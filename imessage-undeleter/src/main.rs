/*!
Event-Driven iMessage Deletion Tracker
*/

use std::path::PathBuf;
use tracing::info;
use clap::{Arg, Command};

mod database;
mod core;

use crate::core::{
    tracker::DeletionTracker,
    config::{TrackerConfig, DatabaseConfig, StateConfig, DetectionConfig, DeletionType, OutputConfig, OutputPlugin, TerminalFormat},
};

fn expand_home_path(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/Users".to_string());
        PathBuf::from(path.replacen("~", &home, 1))
    } else {
        PathBuf::from(path)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let matches = Command::new("iMessage Deletion Tracker")
        .version("3.0.0")
        .about("Event-driven iMessage deletion tracker with WAL monitoring")
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
                .default_value("./deletions.json")
        )
        .arg(
            Arg::new("state-db")
                .short('s')
                .long("state-db")
                .help("Path to persistent state database")
                .value_name("PATH")
                .default_value("./tracker_state.db")
        )
        .arg(
            Arg::new("wal-interval")
                .short('i')
                .long("wal-interval")
                .help("WAL check interval in milliseconds")
                .value_name("MS")
                .default_value("1000")
        )
        .get_matches();

    // Build database path
    let db_path = matches.get_one::<String>("db-path")
        .map(|s| PathBuf::from(s))
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/Users".to_string());
            PathBuf::from(format!("{}/Library/Messages/chat.db", home))
        });

    // Build output path
    let output_path = matches.get_one::<String>("output")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("./deletions.json"));

    // Build state DB path
    let state_db_path = matches.get_one::<String>("state-db")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("./tracker_state.db"));

    // Get WAL interval
    let wal_interval: u64 = matches.get_one::<String>("wal-interval")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000);

    // Create output directory if needed
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Build configuration
    let config = TrackerConfig {
        database: DatabaseConfig {
            imessage_db_path: db_path.clone(),
            wal_check_interval_ms: wal_interval,
            max_batch_size: 100,
        },
        state: StateConfig {
            state_db_path,
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
                    path: output_path.clone(),
                    pretty: true 
                },
                config: serde_json::Value::Null,
                enabled: true,
            },
        ],
    };

    info!("iMessage DB: {:?}", db_path);
    info!("State DB: {:?}", config.state.state_db_path);
    info!("Output: {:?}", output_path);
    info!("⏱WAL interval: {}ms", wal_interval);


    let mut tracker = DeletionTracker::new(config).await?;

    tokio::select! {
        result = tracker.start() => {
            if let Err(e) = result {
                eprintln!("Tracker error: {}", e);
                return Err(e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Shutdown signal received");
        }
    }

    Ok(())
}