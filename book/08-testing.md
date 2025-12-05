# Chapter 8: Testing

Testing ensures your code works correctly and keeps working as you make changes. This chapter covers Rust's testing framework, how to organize tests, and the testing strategies used in LockSmith.

## Why Test?

Testing provides:

1. **Confidence** — Know your code works before shipping
2. **Documentation** — Tests show how code should be used
3. **Regression prevention** — Catch bugs when you change things
4. **Design feedback** — Hard-to-test code often has design problems

## Rust's Testing Framework

Rust has built-in testing support. No external framework needed for basic tests.

### Your First Test

```rust
#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);
}
```

The `#[test]` attribute marks a function as a test. Run tests with:

```
$ cargo test
   Compiling locksmith v1.0.0
    Finished test [unoptimized + debuginfo] target(s) in 2.34s
     Running unittests src\main.rs

running 1 test
test it_works ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Assertion Macros

Rust provides several assertion macros:

```rust
// Equality
assert_eq!(actual, expected);
assert_ne!(actual, not_expected);

// Boolean
assert!(condition);
assert!(!should_be_false);

// With custom message
assert_eq!(result, 42, "Expected 42, got {}", result);
```

If an assertion fails, the test fails with a descriptive error.

### Testing for Panics

Some code should panic on invalid input:

```rust
#[test]
#[should_panic]
fn panics_on_invalid_input() {
    let v = vec![1, 2, 3];
    v[99];  // This panics
}

#[test]
#[should_panic(expected = "out of bounds")]
fn panics_with_specific_message() {
    let v = vec![1, 2, 3];
    v[99];
}
```

## Test Organization

### The tests Module

Tests typically go in a `tests` module at the bottom of your source file:

```rust
// Regular code above...

fn extract_filename(path: &str) -> String {
    path.rsplit('\\').next().unwrap_or(path).to_string()
}

// Tests below...

#[cfg(test)]
mod tests {
    use super::*;  // Import everything from parent module

    #[test]
    fn test_extract_filename_windows_path() {
        assert_eq!(extract_filename(r"C:\Users\Admin\file.txt"), "file.txt");
    }
}
```

### The cfg(test) Attribute

```rust
#[cfg(test)]
mod tests {
```

`#[cfg(test)]` means this module only compiles when running tests. This:
- Keeps test code out of the final binary
- Allows tests to access private functions

### Importing with use super::*

```rust
use super::*;
```

`super` refers to the parent module. `use super::*` imports everything from the parent, giving tests access to all functions and types.

### Nested Test Modules

For better organization, nest related tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod extract_filename_tests {
        use super::*;

        #[test]
        fn test_windows_path() {
            assert_eq!(extract_filename(r"C:\Users\Admin\file.txt"), "file.txt");
        }

        #[test]
        fn test_unc_path() {
            assert_eq!(extract_filename(r"\\server\share\doc.pdf"), "doc.pdf");
        }

        #[test]
        fn test_empty() {
            assert_eq!(extract_filename(""), "");
        }
    }

    mod sort_direction_tests {
        use super::*;

        #[test]
        fn test_toggle() {
            let asc = SortDirection::Ascending;
            assert_eq!(asc.toggle(), SortDirection::Descending);
        }
    }
}
```

Output shows the module hierarchy:

```
running 4 tests
test tests::extract_filename_tests::test_windows_path ... ok
test tests::extract_filename_tests::test_unc_path ... ok
test tests::extract_filename_tests::test_empty ... ok
test tests::sort_direction_tests::test_toggle ... ok
```

## Testing LockSmith's Components

### Testing Helper Functions

```rust
mod extract_filename_tests {
    use super::*;

    #[test]
    fn test_extract_filename_windows_path() {
        assert_eq!(extract_filename(r"C:\Users\Admin\file.txt"), "file.txt");
    }

    #[test]
    fn test_extract_filename_unc_path() {
        assert_eq!(extract_filename(r"\\server\share\doc.pdf"), "doc.pdf");
    }

