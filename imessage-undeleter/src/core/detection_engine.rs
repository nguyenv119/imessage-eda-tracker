/*!
Plugin-based detection engine for identifying different types of message deletions
*/

use std::collections::HashMap;
use async_trait::async_trait;
use tracing::{info, debug, warn};

use crate::core::{
    config::{DetectionConfig, DeletionType},
    state_manager::{MessageFingerprint, DeletionRecord, StateManager},
    event_system::DatabaseEvent,
};

/// Trait for deletion detection plugins
#[async_trait]
pub trait DeletionDetector: Send + Sync {
    /// Name of the detector
    fn name(&self) -> &'static str;
    
    /// Types of deletions this detector can identify
    fn supported_types(&self) -> Vec<DeletionType>;
    
    /// Analyze a message change and determine if it represents a deletion
    async fn detect_deletion(
        &self,
        message_id: i32,
        current_state: Option<&MessageFingerprint>,
        previous_state: Option<&MessageFingerprint>,
        context: &DetectionContext,
    ) -> Result<Option<DetectionResult>, Box<dyn std::error::Error + Send + Sync>>;
}

/// Context provided to detectors
pub struct DetectionContext<'a> {
    pub config: DetectionConfig,
    pub state_manager: std::sync::Arc<tokio::sync::RwLock<StateManager>>,
}

/// Result of deletion detection
#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub deletion_type: DeletionType,

    pub recovered_content: Option<String>,
    pub recovered_attachments: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// Main detection engine that coordinates multiple detectors
pub struct DetectionEngine {
    detectors: Vec<Box<dyn DeletionDetector>>,
    config: DetectionConfig,
    db_path: std::path::PathBuf,
}

impl DetectionEngine {
    pub fn new(config: DetectionConfig) -> Self {
        let mut detectors: Vec<Box<dyn DeletionDetector>> = vec![
            Box::new(FullMessageDeletionDetector),
            Box::new(AttachmentDeletionDetector),
            Box::new(PartialEditDetector),
        ];

        // Filter detectors based on configuration
        detectors.retain(|detector| {
            detector.supported_types().iter().any(|dt| config.deletion_types.contains(dt))
        });

        info!("Initialized detection engine with {} detectors", detectors.len());
        
        let db_path = std::path::PathBuf::from("");
        
        Self { detectors, config, db_path }
    }
    
    pub fn with_db_path(mut self, db_path: std::path::PathBuf) -> Self {
        self.db_path = db_path;
        self
    }

    /// Process a database event and detect any deletions
    pub async fn process_event(
        &self,
        event: &DatabaseEvent,
        context: &DetectionContext,
    ) -> Result<Vec<DeletionRecord>, Box<dyn std::error::Error + Send + Sync>> {
        match event {
            DatabaseEvent::MessagesModified(message_ids) => {
                self.analyze_message_changes(message_ids, context).await
            }
            _ => Ok(vec![]),
        }
    }

