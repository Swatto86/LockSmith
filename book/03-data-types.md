# Chapter 3: Core Data Types

Every application needs to model its domain—the "things" it works with. For LockSmith, we need to represent file locks, errors, and application state. This chapter introduces Rust's type system through the lens of building these data structures.

## What Data Does LockSmith Need?

Before writing code, let's think about what we're modeling:

1. **File Lock Information** — Details about a locked file (path, who locked it, etc.)
2. **Errors** — What can go wrong when enumerating or releasing locks
3. **Sorting State** — Which column to sort by, in which direction
4. **Status Messages** — Feedback to show the user
5. **Persistence** — Settings to save between sessions

Let's build each of these.

## Structs: Grouping Related Data

A **struct** (short for "structure") groups related values together. Here's how we represent a file lock:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileLockInfo {
    file_id: u32,
    server_name: String,
    file_path: String,
    file_name: String,
    username: String,
    permissions: u32,
    num_locks: u32,
}
```

### Understanding the Syntax

```rust
struct FileLockInfo {
```

This declares a new type called `FileLockInfo`. Type names in Rust use `PascalCase` (each word capitalized, no underscores).

```rust
    file_id: u32,
```

Each field has a name and a type, separated by a colon. Fields are comma-separated. Field names use `snake_case` (lowercase with underscores).

### Rust's Numeric Types

The `u32` is an **unsigned 32-bit integer**. Rust has explicit numeric types:

| Type | Description | Range |
|------|-------------|-------|
| `u8` | Unsigned 8-bit | 0 to 255 |
| `u16` | Unsigned 16-bit | 0 to 65,535 |
| `u32` | Unsigned 32-bit | 0 to 4,294,967,295 |
| `u64` | Unsigned 64-bit | 0 to 18 quintillion |
| `i8` | Signed 8-bit | -128 to 127 |
| `i16` | Signed 16-bit | -32,768 to 32,767 |
| `i32` | Signed 32-bit | ±2 billion |
| `i64` | Signed 64-bit | ±9 quintillion |
| `f32` | 32-bit float | Decimal numbers |
| `f64` | 64-bit float | More precision |
| `usize` | Pointer-sized unsigned | For indexing |
| `isize` | Pointer-sized signed | Rare |

Why use `u32` for `file_id`? Because Windows APIs return file IDs as 32-bit unsigned integers.

### String vs &str

You'll notice we use `String` for text fields. Rust has two main string types:

| Type | Ownership | Mutability | Use Case |
|------|-----------|------------|----------|
| `String` | Owned | Growable | Storing text in structs |
| `&str` | Borrowed | Fixed | Function parameters, literals |

Think of `String` as a "box" that owns its text data, while `&str` is a "window" looking at text owned elsewhere.

```rust
let owned: String = "hello".to_string();  // Owns the data
let borrowed: &str = "hello";              // Points to static data
```

We use `String` in our struct because `FileLockInfo` needs to own its data.

### The Derive Attribute

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

`#[derive(...)]` is an **attribute** that automatically implements **traits** for your type. Let's understand each trait:

#### Debug

```rust
#[derive(Debug)]
```

Allows printing the struct with `{:?}` format specifier:

```rust
let lock = FileLockInfo { /* ... */ };
println!("{:?}", lock);
// Output: FileLockInfo { file_id: 123, server_name: "LOCAL", ... }
```

Essential for debugging!

#### Clone

```rust
#[derive(Clone)]
```

Allows creating deep copies with `.clone()`:

```rust
let lock1 = FileLockInfo { /* ... */ };
let lock2 = lock1.clone();  // Independent copy
```

Without `Clone`, you can't duplicate a struct.

#### Serialize and Deserialize

```rust
#[derive(Serialize, Deserialize)]
```

These are from the `serde` crate. They allow converting to/from formats like JSON:

```rust
let json = serde_json::to_string(&lock)?;    // Struct → JSON
let lock: FileLockInfo = serde_json::from_str(&json)?;  // JSON → Struct
```

We use these for saving/loading data.

## Enums: One of Several Possibilities

An **enum** (enumeration) represents a value that can be one of several variants. Here's how we represent possible errors:

```rust
#[derive(Debug, Clone)]
enum FileEnumError {
    AccessDenied,
    NotEnoughMemory,
    WindowsApiError(u32),
}
```

### Understanding Enum Variants

Each line inside the enum is a **variant**:

```rust
AccessDenied,           // A simple variant (no data)
NotEnoughMemory,        // Another simple variant
WindowsApiError(u32),   // A variant carrying data (the error code)
```

Variants can:
- Have no data (`AccessDenied`)
- Have one value (`WindowsApiError(u32)`)
- Have multiple values (`Something(u32, String)`)
- Have named fields (`Something { code: u32, message: String }`)

### Using Enums

```rust
// Creating enum values
let err1 = FileEnumError::AccessDenied;
let err2 = FileEnumError::WindowsApiError(1234);

// Pattern matching (like switch, but more powerful)
match err2 {
    FileEnumError::AccessDenied => println!("Access denied"),
    FileEnumError::NotEnoughMemory => println!("Not enough memory"),
    FileEnumError::WindowsApiError(code) => println!("Error code: {}", code),
}
```

### Our Error Enums

LockSmith has two error types:

```rust
#[derive(Debug, Clone)]
enum FileEnumError {
    AccessDenied,
    NotEnoughMemory,
    WindowsApiError(u32),
}

#[derive(Debug, Clone)]
enum ReleaseLockError {
    AccessDenied,
    FileNotFound,
    WindowsApiError(u32),
}
```

Why two separate enums? Because enumerating files and releasing locks can fail for different reasons. Strong typing helps us handle each case appropriately.

## Implementing Traits Manually

While `#[derive]` implements traits automatically, sometimes we need manual implementations. Here's how we implement `Display` for custom error messages:

```rust
use std::fmt;

impl fmt::Display for FileEnumError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileEnumError::AccessDenied => {
                write!(f, "Access denied. Please run as administrator.")
            }
            FileEnumError::NotEnoughMemory => {
                write!(f, "Not enough memory to complete the operation")
            }
            FileEnumError::WindowsApiError(code) => {
                write!(f, "Windows API error: code {}", code)
            }
        }
    }
}
```

### Breaking Down the Implementation

```rust
impl fmt::Display for FileEnumError {
```

`impl TraitName for TypeName` is how you implement a trait. `Display` is the trait for user-friendly formatting (used by `println!("{}", value)`).

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
```

This is the required method signature. Let's decode it:

- `&self` — A reference to the enum value being formatted
- `f: &mut fmt::Formatter<'_>` — A mutable reference to the formatter
- `-> fmt::Result` — Returns success or an error

The `'_` is a **lifetime** annotation. We'll cover lifetimes later; for now, just know it's required here.

```rust
write!(f, "Access denied. Please run as administrator.")
```

The `write!` macro writes formatted text to the formatter. It's like `println!` but writes to `f` instead of stdout.

### Making Our Errors Work with std::error::Error

To use our errors with Rust's standard error handling, we implement the `Error` trait:

```rust
impl std::error::Error for FileEnumError {}
impl std::error::Error for ReleaseLockError {}
```

Yes, these implementations are empty! The `Error` trait has default implementations for its methods, so we just need to say "this type is an Error."

## Enums for Application State

Enums aren't just for errors. They're perfect for any "one of these options" situation:

### Sort Column

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SortColumn {
    FileName,
    FilePath,
    Username,
    NumLocks,
}
```

Notice the new derive traits:

- **Copy** — Allows implicit copying (like integers)
- **PartialEq** — Allows comparison with `==` and `!=`
- **Eq** — Marks equality as total (if `a == b` and `b == c`, then `a == c`)

### Sort Direction

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SortDirection {
    Ascending,
    Descending,
}
```

### Adding Methods to Enums

Enums can have methods too:

```rust
impl SortDirection {
    fn toggle(&self) -> Self {
        match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        }
    }

    fn arrow(&self) -> &'static str {
        match self {
            SortDirection::Ascending => " ↑",
            SortDirection::Descending => " ↓",
        }
    }
}
```

#### Understanding `&self` and `Self`

- `&self` — A reference to the value the method is called on
- `Self` — An alias for the type being implemented (here, `SortDirection`)

So `fn toggle(&self) -> Self` means "given a reference to a SortDirection, return a new SortDirection."

#### Understanding `&'static str`

```rust
fn arrow(&self) -> &'static str {
```

The `'static` lifetime means "this string reference lives for the entire program." String literals like `" ↑"` are stored in the program's binary and never get deallocated.

### Message Types

For user feedback, we categorize messages:

```rust
#[derive(Clone)]
#[allow(dead_code)]
enum MessageType {
    Info,
    Success,
    Warning,
    Error,
}
```

The `#[allow(dead_code)]` attribute tells the compiler not to warn if some variants aren't used. This is useful during development.

### Status Message Struct

```rust
#[derive(Clone)]
struct StatusMessage {
    text: String,
    message_type: MessageType,
    timestamp: std::time::Instant,
}
```

`std::time::Instant` is a monotonic clock—perfect for tracking when a message was shown so we can auto-hide it after a few seconds.

## Structs for Persistence

Some settings should persist between application runs:

```rust
#[derive(Serialize, Deserialize)]
#[serde(default)]
struct AppPersistence {
    dark_mode: bool,
}

impl Default for AppPersistence {
    fn default() -> Self {
        Self { dark_mode: true }
    }
}
```

### The Default Trait

```rust
impl Default for AppPersistence {
    fn default() -> Self {
        Self { dark_mode: true }
    }
}
```

The `Default` trait provides a `default()` method that creates a default instance. We implement it manually to set `dark_mode: true` (dark theme by default).

`Self` inside `impl` blocks refers to the type being implemented.

### The serde(default) Attribute

```rust
#[serde(default)]
```

This tells serde: "If a field is missing during deserialization, use its `Default` value instead of failing."

This is crucial for backwards compatibility. If we add a new field in a future version, old saved data (without that field) will still load correctly.

## The Main Application State

Here's the struct that holds everything:

```rust
struct LockSmithApp {
    // Data
    all_locks: Vec<FileLockInfo>,
    selected_ids: HashSet<u32>,
    last_selected_index: Option<usize>,

    // UI State
    search_query: String,
    sort_column: Option<SortColumn>,
    sort_direction: SortDirection,
    dark_mode: bool,

    // Status
    status_message: Option<StatusMessage>,
    is_loading: bool,
    is_admin: bool,

    // Confirmation dialog
    show_release_confirm: bool,
    pending_release_ids: Vec<u32>,
}
```

### New Types Introduced

#### Vec<T> — Dynamic Arrays

```rust
all_locks: Vec<FileLockInfo>,
```

`Vec<T>` is a **vector**—a growable array. The `<T>` is a **type parameter** (generics). `Vec<FileLockInfo>` is a vector of `FileLockInfo` values.

```rust
let mut locks = Vec::new();        // Create empty vector
locks.push(some_lock);             // Add item
let first = &locks[0];             // Access by index
for lock in &locks { }             // Iterate
let count = locks.len();           // Get length
```

#### HashSet<T> — Unique Collections

```rust
selected_ids: HashSet<u32>,
```

`HashSet<T>` stores unique values with fast lookup (O(1) average).

```rust
use std::collections::HashSet;

let mut ids = HashSet::new();
ids.insert(1);                     // Add value
ids.insert(1);                     // No effect (already present)
let has_one = ids.contains(&1);    // Check membership: true
ids.remove(&1);                    // Remove value
```

We use this for tracking which file locks are selected (no duplicates, fast membership checking).

#### Option<T> — Maybe a Value

```rust
sort_column: Option<SortColumn>,
```

`Option<T>` represents "either a value of type T, or nothing." It's Rust's way of handling nullable values safely.

```rust
enum Option<T> {
    Some(T),    // Has a value
    None,       // No value
}
```

Usage:

```rust
let maybe_column: Option<SortColumn> = Some(SortColumn::FileName);
let no_column: Option<SortColumn> = None;

// Check and use
if let Some(column) = maybe_column {
    println!("Sorting by {:?}", column);
}

// Or with match
match maybe_column {
    Some(column) => println!("Sorting by {:?}", column),
    None => println!("Not sorted"),
}

// Or with unwrap (panics if None!)
let column = maybe_column.unwrap();

// Safer: provide a default
let column = maybe_column.unwrap_or(SortColumn::FileName);
```

### Implementing Default for LockSmithApp

```rust
impl Default for LockSmithApp {
    fn default() -> Self {
        Self {
            all_locks: Vec::new(),
            selected_ids: HashSet::new(),
            last_selected_index: None,
            search_query: String::new(),
            sort_column: None,
            sort_direction: SortDirection::Ascending,
            dark_mode: true,
            status_message: None,
            is_loading: false,
            is_admin: false,
            show_release_confirm: false,
            pending_release_ids: Vec::new(),
        }
    }
}
```

This sets sensible defaults:
- Empty vectors and sets
- `None` for optional values
- Dark mode enabled
- Not loading, not admin (will be checked at startup)

## Helper Functions

Let's also look at a simple helper function:

```rust
fn extract_filename(path: &str) -> String {
    path.rsplit('\\').next().unwrap_or(path).to_string()
}
```

### Breaking This Down

```rust
fn extract_filename(path: &str) -> String {
```

- `fn` declares a function
- `path: &str` takes a borrowed string slice
- `-> String` returns an owned String

```rust
path.rsplit('\\')
```

`rsplit` splits from the right. For `"C:\Users\Admin\file.txt"`, this produces an iterator yielding: `"file.txt"`, `"Admin"`, `"Users"`, `"C:"`.

```rust
.next()
```

Get the first item from the iterator (the filename). Returns `Option<&str>`.

```rust
.unwrap_or(path)
```

If `next()` returned `None`, use the original path instead.

```rust
.to_string()
```

Convert the `&str` to an owned `String`.

## The Complete Data Types Code

Here's all the data types together:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, Clone)]
enum FileEnumError {
    AccessDenied,
    NotEnoughMemory,
    WindowsApiError(u32),
}

impl fmt::Display for FileEnumError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileEnumError::AccessDenied => {
                write!(f, "Access denied. Please run as administrator.")
            }
            FileEnumError::NotEnoughMemory => {
                write!(f, "Not enough memory to complete the operation")
            }
            FileEnumError::WindowsApiError(code) => {
                write!(f, "Windows API error: code {}", code)
            }
        }
    }
}