    #[test]
    fn test_extract_filename_simple() {
        assert_eq!(extract_filename("simple.txt"), "simple.txt");
    }

    #[test]
    fn test_extract_filename_empty() {
        assert_eq!(extract_filename(""), "");
    }

    #[test]
    fn test_extract_filename_nested_path() {
        assert_eq!(
            extract_filename(r"C:\Users\Admin\Documents\Projects\LockSmith\src\main.rs"),
            "main.rs"
        );
    }

    #[test]
    fn test_extract_filename_no_extension() {
        assert_eq!(extract_filename(r"C:\Users\Admin\Makefile"), "Makefile");
    }

    #[test]
    fn test_extract_filename_hidden_file() {
        assert_eq!(extract_filename(r"C:\Users\Admin\.gitignore"), ".gitignore");
    }

    #[test]
    fn test_extract_filename_trailing_backslash() {
        assert_eq!(extract_filename(r"C:\Users\Admin\"), "");
    }

    #[test]
    fn test_extract_filename_unicode() {
        assert_eq!(extract_filename(r"C:\Users\Admin\文件.txt"), "文件.txt");
    }
}
```

### Testing Enums

```rust
mod sort_direction_tests {
    use super::*;

    #[test]
    fn test_sort_direction_toggle_ascending_to_descending() {
        let dir = SortDirection::Ascending;
        assert_eq!(dir.toggle(), SortDirection::Descending);
    }

    #[test]
    fn test_sort_direction_toggle_descending_to_ascending() {
        let dir = SortDirection::Descending;
        assert_eq!(dir.toggle(), SortDirection::Ascending);
    }

    #[test]
    fn test_sort_direction_double_toggle_identity() {
        let dir = SortDirection::Ascending;
        assert_eq!(dir.toggle().toggle(), SortDirection::Ascending);
    }

    #[test]
    fn test_sort_direction_arrow_ascending() {
        assert_eq!(SortDirection::Ascending.arrow(), " ↑");
    }

    #[test]
    fn test_sort_direction_arrow_descending() {
        assert_eq!(SortDirection::Descending.arrow(), " ↓");
    }
}
```

### Testing Error Types

```rust
mod file_enum_error_tests {
    use super::*;

    #[test]
    fn test_file_enum_error_access_denied_display() {
        let err = FileEnumError::AccessDenied;
        let display = format!("{}", err);
        assert!(display.contains("Access denied"));
        assert!(display.contains("administrator"));
    }

    #[test]
    fn test_file_enum_error_windows_api_error_display() {
        let err = FileEnumError::WindowsApiError(1234);
        let display = format!("{}", err);
        assert!(display.contains("Windows API error"));
        assert!(display.contains("1234"));
    }

    #[test]
    fn test_file_enum_error_is_std_error() {
        // Verify it implements std::error::Error
        let err: Box<dyn std::error::Error> = Box::new(FileEnumError::AccessDenied);
        assert!(!err.to_string().is_empty());
    }
}
```

### Testing Application State

```rust
mod lock_smith_app_tests {
    use super::*;

    fn create_test_app() -> LockSmithApp {
        LockSmithApp::default()
    }

    fn create_sample_locks() -> Vec<FileLockInfo> {
        vec![
            FileLockInfo {
                file_id: 1,
                server_name: "LOCAL".to_string(),
                file_path: r"C:\Data\alpha.txt".to_string(),
                file_name: "alpha.txt".to_string(),
                username: "DOMAIN\\user1".to_string(),
                permissions: 0,
                num_locks: 1,
            },
            FileLockInfo {
                file_id: 2,
                server_name: "LOCAL".to_string(),
                file_path: r"C:\Data\beta.doc".to_string(),
                file_name: "beta.doc".to_string(),
                username: "DOMAIN\\user2".to_string(),
                permissions: 0,
                num_locks: 3,
            },
            FileLockInfo {
                file_id: 3,
                server_name: "LOCAL".to_string(),
                file_path: r"C:\Data\gamma.pdf".to_string(),
                file_name: "gamma.pdf".to_string(),
                username: "DOMAIN\\admin".to_string(),
                permissions: 0,
                num_locks: 2,
            },
        ]
    }

    #[test]
    fn test_app_default_state() {
        let app = create_test_app();
        assert!(app.all_locks.is_empty());
        assert!(app.selected_ids.is_empty());
        assert!(app.search_query.is_empty());
        assert!(app.sort_column.is_none());
        assert_eq!(app.sort_direction, SortDirection::Ascending);
        assert!(app.dark_mode);
        assert!(!app.is_loading);
    }

    #[test]
    fn test_get_filtered_sorted_locks_no_filter() {
        let mut app = create_test_app();
        app.all_locks = create_sample_locks();

        let result = app.get_filtered_sorted_locks();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_get_filtered_sorted_locks_filter_by_filename() {
        let mut app = create_test_app();
        app.all_locks = create_sample_locks();
        app.search_query = "alpha".to_string();

        let result = app.get_filtered_sorted_locks();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].file_name, "alpha.txt");
    }

    #[test]
    fn test_get_filtered_sorted_locks_filter_by_username() {
        let mut app = create_test_app();
        app.all_locks = create_sample_locks();
        app.search_query = "admin".to_string();

        let result = app.get_filtered_sorted_locks();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].username, "DOMAIN\\admin");
    }

    #[test]
    fn test_get_filtered_sorted_locks_case_insensitive() {
        let mut app = create_test_app();
        app.all_locks = create_sample_locks();
        app.search_query = "ALPHA".to_string();

        let result = app.get_filtered_sorted_locks();
        assert_eq!(result.len(), 1);
    }
}
```

### Testing Sorting

```rust
#[test]
fn test_get_filtered_sorted_locks_sort_by_filename_ascending() {
    let mut app = create_test_app();
    app.all_locks = create_sample_locks();
    app.sort_column = Some(SortColumn::FileName);
    app.sort_direction = SortDirection::Ascending;

    let result = app.get_filtered_sorted_locks();
    assert_eq!(result[0].file_name, "alpha.txt");
    assert_eq!(result[1].file_name, "beta.doc");
    assert_eq!(result[2].file_name, "gamma.pdf");
}

