#[cfg(test)]
mod parser_tests {
    use std::env::current_dir;
    use std::fs::File;
    use std::io::Read;
    use std::vec;

    use crate::util::typedstream::{
        models::{Archivable, Class, OutputData},
        parser::TypedStreamReader,
    };

    #[test]
    fn test_parse_header() {
        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/AttributedBodyTextOnly");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        let result = parser.validate_header();

        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_text_mention() {
        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/Mention");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        let result = parser.parse().unwrap();

        println!("\n\nGot data!");
        result.iter().for_each(|item| println!("{item:?}"));

        let expected = vec![
            Archivable::Object(
                Class {
                    name: "NSMutableString".to_string(),
                    version: 1,
                },
                vec![OutputData::String("Test Dad ".to_string())],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(1),
                OutputData::UnsignedInteger(5),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(2),
                OutputData::UnsignedInteger(3),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(2)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMentionConfirmedMention".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String("+15558675309".to_string())],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(1),
                OutputData::UnsignedInteger(1),
            ]),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_text_basic() {
        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/AttributedBodyTextOnly");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        let result = parser.parse().unwrap();

        println!("\n\nGot data!");
        result.iter().for_each(|item| println!("{item:?}"));

        let expected = vec![
            Archivable::Object(
                Class {
                    name: "NSMutableString".to_string(),
                    version: 1,
                },
                vec![OutputData::String("Noter test".to_string())],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(1),
                OutputData::UnsignedInteger(10),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_text_basic_2() {
        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/AttributedBodyTextOnly2");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        let result = parser.parse().unwrap();

        println!("\n\nGot data!");
        result.iter().for_each(|item| println!("\t{item:?}"));

        let expected = vec![
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String("Test 3".to_string())],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(1),
                OutputData::UnsignedInteger(6),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(2)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMBaseWritingDirectionAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(-1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_text_long() {
        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/LongMessage");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        let result = parser.parse().unwrap();

        println!("\n\nGot data!");
        result.iter().for_each(|item| println!("{item:?}"));

        let expected = vec![
            Archivable::Data(vec![
                OutputData::SignedInteger(1),
                OutputData::UnsignedInteger(2359),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
        ];

        assert_eq!(result[1..], expected);
    }

    #[test]
    fn test_parse_text_multi_part() {
        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/Multipart");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        let result = parser.parse().unwrap();

        println!("\n\nGot data!");
        result.iter().for_each(|item| println!("\t{item:?}"));
        println!("\n\n");

        let expected = vec![
            Archivable::Object(
                Class {
                    name: "NSMutableString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "\u{FFFC}test 1\u{FFFC}test 2 \u{FFFC}test 3".to_string(),
                )],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(1),
                OutputData::UnsignedInteger(1),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(2)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMFileTransferGUIDAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "at_0_F0668F79-20C2-49C9-A87F-1B007ABB0CED".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(2),
                OutputData::UnsignedInteger(6),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(3),
                OutputData::UnsignedInteger(1),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(2)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMFileTransferGUIDAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "at_2_F0668F79-20C2-49C9-A87F-1B007ABB0CED".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(2)],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(4),
                OutputData::UnsignedInteger(7),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(3)],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(5),
                OutputData::UnsignedInteger(1),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(2)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMFileTransferGUIDAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "at_4_F0668F79-20C2-49C9-A87F-1B007ABB0CED".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(4)],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(6),
                OutputData::UnsignedInteger(6),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(5)],
            ),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_text_multi_part_deleted() {
        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/MultiPartWithDeleted");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        let result = parser.parse().unwrap();

        println!("\n\nGot data!");
        result.iter().for_each(|item| println!("\t{item:?}"));

        let expected = vec![
            Archivable::Object(
                Class {
                    name: "NSMutableString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "From arbitrary byte stream:\r￼To native Rust data structures:\r".to_string(),
                )],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(1),
                OutputData::UnsignedInteger(28),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(2),
                OutputData::UnsignedInteger(1),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(2)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMFileTransferGUIDAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "D0551D89-4E11-43D0-9A0E-06F19704E97B".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(3),
                OutputData::UnsignedInteger(32),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(2)],
            ),
        ];

        println!("\n\nExpected data!");
        expected.iter().for_each(|item| println!("\t{item:?}"));
        println!("\n\n");

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_text_attachment_float() {
        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/Attachment");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        let result = parser.parse().unwrap();

        println!("\n\nGot data!");
        result.iter().for_each(|item| println!("\t{item:?}"));

        let expected = vec![
            Archivable::Object(
                Class {
                    name: "NSMutableString".to_string(),
                    version: 1,
                },
                vec![OutputData::String("\u{FFFC}This is how the notes look to me fyi, in case it helps make sense of anything".to_string())],
            ),
            Archivable::Data(vec![OutputData::SignedInteger(1), OutputData::UnsignedInteger(1)]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(6)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMFileTransferGUIDAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "at_0_2E5F12C3-E649-48AA-954D-3EA67C016BCC".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMInlineMediaHeightAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::Double(1139.0)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMBaseWritingDirectionAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(-1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMFilenameAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "Messages Image(785748029).png".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMInlineMediaWidthAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::Double(952.0)],
            ),
            Archivable::Data(vec![OutputData::SignedInteger(2), OutputData::UnsignedInteger(77)]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(2)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMBaseWritingDirectionAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(-1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_text_attachment_i16() {
        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/AttachmentI16");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        let result = parser.parse().unwrap();

        println!("\n\nGot data!");
        result.iter().for_each(|item| println!("{item:?}"));
        let expected = vec![
            Archivable::Object(
                Class {
                    name: "NSMutableString".to_string(),
                    version: 1,
                },
                vec![OutputData::String("\u{FFFC}".to_string())],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(1),
                OutputData::UnsignedInteger(1),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(6)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMFileTransferGUIDAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "at_0_BE588799-C4BC-47DF-A56D-7EE90C74911D".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMInlineMediaHeightAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(600)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMBaseWritingDirectionAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(-1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String("__kIMFilenameAttributeName".to_string())],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "brilliant-kids-test-answers-32-93042.jpeg".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMInlineMediaWidthAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(660)],
            ),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_text_url_message() {
        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/URLMessage");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        let result = parser.parse().unwrap();

        println!("\n\nGot data!");
        result
            .iter()
            .enumerate()
            .for_each(|(idx, item)| println!("\t{idx}: {item:?}"));

        let expected_1 = vec![
            Archivable::Object(
                Class {
                    name: "NSMutableString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "https://twitter.com/xxxxxxxxx/status/0000223300009216128".to_string(),
                )],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(1),
                OutputData::UnsignedInteger(56),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(4)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String("__kIMLinkAttributeName".to_string())],
            ),
            Archivable::Object(
                Class {
                    name: "NSURL".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "https://twitter.com/xxxxxxxxx/status/0000223300009216128".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMDataDetectedAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSMutableData".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(604)],
            ),
        ];

        let expected_2 = vec![
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMBaseWritingDirectionAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(-1)],
            ),
        ];

        assert_eq!(result[..10], expected_1);
        assert_eq!(result[11..], expected_2);
    }

    #[test]
    fn test_parse_text_array() {
        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/Array");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        let result = parser.parse().unwrap();

        println!("\n\nGot data!");
        result.iter().for_each(|item| println!("\t{item:?}"));

        // Ignore the large array in the test
        let expected_1 = vec![
            Archivable::Object(
                Class {
                    name: "NSMutableString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "A single ChatGPT instance takes 5MW of power to run".to_string(),
                )],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(1),
                OutputData::UnsignedInteger(32),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(2),
                OutputData::UnsignedInteger(3),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(2)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMDataDetectedAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSData".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(904)],
            ),
        ];

        let expected_2 = vec![
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(1),
                OutputData::UnsignedInteger(16),
            ]),
        ];

        assert_eq!(result[..9], expected_1);
        assert_eq!(result[10..], expected_2);
    }

    #[test]
    fn test_parse_text_app() {
        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/AppMessage");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        let result = parser.parse().unwrap();

        println!("\n\nGot data!");
        result.iter().for_each(|item| println!("{item:?}"));

        let expected = vec![
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String("\u{FFFC}".to_string())],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(1),
                OutputData::UnsignedInteger(1),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(3)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMFileTransferGUIDAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "F0B18A15-E9A5-4B18-A38F-685B7B3FF037".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMBaseWritingDirectionAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(-1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_custom_tapback() {
        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/CustomReaction");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        let result = parser.parse().unwrap();

        println!("\n\nGot data!");
        result.iter().for_each(|item| println!("{item:#?}"));

        let expected = vec![
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "Reacted with a sticker to “Like I wonder if the stickers can be reactions ”￼"
                        .to_string(),
                )],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(1),
                OutputData::UnsignedInteger(75),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(2)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMBaseWritingDirectionAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(-1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(2),
                OutputData::UnsignedInteger(1),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(3)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMFileTransferGUIDAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "41C4376E-397E-4C42-84E2-B16F7801F638".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMBaseWritingDirectionAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(-1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_all_styles() {
        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/TextStyles");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        let result = parser.parse().unwrap();

        println!("\n\nGot data!");
        result
            .iter()
            .enumerate()
            .for_each(|(idx, item)| println!("{idx}: {item:?}"));

        let expected = vec![
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "Bold underline italic strikethrough all four".to_string(),
                )],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(1),
                OutputData::UnsignedInteger(4),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(2)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String("__kIMTextBoldAttributeName".to_string())],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(2),
                OutputData::UnsignedInteger(1),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(3),
                OutputData::UnsignedInteger(9),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(2)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMTextUnderlineAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(2),
                OutputData::UnsignedInteger(1),
            ]),
            Archivable::Data(vec![
                OutputData::SignedInteger(4),
                OutputData::UnsignedInteger(6),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(2)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMTextItalicAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(2),
                OutputData::UnsignedInteger(1),
            ]),
            Archivable::Data(vec![
                OutputData::SignedInteger(5),
                OutputData::UnsignedInteger(13),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(2)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMTextStrikethroughAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(2),
                OutputData::UnsignedInteger(5),
            ]),
            Archivable::Data(vec![
                OutputData::SignedInteger(6),
                OutputData::UnsignedInteger(4),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(5)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String("__kIMTextBoldAttributeName".to_string())],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMTextStrikethroughAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMTextUnderlineAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMTextItalicAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_all_styles_single() {
        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/TextStylesSingleRange");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        let result = parser.parse().unwrap();

        println!("\n\nGot data!");
        result
            .iter()
            .enumerate()
            .for_each(|(idx, item)| println!("{idx}: {item:?}"));

        let expected = vec![
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String("Everything".to_string())],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(1),
                OutputData::UnsignedInteger(10),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(5)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String("__kIMTextBoldAttributeName".to_string())],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMTextStrikethroughAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMTextUnderlineAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMTextItalicAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_all_effects() {
        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/TextEffects");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        let result = parser.parse().unwrap();

        println!("\n\nGot data!");
        result
            .iter()
            .enumerate()
            .for_each(|(idx, item)| println!("{idx}: {item:?}"));

        let expected = vec![
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "Big small shake nod explode ripple bloom jitter".to_string(),
                )],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(1),
                OutputData::UnsignedInteger(3),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(2)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMTextEffectAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(5)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(2),
                OutputData::UnsignedInteger(1),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(3),
                OutputData::UnsignedInteger(6),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(2)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMTextEffectAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(11)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(4),
                OutputData::UnsignedInteger(5),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(2)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMTextEffectAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(9)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(3),
                OutputData::UnsignedInteger(1),
            ]),
            Archivable::Data(vec![
                OutputData::SignedInteger(5),
                OutputData::UnsignedInteger(3),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(2)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMTextEffectAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(8)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(3),
                OutputData::UnsignedInteger(1),
            ]),
            Archivable::Data(vec![
                OutputData::SignedInteger(6),
                OutputData::UnsignedInteger(8),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(2)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMTextEffectAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(12)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(7),
                OutputData::UnsignedInteger(6),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(2)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMTextEffectAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(4)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(6),
                OutputData::UnsignedInteger(1),
            ]),
            Archivable::Data(vec![
                OutputData::SignedInteger(8),
                OutputData::UnsignedInteger(5),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(2)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMTextEffectAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(6)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(6),
                OutputData::UnsignedInteger(1),
            ]),
            Archivable::Data(vec![
                OutputData::SignedInteger(9),
                OutputData::UnsignedInteger(6),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(2)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMTextEffectAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(10)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_effects_styles_mixed() {
        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/TextStylesMixed");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        let result = parser.parse().unwrap();

        println!("\n\nGot data!");
        result
            .iter()
            .enumerate()
            .for_each(|(idx, item)| println!("{idx}: {item:?}"));

        let expected = vec![
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "Underline normal jitter normal".to_string(),
                )],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(1),
                OutputData::UnsignedInteger(9),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(3)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMBaseWritingDirectionAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(-1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMTextUnderlineAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(2),
                OutputData::UnsignedInteger(8),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(2)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMBaseWritingDirectionAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(-1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(3),
                OutputData::UnsignedInteger(6),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(3)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMBaseWritingDirectionAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(-1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMTextEffectAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(10)],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(2),
                OutputData::UnsignedInteger(7),
            ]),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_text_audio_transcription() {
        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/Transcription");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        let result = parser.parse().unwrap();

        println!("\n\nGot data!");
        result.iter().for_each(|item| println!("{item:?}"));

        let expected = vec![
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String("\u{FFFC}".to_string())],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(1),
                OutputData::UnsignedInteger(1),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(4)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMFileTransferGUIDAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "4C339597-EBBB-4978-9B87-521C0471A848".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String("IMAudioTranscription".to_string())],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String("This is a test".to_string())],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMBaseWritingDirectionAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(-1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_text_apple_music_lyrics() {
        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/AppleMusicLyrics");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        let result = parser.parse().unwrap();

        println!("\n\nGot data!");
        result.iter().for_each(|item| println!("{item:#?}"));

        let expected = vec![
            Archivable::Object(
                Class {
                    name: "NSMutableString".to_string(),
                    version: 1,
                },
                vec![OutputData::String("https://music.apple.com/us/lyrics/1329891623?ts=11.108&te=16.031&l=en&tk=2.v1.VsuX9f%2BaT1PyrgMgIT7ANQ%3D%3D&itsct=sharing_msg_lyrics&itscg=50401".to_string())],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(1),
                OutputData::UnsignedInteger(145),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(5)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMBaseWritingDirectionAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(-1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![
                    OutputData::String(
                        "__kIMLinkIsRichLinkAttributeName".to_string(),
                    ),
                ],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![
                    OutputData::SignedInteger(
                        1,
                    ),
                ],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![
                    OutputData::String(
                        "__kIMLinkAttributeName".to_string(),
                    ),
                ],
            ),
            Archivable::Object(
                Class {
                    name: "NSURL".to_string(),
                    version: 0,
                },
                vec![
                    OutputData::SignedInteger(
                        0,
                    ),
                ],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![
                    OutputData::String(
                        "https://music.apple.com/us/lyrics/1329891623?ts=11.108&te=16.031&l=en&tk=2.v1.VsuX9f%2BaT1PyrgMgIT7ANQ%3D%3D&itsct=sharing_msg_lyrics&itscg=50401".to_string(),
                    ),
                ],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![
                    OutputData::String(
                        "__kIMMessagePartAttributeName".to_string(),
                    ),
                ],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![
                    OutputData::SignedInteger(
                        0,
                    ),
                ],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![
                    OutputData::String(
                        "__kIMDataDetectedAttributeName".to_string(),
                    ),
                ],
            ),
            Archivable::Object(
                Class {
                    name: "NSData".to_string(),
                    version: 0,
                },
                vec![
                    OutputData::SignedInteger(
                        675,
                    ),
                ],
            ),
            Archivable::Data(
                vec![
                    OutputData::Array(
                        vec![
                            98,
                            112,
                            108,
                            105,
                            115,
                            116,
                            48,
                            48,
                            212,
                            1,
                            2,
                            3,
                            4,
                            5,
                            6,
                            7,
                            12,
                            88,
                            36,
                            118,
                            101,
                            114,
                            115,
                            105,
                            111,
                            110,
                            89,
                            36,
                            97,
                            114,
                            99,
                            104,
                            105,
                            118,
                            101,
                            114,
                            84,
                            36,
                            116,
                            111,
                            112,
                            88,
                            36,
                            111,
                            98,
                            106,
                            101,
                            99,
                            116,
                            115,
                            18,
                            0,
                            1,
                            134,
                            160,
                            95,
                            16,
                            15,
                            78,
                            83,
                            75,
                            101,
                            121,
                            101,
                            100,
                            65,
                            114,
                            99,
                            104,
                            105,
                            118,
                            101,
                            114,
                            210,
                            8,
                            9,
                            10,
                            11,
                            87,
                            118,
                            101,
                            114,
                            115,
                            105,
                            111,
                            110,
                            89,
                            100,
                            100,
                            45,
                            114,
                            101,
                            115,
                            117,
                            108,
                            116,
                            128,
                            11,
                            128,
                            1,
                            172,
                            13,
                            14,
                            28,
                            36,
                            37,
                            38,
                            44,
                            45,
                            46,
                            50,
                            53,
                            57,
                            85,
                            36,
                            110,
                            117,
                            108,
                            108,
                            215,
                            15,
                            16,
                            17,
                            18,
                            19,
                            20,
                            21,
                            22,
                            23,
                            24,
                            25,
                            26,
                            27,
                            26,
                            82,
                            77,
                            83,
                            86,
                            36,
                            99,
                            108,
                            97,
                            115,
                            115,
                            82,
                            65,
                            82,
                            81,
                            84,
                            81,
                            80,
                            82,
                            83,
                            82,
                            82,
                            86,
                            78,
                            128,
                            6,
                            128,
                            10,
                            128,
                            2,
                            128,
                            7,
                            16,
                            1,
                            128,
                            8,
                            212,
                            29,
                            30,
                            31,
                            16,
                            32,
                            33,
                            34,
                            35,
                            95,
                            16,
                            18,
                            78,
                            83,
                            46,
                            114,
                            97,
                            110,
                            103,
                            101,
                            118,
                            97,
                            108,
                            46,
                            108,
                            101,
                            110,
                            103,
                            116,
                            104,
                            95,
                            16,
                            20,
                            78,
                            83,
                            46,
                            114,
                            97,
                            110,
                            103,
                            101,
                            118,
                            97,
                            108,
                            46,
                            108,
                            111,
                            99,
                            97,
                            116,
                            105,
                            111,
                            110,
                            90,
                            78,
                            83,
                            46,
                            115,
                            112,
                            101,
                            99,
                            105,
                            97,
                            108,
                            128,
                            3,
                            128,
                            4,
                            16,
                            4,
                            128,
                            5,
                            16,
                            145,
                            16,
                            0,
                            210,
                            39,
                            40,
                            41,
                            42,
                            90,
                            36,
                            99,
                            108,
                            97,
                            115,
                            115,
                            110,
                            97,
                            109,
                            101,
                            88,
                            36,
                            99,
                            108,
                            97,
                            115,
                            115,
                            101,
                            115,
                            87,
                            78,
                            83,
                            86,
                            97,
                            108,
                            117,
                            101,
                            162,
                            41,
                            43,
                            88,
                            78,
                            83,
                            79,
                            98,
                            106,
                            101,
                            99,
                            116,
                            95,
                            16,
                            145,
                            104,
                            116,
                            116,
                            112,
                            115,
                            58,
                            47,
                            47,
                            109,
                            117,
                            115,
                            105,
                            99,
                            46,
                            97,
                            112,
                            112,
                            108,
                            101,
                            46,
                            99,
                            111,
                            109,
                            47,
                            117,
                            115,
                            47,
                            108,
                            121,
                            114,
                            105,
                            99,
                            115,
                            47,
                            49,
                            51,
                            50,
                            57,
                            56,
                            57,
                            49,
                            54,
                            50,
                            51,
                            63,
                            116,
                            115,
                            61,
                            49,
                            49,
                            46,
                            49,
                            48,
                            56,
                            38,
                            116,
                            101,
                            61,
                            49,
                            54,
                            46,
                            48,
                            51,
                            49,
                            38,
                            108,
                            61,
                            101,
                            110,
                            38,
                            116,
                            107,
                            61,
                            50,
                            46,
                            118,
                            49,
                            46,
                            86,
                            115,
                            117,
                            88,
                            57,
                            102,
                            37,
                            50,
                            66,
                            97,
                            84,
                            49,
                            80,
                            121,
                            114,
                            103,
                            77,
                            103,
                            73,
                            84,
                            55,
                            65,
                            78,
                            81,
                            37,
                            51,
                            68,
                            37,
                            51,
                            68,
                            38,
                            105,
                            116,
                            115,
                            99,
                            116,
                            61,
                            115,
                            104,
                            97,
                            114,
                            105,
                            110,
                            103,
                            95,
                            109,
                            115,
                            103,
                            95,
                            108,
                            121,
                            114,
                            105,
                            99,
                            115,
                            38,
                            105,
                            116,
                            115,
                            99,
                            103,
                            61,
                            53,
                            48,
                            52,
                            48,
                            49,
                            87,
                            72,
                            116,
                            116,
                            112,
                            85,
                            82,
                            76,
                            210,
                            47,
                            16,
                            48,
                            49,
                            90,
                            78,
                            83,
                            46,
                            111,
                            98,
                            106,
                            101,
                            99,
                            116,
                            115,
                            160,
                            128,
                            9,
                            210,
                            39,
                            40,
                            51,
                            52,
                            87,
                            78,
                            83,
                            65,
                            114,
                            114,
                            97,
                            121,
                            162,
                            51,
                            43,
                            210,
                            39,
                            40,
                            54,
                            55,
                            95,
                            16,
                            15,
                            68,
                            68,
                            83,
                            99,
                            97,
                            110,
                            110,
                            101,
                            114,
                            82,
                            101,
                            115,
                            117,
                            108,
                            116,
                            162,
                            56,
                            43,
                            95,
                            16,
                            15,
                            68,
                            68,
                            83,
                            99,
                            97,
                            110,
                            110,
                            101,
                            114,
                            82,
                            101,
                            115,
                            117,
                            108,
                            116,
                            16,
                            1,
                            0,
                            8,
                            0,
                            17,
                            0,
                            26,
                            0,
                            36,
                            0,
                            41,
                            0,
                            50,
                            0,
                            55,
                            0,
                            73,
                            0,
                            78,
                            0,
                            86,
                            0,
                            96,
                            0,
                            98,
                            0,
                            100,
                            0,
                            113,
                            0,
                            119,
                            0,
                            134,
                            0,
                            137,
                            0,
                            144,
                            0,
                            147,
                            0,
                            149,
                            0,
                            151,
                            0,
                            154,
                            0,
                            157,
                            0,
                            159,
                            0,
                            161,
                            0,
                            163,
                            0,
                            165,
                            0,
                            167,
                            0,
                            169,
                            0,
                            178,
                            0,
                            199,
                            0,
                            222,
                            0,
                            233,
                            0,
                            235,
                            0,
                            237,
                            0,
                            239,
                            0,
                            241,
                            0,
                            243,
                            0,
                            245,
                            0,
                            250,
                            1,
                            5,
                            1,
                            14,
                            1,
                            22,
                            1,
                            25,
                            1,
                            34,
                            1,
                            182,
                            1,
                            190,
                            1,
                            195,
                            1,
                            206,
                            1,
                            207,
                            1,
                            209,
                            1,
                            214,
                            1,
                            222,
                            1,
                            225,
                            1,
                            230,
                            1,
                            248,
                            1,
                            251,
                            2,
                            13,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            2,
                            1,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            58,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            2,
                            15,
                        ],
                    ),
                ],
            )
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_multiple_attachment_dictionaries() {
        let typedstream_path = current_dir()
            .unwrap()
            .as_path()
            .join("test_data/typedstream/MultiAttachment");
        let mut file = File::open(typedstream_path).unwrap();
        let mut bytes = vec![];
        file.read_to_end(&mut bytes).unwrap();

        let mut parser = TypedStreamReader::from(&bytes);
        let result = parser.parse().unwrap();

        println!("\n\nGot data!");
        result
            .iter()
            .enumerate()
            .for_each(|(idx, item)| println!("\t{idx}: {item:?}"));

        let expected = vec![
            Archivable::Object(
                Class {
                    name: "NSMutableString".to_string(),
                    version: 1,
                },
                vec![OutputData::String("\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}\u{FFFC}".to_string())],
            ),
            Archivable::Data(vec![
                OutputData::SignedInteger(1),
                OutputData::UnsignedInteger(1),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(3)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMBaseWritingDirectionAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(-1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(0)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMFileTransferGUIDAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "at_0_48B9C973-3466-438C-BE72-E5B498D30772".to_string(),
                )],
            ),

            Archivable::Data(vec![
                OutputData::SignedInteger(2),
                OutputData::UnsignedInteger(1),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(3)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMBaseWritingDirectionAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(-1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMFileTransferGUIDAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "at_1_48B9C973-3466-438C-BE72-E5B498D30772".to_string(),
                )],
            ),

            Archivable::Data(vec![
                OutputData::SignedInteger(3),
                OutputData::UnsignedInteger(1),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(3)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMBaseWritingDirectionAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(-1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(2)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMFileTransferGUIDAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "at_2_48B9C973-3466-438C-BE72-E5B498D30772".to_string(),
                )],
            ),

            Archivable::Data(vec![
                OutputData::SignedInteger(4),
                OutputData::UnsignedInteger(1),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(3)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMBaseWritingDirectionAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(-1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(3)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMFileTransferGUIDAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "at_3_48B9C973-3466-438C-BE72-E5B498D30772".to_string(),
                )],
            ),

            Archivable::Data(vec![
                OutputData::SignedInteger(5),
                OutputData::UnsignedInteger(1),
            ]),
            Archivable::Object(
                Class {
                    name: "NSDictionary".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(3)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMBaseWritingDirectionAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(-1)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMMessagePartAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSNumber".to_string(),
                    version: 0,
                },
                vec![OutputData::SignedInteger(4)],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "__kIMFileTransferGUIDAttributeName".to_string(),
                )],
            ),
            Archivable::Object(
                Class {
                    name: "NSString".to_string(),
                    version: 1,
                },
                vec![OutputData::String(
                    "at_4_48B9C973-3466-438C-BE72-E5B498D30772".to_string(),
                )],
            ),
        ];

        // For brevity, we don't need to check all 18 attachments
        for idx in 0..expected.len() {
            assert_eq!(result[idx], expected[idx]);
        }
    }
}
