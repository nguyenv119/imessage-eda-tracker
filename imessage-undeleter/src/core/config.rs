/*!
Configuration management for the deletion tracker
*/

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TrackerConfig {
    /// Database monitoring settings
    pub database: DatabaseConfig,
    /// State persistence settings  
    pub state: StateConfig,
    /// Detection behavior settings
    pub detection: DetectionConfig,
    /// Output configuration
    pub outputs: Vec<OutputConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    /// Path to the iMessage database
    pub imessage_db_path: PathBuf,
    /// WAL monitoring interval in milliseconds
    pub wal_check_interval_ms: u64,
    /// Maximum number of transactions to process per batch
    pub max_batch_size: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StateConfig {
    /// Path to persistent state database
    pub state_db_path: PathBuf,
    /// How long to retain deletion records (in days)
    pub retention_days: u32,
    /// Whether to enable state compression
    pub enable_compression: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DetectionConfig {
    /// Types of deletions to track
    pub deletion_types: Vec<DeletionType>,

    /// Whether to track partial message edits as deletions
    pub track_edits_as_deletions: bool,
    /// Conversation filters
    pub conversation_filters: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum DeletionType {
    FullMessage,
    PartialEdit,
    AttachmentOnly,
    MediaContent,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutputConfig {
    /// Output plugin type
    pub plugin: OutputPlugin,
    /// Plugin-specific configuration
    pub config: serde_json::Value,
    /// Whether this output is enabled
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum OutputPlugin {
    Json { path: PathBuf, pretty: bool },
    Sqlite { path: PathBuf, table_name: String },
    Webhook { url: String, auth_token: Option<String> },
    Terminal { format: TerminalFormat },
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum TerminalFormat {
    Plain,
    Colored,
    Json,
}

impl Default for TrackerConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                imessage_db_path: PathBuf::from("~/Library/Messages/chat.db"),
                wal_check_interval_ms: 1000,
                max_batch_size: 100,
            },
            state: StateConfig {
                state_db_path: PathBuf::from("./tracker_state.db"),
                retention_days: 30,
                enable_compression: true,
            },
            detection: DetectionConfig {
                deletion_types: vec![DeletionType::FullMessage, DeletionType::AttachmentOnly],

                track_edits_as_deletions: false,
                conversation_filters: vec![],
            },
            outputs: vec![
                OutputConfig {
                    plugin: OutputPlugin::Terminal { 
                        format: TerminalFormat::Colored 
                    },
                    config: serde_json::Value::Null,
                    enabled: true,
                },
                OutputConfig {
                    plugin: OutputPlugin::Json { 
                        path: PathBuf::from("./deletions.json"),
                        pretty: true 
                    },
                    config: serde_json::Value::Null,
                    enabled: true,
                },
            ],
        }
    }
}
