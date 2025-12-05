//! Test utilities module providing shared helpers, mock data generators,
//! and common test infrastructure for LockSmith tests.
//!
//! This module can be imported by integration tests to reduce duplication
//! and ensure consistent test data across the test suite.

use std::collections::HashSet;

// ============================================================================
// Mock Data Structures
// ============================================================================

/// A simplified version of FileLockInfo for testing purposes.
/// This mirrors the structure in main.rs without requiring the full dependency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MockFileLockInfo {
    pub file_id: u32,
    pub server_name: String,
    pub file_path: String,
    pub file_name: String,
    pub username: String,
    pub permissions: u32,
    pub num_locks: u32,
}

impl MockFileLockInfo {
    /// Creates a new MockFileLockInfo with the given parameters.
    pub fn new(
        file_id: u32,
        file_name: &str,
        file_path: &str,
        username: &str,
        num_locks: u32,
    ) -> Self {
        Self {
            file_id,
            server_name: "LOCAL".to_string(),
            file_path: file_path.to_string(),
            file_name: file_name.to_string(),
            username: username.to_string(),
            permissions: 1,
            num_locks,
        }
    }

    /// Creates a minimal test lock with just an ID.
    pub fn minimal(file_id: u32) -> Self {
        Self::new(
            file_id,
            &format!("file{}.txt", file_id),
            &format!(r"C:\test\file{}.txt", file_id),
            "testuser",
            1,
        )
    }
}

// ============================================================================
// Test Data Generators
// ============================================================================

/// Generates a vector of mock file locks with realistic data.
/// Useful for testing filtering, sorting, and selection operations.
pub fn generate_mock_locks(count: usize) -> Vec<MockFileLockInfo> {
    (0..count)
        .map(|i| MockFileLockInfo {
            file_id: i as u32,
            server_name: "LOCAL".to_string(),
            file_path: format!(
                r"C:\Data\Folder{}\Subfolder{}\file{}.dat",
                i / 1000,
                i / 100,
                i
            ),
            file_name: format!("file{}.dat", i),
            username: format!("DOMAIN\\user{}", i % 100),
            permissions: (i % 4) as u32,
            num_locks: ((i % 20) + 1) as u32,
        })
        .collect()
}

/// Generates a set of realistic file locks representing a typical Windows system.
pub fn generate_realistic_lock_set() -> Vec<MockFileLockInfo> {
    vec![
        MockFileLockInfo::new(
            1001,
            "document.docx",
            r"C:\Users\JohnDoe\Documents\document.docx",
            "DOMAIN\\JohnDoe",
            1,
        ),
        MockFileLockInfo::new(
            1002,
            "spreadsheet.xlsx",
            r"C:\Users\JohnDoe\Documents\spreadsheet.xlsx",
            "DOMAIN\\JohnDoe",
            2,
        ),
        MockFileLockInfo::new(
            1003,
            "database.mdb",
            r"C:\Shared\Database\database.mdb",
            "SYSTEM",
            5,
        ),
        MockFileLockInfo::new(
            1004,
            "config.json",
            r"C:\Program Files\MyApp\config.json",
            "NT AUTHORITY\\SYSTEM",
            1,
        ),
        MockFileLockInfo::new(
            1005,
            "logfile.log",
            r"C:\Logs\Application\logfile.log",
            "NETWORK SERVICE",
            3,
        ),
        MockFileLockInfo::new(
            1006,
            "temp.tmp",
            r"C:\Users\JaneDoe\AppData\Local\Temp\temp.tmp",
            "DOMAIN\\JaneDoe",
            1,
        ),
        MockFileLockInfo::new(
            1007,
            "report.pdf",
            r"\\FileServer\Shared\Reports\report.pdf",
            "DOMAIN\\JohnDoe",
            1,
        ),
        MockFileLockInfo::new(
            1008,
            "backup.zip",
            r"D:\Backups\Weekly\backup.zip",
            "ADMIN",
            2,
        ),
    ]
}

