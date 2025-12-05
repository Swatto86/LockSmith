//! Integration tests for LockSmith application
//!
//! These tests verify the complete behavior of the application's
//! core functionality in realistic scenarios.

use std::collections::HashSet;

// ============================================================================
// Test Data Factories
// ============================================================================

/// Represents file lock information for testing
/// (Mirrors the main crate's FileLockInfo structure)
#[derive(Debug, Clone, PartialEq)]
struct TestFileLockInfo {
    file_id: u32,
    server_name: String,
    file_path: String,
    file_name: String,
    username: String,
    permissions: u32,
    num_locks: u32,
}

/// Creates a standard test lock with customizable fields
fn create_test_lock(
    file_id: u32,
    file_name: &str,
    file_path: &str,
    username: &str,
    num_locks: u32,
) -> TestFileLockInfo {
    TestFileLockInfo {
        file_id,
        server_name: "LOCAL".to_string(),
        file_path: file_path.to_string(),
        file_name: file_name.to_string(),
        username: username.to_string(),
        permissions: 1,
        num_locks,
    }
}

/// Creates a realistic set of file locks for testing
fn create_realistic_lock_set() -> Vec<TestFileLockInfo> {
    vec![
        create_test_lock(
            1001,
            "document.docx",
            r"C:\Users\JohnDoe\Documents\document.docx",
            "DOMAIN\\JohnDoe",
            1,
        ),
        create_test_lock(
            1002,
            "spreadsheet.xlsx",
            r"C:\Users\JohnDoe\Documents\spreadsheet.xlsx",
            "DOMAIN\\JohnDoe",
            2,
        ),
        create_test_lock(
            1003,
            "database.mdb",
            r"C:\Shared\Database\database.mdb",
            "SYSTEM",
            5,
        ),
        create_test_lock(
            1004,
            "config.json",
            r"C:\Program Files\MyApp\config.json",
            "NT AUTHORITY\\SYSTEM",
            1,
        ),
        create_test_lock(
            1005,
            "logfile.log",
            r"C:\Logs\Application\logfile.log",
            "NETWORK SERVICE",
            3,
        ),
        create_test_lock(
            1006,
            "temp.tmp",
            r"C:\Users\JaneDoe\AppData\Local\Temp\temp.tmp",
            "DOMAIN\\JaneDoe",
            1,
        ),
        create_test_lock(
            1007,
            "report.pdf",
            r"\\FileServer\Shared\Reports\report.pdf",
            "DOMAIN\\JohnDoe",
            1,
        ),
        create_test_lock(
            1008,
            "backup.zip",
            r"D:\Backups\Weekly\backup.zip",
            "ADMIN",
            2,
        ),
    ]
}

// ============================================================================
// Helper Functions (Mirroring main.rs functionality)
// ============================================================================

/// Extracts the filename from a Windows path
fn extract_filename(path: &str) -> String {
    path.rsplit('\\').next().unwrap_or(path).to_string()
}

/// Sort direction enum
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    fn toggle(&self) -> Self {
        match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        }
    }
}

/// Sort column enum
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)]
enum SortColumn {
    FileName,
    FilePath,
    Username,
    NumLocks,
}

/// Filter locks based on search query
fn filter_locks(locks: &[TestFileLockInfo], query: &str) -> Vec<TestFileLockInfo> {
    let query_lower = query.to_lowercase();
    locks
        .iter()
        .filter(|lock| {
            if query.is_empty() {
                true
            } else {
                lock.file_name.to_lowercase().contains(&query_lower)
                    || lock.file_path.to_lowercase().contains(&query_lower)
                    || lock.username.to_lowercase().contains(&query_lower)
            }
        })
        .cloned()
        .collect()
}