#[test]
fn test_get_filtered_sorted_locks_sort_by_filename_descending() {
    let mut app = create_test_app();
    app.all_locks = create_sample_locks();
    app.sort_column = Some(SortColumn::FileName);
    app.sort_direction = SortDirection::Descending;

    let result = app.get_filtered_sorted_locks();
    assert_eq!(result[0].file_name, "gamma.pdf");
    assert_eq!(result[1].file_name, "beta.doc");
    assert_eq!(result[2].file_name, "alpha.txt");
}

#[test]
fn test_get_filtered_sorted_locks_sort_by_num_locks() {
    let mut app = create_test_app();
    app.all_locks = create_sample_locks();
    app.sort_column = Some(SortColumn::NumLocks);
    app.sort_direction = SortDirection::Descending;

    let result = app.get_filtered_sorted_locks();
    assert_eq!(result[0].num_locks, 3);  // beta.doc
    assert_eq!(result[1].num_locks, 2);  // gamma.pdf
    assert_eq!(result[2].num_locks, 1);  // alpha.txt
}
```

### Testing Selection

```rust
#[test]
fn test_select_all_visible() {
    let mut app = create_test_app();
    app.all_locks = create_sample_locks();
    
    app.select_all_visible();
    
    assert_eq!(app.selected_ids.len(), 3);
    assert!(app.selected_ids.contains(&1));
    assert!(app.selected_ids.contains(&2));
    assert!(app.selected_ids.contains(&3));
}

#[test]
fn test_select_all_visible_with_filter() {
    let mut app = create_test_app();
    app.all_locks = create_sample_locks();
    app.search_query = "alpha".to_string();
    
    app.select_all_visible();
    
    assert_eq!(app.selected_ids.len(), 1);
    assert!(app.selected_ids.contains(&1));
}

