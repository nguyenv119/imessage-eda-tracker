/*!
Modular output system for different deletion logging formats
*/

use std::fs::OpenOptions;
use std::io::Write;
use async_trait::async_trait;
use serde_json;
use rusqlite::Connection;
use tracing::{info, error};

use crate::core::{
    config::{OutputConfig, OutputPlugin, TerminalFormat},
    state_manager::DeletionRecord,
};

/// Trait for output plugins
#[async_trait]
pub trait OutputHandler: Send {
    /// Name of the output handler
    fn name(&self) -> &'static str;
    
    /// Initialize the output handler (create files, connections, etc.)
    async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Handle a deletion record
    async fn handle_deletion(&mut self, deletion: &DeletionRecord) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Cleanup/finalize the output handler
    async fn finalize(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}

/// Manages multiple output handlers
pub struct OutputManager {
    handlers: Vec<Box<dyn OutputHandler>>,
}

impl OutputManager {
    pub fn new(configs: &[OutputConfig]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut handlers: Vec<Box<dyn OutputHandler>> = Vec::new();

        for config in configs {
            if !config.enabled {
                continue;
            }

            let handler: Box<dyn OutputHandler> = match &config.plugin {
                OutputPlugin::Json { path, pretty } => {
                    Box::new(JsonOutputHandler::new(path.clone(), *pretty))
                }
                OutputPlugin::Sqlite { path, table_name } => {
                    Box::new(SqliteOutputHandler::new(path.clone(), table_name.clone()))
                }
                OutputPlugin::Webhook { url, auth_token } => {
                    Box::new(WebhookOutputHandler::new(url.clone(), auth_token.clone()))
                }
                OutputPlugin::Terminal { format } => {
                    Box::new(TerminalOutputHandler::new(*format))
                }
            };

            handlers.push(handler);
        }

        info!("Initialized output manager with {} handlers", handlers.len());
        Ok(Self { handlers })
    }

    /// Initialize all handlers
    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for handler in &mut self.handlers {
            handler.initialize().await?;
            info!("Initialized output handler: {}", handler.name());
        }
        Ok(())
    }

    /// Send a deletion to all enabled handlers
    pub async fn handle_deletion(&mut self, deletion: &DeletionRecord) -> Result<(), Box<dyn std::error::Error>> {
        for handler in &mut self.handlers {
            if let Err(e) = handler.handle_deletion(deletion).await {
                error!("Handler {} failed to process deletion {}: {}", 
                       handler.name(), deletion.id, e);
            }
        }
        Ok(())
    }

    /// Finalize all handlers
    pub async fn finalize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for handler in &mut self.handlers {
            handler.finalize().await?;
        }
        Ok(())
    }
}

/// JSON file output handler
pub struct JsonOutputHandler {
    file_path: std::path::PathBuf,
    pretty: bool,
    file: Option<std::fs::File>,
}

impl JsonOutputHandler {
    pub fn new(file_path: std::path::PathBuf, pretty: bool) -> Self {
        Self {
            file_path,
            pretty,
            file: None,
        }
    }
}

#[async_trait]
impl OutputHandler for JsonOutputHandler {
    fn name(&self) -> &'static str {
        "JSON"
    }

    async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.file = Some(OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)?);
        Ok(())
    }

    async fn handle_deletion(&mut self, deletion: &DeletionRecord) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut file) = self.file {
            let json_str = if self.pretty {
                serde_json::to_string_pretty(deletion)?
            } else {
                serde_json::to_string(deletion)?
            };
            
            writeln!(file, "{}", json_str)?;
            file.flush()?;
        }
        Ok(())
    }

    async fn finalize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut file) = self.file {
            file.flush()?;
        }
        Ok(())
    }
}

/// SQLite database output handler
pub struct SqliteOutputHandler {
    file_path: std::path::PathBuf,
    table_name: String,
    conn: Option<Connection>,
}

impl SqliteOutputHandler {
    pub fn new(file_path: std::path::PathBuf, table_name: String) -> Self {
        Self {
            file_path,
            table_name,
            conn: None,
        }
    }
}

