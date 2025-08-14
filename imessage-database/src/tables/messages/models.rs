/*!
 This module contains Data structures and models that represent message data.
*/

use std::fmt::{Display, Formatter, Result};

use crate::{
    message_types::text_effects::TextEffect, tables::messages::message::Message,
    util::typedstream::models::Archivable,
};

/// Defines the parts of a message bubble, i.e. the content that can exist in a single message.
///
/// # Component Types
///
/// A single iMessage contains data that may be represented across multiple bubbles.
///
/// iMessage bubbles can only contain data of one variant of this enum at a time.
#[derive(Debug, PartialEq)]
pub enum BubbleComponent<'a> {
    /// A text message with associated formatting, generally representing ranges present in a `NSAttributedString`
    Text(Vec<TextAttributes<'a>>),
    /// An attachment
    Attachment(AttachmentMeta<'a>),
    /// An [app integration](crate::message_types::app)
    App,
    /// A component that was retracted, found by parsing the [`EditedMessage`](crate::message_types::edited::EditedMessage)
    Retracted,
}

/// Defines different types of [services](https://support.apple.com/en-us/104972) we can receive messages from.
#[derive(Debug)]
pub enum Service<'a> {
    /// An iMessage
    #[allow(non_camel_case_types)]
    iMessage,
    /// A message sent as SMS
    SMS,
    /// A message sent as RCS
    RCS,
    /// A message sent via [satellite](https://support.apple.com/en-us/120930)
    Satellite,
    /// Any other type of message
    Other(&'a str),
    /// Used when service field is not set
    Unknown,
}

impl<'a> Service<'a> {
    #[must_use]
    pub fn from(service: Option<&'a str>) -> Self {
        if let Some(service_name) = service {
            return match service_name.trim() {
                "iMessage" => Service::iMessage,
                "iMessageLite" => Service::Satellite,
                "SMS" => Service::SMS,
                "rcs" | "RCS" => Service::RCS,
                service_name => Service::Other(service_name),
            };
        }
        Service::Unknown
    }
}

impl Display for Service<'_> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        match self {
            Service::iMessage => write!(fmt, "iMessage"),
            Service::SMS => write!(fmt, "SMS"),
            Service::RCS => write!(fmt, "RCS"),
            Service::Satellite => write!(fmt, "Satellite"),
            Service::Other(other) => write!(fmt, "{other}"),
            Service::Unknown => write!(fmt, "Unknown"),
        }
    }
}

/// Defines ranges of text and associated attributes parsed from [`typedstream`](crate::util::typedstream) `attributedBody` data.
///
/// Ranges specify locations where attributes are applied to specific portions of a [`Message`]'s [`text`](crate::tables::messages::Message::text). For example, given message text with a [`Mention`](TextEffect::Mention) like:
///
/// ```
/// let message_text = "What's up, Christopher?";
/// ```
///
/// There will be 3 ranges:
///
/// ```
/// use imessage_database::message_types::text_effects::TextEffect;
/// use imessage_database::tables::messages::models::{TextAttributes, BubbleComponent};
///  
/// let result = vec![BubbleComponent::Text(vec![
///     TextAttributes::new(0, 11, TextEffect::Default),  // `What's up, `
///     TextAttributes::new(11, 22, TextEffect::Mention("+5558675309")), // `Christopher`
///     TextAttributes::new(22, 23, TextEffect::Default)  // `?`
/// ])];
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct TextAttributes<'a> {
    /// The start index of the affected range of message text
    pub start: usize,
    /// The end index of the affected range of message text
    pub end: usize,
    /// The effects applied to the specified range
    pub effect: TextEffect<'a>,
}

impl<'a> TextAttributes<'a> {
    #[must_use]
    pub fn new(start: usize, end: usize, effect: TextEffect<'a>) -> Self {
        Self { start, end, effect }
    }
}

