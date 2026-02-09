// Re-export core sorting types for use in CLI
// The sorting logic and FromStr implementations are in hlavi-core
pub use hlavi_core::{sort_tasks, SortField, SortOrder};

#[cfg(test)]
mod tests {
    use super::*;
    use hlavi_core::domain::task::TaskId;
    use hlavi_core::Task;

    #[test]
    fn test_sort_field_parsing() {
        assert!(matches!("id".parse::<SortField>(), Ok(SortField::Id)));
        assert!(matches!("title".parse::<SortField>(), Ok(SortField::Title)));
        assert!(matches!(
            "status".parse::<SortField>(),
            Ok(SortField::Status)
        ));
        assert!("invalid".parse::<SortField>().is_err());
    }

    #[test]
    fn test_sort_order_parsing() {
        assert!(matches!(
            "asc".parse::<SortOrder>(),
            Ok(SortOrder::Ascending)
        ));
        assert!(matches!(
            "desc".parse::<SortOrder>(),
            Ok(SortOrder::Descending)
        ));
        assert!("invalid".parse::<SortOrder>().is_err());
    }

    #[test]
    fn test_case_insensitive_parsing() {
        assert!(matches!("ID".parse::<SortField>(), Ok(SortField::Id)));
        assert!(matches!("Title".parse::<SortField>(), Ok(SortField::Title)));
        assert!(matches!(
            "ASC".parse::<SortOrder>(),
            Ok(SortOrder::Ascending)
        ));
        assert!(matches!(
            "DESC".parse::<SortOrder>(),
            Ok(SortOrder::Descending)
        ));
    }

    #[test]
    fn test_sort_integration() {
        // Test that we can use the re-exported sort_tasks function
        let mut tasks = vec![
            Task::new(TaskId::new(3), "C".to_string()),
            Task::new(TaskId::new(1), "A".to_string()),
            Task::new(TaskId::new(2), "B".to_string()),
        ];

        sort_tasks(&mut tasks, SortField::Id, SortOrder::Ascending);

        assert_eq!(tasks[0].id.as_str(), "HLA1");
        assert_eq!(tasks[1].id.as_str(), "HLA2");
        assert_eq!(tasks[2].id.as_str(), "HLA3");
    }
}
