/*!
Persistent state management for tracking message fingerprints and deletions
*/

use rusqlite::{Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use tracing::{info, debug};
use blake3;

use crate::core::config::StateConfig;

/// Represents a message fingerprint for deletion detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageFingerprint {
    pub message_id: i32,
    pub content_hash: String,
    pub attachment_hashes: Vec<String>,
    pub timestamp: i64,
    pub conversation_id: Option<i32>,
    pub sender_handle: Option<String>,
}

/// Represents a detected deletion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletionRecord {
    pub id: i64,
    pub message_id: i32,
    pub original_fingerprint: MessageFingerprint,
    pub deletion_timestamp: i64,
    pub deletion_type: String,

    pub recovered_content: Option<String>,
    pub recovered_attachments: Vec<String>,
}

/// Manages persistent state for the deletion tracker
pub struct StateManager {
    config: StateConfig,
    conn: Connection,
}

impl StateManager {
    /// Create a new state manager and initialize the database
    pub async fn new(config: StateConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::open(&config.state_db_path)?;
        
        let manager = Self { config, conn };
        manager.initialize_schema().await?;
        manager.cleanup_old_records().await?;
        
        info!("State manager initialized with database: {:?}", manager.config.state_db_path);
        Ok(manager)
    }

    /// Initialize the database schema
    async fn initialize_schema(&self) -> SqliteResult<()> {
        self.conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS message_fingerprints (
                message_id INTEGER PRIMARY KEY,
                content_hash TEXT NOT NULL,
                attachment_hashes TEXT, -- JSON array
                timestamp INTEGER NOT NULL,
                conversation_id INTEGER,
                sender_handle TEXT,
                created_at INTEGER DEFAULT (strftime('%s', 'now'))
            );

            CREATE TABLE IF NOT EXISTS deletion_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                message_id INTEGER NOT NULL,
                original_fingerprint TEXT NOT NULL, -- JSON
                deletion_timestamp INTEGER NOT NULL,
                deletion_type TEXT NOT NULL,

                recovered_content TEXT,
                recovered_attachments TEXT, -- JSON array
                created_at INTEGER DEFAULT (strftime('%s', 'now'))
            );

            CREATE INDEX IF NOT EXISTS idx_fingerprints_timestamp ON message_fingerprints(timestamp);
            CREATE INDEX IF NOT EXISTS idx_deletions_timestamp ON deletion_records(deletion_timestamp);
            CREATE INDEX IF NOT EXISTS idx_fingerprints_conversation ON message_fingerprints(conversation_id);
        "#)?;