/// Generates locks with special characters for edge case testing.
pub fn generate_special_char_locks() -> Vec<MockFileLockInfo> {
    vec![
        MockFileLockInfo::new(2001, "file[1].txt", r"C:\Data\file[1].txt", "user", 1),
        MockFileLockInfo::new(2002, "file(2).txt", r"C:\Data\file(2).txt", "user", 1),
        MockFileLockInfo::new(2003, "file{3}.txt", r"C:\Data\file{3}.txt", "user", 1),
        MockFileLockInfo::new(2004, "file&4.txt", r"C:\Data\file&4.txt", "user", 1),
        MockFileLockInfo::new(2005, "file@5.txt", r"C:\Data\file@5.txt", "user", 1),
    ]
}

/// Generates locks with Unicode characters for internationalization testing.
pub fn generate_unicode_locks() -> Vec<MockFileLockInfo> {
    vec![
        MockFileLockInfo::new(3001, "文档.txt", r"C:\中文\文档.txt", "用户", 1),
        MockFileLockInfo::new(
            3002,
            "ファイル.txt",
            r"C:\日本語\ファイル.txt",
            "ユーザー",
            1,
        ),
        MockFileLockInfo::new(3003, "файл.txt", r"C:\Русский\файл.txt", "пользователь", 1),
        MockFileLockInfo::new(3004, "αβγδ.txt", r"C:\Ελληνικά\αβγδ.txt", "χρήστης", 1),
        MockFileLockInfo::new(3005, "مستند.txt", r"C:\العربية\مستند.txt", "مستخدم", 1),
    ]
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Extracts the filename from a Windows path.
/// Mirrors the implementation in main.rs for test verification.
pub fn extract_filename(path: &str) -> String {
    path.rsplit('\\').next().unwrap_or(path).to_string()
}

/// Filters locks by a search query (case-insensitive).
/// Searches in file_name, file_path, and username fields.
pub fn filter_locks(locks: &[MockFileLockInfo], query: &str) -> Vec<MockFileLockInfo> {
    if query.is_empty() {
        return locks.to_vec();
    }

    let query_lower = query.to_lowercase();
    locks
        .iter()
        .filter(|lock| {
            lock.file_name.to_lowercase().contains(&query_lower)
                || lock.file_path.to_lowercase().contains(&query_lower)
                || lock.username.to_lowercase().contains(&query_lower)
        })
        .cloned()
        .collect()
}

/// Sort column options for testing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SortColumn {
    FileName,
    FilePath,
    Username,
    NumLocks,
}

/// Sort direction options for testing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    /// Toggles the sort direction.
    pub fn toggle(&self) -> Self {
        match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        }
    }

    /// Returns the arrow character for the direction.
    pub fn arrow(&self) -> &'static str {
        match self {
            SortDirection::Ascending => " ↑",
            SortDirection::Descending => " ↓",
        }
    }
}

/// Sorts locks by the specified column and direction.
pub fn sort_locks(
    mut locks: Vec<MockFileLockInfo>,
    column: SortColumn,
    direction: SortDirection,
) -> Vec<MockFileLockInfo> {
    locks.sort_by(|a, b| {
        let cmp = match column {
            SortColumn::FileName => a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()),
            SortColumn::FilePath => a.file_path.to_lowercase().cmp(&b.file_path.to_lowercase()),
            SortColumn::Username => a.username.to_lowercase().cmp(&b.username.to_lowercase()),
            SortColumn::NumLocks => a.num_locks.cmp(&b.num_locks),
        };

        match direction {
            SortDirection::Ascending => cmp,
            SortDirection::Descending => cmp.reverse(),
        }
    });
    locks
}

/// Selects all file IDs from the given locks.
pub fn select_all_ids(locks: &[MockFileLockInfo]) -> HashSet<u32> {
    locks.iter().map(|lock| lock.file_id).collect()
}

// ============================================================================
// Assertion Helpers
// ============================================================================

