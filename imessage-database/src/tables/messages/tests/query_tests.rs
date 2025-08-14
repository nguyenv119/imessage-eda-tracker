#[cfg(test)]
mod exclude_recoverable_tests {
    use std::collections::BTreeSet;

    use crate::{tables::messages::Message, util::query_context::QueryContext};

    #[test]
    fn can_generate_filter_statement_empty() {
        let context = QueryContext::default();

        let statement = Message::generate_filter_statement(&context, false);
        assert_eq!(statement, "");
    }

    #[test]
    fn can_generate_filter_statement_limit() {
        let mut context = QueryContext::default();
        context.set_limit(10);

        let statement = Message::generate_filter_statement(&context, false);
        assert_eq!(statement, "");
    }

    #[test]
    fn can_generate_filter_statement_chat_ids() {
        let mut context = QueryContext::default();
        context.set_selected_chat_ids(BTreeSet::from([1, 2, 3]));

        let statement = Message::generate_filter_statement(&context, false);
        assert_eq!(statement, "WHERE c.chat_id IN (1, 2, 3)");
    }

    #[test]
    fn can_generate_filter_statement_start_end_chat_ids() {
        let mut context = QueryContext::default();
        context.set_limit(10);
        context.set_selected_chat_ids(BTreeSet::from([1, 2, 3]));

        let statement = Message::generate_filter_statement(&context, false);
        assert_eq!(
            statement,
            "WHERE c.chat_id IN (1, 2, 3)"
        );
    }

    #[test]
    fn can_generate_filter_statement_with_empty_chat_ids() {
        let mut context = QueryContext::default();
        context.set_selected_chat_ids(BTreeSet::new());

        let statement = Message::generate_filter_statement(&context, false);
        assert_eq!(statement, "");
    }
}

#[cfg(test)]
mod include_recoverable_tests {
    use std::collections::BTreeSet;

    use crate::{tables::messages::Message, util::query_context::QueryContext};

    #[test]
    fn can_generate_filter_statement_empty() {
        let context = QueryContext::default();

        let statement = Message::generate_filter_statement(&context, true);
        assert_eq!(statement, "");
    }

    #[test]
    fn can_generate_filter_statement_limit() {
        let mut context = QueryContext::default();
        context.set_limit(10);

        let statement = Message::generate_filter_statement(&context, true);
        assert_eq!(statement, "");
    }

    #[test]
    fn can_generate_filter_statement_chat_ids() {
        let mut context = QueryContext::default();
        context.set_selected_chat_ids(BTreeSet::from([1, 2, 3]));

        let statement = Message::generate_filter_statement(&context, true);
        assert_eq!(
            statement,
            "WHERE (c.chat_id IN (1, 2, 3) OR d.chat_id IN (1, 2, 3))"
        );
    }

    #[test]
    fn can_generate_filter_statement_limit_chat_ids() {
        let mut context = QueryContext::default();
        context.set_limit(10);
        context.set_selected_chat_ids(BTreeSet::from([1, 2, 3]));

        let statement = Message::generate_filter_statement(&context, true);
        assert_eq!(
            statement,
            "WHERE (c.chat_id IN (1, 2, 3) OR d.chat_id IN (1, 2, 3))"
        );
    }
}

#[cfg(test)]
mod guid_query_tests {
    use std::env::current_dir;

    use crate::tables::{messages::Message, table::get_connection};

    #[test]
    fn test_cant_query_bad_guid() {
        let db_path = current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .join("imessage-database/test_data/db/test.db");
        let conn = get_connection(&db_path).unwrap();

        let message = Message::from_guid("fake-guid", &conn);

        assert!(message.is_err());
    }

    #[test]
    fn test_can_query_guid() {
        let db_path = current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .join("imessage-database/test_data/db/test.db");
        let conn = get_connection(&db_path).unwrap();

        let mut message =
            Message::from_guid("0355C6E1-D0C8-4212-AA87-DD8AE4FD1203", &conn).unwrap();
        let _ = message.generate_text(&conn);
        println!("{message:#?}");
        assert!(message.components.is_some());
    }

    #[test]
    fn test_empty_guid() {
        let db_path = current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .join("imessage-database/test_data/db/test.db");
        let conn = get_connection(&db_path).unwrap();

        let message = Message::from_guid("", &conn);
        assert!(message.is_err());
    }

    #[test]
    fn test_malformed_guid() {
        let db_path = current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .join("imessage-database/test_data/db/test.db");
        let conn = get_connection(&db_path).unwrap();

        let message = Message::from_guid("not-a-valid-guid-format", &conn);
        assert!(message.is_err());
    }
}

#[cfg(test)]
mod query_string_tests {
    use crate::{
        tables::messages::query_parts,
    };

