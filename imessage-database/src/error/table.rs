/*!
 Errors that can happen when extracting data from a `SQLite` table.
*/

use std::fmt::{Display, Formatter, Result};

/// Errors that can happen when extracting data from a `SQLite` table
#[derive(Debug)]
pub enum TableError {
    Attachment(rusqlite::Error),
    ChatToHandle(rusqlite::Error),
    Chat(rusqlite::Error),
    Handle(rusqlite::Error),
    Messages(rusqlite::Error),
    CannotConnect(String),
    CannotRead(std::io::Error),
}

impl Display for TableError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        match self {
            TableError::Attachment(why) => write!(fmt, "Failed to parse attachment row: {why}"),
            TableError::ChatToHandle(why) => write!(fmt, "Failed to parse chat handle row: {why}"),
            TableError::Chat(why) => write!(fmt, "Failed to parse chat row: {why}"),
            TableError::Handle(why) => write!(fmt, "Failed to parse handle row: {why}"),
            TableError::Messages(why) => write!(fmt, "Failed to parse messages row: {why}"),
            TableError::CannotConnect(why) => write!(fmt, "{why}"),
            TableError::CannotRead(why) => write!(fmt, "{why}"),
        }
    }
}
