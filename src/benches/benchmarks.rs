//! Benchmarks for LockSmith performance-critical operations
//!
//! Run with: cargo bench
//!
//! These benchmarks help ensure that filtering, sorting, and selection
//! operations remain performant as the codebase evolves.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::HashSet;

// ============================================================================
// Test Data Structures (mirrors main crate)
// ============================================================================

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct FileLockInfo {
    file_id: u32,
    server_name: String,
    file_path: String,
    file_name: String,
    username: String,
    permissions: u32,
    num_locks: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SortColumn {
    FileName,
    FilePath,
    Username,
    NumLocks,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SortDirection {
    Ascending,
    Descending,
}

// ============================================================================
// Test Data Generators
// ============================================================================

fn generate_locks(count: usize) -> Vec<FileLockInfo> {
    (0..count)
        .map(|i| FileLockInfo {
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

// ============================================================================
// Core Functions (mirrors main crate)
// ============================================================================

fn extract_filename(path: &str) -> String {
    path.rsplit('\\').next().unwrap_or(path).to_string()
}

fn filter_locks(locks: &[FileLockInfo], query: &str) -> Vec<FileLockInfo> {
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

fn sort_locks(
    mut locks: Vec<FileLockInfo>,
    column: SortColumn,
    direction: SortDirection,
) -> Vec<FileLockInfo> {
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

fn select_all_ids(locks: &[FileLockInfo]) -> HashSet<u32> {
    locks.iter().map(|l| l.file_id).collect()
}

// ============================================================================
// Benchmark Groups
// ============================================================================

fn extract_filename_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("extract_filename");

    let short_path = r"C:\file.txt";
    let medium_path = r"C:\Users\Administrator\Documents\Projects\LockSmith\src\main.rs";
    let long_path: String = format!(
        r"C:\{}",
        (0..50)
            .map(|i| format!("folder{}", i))
            .collect::<Vec<_>>()
            .join("\\")
    );

    group.bench_function("short_path", |b| {
        b.iter(|| extract_filename(black_box(short_path)))
    });

    group.bench_function("medium_path", |b| {
        b.iter(|| extract_filename(black_box(medium_path)))
    });

    group.bench_function("long_path", |b| {
        b.iter(|| extract_filename(black_box(&long_path)))
    });

    group.finish();
}

fn filter_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_locks");

    for size in [100, 1000, 10000].iter() {
        let locks = generate_locks(*size);

        group.bench_with_input(BenchmarkId::new("no_match", size), &locks, |b, locks| {
            b.iter(|| filter_locks(black_box(locks), black_box("nonexistent_query_xyz")))
        });

        group.bench_with_input(
            BenchmarkId::new("some_matches", size),
            &locks,
            |b, locks| b.iter(|| filter_locks(black_box(locks), black_box("file5"))),
        );

        group.bench_with_input(BenchmarkId::new("empty_query", size), &locks, |b, locks| {
            b.iter(|| filter_locks(black_box(locks), black_box("")))
        });

        group.bench_with_input(
            BenchmarkId::new("username_match", size),
            &locks,
            |b, locks| b.iter(|| filter_locks(black_box(locks), black_box("user50"))),
        );
    }

    group.finish();
}

fn sort_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort_locks");

    for size in [100, 1000, 10000].iter() {
        let locks = generate_locks(*size);

        group.bench_with_input(
            BenchmarkId::new("by_filename_asc", size),
            &locks,
            |b, locks| {
                b.iter(|| {
                    sort_locks(
                        black_box(locks.clone()),
                        SortColumn::FileName,
                        SortDirection::Ascending,
                    )
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("by_filename_desc", size),
            &locks,
            |b, locks| {
                b.iter(|| {
                    sort_locks(
                        black_box(locks.clone()),
                        SortColumn::FileName,
                        SortDirection::Descending,
                    )
                })
            },
        );

        group.bench_with_input(BenchmarkId::new("by_path_asc", size), &locks, |b, locks| {
            b.iter(|| {
                sort_locks(
                    black_box(locks.clone()),
                    SortColumn::FilePath,
                    SortDirection::Ascending,
                )
            })
        });

        group.bench_with_input(
            BenchmarkId::new("by_username_asc", size),
            &locks,
            |b, locks| {
                b.iter(|| {
                    sort_locks(
                        black_box(locks.clone()),
                        SortColumn::Username,
                        SortDirection::Ascending,
                    )
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("by_num_locks_desc", size),
            &locks,
            |b, locks| {
                b.iter(|| {
                    sort_locks(
                        black_box(locks.clone()),
                        SortColumn::NumLocks,
                        SortDirection::Descending,
                    )
                })
            },
        );
    }

    group.finish();
}

fn selection_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("select_all");

    for size in [100, 1000, 10000].iter() {
        let locks = generate_locks(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), &locks, |b, locks| {
            b.iter(|| select_all_ids(black_box(locks)))
        });
    }

    group.finish();
}

fn combined_operations_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("combined_operations");

    for size in [1000, 10000].iter() {
        let locks = generate_locks(*size);

        group.bench_with_input(
            BenchmarkId::new("filter_then_sort", size),
            &locks,
            |b, locks| {
                b.iter(|| {
                    let filtered = filter_locks(black_box(locks), "user5");
                    sort_locks(filtered, SortColumn::FileName, SortDirection::Ascending)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("full_pipeline", size),
            &locks,
            |b, locks| {
                b.iter(|| {
                    let filtered = filter_locks(black_box(locks), "file");
                    let sorted =
                        sort_locks(filtered, SortColumn::NumLocks, SortDirection::Descending);
                    select_all_ids(&sorted)
                })
            },
        );
    }

    group.finish();
}

fn hashset_operations_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashset_operations");

    group.bench_function("insert_1000", |b| {
        b.iter(|| {
            let mut set = HashSet::new();
            for i in 0..1000u32 {
                set.insert(black_box(i));
            }
            set
        })
    });

    let set: HashSet<u32> = (0..1000).collect();
    group.bench_function("contains_1000", |b| {
        b.iter(|| {
            let mut found = 0;
            for i in 0..1000u32 {
                if set.contains(black_box(&i)) {
                    found += 1;
                }
            }
            found
        })
    });

    group.bench_function("remove_500_from_1000", |b| {
        b.iter(|| {
            let mut set: HashSet<u32> = (0..1000).collect();
            for i in (0..1000).step_by(2) {
                set.remove(black_box(&i));
            }
            set
        })
    });

    group.finish();
}

fn string_operations_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_operations");

    let short = "FileName.TXT";
    let long = "C:\\Users\\Administrator\\Documents\\MyVeryLongFileName.TXT";
    let haystack = "C:\\Users\\Administrator\\Documents\\Projects\\LockSmith\\src\\main.rs";
    let needle = "LockSmith";

    group.bench_function("to_lowercase_short", |b| {
        b.iter(|| black_box(short).to_lowercase())
    });

    group.bench_function("to_lowercase_long", |b| {
        b.iter(|| black_box(long).to_lowercase())
    });

    group.bench_function("string_contains", |b| {
        b.iter(|| {
            black_box(haystack)
                .to_lowercase()
                .contains(&black_box(needle).to_lowercase())
        })
    });

    group.finish();
}

fn clone_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("clone_operations");

    let single_lock = FileLockInfo {
        file_id: 12345,
        server_name: "LOCAL".to_string(),
        file_path: r"C:\Users\Admin\Documents\Important\File.docx".to_string(),
        file_name: "File.docx".to_string(),
        username: "DOMAIN\\Administrator".to_string(),
        permissions: 3,
        num_locks: 5,
    };

    group.bench_function("single_lock", |b| {
        b.iter(|| black_box(&single_lock).clone())
    });

    for size in [100, 1000, 10000].iter() {
        let locks = generate_locks(*size);

        group.bench_with_input(BenchmarkId::new("vec_locks", size), &locks, |b, locks| {
            b.iter(|| black_box(locks).clone())
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    extract_filename_benchmarks,
    filter_benchmarks,
    sort_benchmarks,
    selection_benchmarks,
    combined_operations_benchmarks,
    hashset_operations_benchmarks,
    string_operations_benchmarks,
    clone_benchmarks,
);

criterion_main!(benches);