    #[test]
    fn can_generate_no_filters_16() {
        let query_string = query_parts::ios_16_newer_query(None, Some("LIMIT 10"));
        let expected = "\nSELECT
    rowid, guid, text, service, handle_id, destination_caller_id, subject, date, date_read, date_delivered, is_from_me, is_read, item_type, other_handle, share_status, share_direction, group_title, group_action_type, associated_message_guid, associated_message_type, balloon_bundle_id, expressive_send_style_id, thread_originator_guid, thread_originator_part, date_edited, associated_message_emoji,
    c.chat_id,
    (SELECT COUNT(*) FROM message_attachment_join a WHERE m.ROWID = a.message_id) as num_attachments,
    d.chat_id as deleted_from,
    (SELECT COUNT(*) FROM message m2 WHERE m2.thread_originator_guid = m.guid) as num_replies
FROM
    message as m
LEFT JOIN chat_message_join as c ON m.ROWID = c.message_id
LEFT JOIN chat_recoverable_message_join as d ON m.ROWID = d.message_id

ORDER BY
    m.date DESC
LIMIT 10;"; 
        assert_eq!(query_string, expected);
    }

    #[test]
    fn can_generate_filters_16() {
        let query_string = query_parts::ios_16_newer_query(Some("WHERE m.guid = \"fake\""), Some("LIMIT 10"));
        let expected = "\nSELECT
    rowid, guid, text, service, handle_id, destination_caller_id, subject, date, date_read, date_delivered, is_from_me, is_read, item_type, other_handle, share_status, share_direction, group_title, group_action_type, associated_message_guid, associated_message_type, balloon_bundle_id, expressive_send_style_id, thread_originator_guid, thread_originator_part, date_edited, associated_message_emoji,
    c.chat_id,
    (SELECT COUNT(*) FROM message_attachment_join a WHERE m.ROWID = a.message_id) as num_attachments,
    d.chat_id as deleted_from,
    (SELECT COUNT(*) FROM message m2 WHERE m2.thread_originator_guid = m.guid) as num_replies
FROM
    message as m
LEFT JOIN chat_message_join as c ON m.ROWID = c.message_id
LEFT JOIN chat_recoverable_message_join as d ON m.ROWID = d.message_id
WHERE m.guid = \"fake\"
ORDER BY
    m.date DESC
LIMIT 10;";
        assert_eq!(query_string, expected);
    }

    #[test]
    fn can_generate_no_filters_14_15() {
        let query_string = query_parts::ios_14_15_query(None, None);
        let expected = "\nSELECT
    *,
    c.chat_id,
    (SELECT COUNT(*) FROM message_attachment_join a WHERE m.ROWID = a.message_id) as num_attachments,
    NULL as deleted_from,
    (SELECT COUNT(*) FROM message m2 WHERE m2.thread_originator_guid = m.guid) as num_replies
FROM
    message as m
LEFT JOIN chat_message_join as c ON m.ROWID = c.message_id

ORDER BY
    m.date DESC
;";
        println!("{query_string}");
        assert_eq!(query_string, expected);
    }

    #[test]
    fn can_generate_filters_14_15() {
        let query_string = query_parts::ios_14_15_query(Some("WHERE m.guid = \"fake\""), Some("LIMIT 10"));
        let expected = "\nSELECT
    *,
    c.chat_id,
    (SELECT COUNT(*) FROM message_attachment_join a WHERE m.ROWID = a.message_id) as num_attachments,
    NULL as deleted_from,
    (SELECT COUNT(*) FROM message m2 WHERE m2.thread_originator_guid = m.guid) as num_replies
FROM
    message as m
LEFT JOIN chat_message_join as c ON m.ROWID = c.message_id
WHERE m.guid = \"fake\"
ORDER BY
    m.date DESC
LIMIT 10;";
        assert_eq!(query_string, expected);
    }

    #[test]
    fn can_generate_no_filters_13() {
        let query_string = query_parts::ios_13_older_query(None, Some("LIMIT 10"));
        let expected = "\nSELECT
    *,
    c.chat_id,
    (SELECT COUNT(*) FROM message_attachment_join a WHERE m.ROWID = a.message_id) as num_attachments,
    NULL as deleted_from,
    0 as num_replies
FROM
    message as m
LEFT JOIN chat_message_join as c ON m.ROWID = c.message_id

ORDER BY
    m.date DESC
LIMIT 10;";
        println!("{query_string}");
        assert_eq!(query_string, expected);
    }

    #[test]
    fn can_generate_filters_13() {
        let query_string = query_parts::ios_13_older_query(Some("WHERE m.guid = \"fake\""), Some("LIMIT 10"));
        let expected = "\nSELECT
    *,
    c.chat_id,
    (SELECT COUNT(*) FROM message_attachment_join a WHERE m.ROWID = a.message_id) as num_attachments,
    NULL as deleted_from,
    0 as num_replies
FROM
    message as m
LEFT JOIN chat_message_join as c ON m.ROWID = c.message_id
WHERE m.guid = \"fake\"
ORDER BY
    m.date DESC
LIMIT 10;";
        assert_eq!(query_string, expected);
    }
}
