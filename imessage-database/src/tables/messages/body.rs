use crate::{
    message_types::{
        edited::{EditStatus, EditedMessage},
        text_effects::{Animation, Style, TextEffect, Unit},
    },
    tables::messages::models::{AttachmentMeta, BubbleComponent, TextAttributes},
    util::typedstream::models::{Archivable, OutputData},
};

/// Character found in message body text that indicates attachment position
const ATTACHMENT_CHAR: char = '\u{FFFC}';
/// Character found in message body text that indicates app message position
const APP_CHAR: char = '\u{FFFD}';
/// A collection of characters that represent non-text content within body text
const REPLACEMENT_CHARS: [char; 2] = [ATTACHMENT_CHAR, APP_CHAR];

pub enum BubbleResult<'a> {
    New(BubbleComponent<'a>),
    Continuation(TextAttributes<'a>),
}

/// Logic to use deserialized typedstream data to parse the message body
pub(crate) fn parse_body_typedstream<'a>(
    components: Option<&'a Vec<Archivable>>,
    text: Option<&'a str>,
    edited_parts: Option<&'a EditedMessage>,
) -> Option<Vec<BubbleComponent<'a>>> {
    // Create the output data
    let mut out_v = vec![];

    // Start to iterate over the ranges
    if let Some(components) = components {
        // The first item is the text itself, so skip over it when iterating
        let mut idx = 1;
        let mut current_start;
        let mut current_end = 0;

        // We want to index into the message text, so we need a table to align
        // Apple's indexes with the actual chars, not the bytes
        let char_index_table: Vec<usize> = text.as_ref()?.char_indices().map(|(a, _)| a).collect();

        while idx < components.len() {
            // The first part of the range sometimes indicates the part number, but not always
            if let Some((_, length)) = get_range(components.get(idx)?) {
                current_start = current_end;
                current_end += *length as usize;
            } else {
                idx += 1;
                continue;
            }

            // The range is followed by a dictionary of attributes that map to that range
            idx += 1;
            let num_attrs = get_attribute_dict_length(components.get(idx));

            // The next set of values alternate between key and value pairs for the dictionary, if there are any
            if num_attrs > 0 {
                idx += 1;
            }

            // If there are no attributes, the default bubble will be applied
            // Otherwise, determine the bubble based on the attributes
            let slice: &[Archivable] = get_n_dict_objects(components, idx, num_attrs);

            // Determine the type of the bubble and add it to the body parts vec
            if let Some(bubble) =
                get_bubble_type(slice, text, current_start, current_end, &char_index_table)
            {
                match bubble {
                    BubbleResult::New(item) => out_v.push(item),
                    BubbleResult::Continuation(effect) => match out_v.last_mut() {
                        Some(BubbleComponent::Text(attrs)) => attrs.push(effect),
                        _ => out_v.push(BubbleComponent::Text(vec![effect])),
                    },
                }
            }

            // Advance the iterator by the number of attributes we just consumed
            idx += slice.len();
        }
    }

    // Add retracted components into the body
    if let Some(edited_message) = &edited_parts {
        for (idx, edited_message_part) in edited_message.parts.iter().enumerate() {
            if matches!(edited_message_part.status, EditStatus::Unsent) {
                if idx >= out_v.len() {
                    out_v.push(BubbleComponent::Retracted);
                } else {
                    out_v.insert(idx, BubbleComponent::Retracted);
                }
            }
        }
    }
    (!out_v.is_empty()).then_some(out_v)
}

fn get_range(component: &Archivable) -> Option<(&i64, &u64)> {
    if let Archivable::Data(items) = component {
        if items.len() == 2 {
            if let (OutputData::SignedInteger(item), OutputData::UnsignedInteger(end)) =
                (items.first()?, items.get(1)?)
            {
                return Some((item, end));
            }
        }
    }
    None
}

/// Given the attributedBody range indexes, get the substring from the Rust representations `char_indices()`
fn get_char_idx(text: &str, idx: usize, char_indices: &[usize]) -> usize {
    char_indices.get(idx).map_or(text.len(), |i| *i)
}

/// Get the number of key/value object pairs in a `NSDictionary`
fn get_attribute_dict_length(component: Option<&Archivable>) -> usize {
    if let Some(Archivable::Object(class, data)) = component {
        if class.name == "NSDictionary" {
            if let Some(OutputData::SignedInteger(length)) = data.first() {
                return (length * 2) as usize;
            }
        }
    }
    0
}