        Ok(())
    }

    /// Store a message fingerprint
    pub async fn store_fingerprint(&self, fingerprint: &MessageFingerprint) -> Result<(), Box<dyn std::error::Error>> {
        let attachment_hashes_json = serde_json::to_string(&fingerprint.attachment_hashes)?;
        
        self.conn.execute(
            "INSERT OR REPLACE INTO message_fingerprints 
             (message_id, content_hash, attachment_hashes, timestamp, conversation_id, sender_handle)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                fingerprint.message_id,
                &fingerprint.content_hash,
                attachment_hashes_json,
                fingerprint.timestamp,
                fingerprint.conversation_id,
                &fingerprint.sender_handle,
            ),
        )?;

        debug!("Stored fingerprint for message {}", fingerprint.message_id);
        Ok(())
    }

    /// Get a stored fingerprint by message ID
    pub async fn get_fingerprint(&self, message_id: i32) -> Result<Option<MessageFingerprint>, Box<dyn std::error::Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT message_id, content_hash, attachment_hashes, timestamp, conversation_id, sender_handle
             FROM message_fingerprints WHERE message_id = ?1"
        )?;

        let fingerprint = stmt.query_row([message_id], |row| {
            let attachment_hashes_json: String = row.get(2)?;
            let attachment_hashes: Vec<String> = serde_json::from_str(&attachment_hashes_json)
                .unwrap_or_default();

            Ok(MessageFingerprint {
                message_id: row.get(0)?,
                content_hash: row.get(1)?,
                attachment_hashes,
                timestamp: row.get(3)?,
                conversation_id: row.get(4)?,
                sender_handle: row.get(5)?,
            })
        });

        match fingerprint {
            Ok(fp) => Ok(Some(fp)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Store a deletion record
    pub async fn store_deletion(&self, deletion: &DeletionRecord) -> Result<i64, Box<dyn std::error::Error>> {
        let fingerprint_json = serde_json::to_string(&deletion.original_fingerprint)?;
        let attachments_json = serde_json::to_string(&deletion.recovered_attachments)?;

        let mut stmt = self.conn.prepare(
            "INSERT INTO deletion_records 
             (message_id, original_fingerprint, deletion_timestamp, deletion_type, recovered_content, recovered_attachments)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
        )?;

        let deletion_id = stmt.insert((
            deletion.message_id,
            fingerprint_json,
            deletion.deletion_timestamp,
            &deletion.deletion_type,

            &deletion.recovered_content,
            attachments_json,
        ))?;

        info!("Stored deletion record {} for message {}", deletion_id, deletion.message_id);
        Ok(deletion_id)
    }

    /// Get all deletion records within a time range
    pub async fn get_deletions_in_range(&self, start_time: i64, end_time: i64) -> Result<Vec<DeletionRecord>, Box<dyn std::error::Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, message_id, original_fingerprint, deletion_timestamp, deletion_type, recovered_content, recovered_attachments
             FROM deletion_records 
             WHERE deletion_timestamp BETWEEN ?1 AND ?2
             ORDER BY deletion_timestamp DESC"
        )?;

        let rows = stmt.query_map([start_time, end_time], |row| {
            let fingerprint_json: String = row.get(2)?;
            let attachments_json: String = row.get(7)?;
            
            let original_fingerprint: MessageFingerprint = serde_json::from_str(&fingerprint_json)
                .map_err(|_e| rusqlite::Error::InvalidColumnType(2, "fingerprint".to_string(), rusqlite::types::Type::Text))?;
            
            let recovered_attachments: Vec<String> = serde_json::from_str(&attachments_json)
                .unwrap_or_default();

            Ok(DeletionRecord {
                id: row.get(0)?,
                message_id: row.get(1)?,
                original_fingerprint,
                deletion_timestamp: row.get(3)?,
                deletion_type: row.get(4)?,

                recovered_content: row.get(6)?,
                recovered_attachments,
            })
        })?;

        let mut deletions = Vec::new();
        for row in rows {
            deletions.push(row?);
        }

        Ok(deletions)
    }

    /// Batch store multiple fingerprints efficiently
    pub async fn batch_store_fingerprints(&self, fingerprints: &[MessageFingerprint]) -> Result<(), Box<dyn std::error::Error>> {
        let tx = self.conn.unchecked_transaction()?;
        
        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO message_fingerprints 
                 (message_id, content_hash, attachment_hashes, timestamp, conversation_id, sender_handle)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
            )?;

            for fingerprint in fingerprints {
                let attachment_hashes_json = serde_json::to_string(&fingerprint.attachment_hashes)?;
                
                stmt.execute((
                    fingerprint.message_id,
                    &fingerprint.content_hash,
                    attachment_hashes_json,
                    fingerprint.timestamp,
                    fingerprint.conversation_id,
                    &fingerprint.sender_handle,
                ))?;
            }
        }

        tx.commit()?;
        debug!("Batch stored {} fingerprints", fingerprints.len());
        Ok(())
    }

    /// Clean up old records based on retention policy
    async fn cleanup_old_records(&self) -> SqliteResult<()> {
        let cutoff_timestamp = chrono::Utc::now().timestamp() - (self.config.retention_days as i64 * 24 * 60 * 60);
        
        let deleted_fingerprints = self.conn.execute(
            "DELETE FROM message_fingerprints WHERE timestamp < ?1",
            [cutoff_timestamp],
        )?;

        let deleted_records = self.conn.execute(
            "DELETE FROM deletion_records WHERE deletion_timestamp < ?1",
            [cutoff_timestamp],
        )?;

        if deleted_fingerprints > 0 || deleted_records > 0 {
            info!("Cleaned up {} old fingerprints and {} old deletion records", 
                  deleted_fingerprints, deleted_records);
        }

        Ok(())
    }

    /// Generate a content hash for a message
    pub fn hash_content(content: &str) -> String {
        blake3::hash(content.as_bytes()).to_hex().to_string()
    }

    /// Generate a hash for attachment metadata
    pub fn hash_attachment(filename: &str, size: u64, modified: Option<i64>) -> String {
        let input = format!("{}:{}:{}", filename, size, modified.unwrap_or(0));
        blake3::hash(input.as_bytes()).to_hex().to_string()
    }
}