impl std::error::Error for FileEnumError {}

#[derive(Debug, Clone)]
enum ReleaseLockError {
    AccessDenied,
    FileNotFound,
    WindowsApiError(u32),
}

impl fmt::Display for ReleaseLockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReleaseLockError::AccessDenied => {
                write!(f, "Access denied. Please run as administrator.")
            }
            ReleaseLockError::FileNotFound => {
                write!(f, "File not found or already closed")
            }
            ReleaseLockError::WindowsApiError(code) => {
                write!(f, "Windows API error: code {}", code)
            }
        }
    }
}

impl std::error::Error for ReleaseLockError {}

// ============================================================================
// Data Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileLockInfo {
    file_id: u32,
    server_name: String,
    file_path: String,
    file_name: String,
    username: String,
    permissions: u32,
    num_locks: u32,
}

// ============================================================================
// Application State Types
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SortColumn {
    FileName,
    FilePath,
    Username,
    NumLocks,
}

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

    fn arrow(&self) -> &'static str {
        match self {
            SortDirection::Ascending => " ↑",
            SortDirection::Descending => " ↓",
        }
    }
}

#[derive(Clone)]
#[allow(dead_code)]
enum MessageType {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone)]
struct StatusMessage {
    text: String,
    message_type: MessageType,
    timestamp: std::time::Instant,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
struct AppPersistence {
    dark_mode: bool,
}

impl Default for AppPersistence {
    fn default() -> Self {
        Self { dark_mode: true }
    }
}

struct LockSmithApp {
    // Data
    all_locks: Vec<FileLockInfo>,
    selected_ids: HashSet<u32>,
    last_selected_index: Option<usize>,

