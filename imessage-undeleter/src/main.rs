/*!
Simple iMessage Deletion Tracker
*/

use std::path::PathBuf;
use std::time::Duration;
use std::collections::HashMap;
use tokio::time::sleep;
use tracing::{info, warn};
use serde::{Serialize, Deserialize};
use clap::{Arg, Command};
use database::{IMessageDatabase, RealMessage};

mod database;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeletionEvent {
    pub message_id: i32,
    pub timestamp: i64,
    pub content: Option<String>,
    pub attachments: Vec<String>,
    pub sender: String,
}

pub struct MessageTracker {
    db_path: PathBuf,
    output_path: PathBuf,
    conversation_filter: Option<String>,
    message_cache: HashMap<i32, RealMessage>,
    imessage_db: Option<IMessageDatabase>,
}

impl MessageTracker {
    pub fn new(db_path: PathBuf, output_path: PathBuf, conversation_filter: Option<String>) -> Self {
        Self {
            db_path,
            output_path,
            conversation_filter,
            message_cache: HashMap::new(),
            imessage_db: None,
        }
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("🚀 Starting iMessage Deletion Tracker");

        // Create output directory
        if let Some(parent) = self.output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Connect to iMessage database
        match IMessageDatabase::new(&self.db_path) {
            Ok(db) => {
                self.imessage_db = Some(db);
            }
            Err(e) => {
                return Err(format!("Failed to connect to iMessage database: {}", e).into());
            }
        }

        // Load initial messages
        self.load_initial_messages().await?;

        // Monitor for changes
        loop {
            self.check_for_changes().await?;
            sleep(Duration::from_millis(500)).await;
        }
    }

    async fn load_initial_messages(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref db) = self.imessage_db {
            let messages = db.get_recent_messages(1000)?;
            
            let filtered_messages: Vec<_> = if let Some(ref filter) = self.conversation_filter {
                messages.into_iter().filter(|msg| {
                    if let Some(handle_id) = msg.handle_id {
                        if let Some(handle) = db.get_handle(handle_id) {
                            return handle.identifier.contains(filter);
                        }
                    }
                    msg.is_from_me
                }).collect()
            } else {
                messages
            };
            
            for message in filtered_messages {
                if message.text.is_some() && message.text.as_ref().unwrap().trim() != "" {
                    self.message_cache.insert(message.id, message);
                }
            }
        }
        Ok(())
    }

    async fn check_for_changes(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.imessage_db.is_none() {
            return Ok(());
        }
        
        let max_cached_id = self.message_cache.keys().max().copied().unwrap_or(0);
        let new_messages = {
            let db = self.imessage_db.as_ref().unwrap();
            db.get_messages_newer_than(max_cached_id)?
        };
        
        let filtered_messages: Vec<_> = if let Some(ref filter) = self.conversation_filter {
            new_messages.into_iter().filter(|msg| {
                if let Some(handle_id) = msg.handle_id {
                    if let Some(handle) = self.imessage_db.as_ref().unwrap().get_handle(handle_id) {
                        return handle.identifier.contains(filter);
                    }
                }
                msg.is_from_me
            }).collect()
        } else {
            new_messages
        };
        
        for message in filtered_messages {
            if !self.message_cache.contains_key(&message.id) {
                if message.text.is_some() && message.text.as_ref().unwrap().trim() != "" {
                    self.message_cache.insert(message.id, message);
                }
            }
        }
        
        let tracked_ids: Vec<i32> = self.message_cache.keys().cloned().collect();
        
        if !tracked_ids.is_empty() {
            let current_messages = {
                let db = self.imessage_db.as_ref().unwrap();
                db.get_messages_by_ids(&tracked_ids)?
            };
            
            for current_msg in current_messages {
                if let Some(cached_msg) = self.message_cache.get(&current_msg.id) {
                    let was_deleted = cached_msg.text.is_some() 
                        && cached_msg.text.as_ref().unwrap().trim() != ""
                        && (current_msg.text.is_none() || current_msg.text.as_ref().unwrap().trim() == "")
                        && current_msg.date_edited.is_some()
                        && current_msg.date_edited > cached_msg.date_edited;
                    
                    if was_deleted {
                        let deletion = {
                            let db = self.imessage_db.as_ref().unwrap();
                            self.create_deletion_event(cached_msg, db).await?
                        };
                        self.handle_deletion(deletion).await?;
                        self.message_cache.insert(current_msg.id, current_msg);
                    }
                }
            }
        }
        
        Ok(())
    }

    async fn create_deletion_event(&self, original_message: &RealMessage, db: &IMessageDatabase) -> Result<DeletionEvent, Box<dyn std::error::Error>> {
        let sender = if let Some(handle_id) = original_message.handle_id {
            if let Some(handle) = db.get_handle(handle_id) {
                handle.identifier.clone()
            } else {
                format!("Unknown (ID: {})", handle_id)
            }
        } else if original_message.is_from_me {
            "Me".to_string()
        } else {
            "Unknown".to_string()
        };

        Ok(DeletionEvent {
            message_id: original_message.id,
            timestamp: original_message.date / 1_000_000_000,
            content: original_message.text.clone(),
            attachments: if original_message.cache_has_attachments {
                vec![format!("attachment_{}.dat", original_message.id)]
            } else {
                vec![]
            },
            sender,
        })
    }

    async fn handle_deletion(&self, deletion: DeletionEvent) -> Result<(), Box<dyn std::error::Error>> {
        warn!("🚨 DELETED/EDITED MESSAGE: \"{}\" from {}", 
            deletion.content.as_deref().unwrap_or("No content"),
            deletion.sender);

        let mut output_data = Vec::new();
        
        if self.output_path.exists() {
            let existing_content = std::fs::read_to_string(&self.output_path)?;
            if !existing_content.trim().is_empty() {
                if let Ok(existing) = serde_json::from_str::<Vec<DeletionEvent>>(&existing_content) {
                    output_data = existing;
                }
            }
        }
        
        output_data.push(deletion);
        let json_content = serde_json::to_string_pretty(&output_data)?;
        std::fs::write(&self.output_path, json_content)?;
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let matches = Command::new("iMessage Deletion Tracker")
        .version("2.0.0")
        .about("Monitors iMessage deletions in real-time")
        .arg(
            Arg::new("db-path")
                .short('p')
                .long("db-path")
                .help("Path to the iMessage database")
                .value_name("PATH")
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Output file path (JSON format)")
                .value_name("PATH")
                .default_value("./undeleted_messages/deletions.json")
        )
        .arg(
            Arg::new("filter")
                .short('t')
                .long("filter")
                .help("Filter conversations by contact")
                .value_name("CONTACT")
        )
        .get_matches();

    let db_path = matches.get_one::<String>("db-path")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/Users".to_string());
            PathBuf::from(format!("{}/Library/Messages/chat.db", home))
        });

    let output_path = matches.get_one::<String>("output")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("./undeleted_messages/deletions.json"));

    let conversation_filter = matches.get_one::<String>("filter").cloned();

    let mut tracker = MessageTracker::new(db_path, output_path, conversation_filter);

    tokio::select! {
        result = tracker.start() => {
            if let Err(e) = result {
                eprintln!("Tracker error: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("🛑 Shutdown");
        }
    }

    Ok(())
}