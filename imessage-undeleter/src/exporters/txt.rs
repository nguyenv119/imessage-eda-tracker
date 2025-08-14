use std::{
    collections::{
        HashMap,
    },
    fs::File,
    io::BufWriter,
};

use crate::app::{
        error::RuntimeError,
        runtime::Config,
    };

use imessage_database::{
    error::table::TableError,
    tables::{
        messages::Message,
        table::{ORPHANED, Table},
    },
    util::dates::format,
};

pub struct TXT<'a> {
    /// Data that is setup from the application's runtime
    pub config: &'a Config,
    /// Handles to files we want to write messages to
    /// Map of resolved chatroom file location to a buffered writer
    pub files: HashMap<String, BufWriter<File>>,
}

impl<'a> TXT<'a> {
    pub fn new(config: &'a Config) -> Result<Self, RuntimeError> {
        let mut orphaned = config.options.export_path.clone();
        orphaned.push(ORPHANED);
        orphaned.set_extension("txt");

        Ok(TXT {
            config,
            files: HashMap::new(),
        })
    }

    pub fn iter_messages(&mut self) -> Result<HashMap<i32, Message>, RuntimeError> {
        // Keep track of current message ROWID
        let mut current_message_row = -1;

        let mut statement =
            Message::stream_rows(self.config.db(), &self.config.options.query_context)?;

        let messages = statement
            .query_map([], |row| Ok(Message::from_row(row)))
            .map_err(|err| RuntimeError::DatabaseError(TableError::Messages(err)))?;

        let mut msgs: HashMap<i32, Message> = HashMap::new();
        for message in messages {
            let msg = Message::extract(message)?;

            // Early escape if we try and render the same message GUID twice
            // See https://github.com/ReagentX/imessage-exporter/issues/135 for rationale
            if msg.rowid == current_message_row {
                continue;
            }
            current_message_row = msg.rowid;
            msgs.insert(msg.rowid, msg).map(|o|println!("{}", o.rowid));
        }
        Ok(msgs)
    }
}

impl TXT<'_> {
    pub fn get_time(&self, message: &Message) -> String {
        let mut date = format(&message.date(&self.config.offset));
        let read_after = message.time_until_read(&self.config.offset);
        if let Some(time) = read_after {
            if !time.is_empty() {
                let who = if message.is_from_me() {
                    "them"
                } else {
                    self.config.options.custom_name.as_deref().unwrap_or("you")
                };
                date.push_str(&format!(" (Read by {who} after {time})"));
            }
        }
        date
    }
}