#[test]
fn test_handle_sort_initial_column() {
    let mut app = create_test_app();
    
    app.handle_sort(SortColumn::FileName);
    
    assert_eq!(app.sort_column, Some(SortColumn::FileName));
    assert_eq!(app.sort_direction, SortDirection::Ascending);
}

#[test]
fn test_handle_sort_same_column_toggles_direction() {
    let mut app = create_test_app();
    
    app.handle_sort(SortColumn::FileName);
    assert_eq!(app.sort_direction, SortDirection::Ascending);
    
    app.handle_sort(SortColumn::FileName);
    assert_eq!(app.sort_direction, SortDirection::Descending);
}
```

## Conditional Tests

Some tests only make sense on certain platforms:

```rust
#[cfg(windows)]
mod error_mapping_tests {
    use super::*;

    #[test]
    fn test_map_windows_error_access_denied() {
        let err = map_windows_error(ERROR_ACCESS_DENIED);
        assert!(matches!(err, FileEnumError::AccessDenied));
    }

    #[test]
    fn test_map_windows_error_not_enough_memory() {
        let err = map_windows_error(ERROR_NOT_ENOUGH_MEMORY);
        assert!(matches!(err, FileEnumError::NotEnoughMemory));
    }
}
```

These tests only compile and run on Windows where the functions exist.

### The matches! Macro

```rust
assert!(matches!(err, FileEnumError::AccessDenied));
```

`matches!` returns `true` if the value matches the pattern. It's cleaner than:

```rust
assert!(match err { FileEnumError::AccessDenied => true, _ => false });
```

## Testing Non-Windows Stubs

```rust
#[cfg(not(windows))]
mod non_windows_stub_tests {
    use super::*;

    #[test]
    fn test_enumerate_files_returns_error_on_non_windows() {
        let result = enumerate_files();
        assert!(result.is_err());
        
        if let Err(FileEnumError::WindowsApiError(code)) = result {
            assert_eq!(code, 0);
        } else {
            panic!("Expected WindowsApiError(0)");
        }
    }

    #[test]
    fn test_is_admin_returns_false_on_non_windows() {
        assert!(!is_admin());
    }
}
```

## Boundary Testing

Test edge cases and limits:

```rust
mod boundary_tests {
    use super::*;

    #[test]
    fn test_file_id_zero() {
        let lock = FileLockInfo {
            file_id: 0,
            // ... other fields
        };
        assert_eq!(lock.file_id, 0);
    }

    #[test]
    fn test_file_id_max() {
        let lock = FileLockInfo {
            file_id: u32::MAX,
            // ... other fields
        };
        assert_eq!(lock.file_id, u32::MAX);
    }

    #[test]
    fn test_empty_strings_in_lock_info() {
        let lock = FileLockInfo {
            file_id: 1,
            server_name: "".to_string(),
            file_path: "".to_string(),
            file_name: "".to_string(),
            username: "".to_string(),
            permissions: 0,
            num_locks: 0,
        };
        // Should not panic
        let _ = format!("{:?}", lock);
    }

    #[test]
    fn test_very_long_strings() {
        let long_string = "a".repeat(10000);
        let lock = FileLockInfo {
            file_id: 1,
            file_path: long_string.clone(),
            file_name: long_string,
            // ... other fields
        };
        assert!(lock.file_path.len() == 10000);
    }

    #[test]
    fn test_unicode_in_all_fields() {
        let lock = FileLockInfo {
            file_id: 1,
            server_name: "サーバー".to_string(),
            file_path: "C:\\用户\\文件.txt".to_string(),
            file_name: "文件.txt".to_string(),
            username: "域\\用户".to_string(),
            permissions: 0,
            num_locks: 0,
        };
        assert!(lock.file_name.contains("文件"));
    }
}
```

## Integration Tests

For larger-scope tests, create separate files in `tests/`:

```
src/
├── tests/
│   ├── integration_tests.rs
│   └── test_utils.rs
```

### tests/test_utils.rs

Shared test utilities:

```rust
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
    pub fn new(id: u32, name: &str, user: &str, locks: u32) -> Self {
        Self {
            file_id: id,
            server_name: "LOCAL".to_string(),
            file_path: format!(r"C:\Data\{}", name),
            file_name: name.to_string(),
            username: user.to_string(),
            permissions: 0,
            num_locks: locks,
        }
    }
}

