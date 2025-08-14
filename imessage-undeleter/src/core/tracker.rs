/*!
Main async coordinator that orchestrates the event-driven deletion tracking system
*/

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::StreamExt;
use tracing::{info, error};

use crate::core::{
    config::TrackerConfig,
    event_system::{EventProcessor, DatabaseEvent},
    state_manager::StateManager,
    detection_engine::DetectionEngine,
    output_plugins::OutputManager,
};

/// Main tracker that coordinates all components
pub struct DeletionTracker {
    config: TrackerConfig,
    event_processor: EventProcessor,
    state_manager: Arc<RwLock<StateManager>>,
    detection_engine: DetectionEngine,
    output_manager: Arc<RwLock<OutputManager>>,
}

impl DeletionTracker {
    /// Create a new deletion tracker
    pub async fn new(config: TrackerConfig) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Initializing new event-driven deletion tracker...");

        // Initialize components
        let event_processor = EventProcessor::new(config.database.clone());
        let state_manager = Arc::new(RwLock::new(
            StateManager::new(config.state.clone()).await?
        ));
        let detection_engine = DetectionEngine::new(config.detection.clone());
        let output_manager = Arc::new(RwLock::new(
            OutputManager::new(&config.outputs)?
        ));

        // Initialize output handlers
        output_manager.write().await.initialize().await?;

        Ok(Self {
            config,
            event_processor,
            state_manager,
            detection_engine,
            output_manager,
        })
    }

    /// Start the async deletion tracking loop
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("🚀 Starting iMessage deletion tracker with new architecture...");
        info!("📊 Monitoring: {:?}", self.config.database.imessage_db_path);
        info!("💾 State DB: {:?}", self.config.state.state_db_path);
        info!("🔍 Detection types: {:?}", self.config.detection.deletion_types);
        // Start the event stream
        let mut event_stream = Box::pin(self.event_processor.start().await);
        // Process events as they arrive  
        while let Some(event) = event_stream.next().await {
            if let Err(e) = self.handle_event(event).await {
                error!("Error handling event: {}", e);
            }
        }

        // Cleanup
        self.output_manager.write().await.finalize().await?;
        info!("🏁 Deletion tracker stopped gracefully");

        Ok(())
    }

    /// Handle a single database event
    async fn handle_event(&self, event: DatabaseEvent) -> Result<(), Box<dyn std::error::Error>> {
        match &event {
            DatabaseEvent::MessagesModified(message_ids) => {
                info!("🔄 Processing {} modified messages", message_ids.len());
                
                // Simplified detection for now - would use full detection engine in production

                // For now, simplify by handling the detection inline
                // This is a simplified approach - in production you'd want a more sophisticated design
                let deletions: Vec<crate::core::state_manager::DeletionRecord> = Vec::new(); // Placeholder for now
                
                // Process each deletion
                for deletion in deletions {
                    info!("🚨 Deletion detected: Message {}", 
                          deletion.message_id);
                    
                    // Store deletion record
                    let deletion_id = self.state_manager.write().await
                        .store_deletion(&deletion).await?;
                    
                    // Send to output handlers
                    let mut deletion_with_id = deletion;
                    deletion_with_id.id = deletion_id;
                    
                    self.output_manager.write().await
                        .handle_deletion(&deletion_with_id).await?;
                }
            }
            DatabaseEvent::TransactionComplete { wal_size, timestamp } => {
                // Periodic housekeeping could go here
                if wal_size % 1000000 == 0 { // Log every 1MB of WAL changes
                    info!("📈 WAL size: {} bytes at {:?}", wal_size, timestamp);
                }
            }
            DatabaseEvent::MonitoringError(error) => {
                error!("⚠️ Monitoring error: {}", error);
                // Could implement reconnection logic here
            }
            _ => {
                // Handle other event types as needed
            }
        }

        Ok(())
    }

    /// Get current tracker statistics
    pub async fn get_stats(&self) -> TrackerStats {
        // This could query the state manager for statistics
        TrackerStats {
            total_deletions_detected: 0, // Would query from state manager
            uptime_seconds: 0,
            events_processed: 0,
            last_event_time: None,
        }
    }
}

/// Statistics about the tracker's operation
#[derive(Debug, Clone)]
pub struct TrackerStats {
    pub total_deletions_detected: u64,
    pub uptime_seconds: u64,
    pub events_processed: u64,
    pub last_event_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// Graceful shutdown handler
pub struct ShutdownHandler {
    tracker: Option<DeletionTracker>,
}

impl ShutdownHandler {
    pub fn new(tracker: DeletionTracker) -> Self {
        Self {
            tracker: Some(tracker),
        }
    }

    /// Handle graceful shutdown
    pub async fn shutdown(mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(tracker) = self.tracker.take() {
            info!("🛑 Initiating graceful shutdown...");
            
            // Finalize output handlers
            tracker.output_manager.write().await.finalize().await?;
            
            info!("✅ Shutdown completed successfully");
        }
        Ok(())
    }
}

/// Helper function to create a tracker from a config file
pub async fn create_tracker_from_config_file<P: AsRef<std::path::Path>>(
    config_path: P,
) -> Result<DeletionTracker, Box<dyn std::error::Error>> {
    let config_content = tokio::fs::read_to_string(config_path).await?;
    let config: TrackerConfig = toml::from_str(&config_content)?;
    DeletionTracker::new(config).await
}

/// Helper function to create a tracker with default config
pub async fn create_default_tracker() -> Result<DeletionTracker, Box<dyn std::error::Error>> {
    let config = TrackerConfig::default();
    DeletionTracker::new(config).await
}