/// Representation of attachment metadata used for rendering message body in a conversation feed.
#[derive(Debug, PartialEq, Default)]
pub struct AttachmentMeta<'a> {
    /// GUID of the attachment in the `attachment` table
    pub guid: Option<&'a str>,
    /// The transcription, if the attachment was an [audio message](https://support.apple.com/guide/iphone/send-and-receive-audio-messages-iph2e42d3117/ios) sent from or received on a [supported platform](https://www.apple.com/ios/feature-availability/#messages-audio-message-transcription).
    pub transcription: Option<&'a str>,
    /// The height of the attachment in points
    pub height: Option<&'a f64>,
    /// The width of the attachment in points
    pub width: Option<&'a f64>,
    /// The attachment's original filename
    pub name: Option<&'a str>,
}

impl<'a> AttachmentMeta<'a> {
    /// Given a slice of parsed [`typedstream`](crate::util::typedstream) data, populate the attachment's metadata fields.
    ///
    /// # Example
    /// ```
    /// use imessage_database::util::typedstream::models::{Archivable, Class, OutputData};
    /// use imessage_database::tables::messages::models::AttachmentMeta;
    ///
    /// // Sample components
    /// let components = vec![
    ///    Archivable::Object(
    ///        Class {
    ///            name: "NSString".to_string(),
    ///            version: 1,
    ///        },
    ///        vec![OutputData::String(
    ///            "__kIMFileTransferGUIDAttributeName".to_string(),
    ///        )],
    ///    ),
    ///    Archivable::Object(
    ///        Class {
    ///            name: "NSString".to_string(),
    ///            version: 1,
    ///        },
    ///        vec![OutputData::String(
    ///            "4C339597-EBBB-4978-9B87-521C0471A848".to_string(),
    ///        )],
    ///    ),
    /// ];
    /// let meta = AttachmentMeta::from_components(&components);
    /// ```
    #[must_use]
    pub fn from_components(components: &'a [Archivable]) -> Option<Self> {
        let mut guid = None;
        let mut transcription = None;
        let mut height = None;
        let mut width = None;
        let mut name = None;

        for (idx, key) in components.iter().enumerate() {
            if let Some(key_name) = key.as_nsstring() {
                match key_name {
                    "__kIMFileTransferGUIDAttributeName" => {
                        guid = components.get(idx + 1)?.as_nsstring();
                    }
                    "IMAudioTranscription" => {
                        transcription = components.get(idx + 1)?.as_nsstring();
                    }
                    "__kIMInlineMediaHeightAttributeName" => {
                        height = components.get(idx + 1)?.as_nsnumber_float();
                    }
                    "__kIMInlineMediaWidthAttributeName" => {
                        width = components.get(idx + 1)?.as_nsnumber_float();
                    }
                    "__kIMFilenameAttributeName" => name = components.get(idx + 1)?.as_nsstring(),
                    _ => {}
                }
            }
        }

        Some(Self {
            guid,
            transcription,
            height,
            width,
            name,
        })
    }
}

/// Represents different types of group message actions that can occur in a chat system
#[derive(Debug)]
pub enum GroupAction<'a> {
    /// A new participant has been added to the group
    ParticipantAdded(i32),
    /// A participant has been removed from the group
    ParticipantRemoved(i32),
    /// The group name has been changed
    NameChange(&'a str),
    /// A participant has voluntarily left the group
    ParticipantLeft,
    /// The group icon/avatar has been updated with a new image
    GroupIconChanged,
    /// The group icon/avatar has been removed, reverting to default
    GroupIconRemoved,
}

impl<'a> GroupAction<'a> {
    /// Creates a new `EventType` based on the provided `item_type` and `group_action_type`
    #[must_use]
    pub fn from_message(message: &'a Message) -> Option<Self> {
        match (
            message.item_type,
            message.group_action_type,
            message.other_handle,
            &message.group_title,
        ) {
            (1, 0, Some(who), _) => Some(Self::ParticipantAdded(who)),
            (1, 1, Some(who), _) => Some(Self::ParticipantRemoved(who)),
            (2, _, _, Some(name)) => Some(Self::NameChange(name)),
            (3, 0, _, _) => Some(Self::ParticipantLeft),
            (3, 1, _, _) => Some(Self::GroupIconChanged),
            (3, 2, _, _) => Some(Self::GroupIconRemoved),
            _ => None,
        }
    }
}