    async fn analyze_message_changes(
        &self,
        message_ids: &[i32],
        context: &DetectionContext,
    ) -> Result<Vec<DeletionRecord>, Box<dyn std::error::Error + Send + Sync>> {
        let mut deletion_records = Vec::new();

        for &message_id in message_ids {
            // Get current and previous state
            let previous_state = context.state_manager.read().await.get_fingerprint(message_id).await
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("{}", e))) })?;
            let current_state = self.build_current_fingerprint(message_id).await?;

            // Run all detectors
            // Filter detectors that require config checks first
            let should_run_partial_edit = context.config.track_edits_as_deletions;
            
            for detector in &self.detectors {
                // Skip PartialEditDetector if config says not to track edits
                if detector.name() == "PartialEdit" && !should_run_partial_edit {
                    continue;
                }
                
                // Clone Arc to avoid Send issues
                let state_mgr = context.state_manager.clone();
                let cfg = context.config.clone();
                let safe_context = DetectionContext {
                    config: cfg,
                    state_manager: state_mgr,
                };
                
                match detector.detect_deletion(
                    message_id,
                    current_state.as_ref(),
                    previous_state.as_ref(),
                    &safe_context,
                ).await {
                    Ok(Some(result)) => {
                        if true { // Always accept detections
                            let deletion_record = DeletionRecord {
                                id: 0, // Will be set by state manager
                                message_id,
                                original_fingerprint: previous_state.clone().unwrap_or_else(|| {
                                    // Create a dummy fingerprint if we don't have previous state
                                    MessageFingerprint {
                                        message_id,
                                        content_hash: "unknown".to_string(),
                                        attachment_hashes: vec![],
                                        timestamp: chrono::Utc::now().timestamp(),
                                        conversation_id: None,
                                        sender_handle: None,
                                    }
                                }),
                                deletion_timestamp: chrono::Utc::now().timestamp(),
                                deletion_type: format!("{:?}", result.deletion_type),

                                recovered_content: result.recovered_content,
                                recovered_attachments: result.recovered_attachments,
                            };

                            info!(
                                "Detected {} deletion for message {}",
                                detector.name(), message_id
                            );

                            deletion_records.push(deletion_record);
                            break; // Only record one deletion per message
                        }
                    }
                    Ok(None) => {
                        debug!("No deletion detected by {} for message {}", detector.name(), message_id);
                    }
                    Err(e) => {
                        warn!("Detector {} failed for message {}: {}", detector.name(), message_id, e);
                    }
                }
            }

            if let Some(current) = &current_state {
                context.state_manager.write().await.store_fingerprint(current).await
                    .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("{}", e))) })?;
            } else if previous_state.is_some() {
                let deleted_fp = MessageFingerprint {
                    message_id,
                    content_hash: "deleted".to_string(),
                    attachment_hashes: vec![],
                    timestamp: chrono::Utc::now().timestamp(),
                    conversation_id: None,
                    sender_handle: None,
                };
                context.state_manager.write().await.store_fingerprint(&deleted_fp).await
                    .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("{}", e))) })?;
            } else {
                if let Some(current) = &current_state {
                    context.state_manager.write().await.store_fingerprint(current).await
                        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("{}", e))) })?;
                }
            }
        }

        Ok(deletion_records)
    }

    async fn build_current_fingerprint(&self, message_id: i32) -> Result<Option<MessageFingerprint>, Box<dyn std::error::Error + Send + Sync>> {
        use rusqlite::Connection;
        use crate::core::state_manager::StateManager;
        
        let conn = Connection::open(&self.db_path)
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { 
                Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to open DB: {}", e))) 
            })?;
        
        let mut stmt = conn.prepare(
            "SELECT ROWID, text, date, date_edited, cache_has_attachments, handle_id
             FROM message WHERE ROWID = ?"
        )
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { 
            Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("SQL error: {}", e))) 
        })?;
        
        let result = stmt.query_row([message_id], |row| {
            let text: Option<String> = row.get(1)?;
            let date: i64 = row.get(2)?;
            let _date_edited: Option<i64> = row.get(3)?;
            let has_attachments: i32 = row.get(4)?;
            let handle_id: Option<i32> = row.get(5)?;
            
            let content_hash = if let Some(ref t) = text {
                StateManager::hash_content(t)
            } else {
                "empty".to_string()
            };
            
            Ok(MessageFingerprint {
                message_id,
                content_hash,
                attachment_hashes: if has_attachments != 0 { vec!["has_attachment".to_string()] } else { vec![] },
                timestamp: date / 1_000_000_000,
                conversation_id: None,
                sender_handle: handle_id.map(|h| h.to_string()),
            })
        });
        
        match result {
            Ok(fp) => Ok(Some(fp)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None), // Message doesn't exist
            Err(e) => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Query error: {}", e))) as Box<dyn std::error::Error + Send + Sync>),
        }
    }
}

/// Detector for complete message deletions
struct FullMessageDeletionDetector;

