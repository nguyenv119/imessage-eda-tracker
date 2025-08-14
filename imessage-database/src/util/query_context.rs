/*!
 Contains logic for handling query filter configurations.
*/
use std::collections::BTreeSet;

#[derive(Debug, Default, PartialEq, Eq)]
/// Represents filter configurations for a SQL query.
pub struct QueryContext {
    pub limit: Option<i32>,
    pub selected_handle_ids: Option<BTreeSet<i32>>,
    /// Selected chat IDs
    pub selected_chat_ids: Option<BTreeSet<i32>>,
}

impl QueryContext {
    /// Populate a [`QueryContext`] with limit on the number of messages retrieved
    /// # Example:
    ///
    /// ```
    /// use imessage_database::util::query_context::QueryContext;
    ///
    /// let mut context = QueryContext::default();
    /// context.set_limit(2);
    /// ```
    pub fn set_limit(&mut self, limit: i32) {
        self.limit = Some(limit);
    }

    /// Populate a [`QueryContext`] with a list of handle IDs to select
    ///
    /// # Example:
    ///
    /// ```
    /// use std::collections::BTreeSet;
    /// use imessage_database::util::query_context::QueryContext;
    ///
    /// let mut context = QueryContext::default();
    /// context.set_selected_handle_ids(BTreeSet::from([1, 2, 3]));
    /// ```
    pub fn set_selected_handle_ids(&mut self, selected_handle_ids: BTreeSet<i32>) {
        self.selected_handle_ids = (!selected_handle_ids.is_empty()).then_some(selected_handle_ids);
    }

    /// Populate a [`QueryContext`] with a list of chat IDs to select
    ///
    /// # Example:
    ///
    /// ```
    /// use std::collections::BTreeSet;
    /// use imessage_database::util::query_context::QueryContext;
    ///
    /// let mut context = QueryContext::default();
    /// context.set_selected_chat_ids(BTreeSet::from([1, 2, 3]));
    /// ```
    pub fn set_selected_chat_ids(&mut self, selected_chat_ids: BTreeSet<i32>) {
        self.selected_chat_ids = (!selected_chat_ids.is_empty()).then_some(selected_chat_ids);
    }

    /// Determine if the current `QueryContext` has any filters present
    ///
    /// # Example:
    ///
    /// ```
    /// use imessage_database::util::query_context::QueryContext;
    ///
    /// let mut context = QueryContext::default();
    /// assert!(!context.has_filters());
    /// context.set_limit(10);
    /// assert!(context.has_filters());
    /// ```
    #[must_use]
    pub fn has_filters(&self) -> bool {
        self.limit.is_some()
            || self.selected_chat_ids.is_some()
            || self.selected_handle_ids.is_some()
    }
}

#[cfg(test)]
mod use_tests {
    use crate::util::{
        query_context::QueryContext,
    };

    #[test]
    fn can_create() {
        let context = QueryContext::default();
        assert!(context.limit.is_none());
        assert!(!context.has_filters());
    }

    #[test]
    fn can_create_limit() {
        let mut context = QueryContext::default();
        context.set_limit(1);

        assert_eq!(context.limit, Some(1));
        assert!(context.limit.is_some());
        assert!(context.has_filters());
    }

}

#[cfg(test)]
mod id_tests {
    use std::collections::BTreeSet;

    use crate::util::query_context::QueryContext;

    #[test]
    fn test_can_set_selected_chat_ids() {
        let mut qc = QueryContext::default();
        qc.set_selected_chat_ids(BTreeSet::from([1, 2, 3]));

        assert_eq!(qc.selected_chat_ids, Some(BTreeSet::from([1, 2, 3])));
        assert!(qc.has_filters());
    }

    #[test]
    fn test_can_set_selected_chat_ids_empty() {
        let mut qc = QueryContext::default();
        qc.set_selected_chat_ids(BTreeSet::new());

        assert_eq!(qc.selected_chat_ids, None);
        assert!(!qc.has_filters());
    }

    #[test]
    fn test_can_overwrite_selected_chat_ids_empty() {
        let mut qc = QueryContext::default();
        qc.set_selected_chat_ids(BTreeSet::from([1, 2, 3]));
        qc.set_selected_chat_ids(BTreeSet::new());

        assert_eq!(qc.selected_chat_ids, None);
        assert!(!qc.has_filters());
    }

    #[test]
    fn test_can_set_selected_handle_ids() {
        let mut qc = QueryContext::default();
        qc.set_selected_handle_ids(BTreeSet::from([1, 2, 3]));

        assert_eq!(qc.selected_handle_ids, Some(BTreeSet::from([1, 2, 3])));
        assert!(qc.has_filters());
    }

    #[test]
    fn test_can_set_selected_handle_ids_empty() {
        let mut qc = QueryContext::default();
        qc.set_selected_handle_ids(BTreeSet::new());

        assert_eq!(qc.selected_handle_ids, None);
        assert!(!qc.has_filters());
    }

    #[test]
    fn test_can_overwrite_selected_handle_ids_empty() {
        let mut qc = QueryContext::default();
        qc.set_selected_handle_ids(BTreeSet::from([1, 2, 3]));
        qc.set_selected_handle_ids(BTreeSet::new());

        assert_eq!(qc.selected_handle_ids, None);
        assert!(!qc.has_filters());
    }
}