/// Get a specific number of objects that represent the content of a dictionary
fn get_n_dict_objects(components: &[Archivable], idx: usize, num_objects: usize) -> &[Archivable] {
    if num_objects == 0 {
        return &[];
    }
    let mut final_idx = idx + num_objects;
    for (idx, component) in components.iter().enumerate().skip(idx) {
        // Break the loop if we encounter a new range, which indicates we should move on to the next part
        if get_range(component).is_some() {
            break;
        }
        final_idx = idx;
    }
    components.get(idx..=final_idx).unwrap_or(&[])
}

/// Determine the type of bubble the current range represents
///
/// App messages are handled in [`body()`](crate::tables::table::AttributedBody); they are detected by the presence of data in the `balloon_bundle_id` column.
fn get_bubble_type<'a>(
    components: &'a [Archivable],
    text: Option<&str>,
    start: usize,
    end: usize,
    char_indices: &[usize],
) -> Option<BubbleResult<'a>> {
    let range_start = get_char_idx(text.as_ref()?, start, char_indices);
    let range_end = get_char_idx(text.as_ref()?, end, char_indices);
    for (idx, key) in components.iter().enumerate() {
        if let Some(key_name) = key.as_nsstring() {
            match key_name {
                "__kIMFileTransferGUIDAttributeName" => {
                    return Some(BubbleResult::New(BubbleComponent::Attachment(
                        AttachmentMeta::from_components(components)?,
                    )));
                }
                "__kIMMentionConfirmedMention" => {
                    return Some(BubbleResult::Continuation(TextAttributes::new(
                        range_start,
                        range_end,
                        TextEffect::Mention(components.get(idx + 1)?.as_nsstring().unwrap_or("")),
                    )));
                }
                "__kIMLinkAttributeName" => {
                    return Some(BubbleResult::Continuation(TextAttributes::new(
                        range_start,
                        range_end,
                        TextEffect::Link(components.get(idx + 2)?.as_nsstring().unwrap_or("#")),
                    )));
                }
                "__kIMOneTimeCodeAttributeName" => {
                    return Some(BubbleResult::Continuation(TextAttributes::new(
                        range_start,
                        range_end,
                        TextEffect::OTP,
                    )));
                }
                "__kIMCalendarEventAttributeName" => {
                    return Some(BubbleResult::Continuation(TextAttributes::new(
                        range_start,
                        range_end,
                        TextEffect::Conversion(Unit::Timezone),
                    )));
                }
                // Any number of text styles can be applied to a message
                "__kIMTextBoldAttributeName"
                | "__kIMTextUnderlineAttributeName"
                | "__kIMTextItalicAttributeName"
                | "__kIMTextStrikethroughAttributeName" => {
                    return Some(BubbleResult::Continuation(TextAttributes::new(
                        range_start,
                        range_end,
                        TextEffect::Styles(resolve_styles(components)),
                    )));
                }
                "__kIMTextEffectAttributeName" => {
                    return Some(BubbleResult::Continuation(TextAttributes::new(
                        range_start,
                        range_end,
                        TextEffect::Animated(Animation::from_id(
                            *components.get(idx + 1)?.as_nsnumber_int().unwrap_or(&0),
                        )),
                    )));
                }
                _ => {}
            }
        }
    }
    Some(BubbleResult::Continuation(TextAttributes::new(
        range_start,
        range_end,
        TextEffect::Default,
    )))
}

/// Extract text styles from a range of key-value pairs
fn resolve_styles(components: &[Archivable]) -> Vec<Style> {
    let mut styles = vec![];
    for key in components {
        if let Some(key_name) = key.as_nsstring() {
            match key_name {
                "__kIMTextBoldAttributeName" => styles.push(Style::Bold),
                "__kIMTextUnderlineAttributeName" => styles.push(Style::Underline),
                "__kIMTextItalicAttributeName" => styles.push(Style::Italic),
                "__kIMTextStrikethroughAttributeName" => styles.push(Style::Strikethrough),
                _ => {}
            }
        }
    }
    styles
}

