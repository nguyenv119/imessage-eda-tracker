/*!
 This module represents common (but not all) columns in the `message` table.

 # Iterating over Message Data

 Generally, use [`Message::get()`] or [`Message::stream_rows()`] to iterate over message rows.

 ## Example
 ```rust
 use imessage_database::{
     tables::{
         messages::Message,
         table::{get_connection, Diagnostic, Table},
     },
     util::dirs::default_db_path,
 };

 let db_path = default_db_path();
 let conn = get_connection(&db_path).unwrap();

 let mut statement = Message::get(&conn).unwrap();

 let messages = statement.query_map([], |row| Ok(Message::from_row(row))).unwrap();

 messages.map(|msg| println!("{:#?}", Message::extract(msg)));
 ```

 # Making Custom Message Queries

 In addition columns from the `messages` table, there are several additional fields represented
 by [`Message`]  that are not present in the database:

 - [`Message::chat_id`]
 - [`Message::num_attachments`]
 - [`Message::deleted_from`]
 - [`Message::num_replies`]

 ## Sample Queries

 To provide a custom query, ensure inclusion of the foregoing columns:

 ```sql
 SELECT
     *,
     c.chat_id,
     (SELECT COUNT(*) FROM message_attachment_join a WHERE m.ROWID = a.message_id) as num_attachments,
     d.chat_id as deleted_from,
     (SELECT COUNT(*) FROM message m2 WHERE m2.thread_originator_guid = m.guid) as num_replies
 FROM
     message as m
 LEFT JOIN chat_message_join as c ON m.ROWID = c.message_id
 LEFT JOIN chat_recoverable_message_join as d ON m.ROWID = d.message_id
 ORDER BY
     m.date;
 ```

 If the source database does not include these required columns, include them as so:

 ```sql
 SELECT
     *,
     c.chat_id,
     (SELECT COUNT(*) FROM message_attachment_join a WHERE m.ROWID = a.message_id) as num_attachments,
     NULL as deleted_from,
     0 as num_replies
 FROM
     message as m
 LEFT JOIN chat_message_join as c ON m.ROWID = c.message_id
 ORDER BY
     m.date;
 ```

 ## Custom Query Example

 The following will return an iterator over messages that have an associated emoji:


 ```rust
 use imessage_database::{
     tables::{
         messages::Message,
         table::{get_connection, Diagnostic, Table},
     },
     util::dirs::default_db_path
 };

 let db_path = default_db_path();
 let db = get_connection(&db_path).unwrap();

 let mut statement = db.prepare("
 SELECT
     *,
     c.chat_id,
     (SELECT COUNT(*) FROM message_attachment_join a WHERE m.ROWID = a.message_id) as num_attachments,
     d.chat_id as deleted_from,
     (SELECT COUNT(*) FROM message m2 WHERE m2.thread_originator_guid = m.guid) as num_replies
 FROM
     message as m
 LEFT JOIN chat_message_join as c ON m.ROWID = c.message_id
 LEFT JOIN chat_recoverable_message_join as d ON m.ROWID = d.message_id
 WHERE m.associated_message_emoji IS NOT NULL
 ORDER BY
     m.date;
 ").unwrap();

 let messages = statement.query_map([], |row| Ok(Message::from_row(row))).unwrap();

 messages.map(|msg| println!("{:#?}", Message::extract(msg)));
 ```
*/

use std::{collections::HashMap, io::Read};

use chrono::{DateTime, offset::Local};
use plist::Value;
use rusqlite::{Connection, Error, Result, Row, Statement, blob::Blob};

use crate::{
    error::{message::MessageError, table::TableError},
    message_types::{
        edited::{EditStatus, EditedMessage},
        expressives::{BubbleEffect, Expressive, ScreenEffect},
        variants::{Announcement, BalloonProvider, CustomBalloon, Tapback, TapbackAction, Variant},
    },
    tables::{
        messages::{
            body::{parse_body_legacy, parse_body_typedstream},
            models::{BubbleComponent, GroupAction, Service},
            query_parts::{ios_13_older_query, ios_14_15_query, ios_16_newer_query},
        },
        table::{
            ATTRIBUTED_BODY, AttributedBody, CHAT_MESSAGE_JOIN, Cacheable, Diagnostic, GetBlob,
            MESSAGE, MESSAGE_ATTACHMENT_JOIN, MESSAGE_PAYLOAD, MESSAGE_SUMMARY_INFO,
            RECENTLY_DELETED, Table,
        },
    },
    util::{
        bundle_id::parse_balloon_bundle_id,
        dates::{get_local_time, readable_diff},
        output::{done_processing, processing},
        query_context::QueryContext,
        streamtyped,
        typedstream::{models::Archivable, parser::TypedStreamReader},
    },
};

/// The required columns, interpolated into the most recent schema due to performance considerations
pub(crate) const COLS: &str = "rowid, guid, text, service, handle_id, destination_caller_id, subject, date, date_read, date_delivered, is_from_me, is_read, item_type, other_handle, share_status, share_direction, group_title, group_action_type, associated_message_guid, associated_message_type, balloon_bundle_id, expressive_send_style_id, thread_originator_guid, thread_originator_part, date_edited, associated_message_emoji";