    // UI State
    search_query: String,
    sort_column: Option<SortColumn>,
    sort_direction: SortDirection,
    dark_mode: bool,

    // Status
    status_message: Option<StatusMessage>,
    is_loading: bool,
    is_admin: bool,

    // Confirmation dialog
    show_release_confirm: bool,
    pending_release_ids: Vec<u32>,
}

impl Default for LockSmithApp {
    fn default() -> Self {
        Self {
            all_locks: Vec::new(),
            selected_ids: HashSet::new(),
            last_selected_index: None,
            search_query: String::new(),
            sort_column: None,
            sort_direction: SortDirection::Ascending,
            dark_mode: true,
            status_message: None,
            is_loading: false,
            is_admin: false,
            show_release_confirm: false,
            pending_release_ids: Vec::new(),
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn extract_filename(path: &str) -> String {
    path.rsplit('\\').next().unwrap_or(path).to_string()
}
```

## What We've Accomplished

You now understand:

✅ Structs — Grouping related data  
✅ Enums — Representing one-of-several values  
✅ Derive attributes — Automatic trait implementations  
✅ Manual trait implementations — Custom Display formatting  
✅ Rust's type system — String, &str, Option, Vec, HashSet  
✅ Methods on types — impl blocks with &self  

## Key Takeaways

1. **Structs group data, enums represent choices** — Use structs when you need "all of these," enums when you need "one of these"

2. **Derive saves boilerplate** — `#[derive(Debug, Clone)]` implements common functionality automatically

3. **Option<T> replaces null** — No null pointer exceptions in Rust

4. **Vec and HashSet are your workhorses** — Dynamic arrays and unique sets

5. **Traits define behavior** — Types implement traits to gain capabilities (Display, Error, Default)

## Looking Ahead

In the next chapter, we'll dive into Windows API integration—calling Windows system functions from Rust to enumerate and release file locks. This is where the real magic happens!

---

**Previous Chapter:** [← Project Structure and Cargo](02-project-structure.md)  
**Next Chapter:** [Windows API Integration →](04-windows-api.md)