/// Fallback logic to parse the body from the message string content
pub(crate) fn parse_body_legacy(text: &Option<String>) -> Vec<BubbleComponent> {
    let mut out_v = vec![];
    // Naive logic for when `typedstream` component parsing fails
    match text {
        Some(text) => {
            let mut start: usize = 0;
            let mut end: usize = 0;

            for (idx, char) in text.char_indices() {
                if REPLACEMENT_CHARS.contains(&char) {
                    if start < end {
                        out_v.push(BubbleComponent::Text(vec![TextAttributes::new(
                            start,
                            idx,
                            TextEffect::Default,
                        )]));
                    }
                    start = idx + 1;
                    end = idx;
                    match char {
                        ATTACHMENT_CHAR => {
                            out_v.push(BubbleComponent::Attachment(AttachmentMeta::default()));
                        }
                        APP_CHAR => out_v.push(BubbleComponent::App),
                        _ => {}
                    }
                } else {
                    if start > end {
                        start = idx;
                    }
                    end = idx;
                }
            }
            if start <= end && start < text.len() {
                out_v.push(BubbleComponent::Text(vec![TextAttributes::new(
                    start,
                    text.len(),
                    TextEffect::Default,
                )]));
            }
            out_v
        }
        None => out_v,
    }
}

#[cfg(test)]
mod typedstream_tests {
    use std::{env::current_dir, fs::File, io::Read};

    use crate::{
        message_types::{
            edited::{EditStatus, EditedEvent, EditedMessage, EditedMessagePart},
            text_effects::{Animation, Style, TextEffect, Unit},
        },
        tables::messages::{
            Message,
            body::parse_body_typedstream,
            models::{AttachmentMeta, BubbleComponent, TextAttributes},
        },
        util::typedstream::parser::TypedStreamReader,
    };

    #[test]
    fn can_get_message_body_simple() {
        let mut m = Message::blank();
        m.text = Some("Noter test".to_string());

        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/AttributedBodyTextOnly");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        m.components = parser.parse().ok();

        assert_eq!(
            parse_body_typedstream(
                m.components.as_ref(),
                m.text.as_deref(),
                m.edited_parts.as_ref()
            )
            .unwrap(),
            vec![BubbleComponent::Text(vec![TextAttributes::new(
                0,
                10,
                TextEffect::Default
            )])]
        );
    }

    #[test]
    fn can_get_message_body_app() {
        let mut m = Message::blank();
        m.text = Some("\u{FFFC}".to_string());

        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/AppMessage");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        m.components = parser.parse().ok();

        assert_eq!(
            parse_body_typedstream(
                m.components.as_ref(),
                m.text.as_deref(),
                m.edited_parts.as_ref()
            )
            .unwrap(),
            vec![BubbleComponent::Attachment(AttachmentMeta {
                guid: Some("F0B18A15-E9A5-4B18-A38F-685B7B3FF037"),
                transcription: None,
                height: None,
                width: None,
                name: None
            })]
        );
    }

    #[test]
    fn can_get_message_body_simple_two() {
        let mut m = Message::blank();
        m.text = Some("Test 3".to_string());

        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/AttributedBodyTextOnly2");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        m.components = parser.parse().ok();

        assert_eq!(
            parse_body_typedstream(
                m.components.as_ref(),
                m.text.as_deref(),
                m.edited_parts.as_ref()
            )
            .unwrap(),
            vec![BubbleComponent::Text(vec![TextAttributes::new(
                0,
                6,
                TextEffect::Default
            )])]
        );
    }

    #[test]
    fn can_get_message_body_multi_part() {
        let mut m = Message::blank();
        m.text = Some("\u{FFFC}test 1\u{FFFC}test 2 \u{FFFC}test 3".to_string());

        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/Multipart");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        m.components = parser.parse().ok();

        assert_eq!(
            parse_body_typedstream(
                m.components.as_ref(),
                m.text.as_deref(),
                m.edited_parts.as_ref()
            )
            .unwrap(),
            vec![
                BubbleComponent::Attachment(AttachmentMeta {
                    guid: Some("at_0_F0668F79-20C2-49C9-A87F-1B007ABB0CED"),
                    transcription: None,
                    height: None,
                    width: None,
                    name: None
                }),
                BubbleComponent::Text(vec![TextAttributes::new(3, 9, TextEffect::Default)]),
                BubbleComponent::Attachment(AttachmentMeta {
                    guid: Some("at_2_F0668F79-20C2-49C9-A87F-1B007ABB0CED"),
                    transcription: None,
                    height: None,
                    width: None,
                    name: None
                }),
                BubbleComponent::Text(vec![TextAttributes::new(12, 19, TextEffect::Default)]),
                BubbleComponent::Attachment(AttachmentMeta {
                    guid: Some("at_4_F0668F79-20C2-49C9-A87F-1B007ABB0CED"),
                    transcription: None,
                    height: None,
                    width: None,
                    name: None
                }),
                BubbleComponent::Text(vec![TextAttributes::new(22, 28, TextEffect::Default)]),
            ]
        );
    }