/// Represents a single row in the `message` table.
///
/// Additional information is available in the [parent](crate::tables::messages::message) module.
#[derive(Debug)]
#[allow(non_snake_case)]
pub struct Message {
    pub rowid: i32,
    pub guid: String,
    /// The text of the message, which may require calling [`Self::generate_text()`] to populate
    pub text: Option<String>,
    /// The service the message was sent from
    pub service: Option<String>,
    /// The ID of the person who sent the message
    pub handle_id: Option<i32>,
    /// The address the database owner received the message at, i.e. a phone number or email
    pub destination_caller_id: Option<String>,
    /// The content of the Subject field
    pub subject: Option<String>,
    /// The date the message was written to the database
    pub date: i64,
    /// The date the message was read
    pub date_read: i64,
    /// The date a message was delivered
    pub date_delivered: i64,
    /// `true` if the database owner sent the message, else `false`
    pub is_from_me: bool,
    /// `true` if the message was read by the recipient, else `false`
    pub is_read: bool,
    /// Intermediate data for determining the [`Variant`] of a message
    pub item_type: i32,
    /// Optional handle for the recipient of a message that includes shared content
    pub other_handle: Option<i32>,
    /// Boolean determining whether some shared data is active or inactive, i.e. shared location being enabled or disabled
    pub share_status: bool,
    /// Boolean determining the direction shared data was sent; `false` indicates it was sent from the database owner, `true` indicates it was sent to the database owner
    pub share_direction: Option<bool>,
    /// If the message updates the [`display_name`](crate::tables::chat::Chat::display_name) of the chat, this field will be populated
    pub group_title: Option<String>,
    /// If the message modified for a group, this will be nonzero
    pub group_action_type: i32,
    /// The message GUID of a message associated with this one
    pub associated_message_guid: Option<String>,
    /// Intermediate data for determining the [`Variant`] of a message
    pub associated_message_type: Option<i32>,
    /// The [bundle ID](https://developer.apple.com/help/app-store-connect/reference/app-bundle-information) of the app that generated the [`AppMessage`](crate::message_types::app::AppMessage)
    pub balloon_bundle_id: Option<String>,
    /// Intermediate data for determining the [`expressive`](crate::message_types::expressives) of a message
    pub expressive_send_style_id: Option<String>,
    /// Indicates the first message in a thread of replies in [`get_replies()`](crate::tables::messages::Message::get_replies)
    pub thread_originator_guid: Option<String>,
    /// Indicates the part of a message a reply is pointing to
    pub thread_originator_part: Option<String>,
    /// The date the message was most recently edited
    pub date_edited: i64,
    /// If present, this is the emoji associated with a custom emoji tapback
    pub associated_message_emoji: Option<String>,
    /// The [`identifier`](crate::tables::chat::Chat::chat_identifier) of the chat the message belongs to
    pub chat_id: Option<i32>,
    /// The number of attached files included in the message
    pub num_attachments: i32,
    /// The [`identifier`](crate::tables::chat::Chat::chat_identifier) of the chat the message was deleted from
    pub deleted_from: Option<i32>,
    /// The number of replies to the message
    pub num_replies: i32,
    /// The components of the message body, parsed by [`TypedStreamReader`]
    pub components: Option<Vec<Archivable>>,
    /// The components of the message that may or may not have been edited or unsent
    pub edited_parts: Option<EditedMessage>,
}

impl Table for Message {
    fn from_row(row: &Row) -> Result<Message> {
        Ok(Message {
            rowid: row.get("rowid")?,
            guid: row.get("guid")?,
            text: row.get("text").unwrap_or(None),
            service: row.get("service").unwrap_or(None),
            handle_id: row.get("handle_id").unwrap_or(None),
            destination_caller_id: row.get("destination_caller_id").unwrap_or(None),
            subject: row.get("subject").unwrap_or(None),
            date: row.get("date")?,
            date_read: row.get("date_read").unwrap_or(0),
            date_delivered: row.get("date_delivered").unwrap_or(0),
            is_from_me: row.get("is_from_me")?,
            is_read: row.get("is_read").unwrap_or(false),
            item_type: row.get("item_type").unwrap_or_default(),
            other_handle: row.get("other_handle").unwrap_or(None),
            share_status: row.get("share_status").unwrap_or(false),
            share_direction: row.get("share_direction").unwrap_or(None),
            group_title: row.get("group_title").unwrap_or(None),
            group_action_type: row.get("group_action_type").unwrap_or(0),
            associated_message_guid: row.get("associated_message_guid").unwrap_or(None),
            associated_message_type: row.get("associated_message_type").unwrap_or(None),
            balloon_bundle_id: row.get("balloon_bundle_id").unwrap_or(None),
            expressive_send_style_id: row.get("expressive_send_style_id").unwrap_or(None),
            thread_originator_guid: row.get("thread_originator_guid").unwrap_or(None),
            thread_originator_part: row.get("thread_originator_part").unwrap_or(None),
            date_edited: row.get("date_edited").unwrap_or(0),
            associated_message_emoji: row.get("associated_message_emoji").unwrap_or(None),
            chat_id: row.get("chat_id").unwrap_or(None),
            num_attachments: row.get("num_attachments")?,
            deleted_from: row.get("deleted_from").unwrap_or(None),
            num_replies: row.get("num_replies")?,
            components: None,
            edited_parts: None,
        })
    }

    /// Convert data from the messages table to native Rust data structures, falling back to
    /// more compatible queries to ensure compatibility with older database schemas
    fn get(db: &Connection) -> Result<Statement, TableError> {
        db.prepare(&ios_16_newer_query(None, None))
            .or_else(|_| db.prepare(&ios_14_15_query(None, None)))
            .or_else(|_| db.prepare(&ios_13_older_query(None, None)))
            .map_err(TableError::Messages)
    }

    fn extract(message: Result<Result<Self, Error>, Error>) -> Result<Self, TableError> {
        match message {
            Ok(Ok(message)) => Ok(message),
            Err(why) | Ok(Err(why)) => Err(TableError::Messages(why)),
        }
    }
}