pub fn generate_mock_locks(count: usize) -> Vec<MockFileLockInfo> {
    (0..count)
        .map(|i| MockFileLockInfo::new(
            i as u32,
            &format!("file{}.txt", i),
            &format!("DOMAIN\\user{}", i % 10),
            (i % 5) as u32,
        ))
        .collect()
}
```

### tests/integration_tests.rs

Integration tests testing larger workflows:

```rust
mod scenario_tests {
    use super::*;

    #[test]
    fn test_complete_user_workflow() {
        // Simulate: User opens app, searches, sorts, selects, releases
        
        // 1. Create app with locks
        let locks = create_realistic_lock_set();
        
        // 2. Search for specific files
        let filtered = filter_locks(&locks, "document");
        assert!(!filtered.is_empty());
        
        // 3. Sort by username
        let sorted = sort_locks(
            filtered.clone(),
            SortColumn::Username,
            SortDirection::Ascending,
        );
        
        // 4. Verify order
        for i in 1..sorted.len() {
            assert!(sorted[i-1].username <= sorted[i].username);
        }
        
        // 5. Select all
        let selected = select_all_visible(&sorted);
        assert_eq!(selected.len(), sorted.len());
    }

    #[test]
    fn test_find_and_release_specific_lock() {
        let locks = create_realistic_lock_set();
        
        // Find a specific file
        let target = locks.iter()
            .find(|l| l.file_name.contains("important"))
            .expect("Test data should contain 'important' file");
        
        // Would release this lock
        assert!(target.file_id > 0);
    }
}
```

## Running Tests

### All Tests

```
$ cargo test
```

### Specific Test

```
$ cargo test test_extract_filename_windows_path
```

### Tests Matching Pattern

```
$ cargo test extract_filename
```

### Tests in Specific Module

```
$ cargo test sort_direction_tests
```

### Show Output (println! in tests)

```
$ cargo test -- --nocapture
```

### Run Ignored Tests

```rust
#[test]
#[ignore]
fn expensive_test() {
    // Takes a long time
}
```

```
$ cargo test -- --ignored
```

## Test Output

Successful run:

```
running 45 tests
test tests::extract_filename_tests::test_empty ... ok
test tests::extract_filename_tests::test_simple ... ok
...
test result: ok. 45 passed; 0 failed; 0 ignored
```

Failed test:

```
running 1 test
test tests::my_test ... FAILED

failures:

---- tests::my_test stdout ----
thread 'tests::my_test' panicked at 'assertion failed: `(left == right)`
  left: `"hello"`,
 right: `"world"`', src\main.rs:123:9

failures:
    tests::my_test

test result: FAILED. 0 passed; 1 failed; 0 ignored
```

## What We've Accomplished

You now understand:

✅ Rust's testing framework — #[test], assertions, cargo test  
✅ Test organization — Modules, nesting, cfg(test)  
✅ Testing strategies — Unit tests, boundary tests, integration tests  
✅ Conditional compilation — Platform-specific tests  
✅ Test patterns — matches!, helper functions, test fixtures  

## Key Takeaways

1. **Tests go in #[cfg(test)] modules** — Keeps them separate from production code
2. **Use nested modules for organization** — Groups related tests together
3. **Test edge cases** — Empty inputs, max values, unicode
4. **Conditional tests for platform-specific code** — cfg(windows) and cfg(not(windows))
5. **Helper functions reduce duplication** — Create test fixtures and utilities

## Looking Ahead

In the next chapter, we'll cover benchmarking—measuring performance and ensuring our code stays fast.

---

**Previous Chapter:** [← Polish and Finishing Touches](07-polish.md)  
**Next Chapter:** [Benchmarking →](09-benchmarking.md)