#[async_trait]
impl DeletionDetector for FullMessageDeletionDetector {
    fn name(&self) -> &'static str {
        "FullMessageDeletion"
    }

    fn supported_types(&self) -> Vec<DeletionType> {
        vec![DeletionType::FullMessage]
    }

    async fn detect_deletion(
        &self,
        _message_id: i32,
        current_state: Option<&MessageFingerprint>,
        previous_state: Option<&MessageFingerprint>,
        _context: &DetectionContext,
    ) -> Result<Option<DetectionResult>, Box<dyn std::error::Error + Send + Sync>> {
        match (previous_state, current_state) {
            (Some(prev), None) => {
                Ok(Some(DetectionResult {
                    deletion_type: DeletionType::FullMessage,
                    recovered_content: Some(format!("Message {} was fully deleted", prev.message_id)),
                    recovered_attachments: prev.attachment_hashes.clone(),
                    metadata: HashMap::new(),
                }))
            }
            // Message existed before and still exists, but content was deleted
            (Some(prev), Some(curr)) => {
                // If previous had content but current is empty/deleted, it's a deletion
                if prev.content_hash != "empty" && prev.content_hash != "deleted" 
                    && (curr.content_hash == "empty" || curr.content_hash == "deleted") {
                    Ok(Some(DetectionResult {
                        deletion_type: DeletionType::FullMessage,
                        recovered_content: Some(format!("Message {} content was deleted", prev.message_id)),
                        recovered_attachments: prev.attachment_hashes.clone(),
                        metadata: HashMap::new(),
                    }))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }
}

/// Detector for attachment-only deletions
struct AttachmentDeletionDetector;

#[async_trait]
impl DeletionDetector for AttachmentDeletionDetector {
    fn name(&self) -> &'static str {
        "AttachmentDeletion"
    }

    fn supported_types(&self) -> Vec<DeletionType> {
        vec![DeletionType::AttachmentOnly]
    }

    async fn detect_deletion(
        &self,
        _message_id: i32,
        current_state: Option<&MessageFingerprint>,
        previous_state: Option<&MessageFingerprint>,
        _context: &DetectionContext,
    ) -> Result<Option<DetectionResult>, Box<dyn std::error::Error + Send + Sync>> {
        match (previous_state, current_state) {
            (Some(prev), Some(curr)) => {
                if prev.content_hash == curr.content_hash && prev.attachment_hashes != curr.attachment_hashes {
                    // Same text content but different attachments
                    let missing_attachments: Vec<String> = prev.attachment_hashes
                        .iter()
                        .filter(|hash| !curr.attachment_hashes.contains(hash))
                        .cloned()
                        .collect();

                    if !missing_attachments.is_empty() {
                        return Ok(Some(DetectionResult {
                            deletion_type: DeletionType::AttachmentOnly,

                            recovered_content: None,
                            recovered_attachments: missing_attachments,
                            metadata: HashMap::new(),
                        }));
                    }
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}

/// Detector for partial edits that remove content
struct PartialEditDetector;

#[async_trait]
impl DeletionDetector for PartialEditDetector {
    fn name(&self) -> &'static str {
        "PartialEdit"
    }

    fn supported_types(&self) -> Vec<DeletionType> {
        vec![DeletionType::PartialEdit]
    }

    async fn detect_deletion(
        &self,
        _message_id: i32,
        current_state: Option<&MessageFingerprint>,
        previous_state: Option<&MessageFingerprint>,
        _context: &DetectionContext,
    ) -> Result<Option<DetectionResult>, Box<dyn std::error::Error + Send + Sync>> {

        match (previous_state, current_state) {
            (Some(prev), Some(curr)) => {
                if prev.content_hash != curr.content_hash {
                    Ok(Some(DetectionResult {
                        deletion_type: DeletionType::PartialEdit,
                        recovered_content: Some("Content changed".to_string()),
                        recovered_attachments: vec![],
                        metadata: HashMap::new(),
                    }))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }
}