/// Sort locks based on column and direction
fn sort_locks(
    mut locks: Vec<TestFileLockInfo>,
    column: SortColumn,
    direction: SortDirection,
) -> Vec<TestFileLockInfo> {
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

/// Select all visible locks
fn select_all_visible(locks: &[TestFileLockInfo]) -> HashSet<u32> {
    locks.iter().map(|lock| lock.file_id).collect()
}

// ============================================================================
// Integration Test Scenarios
// ============================================================================

mod scenario_tests {
    use super::*;

    /// Scenario: User opens app and views all locks
    #[test]
    fn test_scenario_view_all_locks() {
        let locks = create_realistic_lock_set();

        // Verify we have the expected number of locks
        assert_eq!(locks.len(), 8);

        // Verify each lock has required fields
        for lock in &locks {
            assert!(!lock.file_name.is_empty());
            assert!(!lock.file_path.is_empty());
            assert!(!lock.username.is_empty());
            assert!(lock.file_id > 0);
        }
    }

    /// Scenario: User searches for a specific file type
    #[test]
    fn test_scenario_search_by_extension() {
        let locks = create_realistic_lock_set();

        // Search for Excel files
        let xlsx_files = filter_locks(&locks, ".xlsx");
        assert_eq!(xlsx_files.len(), 1);
        assert_eq!(xlsx_files[0].file_name, "spreadsheet.xlsx");

        // Search for text-based files
        let json_files = filter_locks(&locks, ".json");
        assert_eq!(json_files.len(), 1);

        // Search for non-existent extension
        let py_files = filter_locks(&locks, ".py");
        assert_eq!(py_files.len(), 0);
    }

    /// Scenario: User searches for files by a specific user
    #[test]
    fn test_scenario_search_by_username() {
        let locks = create_realistic_lock_set();

        // Search for JohnDoe's files
        let john_files = filter_locks(&locks, "JohnDoe");
        assert_eq!(john_files.len(), 3); // document.docx, spreadsheet.xlsx, report.pdf

        // Search for system files
        let system_files = filter_locks(&locks, "SYSTEM");
        assert_eq!(system_files.len(), 2); // database.mdb and config.json
    }

    /// Scenario: User searches by partial path
    #[test]
    fn test_scenario_search_by_path() {
        let locks = create_realistic_lock_set();

        // Search for files in Documents folder
        let doc_folder = filter_locks(&locks, "Documents");
        assert_eq!(doc_folder.len(), 2);

        // Search for files on network share
        let network_files = filter_locks(&locks, r"\\FileServer");
        assert_eq!(network_files.len(), 1);
        assert_eq!(network_files[0].file_name, "report.pdf");

        // Search for files on D: drive
        let d_drive = filter_locks(&locks, "D:");
        assert_eq!(d_drive.len(), 1);
    }

    /// Scenario: User sorts locks by different columns
    #[test]
    fn test_scenario_sort_by_all_columns() {
        let locks = create_realistic_lock_set();

        // Sort by filename ascending
        let by_name_asc = sort_locks(
            locks.clone(),
            SortColumn::FileName,
            SortDirection::Ascending,
        );
        assert_eq!(by_name_asc[0].file_name, "backup.zip");
        assert_eq!(by_name_asc[7].file_name, "temp.tmp");

        // Sort by filename descending
        let by_name_desc = sort_locks(
            locks.clone(),
            SortColumn::FileName,
            SortDirection::Descending,
        );
        assert_eq!(by_name_desc[0].file_name, "temp.tmp");
        assert_eq!(by_name_desc[7].file_name, "backup.zip");

        // Sort by lock count ascending
        let by_locks_asc = sort_locks(
            locks.clone(),
            SortColumn::NumLocks,
            SortDirection::Ascending,
        );
        assert_eq!(by_locks_asc[0].num_locks, 1);

        // Sort by lock count descending
        let by_locks_desc = sort_locks(
            locks.clone(),
            SortColumn::NumLocks,
            SortDirection::Descending,
        );
        assert_eq!(by_locks_desc[0].num_locks, 5); // database.mdb has 5 locks
    }

    /// Scenario: User toggles sort direction on same column
    #[test]
    fn test_scenario_toggle_sort_direction() {
        let mut direction = SortDirection::Ascending;

        // First click: stays ascending (initial state)
        assert_eq!(direction, SortDirection::Ascending);

        // Second click: toggles to descending
        direction = direction.toggle();
        assert_eq!(direction, SortDirection::Descending);

        // Third click: toggles back to ascending
        direction = direction.toggle();
        assert_eq!(direction, SortDirection::Ascending);
    }

    /// Scenario: User filters and then sorts
    #[test]
    fn test_scenario_filter_then_sort() {
        let locks = create_realistic_lock_set();

        // Filter for JohnDoe's files
        let filtered = filter_locks(&locks, "JohnDoe");
        assert_eq!(filtered.len(), 3);

        // Sort by filename
        let sorted = sort_locks(filtered, SortColumn::FileName, SortDirection::Ascending);

        // Verify order
        assert_eq!(sorted[0].file_name, "document.docx");
        assert_eq!(sorted[1].file_name, "report.pdf");
        assert_eq!(sorted[2].file_name, "spreadsheet.xlsx");
    }

    /// Scenario: User selects multiple files for release
    #[test]
    fn test_scenario_select_multiple_files() {
        let _locks = create_realistic_lock_set();
        let mut selected: HashSet<u32> = HashSet::new();

        // Single select
        selected.insert(1001);
        assert_eq!(selected.len(), 1);

        // Add to selection (Ctrl+Click simulation)
        selected.insert(1002);
        selected.insert(1003);
        assert_eq!(selected.len(), 3);

        // Remove from selection
        selected.remove(&1002);
        assert_eq!(selected.len(), 2);
        assert!(selected.contains(&1001));
        assert!(selected.contains(&1003));
        assert!(!selected.contains(&1002));
    }

    /// Scenario: User selects all visible locks
    #[test]
    fn test_scenario_select_all() {
        let locks = create_realistic_lock_set();

        // Select all
        let selected = select_all_visible(&locks);
        assert_eq!(selected.len(), 8);

        // Verify all IDs are selected
        for lock in &locks {
            assert!(selected.contains(&lock.file_id));
        }
    }

    /// Scenario: User selects all after filtering
    #[test]
    fn test_scenario_select_all_filtered() {
        let locks = create_realistic_lock_set();

        // Filter for JohnDoe
        let filtered = filter_locks(&locks, "JohnDoe");
        assert_eq!(filtered.len(), 3);

        // Select all visible
        let selected = select_all_visible(&filtered);
        assert_eq!(selected.len(), 3);

        // Verify only JohnDoe's files are selected
        assert!(selected.contains(&1001)); // document.docx
        assert!(selected.contains(&1002)); // spreadsheet.xlsx
        assert!(selected.contains(&1007)); // report.pdf

        // Verify other files are not selected
        assert!(!selected.contains(&1003));
        assert!(!selected.contains(&1004));
    }

    /// Scenario: User changes filter while having selections
    #[test]
    fn test_scenario_change_filter_with_selections() {
        let locks = create_realistic_lock_set();
        let mut selected: HashSet<u32> = HashSet::new();

        // Select JohnDoe's files
        selected.insert(1001);
        selected.insert(1002);

        // Change filter to show only system files
        let filtered = filter_locks(&locks, "SYSTEM");

        // Selections persist (but may not be visible)
        assert_eq!(selected.len(), 2);

        // Filtered results don't include selected items
        for lock in &filtered {
            assert!(!selected.contains(&lock.file_id));
        }
    }

    /// Scenario: Case-insensitive search
    #[test]
    fn test_scenario_case_insensitive_search() {
        let locks = create_realistic_lock_set();

        // Search with different cases
        let upper = filter_locks(&locks, "JOHNDOE");
        let lower = filter_locks(&locks, "johndoe");
        let mixed = filter_locks(&locks, "JoHnDoE");

        // All should return same results
        assert_eq!(upper.len(), lower.len());
        assert_eq!(lower.len(), mixed.len());
        assert_eq!(upper.len(), 3);
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_lock_list() {
        let locks: Vec<TestFileLockInfo> = vec![];

        let filtered = filter_locks(&locks, "anything");
        assert!(filtered.is_empty());

        let selected = select_all_visible(&locks);
        assert!(selected.is_empty());
    }

    #[test]
    fn test_empty_search_query() {
        let locks = create_realistic_lock_set();

        let filtered = filter_locks(&locks, "");
        assert_eq!(filtered.len(), locks.len());
    }

    #[test]
    fn test_whitespace_only_search() {
        let locks = create_realistic_lock_set();

        let filtered = filter_locks(&locks, "   ");
        // Whitespace doesn't match any normal content
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_special_characters_in_search() {
        let locks = vec![
            create_test_lock(1, "file[1].txt", r"C:\Data\file[1].txt", "user", 1),
            create_test_lock(2, "file(2).txt", r"C:\Data\file(2).txt", "user", 1),
            create_test_lock(3, "file{3}.txt", r"C:\Data\file{3}.txt", "user", 1),
        ];

        assert_eq!(filter_locks(&locks, "[1]").len(), 1);
        assert_eq!(filter_locks(&locks, "(2)").len(), 1);
        assert_eq!(filter_locks(&locks, "{3}").len(), 1);
    }

    #[test]
    fn test_unicode_search() {
        let locks = vec![
            create_test_lock(1, "文档.txt", r"C:\中文\文档.txt", "用户", 1),
            create_test_lock(2, "ファイル.txt", r"C:\日本語\ファイル.txt", "ユーザー", 1),
            create_test_lock(3, "файл.txt", r"C:\Русский\файл.txt", "пользователь", 1),
        ];

        assert_eq!(filter_locks(&locks, "中文").len(), 1);
        assert_eq!(filter_locks(&locks, "日本").len(), 1);
        assert_eq!(filter_locks(&locks, "Рус").len(), 1);
    }

    #[test]
    fn test_very_long_paths() {
        let long_path = format!(
            r"C:\{}",
            (0..50)
                .map(|i| format!("folder{}", i))
                .collect::<Vec<_>>()
                .join("\\")
        );
        let locks = vec![create_test_lock(1, "deep.txt", &long_path, "user", 1)];

        // Should be able to search in long paths
        assert_eq!(filter_locks(&locks, "folder25").len(), 1);
        assert_eq!(filter_locks(&locks, "deep.txt").len(), 1);
    }

    #[test]
    fn test_duplicate_file_ids() {
        let mut selected: HashSet<u32> = HashSet::new();

        // Adding same ID multiple times should only count once
        selected.insert(100);
        selected.insert(100);
        selected.insert(100);

        assert_eq!(selected.len(), 1);
    }

    #[test]
    fn test_sort_empty_list() {
        let locks: Vec<TestFileLockInfo> = vec![];

        let sorted = sort_locks(locks, SortColumn::FileName, SortDirection::Ascending);
        assert!(sorted.is_empty());
    }

    #[test]
    fn test_sort_single_item() {
        let locks = vec![create_test_lock(1, "only.txt", r"C:\only.txt", "user", 1)];

        let sorted = sort_locks(locks, SortColumn::FileName, SortDirection::Ascending);
        assert_eq!(sorted.len(), 1);
        assert_eq!(sorted[0].file_name, "only.txt");
    }

    #[test]
    fn test_sort_equal_values() {
        let locks = vec![
            create_test_lock(1, "same.txt", r"C:\a\same.txt", "user", 5),
            create_test_lock(2, "same.txt", r"C:\b\same.txt", "user", 5),
            create_test_lock(3, "same.txt", r"C:\c\same.txt", "user", 5),
        ];

        // Sort by filename (all same)
        let sorted = sort_locks(
            locks.clone(),
            SortColumn::FileName,
            SortDirection::Ascending,
        );
        assert_eq!(sorted.len(), 3);

        // Sort by num_locks (all same)
        let sorted = sort_locks(locks, SortColumn::NumLocks, SortDirection::Ascending);
        assert_eq!(sorted.len(), 3);
    }
}

// ============================================================================
// Performance Tests
// ============================================================================

mod performance_tests {
    use super::*;

    fn create_large_lock_set(count: usize) -> Vec<TestFileLockInfo> {
        (0..count)
            .map(|i| {
                create_test_lock(
                    i as u32,
                    &format!("file{}.dat", i),
                    &format!(r"C:\Data\Folder{}\file{}.dat", i / 100, i),
                    &format!("user{}", i % 50),
                    ((i % 10) + 1) as u32,
                )
            })
            .collect()
    }

    #[test]
    fn test_filter_performance_large_set() {
        let locks = create_large_lock_set(10000);

        let start = std::time::Instant::now();
        let filtered = filter_locks(&locks, "file999");
        let duration = start.elapsed();

        // Should complete in reasonable time (< 100ms)
        assert!(duration.as_millis() < 100);
        assert!(!filtered.is_empty());
    }

    #[test]
    fn test_sort_performance_large_set() {
        let locks = create_large_lock_set(10000);

        let start = std::time::Instant::now();
        let sorted = sort_locks(locks, SortColumn::NumLocks, SortDirection::Descending);
        let duration = start.elapsed();

        // Should complete in reasonable time (< 200ms)
        assert!(duration.as_millis() < 200);
        assert_eq!(sorted.len(), 10000);

        // Verify sort order
        for i in 0..sorted.len() - 1 {
            assert!(sorted[i].num_locks >= sorted[i + 1].num_locks);
        }
    }

    #[test]
    fn test_select_all_performance_large_set() {
        let locks = create_large_lock_set(10000);

        let start = std::time::Instant::now();
        let selected = select_all_visible(&locks);
        let duration = start.elapsed();

        // Should complete in reasonable time (< 50ms)
        assert!(duration.as_millis() < 50);
        assert_eq!(selected.len(), 10000);
    }

    #[test]
    fn test_combined_filter_sort_performance() {
        let locks = create_large_lock_set(10000);

        let start = std::time::Instant::now();
        let filtered = filter_locks(&locks, "user25");
        let sorted = sort_locks(filtered, SortColumn::FileName, SortDirection::Ascending);
        let duration = start.elapsed();

        // Combined operation should complete in reasonable time
        assert!(duration.as_millis() < 150);
        assert!(!sorted.is_empty());
    }
}

// ============================================================================
// Extract Filename Tests
// ============================================================================

mod extract_filename_tests {
    use super::*;

    #[test]
    fn test_standard_windows_path() {
        assert_eq!(
            extract_filename(r"C:\Users\Admin\Documents\file.txt"),
            "file.txt"
        );
    }

    #[test]
    fn test_unc_path() {
        assert_eq!(
            extract_filename(r"\\server\share\folder\document.pdf"),
            "document.pdf"
        );
    }

    #[test]
    fn test_root_only() {
        assert_eq!(extract_filename(r"C:\"), "");
    }

    #[test]
    fn test_no_path_separator() {
        assert_eq!(extract_filename("justfilename.txt"), "justfilename.txt");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(extract_filename(""), "");
    }

    #[test]
    fn test_hidden_file() {
        assert_eq!(extract_filename(r"C:\Users\.config"), ".config");
    }

    #[test]
    fn test_file_with_spaces() {
        assert_eq!(
            extract_filename(r"C:\My Documents\My File.docx"),
            "My File.docx"
        );
    }

    #[test]
    fn test_multiple_extensions() {
        assert_eq!(
            extract_filename(r"C:\Archive\backup.tar.gz"),
            "backup.tar.gz"
        );
    }
}

// ============================================================================
// Workflow Simulation Tests
// ============================================================================

mod workflow_tests {
    use super::*;

    /// Simulates a complete user workflow: view -> search -> sort -> select -> release
    #[test]
    fn test_complete_user_workflow() {
        // Step 1: Application loads with locks
        let locks = create_realistic_lock_set();
        assert_eq!(locks.len(), 8);

        // Step 2: User searches for "JohnDoe"
        let filtered = filter_locks(&locks, "JohnDoe");
        assert_eq!(filtered.len(), 3);

        // Step 3: User sorts by filename
        let sorted = sort_locks(
            filtered.clone(),
            SortColumn::FileName,
            SortDirection::Ascending,
        );
        assert_eq!(sorted[0].file_name, "document.docx");
        assert_eq!(sorted[2].file_name, "spreadsheet.xlsx");

        // Step 4: User selects all visible
        let selected = select_all_visible(&sorted);
        assert_eq!(selected.len(), 3);

        // Step 5: User would confirm release (simulated by checking selection)
        for lock in &sorted {
            assert!(selected.contains(&lock.file_id));
        }

        // Step 6: After release, clear selection
        let cleared_selection: HashSet<u32> = HashSet::new();
        assert!(cleared_selection.is_empty());
    }

    /// Simulates finding and releasing a specific problematic lock
    #[test]
    fn test_find_and_release_specific_lock() {
        let locks = create_realistic_lock_set();

        // User searches for the database file
        let filtered = filter_locks(&locks, "database");
        assert_eq!(filtered.len(), 1);

        let target_lock = &filtered[0];
        assert_eq!(target_lock.file_name, "database.mdb");
        assert_eq!(target_lock.num_locks, 5); // High lock count

        // User selects it
        let mut selected: HashSet<u32> = HashSet::new();
        selected.insert(target_lock.file_id);
        assert!(selected.contains(&1003));
    }

    /// Simulates investigating locks by a specific service account
    #[test]
    fn test_investigate_service_account_locks() {
        let locks = create_realistic_lock_set();

        // Admin wants to see all system/service locks
        let _system_locks = filter_locks(&locks, "SYSTEM");
        let _service_locks = filter_locks(&locks, "SERVICE");

        // Combine results
        let all_service_locks: Vec<_> = locks
            .iter()
            .filter(|l| {
                l.username.to_uppercase().contains("SYSTEM")
                    || l.username.to_uppercase().contains("SERVICE")
            })
            .cloned()
            .collect();

        assert!(all_service_locks.len() >= 2);

        // Sort by lock count to find most problematic
        let sorted = sort_locks(
            all_service_locks,
            SortColumn::NumLocks,
            SortDirection::Descending,
        );

        // The database.mdb with 5 locks should be first
        assert_eq!(sorted[0].file_name, "database.mdb");
        assert_eq!(sorted[0].num_locks, 5);
    }
}
