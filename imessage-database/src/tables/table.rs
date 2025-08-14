/*!
 This module defines traits for table representations and stores some shared table constants.
*/

use std::{collections::HashMap, fs::metadata, path::Path};

use rusqlite::{Connection, Error, OpenFlags, Result, Row, Statement, blob::Blob};

use crate::{error::table::TableError, tables::messages::models::BubbleComponent};

/// Defines behavior for SQL Table data
pub trait Table {
    /// Deserializes a single row of data into an instance of the struct that implements this Trait
    fn from_row(row: &Row) -> Result<Self>
    where
        Self: Sized;
    /// Gets a statement we can execute to iterate over the data in the table
    fn get(db: &Connection) -> Result<Statement, TableError>;

    /// Extract valid row data while handling both types of query errors
    fn extract(item: Result<Result<Self, Error>, Error>) -> Result<Self, TableError>
    where
        Self: Sized;
}

/// Defines behavior for table data that can be cached in memory
pub trait Cacheable {
    type K;
    type V;
    fn cache(db: &Connection) -> Result<HashMap<Self::K, Self::V>, TableError>;
}

/// Defines behavior for deduplicating data in a table
pub trait Deduplicate {
    type T;
    fn dedupe(duplicated_data: &HashMap<i32, Self::T>) -> HashMap<i32, i32>;
}

/// Defines behavior for printing diagnostic information for a table
pub trait Diagnostic {
    /// Emit diagnostic data about the table to `stdout`
    fn run_diagnostic(db: &Connection) -> Result<(), TableError>;
}

/// Defines behavior for getting BLOB data from from a table
pub trait GetBlob {
    /// Retreive `BLOB` data from a table
    fn get_blob<'a>(&self, db: &'a Connection, column: &str) -> Option<Blob<'a>>;
}

/// Defines behavior for deserializing a message's [`typedstream`](crate::util::typedstream) body data in native Rust
pub trait AttributedBody {
    /// Get a vector of a message body's components. If the text has not been captured, the vector will be empty.
    ///
    /// # Parsing
    ///
    /// There are two different ways this crate will attempt to parse this data.
    ///
    /// ## Default parsing
    ///
    /// In most cases, the message body will be deserialized using the [`typedstream`](crate::util::typedstream) deserializer.
    ///
    /// *Note*: message body text can be formatted with a [`Vec`] of [`TextAttributes`](crate::tables::messages::models::TextAttributes).
    ///
    /// ## Legacy parsing
    ///
    /// If the `typedstream` data cannot be deserialized, this method falls back to a legacy string parsing algorithm that
    /// only supports unstyled text.
    ///
    /// If the message has attachments, there will be one [`U+FFFC`](https://www.compart.com/en/unicode/U+FFFC) character
    /// for each attachment and one [`U+FFFD`](https://www.compart.com/en/unicode/U+FFFD) for app messages that we need
    /// to format.
    ///
    /// ## Sample
    ///
    /// An iMessage that contains body text like:
    ///
    /// ```
    /// let message_text = "\u{FFFC}Check out this photo!";
    /// ```
    ///
    /// Will have a `body()` of:
    ///
    /// ```
    /// use imessage_database::message_types::text_effects::TextEffect;
    /// use imessage_database::tables::messages::{models::{TextAttributes, BubbleComponent, AttachmentMeta}};
    ///  
    /// let result = vec![
    ///     BubbleComponent::Attachment(AttachmentMeta::default()),
    ///     BubbleComponent::Text(vec![TextAttributes::new(3, 24, TextEffect::Default)]),
    /// ];
    /// ```
    fn body(&self) -> Vec<BubbleComponent>;
}

/// Get a connection to the iMessage `SQLite` database
// # Example:
///
/// ```
/// use imessage_database::{
///     util::dirs::default_db_path,
///     tables::table::get_connection
/// };
///
/// let db_path = default_db_path();
/// let connection = get_connection(&db_path);
/// ```
pub fn get_connection(path: &Path) -> Result<Connection, TableError> {
    if path.exists() && path.is_file() {
        return match Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        ) {
            Ok(res) => Ok(res),
            Err(why) => Err(TableError::CannotConnect(format!(
                "Unable to read from chat database: {why}\nEnsure full disk access is enabled for your terminal emulator in System Settings > Privacy & Security > Full Disk Access"
            ))),
        };
    }

    // Path does not point to a file
    if path.exists() && !path.is_file() {
        return Err(TableError::CannotConnect(format!(
            "Specified path `{}` is not a database!",
            &path.to_str().unwrap_or("Unknown")
        )));
    }

    // File is missing
    Err(TableError::CannotConnect(format!(
        "Database not found at {}",
        &path.to_str().unwrap_or("Unknown")
    )))
}

/// Get the size of the database on the disk
// # Example:
///
/// ```
/// use imessage_database::{
///     util::dirs::default_db_path,
///     tables::table::get_db_size
/// };
///
/// let db_path = default_db_path();
/// let database_size_in_bytes = get_db_size(&db_path);
/// ```
pub fn get_db_size(path: &Path) -> Result<u64, TableError> {
    Ok(metadata(path).map_err(TableError::CannotRead)?.len())
}

// Table Names
/// Handle table name
pub const HANDLE: &str = "handle";
/// Message table name
pub const MESSAGE: &str = "message";
/// Chat table name
pub const CHAT: &str = "chat";
/// Attachment table name
pub const ATTACHMENT: &str = "attachment";
/// Chat to message join table name
pub const CHAT_MESSAGE_JOIN: &str = "chat_message_join";
/// Message to attachment join table name
pub const MESSAGE_ATTACHMENT_JOIN: &str = "message_attachment_join";
/// Chat to handle join table name
pub const CHAT_HANDLE_JOIN: &str = "chat_handle_join";
/// Recently deleted messages table
pub const RECENTLY_DELETED: &str = "chat_recoverable_message_join";

// Column names
/// The payload data column contains `plist`-encoded app message data
pub const MESSAGE_PAYLOAD: &str = "payload_data";
/// The message summary info column contains `plist`-encoded edited message information
pub const MESSAGE_SUMMARY_INFO: &str = "message_summary_info";
/// The `attributedBody` column contains [`typedstream`](crate::util::typedstream)-encoded a message's body text with many other attributes
pub const ATTRIBUTED_BODY: &str = "attributedBody";
/// The sticker user info column contains `plist`-encoded metadata for sticker attachments
pub const STICKER_USER_INFO: &str = "sticker_user_info";
/// The attribution info contains `plist`-encoded metadata for sticker attachments
pub const ATTRIBUTION_INFO: &str = "attribution_info";

// Default information
/// Name used for messages sent by the database owner in a first-person context
pub const ME: &str = "Me";
/// Name used for messages sent by the database owner in a second-person context
pub const YOU: &str = "You";
/// Name used for contacts or chats where the name cannot be discovered
pub const UNKNOWN: &str = "Unknown";
/// Default location for the Messages database on macOS
pub const DEFAULT_PATH_MACOS: &str = "Library/Messages/chat.db";
/// Default location for the Messages database in an iOS backup
pub const DEFAULT_PATH_IOS: &str = "3d/3d0d7e5fb2ce288813306e4d4636395e047a3d28";
/// Chat name reserved for messages that do not belong to a chat in the table
pub const ORPHANED: &str = "orphaned";
/// Replacement text sent in Fitness.app messages
pub const FITNESS_RECEIVER: &str = "$(kIMTranscriptPluginBreadcrumbTextReceiverIdentifier)";
/// Name for attachments directory in exports
pub const ATTACHMENTS_DIR: &str = "attachments";