    #[test]
    fn can_get_message_body_multi_part_deleted() {
        let mut m = Message::blank();
        m.text = Some(
            "From arbitrary byte stream:\r\u{FFFC}To native Rust data structures:\r".to_string(),
        );

        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/MultiPartWithDeleted");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        m.components = parser.parse().ok();

        assert_eq!(
            parse_body_typedstream(
                m.components.as_ref(),
                m.text.as_deref(),
                m.edited_parts.as_ref()
            )
            .unwrap(),
            vec![
                BubbleComponent::Text(vec![TextAttributes::new(0, 28, TextEffect::Default)]),
                BubbleComponent::Attachment(AttachmentMeta {
                    guid: Some("D0551D89-4E11-43D0-9A0E-06F19704E97B"),
                    transcription: None,
                    height: None,
                    width: None,
                    name: None
                }),
                BubbleComponent::Text(vec![TextAttributes::new(31, 63, TextEffect::Default)]),
            ]
        );
    }

    #[test]
    fn can_get_message_body_multi_part_deleted_edited() {
        let mut m = Message::blank();
        m.text = Some(
            "From arbitrary byte stream:\r\u{FFFC}To native Rust data structures:\r".to_string(),
        );

        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/MultiPartWithDeleted");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        m.components = parser.parse().ok();

        m.edited_parts = Some(EditedMessage {
            parts: vec![
                EditedMessagePart {
                    status: EditStatus::Original,
                    edit_history: vec![],
                },
                EditedMessagePart {
                    status: EditStatus::Original,
                    edit_history: vec![],
                },
                EditedMessagePart {
                    status: EditStatus::Edited,
                    edit_history: vec![
                        EditedEvent::new(
                            743907435000000000,
                            Some("Second test".to_string()),
                            None,
                            None,
                        ),
                        EditedEvent::new(
                            743907448000000000,
                            Some("Second test was edited!".to_string()),
                            None,
                            None,
                        ),
                    ],
                },
                EditedMessagePart {
                    status: EditStatus::Unsent,
                    edit_history: vec![],
                },
            ],
        });

        assert_eq!(
            parse_body_typedstream(
                m.components.as_ref(),
                m.text.as_deref(),
                m.edited_parts.as_ref()
            )
            .unwrap(),
            vec![
                BubbleComponent::Text(vec![TextAttributes::new(0, 28, TextEffect::Default)]),
                BubbleComponent::Attachment(AttachmentMeta {
                    guid: Some("D0551D89-4E11-43D0-9A0E-06F19704E97B"),
                    transcription: None,
                    height: None,
                    width: None,
                    name: None
                }),
                BubbleComponent::Text(vec![TextAttributes::new(31, 63, TextEffect::Default)]),
                BubbleComponent::Retracted,
            ]
        );
    }

    #[test]
    fn can_get_message_body_attachment() {
        let mut m = Message::blank();
        m.text = Some(
            "\u{FFFC}This is how the notes look to me fyi, in case it helps make sense of anything"
                .to_string(),
        );

        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/Attachment");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        m.components = parser.parse().ok();

        assert_eq!(
            parse_body_typedstream(
                m.components.as_ref(),
                m.text.as_deref(),
                m.edited_parts.as_ref()
            )
            .unwrap(),
            vec![
                BubbleComponent::Attachment(AttachmentMeta {
                    guid: Some("at_0_2E5F12C3-E649-48AA-954D-3EA67C016BCC"),
                    transcription: None,
                    height: Some(&1139.0),
                    width: Some(&952.0),
                    name: Some("Messages Image(785748029).png")
                }),
                BubbleComponent::Text(vec![TextAttributes::new(3, 80, TextEffect::Default)]),
            ]
        );
    }

    #[test]
    fn can_get_message_body_attachment_i16() {
        let mut m = Message::blank();
        m.text = Some("\u{FFFC}".to_string());

        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/AttachmentI16");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        m.components = parser.parse().ok();

        assert_eq!(
            parse_body_typedstream(
                m.components.as_ref(),
                m.text.as_deref(),
                m.edited_parts.as_ref()
            )
            .unwrap(),
            vec![BubbleComponent::Attachment(AttachmentMeta {
                guid: Some("at_0_BE588799-C4BC-47DF-A56D-7EE90C74911D"),
                transcription: None,
                height: None,
                width: None,
                name: Some("brilliant-kids-test-answers-32-93042.jpeg")
            })]
        );
    }