impl Diagnostic for Message {
    /// Emit diagnostic data for the Messages table
    ///
    /// # Example:
    ///
    /// ```
    /// use imessage_database::util::dirs::default_db_path;
    /// use imessage_database::tables::table::{Diagnostic, get_connection};
    /// use imessage_database::tables::messages::Message;
    ///
    /// let db_path = default_db_path();
    /// let conn = get_connection(&db_path).unwrap();
    /// Message::run_diagnostic(&conn);
    /// ```
    fn run_diagnostic(db: &Connection) -> Result<(), TableError> {
        processing();
        let mut messages_without_chat = db
            .prepare(&format!(
                "
            SELECT
                COUNT(m.rowid)
            FROM
            {MESSAGE} as m
            LEFT JOIN {CHAT_MESSAGE_JOIN} as c ON m.rowid = c.message_id
            WHERE
                c.chat_id is NULL
            ORDER BY
                m.date
            "
            ))
            .map_err(TableError::Messages)?;

        let num_dangling: i32 = messages_without_chat
            .query_row([], |r| r.get(0))
            .unwrap_or(0);

        let mut messages_in_more_than_one_chat_q = db
            .prepare(&format!(
                "
            SELECT
                COUNT(*)
            FROM (
            SELECT DISTINCT
                message_id
              , COUNT(chat_id) AS c
            FROM {CHAT_MESSAGE_JOIN}
            GROUP BY
                message_id
            HAVING c > 1);
            "
            ))
            .map_err(TableError::Messages)?;

        let messages_in_more_than_one_chat: i32 = messages_in_more_than_one_chat_q
            .query_row([], |r| r.get(0))
            .unwrap_or(0);

        let mut messages_count = db
            .prepare(&format!(
                "
            SELECT
                COUNT(rowid)
            FROM
                {MESSAGE}
            "
            ))
            .map_err(TableError::Messages)?;

        let total_messages: i64 = messages_count.query_row([], |r| r.get(0)).unwrap_or(0);

        done_processing();

        println!("Message diagnostic data:");
        println!("    Total messages: {total_messages}");
        if num_dangling > 0 {
            println!("    Messages not associated with a chat: {num_dangling}");
        }
        if messages_in_more_than_one_chat > 0 {
            println!(
                "    Messages belonging to more than one chat: {messages_in_more_than_one_chat}"
            );
        }
        Ok(())
    }
}

impl Cacheable for Message {
    type K = String;
    type V = HashMap<usize, Vec<Self>>;
    /// Used for tapbacks that do not exist in a foreign key table
    ///
    /// Builds a map like:
    ///
    /// ```json
    /// {
    ///     "message_guid": {
    ///         0: [Message, Message],
    ///         1: [Message]
    ///     }
    /// }
    /// ```
    ///
    /// Where the `0` and `1` are the tapback indexes in the body of the message mapped by `message_guid`
    fn cache(db: &Connection) -> Result<HashMap<Self::K, Self::V>, TableError> {
        // Create cache for user IDs
        let mut map: HashMap<Self::K, Self::V> = HashMap::new();

        // Create query
        let statement = db.prepare(&format!(
            "SELECT
                 {COLS},
                 c.chat_id,
                 (SELECT COUNT(*) FROM {MESSAGE_ATTACHMENT_JOIN} a WHERE m.ROWID = a.message_id) as num_attachments,
                 NULL as deleted_from,
                 0 as num_replies
             FROM
                 {MESSAGE} as m
             LEFT JOIN {CHAT_MESSAGE_JOIN} as c ON m.ROWID = c.message_id
             WHERE m.associated_message_guid IS NOT NULL
            "
        )).or_else(|_| db.prepare(&format!(
            "SELECT
                 *,
                 c.chat_id,
                 (SELECT COUNT(*) FROM {MESSAGE_ATTACHMENT_JOIN} a WHERE m.ROWID = a.message_id) as num_attachments,
                 NULL as deleted_from,
                 0 as num_replies
             FROM
                 {MESSAGE} as m
             LEFT JOIN {CHAT_MESSAGE_JOIN} as c ON m.ROWID = c.message_id
             WHERE m.associated_message_guid IS NOT NULL
            "
        )));

        if let Ok(mut statement) = statement {
            // Execute query to build the message tapback map
            let messages = statement
                .query_map([], |row| Ok(Message::from_row(row)))
                .map_err(TableError::Messages)?;

            // Iterate over the messages and update the map
            for message in messages {
                let message = Self::extract(message)?;
                if message.is_tapback() {
                    if let Some((idx, tapback_target_guid)) = message.clean_associated_guid() {
                        map.entry(tapback_target_guid.to_string())
                            .or_insert_with(HashMap::new)
                            .entry(idx)
                            .or_insert_with(Vec::new)
                            .push(message);
                    }
                }
            }
        }

        Ok(map)
    }
}

impl GetBlob for Message {
    /// Extract a blob of data that belongs to a single message from a given column
    fn get_blob<'a>(&self, db: &'a Connection, column: &str) -> Option<Blob<'a>> {
        db.blob_open(
            rusqlite::MAIN_DB,
            MESSAGE,
            column,
            i64::from(self.rowid),
            true,
        )
        .ok()
    }
}