/// Asserts that two vectors of locks have the same file IDs (in any order).
pub fn assert_same_lock_ids(actual: &[MockFileLockInfo], expected: &[MockFileLockInfo]) {
    let actual_ids: HashSet<u32> = actual.iter().map(|l| l.file_id).collect();
    let expected_ids: HashSet<u32> = expected.iter().map(|l| l.file_id).collect();
    assert_eq!(
        actual_ids, expected_ids,
        "Lock IDs do not match.\nActual: {:?}\nExpected: {:?}",
        actual_ids, expected_ids
    );
}

/// Asserts that locks are sorted by filename in ascending order.
pub fn assert_sorted_by_filename_asc(locks: &[MockFileLockInfo]) {
    for i in 0..locks.len().saturating_sub(1) {
        assert!(
            locks[i].file_name.to_lowercase() <= locks[i + 1].file_name.to_lowercase(),
            "Locks not sorted by filename at index {}: {} > {}",
            i,
            locks[i].file_name,
            locks[i + 1].file_name
        );
    }
}

/// Asserts that locks are sorted by num_locks in descending order.
pub fn assert_sorted_by_num_locks_desc(locks: &[MockFileLockInfo]) {
    for i in 0..locks.len().saturating_sub(1) {
        assert!(
            locks[i].num_locks >= locks[i + 1].num_locks,
            "Locks not sorted by num_locks (desc) at index {}: {} < {}",
            i,
            locks[i].num_locks,
            locks[i + 1].num_locks
        );
    }
}

/// Asserts that all locks match the given username filter.
pub fn assert_all_match_username(locks: &[MockFileLockInfo], username_filter: &str) {
    let filter_lower = username_filter.to_lowercase();
    for lock in locks {
        assert!(
            lock.username.to_lowercase().contains(&filter_lower),
            "Lock {} does not match username filter '{}': actual username is '{}'",
            lock.file_id,
            username_filter,
            lock.username
        );
    }
}

// ============================================================================
// Mock State Management
// ============================================================================

/// A mock application state for testing state transitions.
pub struct MockAppState {
    pub locks: Vec<MockFileLockInfo>,
    pub selected_ids: HashSet<u32>,
    pub search_query: String,
    pub sort_column: Option<SortColumn>,
    pub sort_direction: SortDirection,
    pub is_loading: bool,
    pub dark_mode: bool,
}

impl Default for MockAppState {
    fn default() -> Self {
        Self {
            locks: Vec::new(),
            selected_ids: HashSet::new(),
            search_query: String::new(),
            sort_column: None,
            sort_direction: SortDirection::Ascending,
            is_loading: false,
            dark_mode: true,
        }
    }
}

impl MockAppState {
    /// Creates a new mock app state with the given locks.
    pub fn with_locks(locks: Vec<MockFileLockInfo>) -> Self {
        Self {
            locks,
            ..Default::default()
        }
    }

    /// Returns filtered and sorted locks based on current state.
    pub fn get_visible_locks(&self) -> Vec<MockFileLockInfo> {
        let filtered = filter_locks(&self.locks, &self.search_query);

        if let Some(column) = self.sort_column {
            sort_locks(filtered, column, self.sort_direction)
        } else {
            filtered
        }
    }

    /// Selects all visible locks.
    pub fn select_all_visible(&mut self) {
        let visible = self.get_visible_locks();
        self.selected_ids = select_all_ids(&visible);
    }

    /// Clears the current selection.
    pub fn clear_selection(&mut self) {
        self.selected_ids.clear();
    }

    /// Toggles selection for a specific file ID.
    pub fn toggle_selection(&mut self, file_id: u32) {
        if self.selected_ids.contains(&file_id) {
            self.selected_ids.remove(&file_id);
        } else {
            self.selected_ids.insert(file_id);
        }
    }

    /// Handles a sort column click (sets column or toggles direction).
    pub fn handle_sort(&mut self, column: SortColumn) {
        if self.sort_column == Some(column) {
            self.sort_direction = self.sort_direction.toggle();
        } else {
            self.sort_column = Some(column);
            self.sort_direction = SortDirection::Ascending;
        }
    }
}