    #[test]
    fn can_get_message_body_url() {
        let mut m = Message::blank();
        m.text = Some("https://twitter.com/xxxxxxxxx/status/0000223300009216128".to_string());

        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/URLMessage");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        m.components = parser.parse().ok();

        m.components
            .as_ref()
            .unwrap()
            .iter()
            .enumerate()
            .for_each(|(idx, item)| println!("\t{idx}: {item:?}"));

        assert_eq!(
            parse_body_typedstream(
                m.components.as_ref(),
                m.text.as_deref(),
                m.edited_parts.as_ref()
            )
            .unwrap(),
            vec![BubbleComponent::Text(vec![TextAttributes::new(
                0,
                56,
                TextEffect::Link("https://twitter.com/xxxxxxxxx/status/0000223300009216128")
            )]),]
        );
    }

    #[test]
    fn can_get_message_body_mention() {
        let mut m = Message::blank();
        m.text = Some("Test Dad ".to_string());

        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/Mention");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        m.components = parser.parse().ok();

        m.components
            .as_ref()
            .unwrap()
            .iter()
            .enumerate()
            .for_each(|(idx, item)| println!("\t{idx}: {item:?}"));

        assert_eq!(
            parse_body_typedstream(
                m.components.as_ref(),
                m.text.as_deref(),
                m.edited_parts.as_ref()
            )
            .unwrap(),
            vec![BubbleComponent::Text(vec![
                TextAttributes::new(0, 5, TextEffect::Default),
                TextAttributes::new(5, 8, TextEffect::Mention("+15558675309")),
                TextAttributes::new(8, 9, TextEffect::Default)
            ])]
        );
    }

    #[test]
    fn can_get_message_body_code() {
        let mut m = Message::blank();
        m.text = Some("000123 is your security code. Don't share your code.".to_string());

        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/Code");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        m.components = parser.parse().ok();

        m.components
            .as_ref()
            .unwrap()
            .iter()
            .enumerate()
            .for_each(|(idx, item)| println!("\t{idx}: {item:?}"));

        assert_eq!(
            parse_body_typedstream(
                m.components.as_ref(),
                m.text.as_deref(),
                m.edited_parts.as_ref()
            )
            .unwrap(),
            vec![BubbleComponent::Text(vec![
                TextAttributes::new(0, 6, TextEffect::OTP),
                TextAttributes::new(6, 52, TextEffect::Default),
            ])]
        );
    }

    #[test]
    fn can_get_message_body_phone() {
        let mut m = Message::blank();
        m.text = Some("What about 0000000000".to_string());

        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/PhoneNumber");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        m.components = parser.parse().ok();

        m.components
            .as_ref()
            .unwrap()
            .iter()
            .enumerate()
            .for_each(|(idx, item)| println!("\t{idx}: {item:?}"));

        assert_eq!(
            parse_body_typedstream(
                m.components.as_ref(),
                m.text.as_deref(),
                m.edited_parts.as_ref()
            )
            .unwrap(),
            vec![BubbleComponent::Text(vec![
                TextAttributes::new(0, 11, TextEffect::Default),
                TextAttributes::new(11, 21, TextEffect::Link("tel:0000000000")),
            ])]
        );
    }

    #[test]
    fn can_get_message_body_email() {
        let mut m = Message::blank();
        m.text = Some("asdfghjklq@gmail.com might work".to_string());

        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/Email");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        m.components = parser.parse().ok();

        m.components
            .as_ref()
            .unwrap()
            .iter()
            .enumerate()
            .for_each(|(idx, item)| println!("\t{idx}: {item:?}"));

        assert_eq!(
            parse_body_typedstream(
                m.components.as_ref(),
                m.text.as_deref(),
                m.edited_parts.as_ref()
            )
            .unwrap(),
            vec![BubbleComponent::Text(vec![
                TextAttributes::new(0, 20, TextEffect::Link("mailto:asdfghjklq@gmail.com")),
                TextAttributes::new(20, 31, TextEffect::Default),
            ])]
        );
    }

    #[test]
    fn can_get_message_body_date() {
        let mut m = Message::blank();
        m.text = Some("Hi. Right now or tomorrow?".to_string());

        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/Date");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        m.components = parser.parse().ok();

        m.components
            .as_ref()
            .unwrap()
            .iter()
            .enumerate()
            .for_each(|(idx, item)| println!("\t{idx}: {item:?}"));

        assert_eq!(
            parse_body_typedstream(
                m.components.as_ref(),
                m.text.as_deref(),
                m.edited_parts.as_ref()
            )
            .unwrap(),
            vec![BubbleComponent::Text(vec![
                TextAttributes::new(0, 17, TextEffect::Default),
                TextAttributes::new(17, 25, TextEffect::Conversion(Unit::Timezone)),
                TextAttributes::new(25, 26, TextEffect::Default),
            ])]
        );
    }