impl AttributedBody for Message {
    /// Get a vector of a message body's components. If the text has not been captured with [`Self::generate_text()`], the vector will be empty.
    ///
    /// For more detail see the trait documentation [here](crate::tables::table::AttributedBody).
    fn body(&self) -> Vec<BubbleComponent> {
        // If the message is an app, it will be rendered differently, so just escape there
        if self.balloon_bundle_id.is_some() {
            return vec![BubbleComponent::App];
        }

        if let Some(body) = parse_body_typedstream(
            self.components.as_ref(),
            self.text.as_deref(),
            self.edited_parts.as_ref(),
        ) {
            return body;
        }

        // Naive logic for when `typedstream` component parsing fails
        parse_body_legacy(&self.text)
    }
}

impl Message {
    /// Generate the text of a message, deserializing it as [`typedstream`](crate::util::typedstream) (and falling back to [`streamtyped`]) data if necessary.
    pub fn generate_text<'a>(&'a mut self, db: &'a Connection) -> Result<&'a str, MessageError> {
        // Grab the body data from the table
        if let Some(body) = self.attributed_body(db) {
            // Attempt to deserialize the typedstream data
            let mut typedstream = TypedStreamReader::from(&body);
            self.components = typedstream.parse().ok();

            // If we deserialize the typedstream, use that data
            self.text = self
                .components
                .as_ref()
                .and_then(|items| items.first())
                .and_then(|item| item.as_nsstring())
                .map(String::from);

            // If the above parsing failed, fall back to the legacy parser instead
            if self.text.is_none() {
                self.text =
                    Some(streamtyped::parse(body).map_err(MessageError::StreamTypedParseError)?);
            }
        }

        // Generate the edited message data
        self.edited_parts = self
            .is_edited()
            .then(|| self.message_summary_info(db))
            .flatten()
            .as_ref()
            .and_then(|payload| EditedMessage::from_map(payload).ok());

        if let Some(t) = &self.text {
            Ok(t)
        } else {
            Err(MessageError::NoText)
        }
    }

    /// Calculates the date a message was written to the database.
    ///
    /// This field is stored as a unix timestamp with an epoch of `2001-01-01 00:00:00` in the local time zone
    ///
    /// `offset` can be provided by [`get_offset`](crate::util::dates::get_offset) or manually.
    pub fn date(&self, offset: &i64) -> Result<DateTime<Local>, MessageError> {
        get_local_time(&self.date, offset)
    }

    /// Calculates the date a message was marked as delivered.
    ///
    /// This field is stored as a unix timestamp with an epoch of `2001-01-01 00:00:00` in the local time zone
    ///
    /// `offset` can be provided by [`get_offset`](crate::util::dates::get_offset) or manually.
    pub fn date_delivered(&self, offset: &i64) -> Result<DateTime<Local>, MessageError> {
        get_local_time(&self.date_delivered, offset)
    }

    /// Calculates the date a message was marked as read.
    ///
    /// This field is stored as a unix timestamp with an epoch of `2001-01-01 00:00:00` in the local time zone
    ///
    /// `offset` can be provided by [`get_offset`](crate::util::dates::get_offset) or manually.
    pub fn date_read(&self, offset: &i64) -> Result<DateTime<Local>, MessageError> {
        get_local_time(&self.date_read, offset)
    }

    /// Calculates the date a message was most recently edited.
    ///
    /// This field is stored as a unix timestamp with an epoch of `2001-01-01 00:00:00` in the local time zone
    ///
    /// `offset` can be provided by [`get_offset`](crate::util::dates::get_offset) or manually.
    pub fn date_edited(&self, offset: &i64) -> Result<DateTime<Local>, MessageError> {
        get_local_time(&self.date_edited, offset)
    }

    /// Gets the time until the message was read. This can happen in two ways:
    ///
    /// - You received a message, then waited to read it
    /// - You sent a message, and the recipient waited to read it
    ///
    /// In the former case, this subtracts the date read column (`date_read`) from the date received column (`date`).
    /// In the latter case, this subtracts the date delivered column (`date_delivered`) from the date received column (`date`).
    ///
    /// Not all messages get tagged with the read properties.
    /// If more than one message has been sent in a thread before getting read,
    /// only the most recent message will get the tag.
    ///
    /// `offset` can be provided by [`get_offset`](crate::util::dates::get_offset) or manually.
    #[must_use]
    pub fn time_until_read(&self, offset: &i64) -> Option<String> {
        // Message we received
        if !self.is_from_me && self.date_read != 0 && self.date != 0 {
            return readable_diff(self.date(offset), self.date_read(offset));
        }
        // Message we sent
        else if self.is_from_me && self.date_delivered != 0 && self.date != 0 {
            return readable_diff(self.date(offset), self.date_delivered(offset));
        }
        None
    }

    /// `true` if the message is a response to a thread, else `false`
    #[must_use]
    pub fn is_reply(&self) -> bool {
        self.thread_originator_guid.is_some()
    }

    /// `true` if the message is an [`Announcement`], else `false`
    #[must_use]
    pub fn is_announcement(&self) -> bool {
        self.get_announcement().is_some()
    }

    /// `true` if the message is a [`Tapback`] to another message, else `false`
    #[must_use]
    pub fn is_tapback(&self) -> bool {
        matches!(self.variant(), Variant::Tapback(..))
    }

    /// `true` if the message has an [`Expressive`], else `false`
    #[must_use]
    pub fn is_expressive(&self) -> bool {
        self.expressive_send_style_id.is_some()
    }

    /// `true` if the message has a [URL preview](crate::message_types::url), else `false`
    #[must_use]
    pub fn is_url(&self) -> bool {
        matches!(self.variant(), Variant::App(CustomBalloon::URL))
    }

    /// `true` if the message is a [`HandwrittenMessage`](crate::message_types::handwriting::models::HandwrittenMessage), else `false`
    #[must_use]
    pub fn is_handwriting(&self) -> bool {
        matches!(self.variant(), Variant::App(CustomBalloon::Handwriting))
    }

