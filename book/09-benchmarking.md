# Chapter 9: Benchmarking

Performance matters. This chapter covers how to measure your code's speed using Rust's benchmarking tools and the Criterion library.

## Why Benchmark?

Benchmarking helps you:

1. **Find bottlenecks** — Identify slow code before users complain
2. **Compare implementations** — Measure which approach is faster
3. **Prevent regressions** — Catch performance degradation early
4. **Make informed decisions** — Data beats intuition

## Setting Up Criterion

Rust's built-in benchmarking is nightly-only. For stable Rust, we use **Criterion**, a statistics-driven benchmarking library.

### Cargo.toml Configuration

```toml
[[bench]]
name = "benchmarks"
harness = false

[dev-dependencies]
criterion = "0.5"
```

- `[[bench]]` — Defines a benchmark target (double brackets for arrays)
- `name = "benchmarks"` — The benchmark file name (without `.rs`)
- `harness = false` — Don't use Rust's built-in test harness
- `criterion = "0.5"` — The benchmarking library

### Creating the Benchmark File

Create `benches/benchmarks.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn my_benchmark(c: &mut Criterion) {
    c.bench_function("my_function", |b| {
        b.iter(|| {
            // Code to benchmark
            black_box(do_something())
        })
    });
}

criterion_group!(benches, my_benchmark);
criterion_main!(benches);
```

## Understanding Criterion Basics

### The black_box Function

```rust
use criterion::black_box;

b.iter(|| {
    black_box(compute_result())
})
```

`black_box` prevents the compiler from optimizing away your code. Without it, the compiler might:
- Remove unused calculations
- Move computations outside the timing loop
- Constant-fold known values

`black_box` acts as an optimization barrier—the compiler can't see through it.

### The bench_function Method

```rust
c.bench_function("descriptive_name", |b| {
    b.iter(|| {
        // This closure runs many times
        // Criterion measures how long it takes
    })
});
```

Criterion:
1. Runs your code many times
2. Measures elapsed time
3. Computes statistics (mean, median, standard deviation)
4. Compares against previous runs (if available)

## LockSmith's Benchmarks

Let's look at LockSmith's actual benchmarks:

### Data Structures for Benchmarks

```rust
#[derive(Debug, Clone)]
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
```

We duplicate types in benchmarks because benchmarks compile separately from the main crate.

### Test Data Generators

```rust
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
```

This generates realistic test data with varied values.

### Benchmarking extract_filename

```rust
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
```

### Benchmark Groups

```rust
let mut group = c.benchmark_group("extract_filename");
// ... benchmarks ...
group.finish();
```

Groups organize related benchmarks:
- Output is grouped in reports
- Easy to compare related functions
- Can set group-wide configuration

### Benchmarking with Different Input Sizes

```rust
fn filter_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_locks");

    for size in [100, 1000, 10000].iter() {
        let locks = generate_locks(*size);

        group.bench_with_input(
            BenchmarkId::new("no_match", size),
            &locks,
            |b, locks| {
                b.iter(|| filter_locks(black_box(locks), black_box("nonexistent_query_xyz")))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("some_matches", size),
            &locks,
            |b, locks| {
                b.iter(|| filter_locks(black_box(locks), black_box("file5")))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("empty_query", size),
            &locks,
            |b, locks| {
                b.iter(|| filter_locks(black_box(locks), black_box("")))
            },
        );
    }

    group.finish();
}
```

### Understanding bench_with_input

```rust
group.bench_with_input(
    BenchmarkId::new("name", parameter),
    &input_data,
    |b, input| {
        b.iter(|| do_something(black_box(input)))
    },
);
```

- `BenchmarkId::new("name", param)` — Creates an ID that includes the parameter
- `&input_data` — Reference to data (not recreated each iteration)
- `|b, input|` — Closure receives the bencher and input reference

### Benchmarking Sorting

```rust
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
```

Note: We `.clone()` inside `iter()` because sorting is in-place. Each iteration needs fresh data.

### Benchmarking Selection

```rust
fn selection_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("select_all");

    for size in [100, 1000, 10000].iter() {
        let locks = generate_locks(*size);

        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &locks,
            |b, locks| {
                b.iter(|| select_all_ids(black_box(locks)))
            },
        );
    }

    group.finish();
}
```