    #[test]
    fn can_get_message_body_custom_tapback() {
        let mut m = Message::blank();
        m.text = Some(String::new());

        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/CustomReaction");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        m.components = parser.parse().ok();

        m.components
            .as_ref()
            .unwrap()
            .iter()
            .enumerate()
            .for_each(|(idx, item)| println!("\t{idx}: {item:?}"));

        assert_eq!(
            parse_body_typedstream(
                m.components.as_ref(),
                m.text.as_deref(),
                m.edited_parts.as_ref()
            )
            .unwrap(),
            vec![
                BubbleComponent::Text(vec![TextAttributes::new(0, 0, TextEffect::Default)]),
                BubbleComponent::Attachment(AttachmentMeta {
                    guid: Some("41C4376E-397E-4C42-84E2-B16F7801F638"),
                    transcription: None,
                    height: None,
                    width: None,
                    name: None
                })
            ]
        );
    }

    #[test]
    fn can_get_message_body_deleted_only() {
        let mut m = Message::blank();
        m.edited_parts = Some(EditedMessage {
            parts: vec![EditedMessagePart {
                status: EditStatus::Unsent,
                edit_history: vec![],
            }],
        });

        assert_eq!(
            parse_body_typedstream(
                m.components.as_ref(),
                m.text.as_deref(),
                m.edited_parts.as_ref()
            )
            .unwrap(),
            vec![BubbleComponent::Retracted,]
        );
    }

    #[test]
    fn can_get_message_body_text_styles() {
        let mut m = Message::blank();
        m.text = Some("Bold underline italic strikethrough all four".to_string());

        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/TextStyles");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        m.components = parser.parse().ok();

        m.components
            .as_ref()
            .unwrap()
            .iter()
            .enumerate()
            .for_each(|(idx, item)| println!("\t{idx}: {item:?}"));

        assert_eq!(
            parse_body_typedstream(
                m.components.as_ref(),
                m.text.as_deref(),
                m.edited_parts.as_ref()
            )
            .unwrap(),
            vec![BubbleComponent::Text(vec![
                TextAttributes::new(0, 4, TextEffect::Styles(vec![Style::Bold])),
                TextAttributes::new(4, 5, TextEffect::Default),
                TextAttributes::new(5, 14, TextEffect::Styles(vec![Style::Underline])),
                TextAttributes::new(14, 15, TextEffect::Default),
                TextAttributes::new(15, 21, TextEffect::Styles(vec![Style::Italic])),
                TextAttributes::new(21, 22, TextEffect::Default),
                TextAttributes::new(22, 35, TextEffect::Styles(vec![Style::Strikethrough])),
                TextAttributes::new(35, 40, TextEffect::Default),
                TextAttributes::new(
                    40,
                    44,
                    TextEffect::Styles(vec![
                        Style::Bold,
                        Style::Strikethrough,
                        Style::Underline,
                        Style::Italic
                    ])
                ),
            ]),]
        );
    }

    #[test]
    fn can_get_message_body_text_effects() {
        let mut m = Message::blank();
        m.text = Some("Big small shake nod explode ripple bloom jitter".to_string());

        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/TextEffects");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        m.components = parser.parse().ok();

        m.components
            .as_ref()
            .unwrap()
            .iter()
            .enumerate()
            .for_each(|(idx, item)| println!("\t{idx}: {item:?}"));

        assert_eq!(
            parse_body_typedstream(
                m.components.as_ref(),
                m.text.as_deref(),
                m.edited_parts.as_ref()
            )
            .unwrap(),
            vec![BubbleComponent::Text(vec![
                TextAttributes::new(0, 3, TextEffect::Animated(Animation::Big)),
                TextAttributes::new(3, 4, TextEffect::Default),
                TextAttributes::new(4, 10, TextEffect::Animated(Animation::Small)),
                TextAttributes::new(10, 15, TextEffect::Animated(Animation::Shake)),
                TextAttributes::new(15, 16, TextEffect::Default),
                TextAttributes::new(16, 19, TextEffect::Animated(Animation::Nod)),
                TextAttributes::new(19, 20, TextEffect::Default),
                TextAttributes::new(20, 28, TextEffect::Animated(Animation::Explode)),
                TextAttributes::new(28, 34, TextEffect::Animated(Animation::Ripple)),
                TextAttributes::new(34, 35, TextEffect::Default),
                TextAttributes::new(35, 40, TextEffect::Animated(Animation::Bloom)),
                TextAttributes::new(40, 41, TextEffect::Default),
                TextAttributes::new(41, 47, TextEffect::Animated(Animation::Jitter)),
            ]),]
        );
    }

