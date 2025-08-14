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
        context: &DetectionContext<'_>,
    ) -> Result<Option<DetectionResult>, Box<dyn std::error::Error + Send + Sync>>;
}

/// Context provided to detectors
pub struct DetectionContext<'a> {
    pub config: DetectionConfig,
    pub state_manager: &'a StateManager,
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
        
        Self { detectors, config }
    }

    /// Process a database event and detect any deletions
    pub async fn process_event(
        &self,
        event: &DatabaseEvent,
        context: &DetectionContext<'_>,
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
        context: &DetectionContext<'_>,
    ) -> Result<Vec<DeletionRecord>, Box<dyn std::error::Error + Send + Sync>> {
        let mut deletion_records = Vec::new();

        for &message_id in message_ids {
            // Get current and previous state
            let previous_state = context.state_manager.get_fingerprint(message_id).await?;
            let current_state = self.build_current_fingerprint(message_id).await?;

            // Run all detectors
            for detector in &self.detectors {
                match detector.detect_deletion(
                    message_id,
                    current_state.as_ref(),
                    previous_state.as_ref(),
                    context,
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

            // Update the fingerprint for future comparisons
            if let Some(current) = current_state {
                context.state_manager.store_fingerprint(&current).await?;
            }
        }

        Ok(deletion_records)
    }

    async fn build_current_fingerprint(&self, _message_id: i32) -> Result<Option<MessageFingerprint>, Box<dyn std::error::Error + Send + Sync>> {
        // This would query the current iMessage database to build a fingerprint
        // Implementation depends on your database access layer
        // For now, returning None as placeholder
        Ok(None)
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
        _context: &DetectionContext<'_>,
    ) -> Result<Option<DetectionResult>, Box<dyn std::error::Error + Send + Sync>> {
        match (previous_state, current_state) {
            (Some(prev), None) => {
                // Message existed before but doesn't now - likely deleted
                Ok(Some(DetectionResult {
                    deletion_type: DeletionType::FullMessage,

                    recovered_content: Some("Full message content".to_string()), // Would extract from prev
                    recovered_attachments: prev.attachment_hashes.clone(),
                    metadata: HashMap::new(),
                }))
            }
            (Some(prev), Some(curr)) if prev.content_hash != curr.content_hash => {
                // Content changed - might be an edit or deletion
                Ok(Some(DetectionResult {
                    deletion_type: DeletionType::FullMessage,

                    recovered_content: Some("Modified content".to_string()),
                    recovered_attachments: vec![],
                    metadata: HashMap::new(),
                }))
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
        _context: &DetectionContext<'_>,
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
        context: &DetectionContext<'_>,
    ) -> Result<Option<DetectionResult>, Box<dyn std::error::Error + Send + Sync>> {
        if !context.config.track_edits_as_deletions {
            return Ok(None);
        }

        match (previous_state, current_state) {
            (Some(prev), Some(curr)) => {
                if prev.content_hash != curr.content_hash {
                    // This is a simplified heuristic - in practice you'd do more sophisticated
                    // text diff analysis to determine if content was removed vs. just changed
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