`BenchmarkId::from_parameter(size)` — Uses just the size as the ID when the name isn't important.

### Combined Operations

Real usage often involves multiple operations:

```rust
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
                    let sorted = sort_locks(
                        filtered,
                        SortColumn::NumLocks,
                        SortDirection::Descending,
                    );
                    select_all_ids(&sorted)
                })
            },
        );
    }

    group.finish();
}
```

### Micro-Benchmarks

Sometimes you need to benchmark individual operations:

```rust
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

    group.finish();
}
```

### Registering Benchmarks

```rust
criterion_group!(
    benches,
    extract_filename_benchmarks,
    filter_benchmarks,
    sort_benchmarks,
    selection_benchmarks,
    combined_operations_benchmarks,
    hashset_operations_benchmarks,
    string_operations_benchmarks,
);

criterion_main!(benches);
```

`criterion_group!` creates a group of benchmark functions.
`criterion_main!` creates the main function that runs them.

## Running Benchmarks

### Basic Run

```
$ cargo bench
```

This compiles in release mode and runs all benchmarks.

### Run Specific Benchmark

```
$ cargo bench filter_locks
```

Runs benchmarks with "filter_locks" in their name.

### Benchmark Output

```
filter_locks/no_match/100
                        time:   [1.2345 µs 1.2456 µs 1.2567 µs]
                        change: [-2.1234% -0.1234% +1.8766%] (p = 0.12 > 0.05)
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
```

The output shows:
- **Time range** — [lower bound, estimate, upper bound] with 95% confidence
- **Change** — Comparison to previous run
- **Statistical significance** — p-value and conclusion
- **Outliers** — Anomalous measurements

### Understanding Time Units

| Unit | Name | Value |
|------|------|-------|
| ns | Nanosecond | 0.000000001 seconds |
| µs | Microsecond | 0.000001 seconds |
| ms | Millisecond | 0.001 seconds |
| s | Second | 1 second |

For reference:
- Function call overhead: ~1 ns
- Hash table lookup: ~10-100 ns
- String comparison: ~10-1000 ns (length dependent)
- Disk access: ~1-10 ms

### HTML Reports

Criterion generates HTML reports in `target/criterion/`. Open `report/index.html` for:
- Interactive charts
- Comparison between runs
- Detailed statistics

## Interpreting Results

### What to Look For

1. **Absolute time** — Is it fast enough for your use case?
2. **Scaling** — Does it grow linearly, quadratically, etc.?
3. **Variance** — High variance suggests inconsistent performance
4. **Regressions** — Did recent changes make things slower?

### Common Patterns

**O(n) - Linear:**
```
size=100:   100 µs
size=1000:  1000 µs
size=10000: 10000 µs
```

**O(n log n) - Linearithmic (sorting):**
```
size=100:   100 µs
size=1000:  1500 µs  (not 1000)
size=10000: 20000 µs (not 10000)
```

**O(n²) - Quadratic (bad!):**
```
size=100:   100 µs
size=1000:  10000 µs (100x for 10x size)
size=10000: 1000000 µs
```

## Optimization Tips

Based on benchmarking, common optimizations:

### 1. Reduce Allocations

Bad:
```rust
fn process(items: &[Item]) -> Vec<String> {
    items.iter()
        .map(|i| i.name.clone())  // Allocates for each item
        .collect()
}
```

Better (if you only need references):
```rust
fn process(items: &[Item]) -> Vec<&str> {
    items.iter()
        .map(|i| i.name.as_str())  // No allocation
        .collect()
}
```

### 2. Pre-Allocate Vectors

Bad:
```rust
let mut results = Vec::new();  // Grows incrementally
for item in items {
    results.push(process(item));
}
```

Better:
```rust
let mut results = Vec::with_capacity(items.len());  // Pre-allocate
for item in items {
    results.push(process(item));
}
```

### 3. Avoid Repeated Work

Bad:
```rust
for item in items {
    if item.name.to_lowercase().contains(&query.to_lowercase()) {
        // query.to_lowercase() called every iteration!
    }
}
```

Better:
```rust
let query_lower = query.to_lowercase();  // Once
for item in items {
    if item.name.to_lowercase().contains(&query_lower) {
        // Reuse query_lower
    }
}
```

### 4. Use Appropriate Data Structures