// ============================================================================
// Test Module for Self-Testing
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_file_lock_info_new() {
        let lock = MockFileLockInfo::new(1, "test.txt", r"C:\test.txt", "user", 5);
        assert_eq!(lock.file_id, 1);
        assert_eq!(lock.file_name, "test.txt");
        assert_eq!(lock.num_locks, 5);
    }

    #[test]
    fn test_mock_file_lock_info_minimal() {
        let lock = MockFileLockInfo::minimal(42);
        assert_eq!(lock.file_id, 42);
        assert!(lock.file_name.contains("42"));
    }

    #[test]
    fn test_generate_mock_locks() {
        let locks = generate_mock_locks(100);
        assert_eq!(locks.len(), 100);

        // Check all IDs are unique
        let ids: HashSet<u32> = locks.iter().map(|l| l.file_id).collect();
        assert_eq!(ids.len(), 100);
    }

    #[test]
    fn test_generate_realistic_lock_set() {
        let locks = generate_realistic_lock_set();
        assert!(!locks.is_empty());

        // Check for expected file types
        let has_docx = locks.iter().any(|l| l.file_name.ends_with(".docx"));
        let has_xlsx = locks.iter().any(|l| l.file_name.ends_with(".xlsx"));
        assert!(has_docx);
        assert!(has_xlsx);
    }

    #[test]
    fn test_extract_filename() {
        assert_eq!(extract_filename(r"C:\Users\file.txt"), "file.txt");
        assert_eq!(extract_filename("simple.txt"), "simple.txt");
        assert_eq!(extract_filename(""), "");
    }

    #[test]
    fn test_filter_locks_empty_query() {
        let locks = generate_mock_locks(10);
        let filtered = filter_locks(&locks, "");
        assert_eq!(filtered.len(), 10);
    }

    #[test]
    fn test_filter_locks_with_match() {
        let locks = generate_realistic_lock_set();
        let filtered = filter_locks(&locks, "JohnDoe");
        assert!(!filtered.is_empty());
        assert!(filtered
            .iter()
            .all(|l| l.file_name.to_lowercase().contains("johndoe")
                || l.file_path.to_lowercase().contains("johndoe")
                || l.username.to_lowercase().contains("johndoe")));
    }

    #[test]
    fn test_sort_direction_toggle() {
        let dir = SortDirection::Ascending;
        assert_eq!(dir.toggle(), SortDirection::Descending);
        assert_eq!(dir.toggle().toggle(), SortDirection::Ascending);
    }

    #[test]
    fn test_sort_locks() {
        let locks = generate_realistic_lock_set();
        let sorted = sort_locks(locks, SortColumn::NumLocks, SortDirection::Descending);
        assert_sorted_by_num_locks_desc(&sorted);
    }

    #[test]
    fn test_mock_app_state_default() {
        let state = MockAppState::default();
        assert!(state.locks.is_empty());
        assert!(state.selected_ids.is_empty());
        assert!(state.dark_mode);
    }

    #[test]
    fn test_mock_app_state_select_all_visible() {
        let mut state = MockAppState::with_locks(generate_mock_locks(10));
        state.select_all_visible();
        assert_eq!(state.selected_ids.len(), 10);
    }

    #[test]
    fn test_mock_app_state_handle_sort() {
        let mut state = MockAppState::default();

        state.handle_sort(SortColumn::FileName);
        assert_eq!(state.sort_column, Some(SortColumn::FileName));
        assert_eq!(state.sort_direction, SortDirection::Ascending);

        state.handle_sort(SortColumn::FileName);
        assert_eq!(state.sort_direction, SortDirection::Descending);

        state.handle_sort(SortColumn::NumLocks);
        assert_eq!(state.sort_column, Some(SortColumn::NumLocks));
        assert_eq!(state.sort_direction, SortDirection::Ascending);
    }
}