    #[test]
    fn can_get_message_body_text_effects_styles_mixed() {
        let mut m = Message::blank();
        m.text = Some("Underline normal jitter normal".to_string());

        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/TextStylesMixed");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        m.components = parser.parse().ok();

        m.components
            .as_ref()
            .unwrap()
            .iter()
            .enumerate()
            .for_each(|(idx, item)| println!("\t{idx}: {item:?}"));

        assert_eq!(
            parse_body_typedstream(
                m.components.as_ref(),
                m.text.as_deref(),
                m.edited_parts.as_ref()
            )
            .unwrap(),
            vec![BubbleComponent::Text(vec![
                TextAttributes::new(0, 9, TextEffect::Styles(vec![Style::Underline])),
                TextAttributes::new(9, 17, TextEffect::Default),
                TextAttributes::new(17, 23, TextEffect::Animated(Animation::Jitter)),
                TextAttributes::new(23, 30, TextEffect::Default),
            ]),]
        );
    }

    #[test]
    fn can_get_message_body_audio_message() {
        let mut m = Message::blank();
        m.text = Some("\u{FFFC}".to_string());

        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/Transcription");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        m.components = parser.parse().ok();

        m.components
            .as_ref()
            .unwrap()
            .iter()
            .enumerate()
            .for_each(|(idx, item)| println!("\t{idx}: {item:?}"));

        assert_eq!(
            parse_body_typedstream(
                m.components.as_ref(),
                m.text.as_deref(),
                m.edited_parts.as_ref()
            )
            .unwrap(),
            vec![BubbleComponent::Attachment(AttachmentMeta {
                guid: Some("4C339597-EBBB-4978-9B87-521C0471A848"),
                transcription: Some("This is a test"),
                height: None,
                width: None,
                name: None
            }),]
        );
    }

    #[test]
    fn can_get_message_body_apple_music_lyrics() {
        let mut m = Message::blank();
        m.text = Some("\u{FFFC}".to_string());

        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/AppleMusicLyrics");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        m.components = parser.parse().ok();

        parse_body_typedstream(
            m.components.as_ref(),
            m.text.as_deref(),
            m.edited_parts.as_ref(),
        )
        .unwrap()
        .iter()
        .enumerate()
        .for_each(|(idx, item)| println!("\t{idx}: {item:#?}"));

        assert_eq!(
            parse_body_typedstream(
                m.components.as_ref(),
                m.text.as_deref(),
                m.edited_parts.as_ref()
            )
            .unwrap(),
            vec![BubbleComponent::Text(vec![TextAttributes::new(
                0,
                3,
                TextEffect::Link(
                    "https://music.apple.com/us/lyrics/1329891623?ts=11.108&te=16.031&l=en&tk=2.v1.VsuX9f%2BaT1PyrgMgIT7ANQ%3D%3D&itsct=sharing_msg_lyrics&itscg=50401"
                )
            ),])]
        );
    }

    #[test]
    fn can_get_message_body_multiple_attachment() {
        let mut m = Message::blank();
        m.text = Some("\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}".to_string());

        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/MultiAttachment");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        m.components = parser.parse().ok();

        parse_body_typedstream(
            m.components.as_ref(),
            m.text.as_deref(),
            m.edited_parts.as_ref(),
        )
        .unwrap()
        .iter()
        .enumerate()
        .for_each(|(idx, item)| println!("\t{idx}: {item:#?}"));

        assert_eq!(
            parse_body_typedstream(
                m.components.as_ref(),
                m.text.as_deref(),
                m.edited_parts.as_ref()
            )
            .unwrap()[..5],
            vec![
                BubbleComponent::Attachment(AttachmentMeta {
                    guid: Some("at_0_48B9C973-3466-438C-BE72-E5B498D30772"),
                    transcription: None,
                    height: None,
                    width: None,
                    name: None
                }),
                BubbleComponent::Attachment(AttachmentMeta {
                    guid: Some("at_1_48B9C973-3466-438C-BE72-E5B498D30772"),
                    transcription: None,
                    height: None,
                    width: None,
                    name: None
                }),
                BubbleComponent::Attachment(AttachmentMeta {
                    guid: Some("at_2_48B9C973-3466-438C-BE72-E5B498D30772"),
                    transcription: None,
                    height: None,
                    width: None,
                    name: None
                }),
                BubbleComponent::Attachment(AttachmentMeta {
                    guid: Some("at_3_48B9C973-3466-438C-BE72-E5B498D30772"),
                    transcription: None,
                    height: None,
                    width: None,
                    name: None
                }),
                BubbleComponent::Attachment(AttachmentMeta {
                    guid: Some("at_4_48B9C973-3466-438C-BE72-E5B498D30772"),
                    transcription: None,
                    height: None,
                    width: None,
                    name: None
                })
            ]
        );
    }
}

