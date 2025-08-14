use std::{fs::File, io::BufWriter};

use imessage_database::{
    error::{plist::PlistParseError, table::TableError},
    message_types::edited::EditedMessage,
    tables::{
        attachment::Attachment,
        messages::{
            Message,
            models::{AttachmentMeta, TextAttributes},
        },
    },
};

use crate::app::{error::RuntimeError};

pub(crate) const ATTACHMENT_NO_FILENAME: &str = "Attachment missing name metadata!";

/// Defines behavior for formatting message instances to the desired output format
pub trait Writer<'a> {
    /// Format a message, including its tapbacks and replies
    fn format_message(&self, msg: &Message, indent: usize) -> Result<String, TableError>;
    /// Format an attachment, possibly by reading the disk
    fn format_attachment(
        &self,
        attachment: &'a mut Attachment,
        msg: &'a Message,
        metadata: &AttachmentMeta,
    ) -> Result<String, &'a str>;
    /// Format a sticker, possibly by reading the disk
    fn format_sticker(&self, attachment: &'a mut Attachment, msg: &'a Message) -> String;
    /// Format an app message by parsing some of its fields
    fn format_app(
        &self,
        msg: &'a Message,
        attachments: &mut Vec<Attachment>,
        indent: &str,
    ) -> Result<String, PlistParseError>;
    /// Format a tapback (displayed under a message)
    fn format_tapback(&self, msg: &Message) -> Result<String, TableError>;
    /// Format an expressive message
    fn format_expressive(&self, msg: &'a Message) -> &'a str;
    /// Format an announcement message
    fn format_announcement(&self, msg: &'a Message) -> String;
    /// Format a `SharePlay` message
    fn format_shareplay(&self) -> &str;
    /// Format a legacy Shared Location message
    fn format_shared_location(&self, msg: &'a Message) -> &str;
    /// Format an edited message
    fn format_edited(
        &self,
        msg: &'a Message,
        edited_message: &'a EditedMessage,
        message_part_idx: usize,
        indent: &str,
    ) -> Option<String>;
    /// Format all [`TextAttributes`]s applied to a given set of text
    fn format_attributes(&'a self, text: &'a str, attributes: &'a [TextAttributes]) -> String;
    fn write_to_file(file: &mut BufWriter<File>, text: &str) -> Result<(), RuntimeError>;
}