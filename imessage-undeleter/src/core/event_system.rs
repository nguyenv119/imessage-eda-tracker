/*!
Event-driven system for monitoring database changes via SQLite WAL
*/

use std::time::{Duration, Instant};
use tokio_stream::{wrappers::IntervalStream, StreamExt};
use rusqlite::Connection;
use tracing::{info, debug, error};

use crate::core::config::DatabaseConfig;

/// Events emitted by the database monitoring system
#[derive(Debug, Clone)]
pub enum DatabaseEvent {
    /// New messages detected
    MessagesAdded(Vec<i32>),
    /// Messages modified (potential deletions)
    MessagesModified(Vec<i32>),
    /// Database transaction completed
    TransactionComplete { wal_size: u64, timestamp: Instant },
    /// Error occurred during monitoring
    MonitoringError(String),
}

/// Monitors SQLite WAL (Write-Ahead Log) for changes
pub struct WalMonitor {
    config: DatabaseConfig,
    last_wal_size: u64,
    last_check: Instant,
}

impl WalMonitor {
    pub fn new(config: DatabaseConfig) -> Self {
        Self {
            config,
            last_wal_size: 0,
            last_check: Instant::now(),
        }
    }

    /// Start monitoring the database for changes  
    pub async fn start_monitoring(&mut self) -> impl StreamExt<Item = DatabaseEvent> {
        let interval = Duration::from_millis(self.config.wal_check_interval_ms);
        let mut interval_stream = IntervalStream::new(tokio::time::interval(interval));
        
        async_stream::stream! {
            while let Some(_) = interval_stream.next().await {
                match self.check_for_changes().await {
                    Ok(events) => {
                        for event in events {
                            yield event;
                        }
                    }
                    Err(e) => {
                        error!("WAL monitoring error: {}", e);
                        yield DatabaseEvent::MonitoringError(e.to_string());
                    }
                }
            }
        }
    }

    async fn check_for_changes(&mut self) -> Result<Vec<DatabaseEvent>, Box<dyn std::error::Error>> {
        let wal_path = self.get_wal_path();
        
        if !wal_path.exists() {
            return Ok(vec![]);
        }

        let current_size = tokio::fs::metadata(&wal_path).await?.len();
        let mut events = Vec::new();

        if current_size != self.last_wal_size {
            debug!("WAL size changed: {} -> {}", self.last_wal_size, current_size);
            
            // Check for specific changes in the messages table
            let changed_messages = self.detect_message_changes().await?;
            
            if !changed_messages.is_empty() {
                events.push(DatabaseEvent::MessagesModified(changed_messages));
            }

            events.push(DatabaseEvent::TransactionComplete {
                wal_size: current_size,
                timestamp: Instant::now(),
            });

            self.last_wal_size = current_size;
        }

        self.last_check = Instant::now();
        Ok(events)
    }

    fn get_wal_path(&self) -> std::path::PathBuf {
        let mut wal_path = self.config.imessage_db_path.clone();
        wal_path.set_extension("db-wal");
        wal_path
    }

    async fn detect_message_changes(&self) -> Result<Vec<i32>, Box<dyn std::error::Error>> {
        // This is a simplified approach - in practice, you'd want more sophisticated
        // change detection by parsing the WAL file or using triggers
        let conn = Connection::open(&self.config.imessage_db_path)?;
        
        let mut stmt = conn.prepare("
            SELECT ROWID 
            FROM message 
            WHERE date > ?
            ORDER BY date DESC 
            LIMIT ?
        ")?;

        let since_timestamp = (self.last_check.elapsed().as_secs() as i64) * -1000000000;
        let rows = stmt.query_map([since_timestamp, self.config.max_batch_size as i64], |row| {
            Ok(row.get::<_, i32>(0)?)
        })?;

        let mut message_ids = Vec::new();
        for row in rows {
            message_ids.push(row?);
        }

        Ok(message_ids)
    }
}

/// Higher-level event processor that coordinates different monitoring strategies
pub struct EventProcessor {
    wal_monitor: WalMonitor,
}

impl EventProcessor {
    pub fn new(config: DatabaseConfig) -> Self {
        Self {
            wal_monitor: WalMonitor::new(config),
        }
    }

    /// Start the event processing system
    pub async fn start(&mut self) -> impl StreamExt<Item = DatabaseEvent> {
        info!("Starting event-driven database monitoring...");
        self.wal_monitor.start_monitoring().await
    }
}