- **HashSet** for membership testing — O(1) vs O(n) for Vec
- **BTreeMap** for sorted keys
- **Vec** for sequential access

## Complete Benchmark File

Here's the complete `benches/benchmarks.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::HashSet;

// Data structures (duplicated from main crate)
#[derive(Debug, Clone)]
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
enum SortColumn { FileName, FilePath, Username, NumLocks }

#[derive(Clone, Copy, PartialEq, Eq)]
enum SortDirection { Ascending, Descending }

// Test data generator
fn generate_locks(count: usize) -> Vec<FileLockInfo> {
    (0..count)
        .map(|i| FileLockInfo {
            file_id: i as u32,
            server_name: "LOCAL".to_string(),
            file_path: format!(r"C:\Data\Folder{}\file{}.dat", i / 100, i),
            file_name: format!("file{}.dat", i),
            username: format!("DOMAIN\\user{}", i % 100),
            permissions: (i % 4) as u32,
            num_locks: ((i % 20) + 1) as u32,
        })
        .collect()
}

// Functions to benchmark
fn extract_filename(path: &str) -> String {
    path.rsplit('\\').next().unwrap_or(path).to_string()
}

fn filter_locks(locks: &[FileLockInfo], query: &str) -> Vec<FileLockInfo> {
    if query.is_empty() { return locks.to_vec(); }
    let query_lower = query.to_lowercase();
    locks.iter()
        .filter(|l| l.file_name.to_lowercase().contains(&query_lower)
                 || l.file_path.to_lowercase().contains(&query_lower)
                 || l.username.to_lowercase().contains(&query_lower))
        .cloned()
        .collect()
}

fn sort_locks(mut locks: Vec<FileLockInfo>, col: SortColumn, dir: SortDirection) -> Vec<FileLockInfo> {
    locks.sort_by(|a, b| {
        let cmp = match col {
            SortColumn::FileName => a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()),
            SortColumn::FilePath => a.file_path.to_lowercase().cmp(&b.file_path.to_lowercase()),
            SortColumn::Username => a.username.to_lowercase().cmp(&b.username.to_lowercase()),
            SortColumn::NumLocks => a.num_locks.cmp(&b.num_locks),
        };
        match dir {
            SortDirection::Ascending => cmp,
            SortDirection::Descending => cmp.reverse(),
        }
    });
    locks
}

fn select_all_ids(locks: &[FileLockInfo]) -> HashSet<u32> {
    locks.iter().map(|l| l.file_id).collect()
}

// Benchmark functions
fn filter_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_locks");
    for size in [100, 1000, 10000] {
        let locks = generate_locks(size);
        group.bench_with_input(BenchmarkId::new("some_matches", size), &locks, |b, locks| {
            b.iter(|| filter_locks(black_box(locks), black_box("file5")))
        });
    }
    group.finish();
}

fn sort_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort_locks");
    for size in [100, 1000, 10000] {
        let locks = generate_locks(size);
        group.bench_with_input(BenchmarkId::new("by_filename", size), &locks, |b, locks| {
            b.iter(|| sort_locks(black_box(locks.clone()), SortColumn::FileName, SortDirection::Ascending))
        });
    }
    group.finish();
}

criterion_group!(benches, filter_benchmarks, sort_benchmarks);
criterion_main!(benches);
```

## What We've Accomplished

You now understand:

✅ Criterion basics — bench_function, black_box, groups  
✅ Parameterized benchmarks — bench_with_input, BenchmarkId  
✅ Benchmark organization — Groups, micro-benchmarks  
✅ Running and interpreting — cargo bench, understanding output  
✅ Optimization strategies — Pre-allocation, avoiding repeated work  

## Key Takeaways

1. **Use black_box** — Prevents compiler from optimizing away benchmarked code
2. **Test multiple sizes** — Understand how performance scales
3. **Benchmark realistic operations** — Combined operations, not just micro-benchmarks
4. **Compare against baselines** — Criterion tracks historical performance
5. **Profile before optimizing** — Measure, don't guess

## Looking Ahead

In the next chapter, we'll cover build configuration—release optimizations, the Windows manifest, and creating a distributable application.

---

**Previous Chapter:** [← Testing](08-testing.md)  
**Next Chapter:** [Build Configuration →](10-build-configuration.md)