#[cfg(test)]
mod legacy_tests {
    use crate::{
        message_types::text_effects::TextEffect,
        tables::messages::{
            Message,
            body::parse_body_legacy,
            models::{AttachmentMeta, BubbleComponent, TextAttributes},
        },
    };

    #[test]
    fn can_get_message_body_single_emoji() {
        let mut m = Message::blank();
        m.text = Some("🙈".to_string());
        assert_eq!(
            parse_body_legacy(&m.text),
            vec![BubbleComponent::Text(vec![TextAttributes::new(
                0,
                4,
                TextEffect::Default
            ),])]
        );
    }

    #[test]
    fn can_get_message_body_multiple_emoji() {
        let mut m = Message::blank();
        m.text = Some("🙈🙈🙈".to_string());
        assert_eq!(
            parse_body_legacy(&m.text),
            vec![BubbleComponent::Text(vec![TextAttributes::new(
                0,
                12,
                TextEffect::Default
            ),])]
        );
    }

    #[test]
    fn can_get_message_body_text_only() {
        let mut m = Message::blank();
        m.text = Some("Hello world".to_string());
        assert_eq!(
            parse_body_legacy(&m.text),
            vec![BubbleComponent::Text(vec![TextAttributes::new(
                0,
                11,
                TextEffect::Default
            ),])]
        );
    }

    #[test]
    fn can_get_message_body_attachment_text() {
        let mut m = Message::blank();
        m.text = Some("\u{FFFC}Hello world".to_string());
        assert_eq!(
            parse_body_legacy(&m.text),
            vec![
                BubbleComponent::Attachment(AttachmentMeta::default()),
                BubbleComponent::Text(vec![TextAttributes::new(3, 14, TextEffect::Default),])
            ]
        );
    }

    #[test]
    fn can_get_message_body_app_text() {
        let mut m = Message::blank();
        m.text = Some("\u{FFFD}Hello world".to_string());
        assert_eq!(
            parse_body_legacy(&m.text),
            vec![
                BubbleComponent::App,
                BubbleComponent::Text(vec![TextAttributes::new(3, 14, TextEffect::Default),])
            ]
        );
    }

    #[test]
    fn can_get_message_body_app_attachment_text_mixed_start_text() {
        let mut m = Message::blank();
        m.text = Some("One\u{FFFD}\u{FFFC}Two\u{FFFC}Three\u{FFFC}four".to_string());
        assert_eq!(
            parse_body_legacy(&m.text),
            vec![
                BubbleComponent::Text(vec![TextAttributes::new(0, 3, TextEffect::Default),]),
                BubbleComponent::App,
                BubbleComponent::Attachment(AttachmentMeta::default()),
                BubbleComponent::Text(vec![TextAttributes::new(9, 12, TextEffect::Default),]),
                BubbleComponent::Attachment(AttachmentMeta::default()),
                BubbleComponent::Text(vec![TextAttributes::new(15, 20, TextEffect::Default),]),
                BubbleComponent::Attachment(AttachmentMeta::default()),
                BubbleComponent::Text(vec![TextAttributes::new(23, 27, TextEffect::Default),]),
            ]
        );
    }

    #[test]
    fn can_get_message_body_app_attachment_text_mixed_start_app() {
        let mut m = Message::blank();
        m.text = Some("\u{FFFD}\u{FFFC}Two\u{FFFC}Three\u{FFFC}".to_string());
        assert_eq!(
            parse_body_legacy(&m.text),
            vec![
                BubbleComponent::App,
                BubbleComponent::Attachment(AttachmentMeta::default()),
                BubbleComponent::Text(vec![TextAttributes::new(6, 9, TextEffect::Default),]),
                BubbleComponent::Attachment(AttachmentMeta::default()),
                BubbleComponent::Text(vec![TextAttributes::new(12, 17, TextEffect::Default),]),
                BubbleComponent::Attachment(AttachmentMeta::default()),
            ]
        );
    }
}