    /// `true` if the message is a [`Digital Touch`](crate::message_types::digital_touch::models), else `false`
    #[must_use]
    pub fn is_digital_touch(&self) -> bool {
        matches!(self.variant(), Variant::App(CustomBalloon::DigitalTouch))
    }

    /// `true` if the message was [`Edited`](crate::message_types::edited), else `false`
    #[must_use]
    pub fn is_edited(&self) -> bool {
        self.date_edited != 0
    }

    /// `true` if the specified message component was [edited](crate::message_types::edited::EditStatus::Edited), else `false`
    #[must_use]
    pub fn is_part_edited(&self, index: usize) -> bool {
        if let Some(edited_parts) = &self.edited_parts {
            if let Some(part) = edited_parts.part(index) {
                return matches!(part.status, EditStatus::Edited);
            }
        }
        false
    }

    /// `true` if all message components were [unsent](crate::message_types::edited::EditStatus::Unsent), else `false`
    #[must_use]
    pub fn is_fully_unsent(&self) -> bool {
        self.edited_parts.as_ref().is_some_and(|ep| {
            ep.parts
                .iter()
                .all(|part| matches!(part.status, EditStatus::Unsent))
        })
    }

    /// `true` if the message contains [`Attachment`](crate::tables::attachment::Attachment)s, else `false`
    ///
    /// Attachments can be queried with [`Attachment::from_message()`](crate::tables::attachment::Attachment::from_message).
    #[must_use]
    pub fn has_attachments(&self) -> bool {
        self.num_attachments > 0
    }

    /// `true` if the message begins a thread, else `false`
    #[must_use]
    pub fn has_replies(&self) -> bool {
        self.num_replies > 0
    }

    /// `true` if the message indicates a sent audio message was kept, else `false`
    #[must_use]
    pub fn is_kept_audio_message(&self) -> bool {
        self.item_type == 5
    }

    /// `true` if the message is a [SharePlay/FaceTime](crate::message_types::variants::Variant::SharePlay) message, else `false`
    #[must_use]
    pub fn is_shareplay(&self) -> bool {
        self.item_type == 6
    }

    /// `true` if the message was sent by the database owner, else `false`
    #[must_use]
    pub fn is_from_me(&self) -> bool {
        if let (Some(other_handle), Some(share_direction)) =
            (self.other_handle, self.share_direction)
        {
            self.is_from_me || other_handle != 0 && !share_direction
        } else {
            self.is_from_me
        }
    }

    /// Get the group action for the current message
    #[must_use]
    pub fn group_action(&self) -> Option<GroupAction> {
        GroupAction::from_message(self)
    }

    /// `true` if the message indicates a sender started sharing their location, else `false`
    #[must_use]
    pub fn started_sharing_location(&self) -> bool {
        self.item_type == 4 && self.group_action_type == 0 && !self.share_status
    }

    /// `true` if the message indicates a sender stopped sharing their location, else `false`
    #[must_use]
    pub fn stopped_sharing_location(&self) -> bool {
        self.item_type == 4 && self.group_action_type == 0 && self.share_status
    }

    /// `true` if the message was deleted and is recoverable, else `false`
    ///
    /// Messages removed by deleting an entire conversation or by deleting a single message
    /// from a conversation are moved to a separate collection for up to 30 days. Messages
    /// present in this collection are restored to the conversations they belong to. Apple
    /// details this process [here](https://support.apple.com/en-us/HT202549#delete).
    ///
    /// Messages that have expired from this restoration process are permanently deleted and
    /// cannot be recovered.
    ///
    /// Note: This is not the same as an [`Unsent`](crate::message_types::edited::EditStatus::Unsent) message.
    #[must_use]
    pub fn is_deleted(&self) -> bool {
        self.deleted_from.is_some()
    }

    /// Get the index of the part of a message a reply is pointing to
    fn get_reply_index(&self) -> usize {
        if let Some(parts) = &self.thread_originator_part {
            return match parts.split(':').next() {
                Some(part) => str::parse::<usize>(part).unwrap_or(0),
                None => 0,
            };
        }
        0
    }

    /// Generate the SQL `WHERE` clause described by a [`QueryContext`].
    ///
    /// If `include_recoverable` is `true`, the filter includes messages from the recently deleted messages
    /// table that match the chat IDs. This allows recovery of deleted messages that are still
    /// present in the database but no longer visible in the Messages app.
    pub(crate) fn generate_filter_statement(
        context: &QueryContext,
        include_recoverable: bool,
    ) -> String {
        let mut filters = String::new();

        // Chat ID filter, optionally including recoverable messages
        if let Some(chat_ids) = &context.selected_chat_ids {
            // Allocate the filter string for interpolation
            let ids = chat_ids
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<String>>()
                .join(", ");

            if include_recoverable {
                filters.push_str(&format!(
                    "WHERE (c.chat_id IN ({ids}) OR d.chat_id IN ({ids}))"
                ));
            } else {
                filters.push_str(&format!("WHERE c.chat_id IN ({ids})"));
            }
        }

        filters
    }

    pub(crate) fn generate_limit_statement(context: &QueryContext) -> String {
        let mut limit_statement = String::new();
        // limit filter
        if let Some(limit) = context.limit {
            limit_statement.push_str(&format!("LIMIT {limit}"));
        }

        limit_statement
    }