#[async_trait]
impl OutputHandler for SqliteOutputHandler {
    fn name(&self) -> &'static str {
        "SQLite"
    }

    async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = Connection::open(&self.file_path)?;
        
        // Create the output table
        conn.execute(&format!(r#"
            CREATE TABLE IF NOT EXISTS {} (
                id INTEGER PRIMARY KEY,
                message_id INTEGER NOT NULL,
                deletion_timestamp INTEGER NOT NULL,
                deletion_type TEXT NOT NULL,

                recovered_content TEXT,
                recovered_attachments TEXT,
                original_fingerprint TEXT,
                created_at INTEGER DEFAULT (strftime('%s', 'now'))
            )
        "#, self.table_name), [])?;

        self.conn = Some(conn);
        Ok(())
    }

    async fn handle_deletion(&mut self, deletion: &DeletionRecord) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref conn) = self.conn {
            let fingerprint_json = serde_json::to_string(&deletion.original_fingerprint)?;
            let attachments_json = serde_json::to_string(&deletion.recovered_attachments)?;

            conn.execute(&format!(
                "INSERT INTO {} (message_id, deletion_timestamp, deletion_type, recovered_content, recovered_attachments, original_fingerprint)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)", 
                self.table_name
            ), (
                deletion.message_id,
                deletion.deletion_timestamp,
                &deletion.deletion_type,
                deletion.confidence,
                &deletion.recovered_content,
                attachments_json,
                fingerprint_json,
            ))?;
        }
        Ok(())
    }

    async fn finalize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // SQLite auto-commits, no special finalization needed
        Ok(())
    }
}

/// Webhook output handler
pub struct WebhookOutputHandler {
    url: String,
    auth_token: Option<String>,
    client: reqwest::Client,
}

impl WebhookOutputHandler {
    pub fn new(url: String, auth_token: Option<String>) -> Self {
        Self {
            url,
            auth_token,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl OutputHandler for WebhookOutputHandler {
    fn name(&self) -> &'static str {
        "Webhook"
    }

    async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Test the webhook endpoint
        let mut request = self.client.post(&self.url);
        
        if let Some(ref token) = self.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let test_payload = serde_json::json!({
            "test": true,
            "timestamp": chrono::Utc::now().timestamp()
        });

        let response = request
            .json(&test_payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Webhook test failed: {}", response.status()).into());
        }

        Ok(())
    }

    async fn handle_deletion(&mut self, deletion: &DeletionRecord) -> Result<(), Box<dyn std::error::Error>> {
        let mut request = self.client.post(&self.url);
        
        if let Some(ref token) = self.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request
            .json(deletion)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Webhook delivery failed: {}", response.status()).into());
        }

        Ok(())
    }

    async fn finalize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

/// Terminal output handler
pub struct TerminalOutputHandler {
    format: TerminalFormat,
}

impl TerminalOutputHandler {
    pub fn new(format: TerminalFormat) -> Self {
        Self { format }
    }

    fn format_deletion(&self, deletion: &DeletionRecord) -> String {
        match self.format {
            TerminalFormat::Plain => {
                format!(
                    "DELETION DETECTED: Message {} deleted at {} (confidence: {:.2})\nContent: {}\nAttachments: {:?}",
                    deletion.message_id,
                    chrono::DateTime::from_timestamp(deletion.deletion_timestamp, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "Unknown".to_string()),
                    deletion.confidence,
                    deletion.recovered_content.as_deref().unwrap_or("[No content]"),
                    deletion.recovered_attachments
                )
            }
            TerminalFormat::Colored => {
                format!(
                    "\x1b[31m🚨 DELETION DETECTED\x1b[0m\n\
                     \x1b[36m📱 Message ID:\x1b[0m {}\n\
                     \x1b[36m⏰ Timestamp:\x1b[0m {}\n\
                     \x1b[36m🎯 Confidence:\x1b[0m {:.2}\n\
                     \x1b[36m📝 Content:\x1b[0m {}\n\
                     \x1b[36m📎 Attachments:\x1b[0m {:?}",
                    deletion.message_id,
                    chrono::DateTime::from_timestamp(deletion.deletion_timestamp, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "Unknown".to_string()),
                    deletion.confidence,
                    deletion.recovered_content.as_deref().unwrap_or("[No content]"),
                    deletion.recovered_attachments
                )
            }
            TerminalFormat::Json => {
                serde_json::to_string_pretty(deletion).unwrap_or_else(|_| "JSON serialization failed".to_string())
            }
        }
    }
}

#[async_trait]
impl OutputHandler for TerminalOutputHandler {
    fn name(&self) -> &'static str {
        "Terminal"
    }

    async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.format {
            TerminalFormat::Colored => {
                println!("\x1b[32m🚀 iMessage Deletion Tracker Started\x1b[0m");
            }
            _ => {
                println!("🚀 iMessage Deletion Tracker Started");
            }
        }
        Ok(())
    }

    async fn handle_deletion(&mut self, deletion: &DeletionRecord) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", self.format_deletion(deletion));
        println!(); // Add spacing between deletions
        Ok(())
    }

    async fn finalize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.format {
            TerminalFormat::Colored => {
                println!("\x1b[33m🏁 iMessage Deletion Tracker Stopped\x1b[0m");
            }
            _ => {
                println!("🏁 iMessage Deletion Tracker Stopped");
            }
        }
        Ok(())
    }
}