    /// Get the number of messages in the database
    ///
    /// # Example:
    ///
    /// ```
    /// use imessage_database::util::dirs::default_db_path;
    /// use imessage_database::tables::table::{Diagnostic, get_connection};
    /// use imessage_database::tables::messages::Message;
    /// use imessage_database::util::query_context::QueryContext;
    ///
    /// let db_path = default_db_path();
    /// let conn = get_connection(&db_path).unwrap();
    /// let context = QueryContext::default();
    /// Message::get_count(&conn, &context);
    /// ```
    pub fn get_count(db: &Connection, context: &QueryContext) -> Result<u64, TableError> {
        let mut statement = if context.has_filters() {
            db.prepare(&format!(
                "SELECT
                     COUNT(*)
                 FROM {MESSAGE} as m
                 LEFT JOIN {CHAT_MESSAGE_JOIN} as c ON m.ROWID = c.message_id
                 LEFT JOIN {RECENTLY_DELETED} as d ON m.ROWID = d.message_id
                 {}",
                Self::generate_filter_statement(context, true)
            ))
            .or_else(|_| {
                db.prepare(&format!(
                    "SELECT
                         COUNT(*)
                     FROM {MESSAGE} as m
                     LEFT JOIN {CHAT_MESSAGE_JOIN} as c ON m.ROWID = c.message_id
                    {}",
                    Self::generate_filter_statement(context, false)
                ))
            })
            .map_err(TableError::Messages)?
        } else {
            db.prepare(&format!("SELECT COUNT(*) FROM {MESSAGE}"))
                .map_err(TableError::Messages)?
        };
        // Execute query, defaulting to zero if it fails
        let count: u64 = statement.query_row([], |r| r.get(0)).unwrap_or(0);

        Ok(count)
    }

    /// Stream messages from the database with optional filters.
    ///
    /// # Example:
    ///
    /// ```
    /// use imessage_database::util::dirs::default_db_path;
    /// use imessage_database::tables::table::{Diagnostic, get_connection};
    /// use imessage_database::tables::{messages::Message, table::Table};
    /// use imessage_database::util::query_context::QueryContext;
    ///
    /// let db_path = default_db_path();
    /// let conn = get_connection(&db_path).unwrap();
    /// let context = QueryContext::default();
    ///
    /// let mut statement = Message::stream_rows(&conn, &context).unwrap();
    ///
    /// let messages = statement.query_map([], |row| Ok(Message::from_row(row))).unwrap();
    ///
    /// messages.map(|msg| println!("{:#?}", Message::extract(msg)));
    /// ```
    pub fn stream_rows<'a>(
        db: &'a Connection,
        context: &'a QueryContext,
    ) -> Result<Statement<'a>, TableError> {
        if !context.has_filters() {
            return Self::get(db);
        }
        db.prepare(&ios_16_newer_query(
            Some(&Self::generate_filter_statement(context, true)),
            Some(&Self::generate_limit_statement(context)),
        ))
        .or_else(|_| {
            db.prepare(&ios_14_15_query(
                Some(&Self::generate_filter_statement(context, false)),
                Some(&Self::generate_limit_statement(context)),
            ))
        })
        .or_else(|_| {
            db.prepare(&ios_13_older_query(
                Some(&Self::generate_filter_statement(context, false)),
                Some(&Self::generate_limit_statement(context)),
            ))
        })
        .map_err(TableError::Messages)
    }

    /// See [`Tapback`] for details on this data.
    #[must_use]
    pub fn clean_associated_guid(&self) -> Option<(usize, &str)> {
        if let Some(guid) = &self.associated_message_guid {
            if guid.starts_with("p:") {
                let mut split = guid.split('/');
                let index_str = split.next()?;
                let message_id = split.next()?;
                let index = str::parse::<usize>(&index_str.replace("p:", "")).unwrap_or(0);
                return Some((index, message_id.get(0..36)?));
            } else if guid.starts_with("bp:") {
                return Some((0, guid.get(3..39)?));
            }

            return Some((0, guid.get(0..36)?));
        }
        None
    }

    /// Parse the index of a tapback from it's associated GUID field
    fn tapback_index(&self) -> usize {
        match self.clean_associated_guid() {
            Some((x, _)) => x,
            None => 0,
        }
    }

    /// Build a `HashMap` of message component index to messages that reply to that component
    pub fn get_replies(&self, db: &Connection) -> Result<HashMap<usize, Vec<Self>>, TableError> {
        let mut out_h: HashMap<usize, Vec<Self>> = HashMap::new();

        // No need to hit the DB if we know we don't have replies
        if self.has_replies() {
            let filters = format!("WHERE m.thread_originator_guid = \"{}\"", self.guid);

            // No iOS 13 and prior used here because `thread_originator_guid` is not present in that schema
            let mut statement = db
                .prepare(&ios_16_newer_query(Some(&filters), None))
                .or_else(|_| db.prepare(&ios_14_15_query(Some(&filters), None)))
                .map_err(TableError::Messages)?;

            let iter = statement
                .query_map([], |row| Ok(Message::from_row(row)))
                .map_err(TableError::Messages)?;

            for message in iter {
                let m = Message::extract(message)?;
                let idx = m.get_reply_index();
                match out_h.get_mut(&idx) {
                    Some(body_part) => body_part.push(m),
                    None => {
                        out_h.insert(idx, vec![m]);
                    }
                }
            }
        }

        Ok(out_h)
    }

    /// Get the variant of a message, see [`variants`](crate::message_types::variants) for detail.
    #[must_use]
    pub fn variant(&self) -> Variant {
        // Check if a message was edited first as those have special properties
        if self.is_edited() {
            return Variant::Edited;
        }

        // Handle different types of bundle IDs next, as those are most common
        if let Some(associated_message_type) = self.associated_message_type {
            return match associated_message_type {
                // Standard iMessages with either text or a message payload
                0 | 2 | 3 => match parse_balloon_bundle_id(self.balloon_bundle_id.as_deref()) {
                    Some(bundle_id) => match bundle_id {
                        "com.apple.messages.URLBalloonProvider" => Variant::App(CustomBalloon::URL),
                        "com.apple.Handwriting.HandwritingProvider" => {
                            Variant::App(CustomBalloon::Handwriting)
                        }
                        "com.apple.DigitalTouchBalloonProvider" => {
                            Variant::App(CustomBalloon::DigitalTouch)
                        }
                        "com.apple.PassbookUIService.PeerPaymentMessagesExtension" => {
                            Variant::App(CustomBalloon::ApplePay)
                        }
                        "com.apple.ActivityMessagesApp.MessagesExtension" => {
                            Variant::App(CustomBalloon::Fitness)
                        }
                        "com.apple.mobileslideshow.PhotosMessagesApp" => {
                            Variant::App(CustomBalloon::Slideshow)
                        }
                        "com.apple.SafetyMonitorApp.SafetyMonitorMessages" => {
                            Variant::App(CustomBalloon::CheckIn)
                        }
                        "com.apple.findmy.FindMyMessagesApp" => Variant::App(CustomBalloon::FindMy),
                        _ => Variant::App(CustomBalloon::Application(bundle_id)),
                    },
                    // This is the most common case
                    None => Variant::Normal,
                },

                // Stickers overlaid on messages
                1000 => {
                    Variant::Tapback(self.tapback_index(), TapbackAction::Added, Tapback::Sticker)
                }

                // Tapbacks
                2000 => {
                    Variant::Tapback(self.tapback_index(), TapbackAction::Added, Tapback::Loved)
                }
                2001 => {
                    Variant::Tapback(self.tapback_index(), TapbackAction::Added, Tapback::Liked)
                }
                2002 => Variant::Tapback(
                    self.tapback_index(),
                    TapbackAction::Added,
                    Tapback::Disliked,
                ),
                2003 => {
                    Variant::Tapback(self.tapback_index(), TapbackAction::Added, Tapback::Laughed)
                }
                2004 => Variant::Tapback(
                    self.tapback_index(),
                    TapbackAction::Added,
                    Tapback::Emphasized,
                ),
                2005 => Variant::Tapback(
                    self.tapback_index(),
                    TapbackAction::Added,
                    Tapback::Questioned,
                ),
                2006 => Variant::Tapback(
                    self.tapback_index(),
                    TapbackAction::Added,
                    Tapback::Emoji(self.associated_message_emoji.as_deref()),
                ),
                2007 => {
                    Variant::Tapback(self.tapback_index(), TapbackAction::Added, Tapback::Sticker)
                }
                3000 => {
                    Variant::Tapback(self.tapback_index(), TapbackAction::Removed, Tapback::Loved)
                }
                3001 => {
                    Variant::Tapback(self.tapback_index(), TapbackAction::Removed, Tapback::Liked)
                }
                3002 => Variant::Tapback(
                    self.tapback_index(),
                    TapbackAction::Removed,
                    Tapback::Disliked,
                ),
                3003 => Variant::Tapback(
                    self.tapback_index(),
                    TapbackAction::Removed,
                    Tapback::Laughed,
                ),
                3004 => Variant::Tapback(
                    self.tapback_index(),
                    TapbackAction::Removed,
                    Tapback::Emphasized,
                ),
                3005 => Variant::Tapback(
                    self.tapback_index(),
                    TapbackAction::Removed,
                    Tapback::Questioned,
                ),
                3006 => Variant::Tapback(
                    self.tapback_index(),
                    TapbackAction::Removed,
                    Tapback::Emoji(self.associated_message_emoji.as_deref()),
                ),
                3007 => Variant::Tapback(
                    self.tapback_index(),
                    TapbackAction::Removed,
                    Tapback::Sticker,
                ),

                // Unknown
                x => Variant::Unknown(x),
            };
        }

        // Any other rarer cases belong here
        if self.is_shareplay() {
            return Variant::SharePlay;
        }

        Variant::Normal
    }

    /// Determine the type of announcement a message contains, if it contains one
    #[must_use]
    pub fn get_announcement(&self) -> Option<Announcement> {
        if let Some(action) = self.group_action() {
            return Some(Announcement::GroupAction(action));
        }

        if self.is_fully_unsent() {
            return Some(Announcement::FullyUnsent);
        }

        if self.is_kept_audio_message() {
            return Some(Announcement::AudioMessageKept);
        }

        None
    }

    /// Determine the service the message was sent from, i.e. iMessage, SMS, IRC, etc.
    #[must_use]
    pub fn service(&self) -> Service {
        Service::from(self.service.as_deref())
    }

    /// Get a message's plist from the [`MESSAGE_PAYLOAD`] BLOB column
    ///
    /// Calling this hits the database, so it is expensive and should
    /// only get invoked when needed.
    ///
    /// This column contains data used by iMessage app balloons and can be parsed with
    /// [`parse_ns_keyed_archiver()`](crate::util::plist::parse_ns_keyed_archiver).
    pub fn payload_data(&self, db: &Connection) -> Option<Value> {
        Value::from_reader(self.get_blob(db, MESSAGE_PAYLOAD)?).ok()
    }

    /// Get a message's raw data from the [`MESSAGE_PAYLOAD`] BLOB column
    ///
    /// Calling this hits the database, so it is expensive and should
    /// only get invoked when needed.
    ///
    /// This column contains data used by [`HandwrittenMessage`](crate::message_types::handwriting::HandwrittenMessage)s.
    pub fn raw_payload_data(&self, db: &Connection) -> Option<Vec<u8>> {
        let mut buf = Vec::new();
        self.get_blob(db, MESSAGE_PAYLOAD)?
            .read_to_end(&mut buf)
            .ok()?;
        Some(buf)
    }

    /// Get a message's plist from the [`MESSAGE_SUMMARY_INFO`] BLOB column
    ///
    /// Calling this hits the database, so it is expensive and should
    /// only get invoked when needed.
    ///
    /// This column contains data used by [`edited`](crate::message_types::edited) iMessages.
    pub fn message_summary_info(&self, db: &Connection) -> Option<Value> {
        Value::from_reader(self.get_blob(db, MESSAGE_SUMMARY_INFO)?).ok()
    }

    /// Get a message's [typedstream](crate::util::typedstream) from the [`ATTRIBUTED_BODY`] BLOB column
    ///
    /// Calling this hits the database, so it is expensive and should
    /// only get invoked when needed.
    ///
    /// This column contains the message's body text with any other attributes.
    pub fn attributed_body(&self, db: &Connection) -> Option<Vec<u8>> {
        let mut body = vec![];
        self.get_blob(db, ATTRIBUTED_BODY)?
            .read_to_end(&mut body)
            .ok();
        Some(body)
    }

    /// Determine which [`Expressive`] the message was sent with
    #[must_use]
    pub fn get_expressive(&self) -> Expressive {
        match &self.expressive_send_style_id {
            Some(content) => match content.as_str() {
                "com.apple.MobileSMS.expressivesend.gentle" => {
                    Expressive::Bubble(BubbleEffect::Gentle)
                }
                "com.apple.MobileSMS.expressivesend.impact" => {
                    Expressive::Bubble(BubbleEffect::Slam)
                }
                "com.apple.MobileSMS.expressivesend.invisibleink" => {
                    Expressive::Bubble(BubbleEffect::InvisibleInk)
                }
                "com.apple.MobileSMS.expressivesend.loud" => Expressive::Bubble(BubbleEffect::Loud),
                "com.apple.messages.effect.CKConfettiEffect" => {
                    Expressive::Screen(ScreenEffect::Confetti)
                }
                "com.apple.messages.effect.CKEchoEffect" => Expressive::Screen(ScreenEffect::Echo),
                "com.apple.messages.effect.CKFireworksEffect" => {
                    Expressive::Screen(ScreenEffect::Fireworks)
                }
                "com.apple.messages.effect.CKHappyBirthdayEffect" => {
                    Expressive::Screen(ScreenEffect::Balloons)
                }
                "com.apple.messages.effect.CKHeartEffect" => {
                    Expressive::Screen(ScreenEffect::Heart)
                }
                "com.apple.messages.effect.CKLasersEffect" => {
                    Expressive::Screen(ScreenEffect::Lasers)
                }
                "com.apple.messages.effect.CKShootingStarEffect" => {
                    Expressive::Screen(ScreenEffect::ShootingStar)
                }
                "com.apple.messages.effect.CKSparklesEffect" => {
                    Expressive::Screen(ScreenEffect::Sparkles)
                }
                "com.apple.messages.effect.CKSpotlightEffect" => {
                    Expressive::Screen(ScreenEffect::Spotlight)
                }
                _ => Expressive::Unknown(content),
            },
            None => Expressive::None,
        }
    }

    /// Create a message from a given GUID; useful for debugging
    ///
    /// # Example
    /// ```rust
    /// use imessage_database::{
    ///     tables::{
    ///         messages::Message,
    ///         table::get_connection,
    ///     },
    ///     util::dirs::default_db_path,
    /// };
    ///
    /// let db_path = default_db_path();
    /// let conn = get_connection(&db_path).unwrap();
    ///
    /// if let Ok(mut message) = Message::from_guid("example-guid", &conn) {
    ///     let _ = message.generate_text(&conn);
    ///     println!("{:#?}", message)
    /// }
    ///```
    pub fn from_guid(guid: &str, db: &Connection) -> Result<Self, TableError> {
        // If the database has `chat_recoverable_message_join`, we can restore some deleted messages.
        // If database has `thread_originator_guid`, we can parse replies, otherwise default to 0
        let filters = format!("WHERE m.guid = \"{guid}\"");

        let mut statement = db
            .prepare(&ios_16_newer_query(Some(&filters), None))
            .or_else(|_| db.prepare(&ios_14_15_query(Some(&filters), None)))
            .or_else(|_| db.prepare(&ios_13_older_query(Some(&filters), None)))
            .map_err(TableError::Messages)?;

        Message::extract(statement.query_row([], |row| Ok(Message::from_row(row))))
    }
}

#[cfg(test)]
impl Message {
    #[must_use]
    pub fn blank() -> Message {
        Message {
            rowid: i32::default(),
            guid: String::default(),
            text: None,
            service: Some("iMessage".to_string()),
            handle_id: Some(i32::default()),
            destination_caller_id: None,
            subject: None,
            date: i64::default(),
            date_read: i64::default(),
            date_delivered: i64::default(),
            is_from_me: false,
            is_read: false,
            item_type: 0,
            other_handle: None,
            share_status: false,
            share_direction: None,
            group_title: None,
            group_action_type: 0,
            associated_message_guid: None,
            associated_message_type: None,
            balloon_bundle_id: None,
            expressive_send_style_id: None,
            thread_originator_guid: None,
            thread_originator_part: None,
            date_edited: 0,
            associated_message_emoji: None,
            chat_id: None,
            num_attachments: 0,
            deleted_from: None,
            num_replies: 0,
            components: None,
            edited_parts: None,
        }
    }
}
