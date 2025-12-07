//! # LockSmith - A Windows File Lock Management Utility
//!
//! This program is a graphical desktop application that helps Windows users
//! deal with a common frustration: when you try to delete, move, or rename
//! a file and Windows tells you "The file is in use by another program."
//!
//! LockSmith lets you:
//! 1. See all the files that are currently "locked" (held open) on your system
//! 2. Find out WHO or WHAT program has them locked
//! 3. Forcibly release those locks when needed (like when a program crashes
//!    but didn't properly close its files)
//!
//! Think of it like a locksmith who can help you get into your own house
//! when you've lost the key - but for your files!
//!
//! ---
//!
//! # HOW TO READ THIS CODE - A Guide for Learners
//!
//! This file is extensively commented to serve as a learning resource.
//! Here's how to approach reading it:
//!
//! ## Reading Strategy
//!
//! **If you're new to Rust:**
//! 1. Start with the DATA TYPES section - understand what data we work with
//! 2. Read the HELPER FUNCTIONS - these are small and self-contained
//! 3. Look at the APPLICATION STATE struct - this is the "brain" of the app
//! 4. Skim the UI code to see how it all connects
//!
//! **If you're experienced but new to Windows programming:**
//! 1. Focus on the WINDOWS FFI DECLARATIONS section
//! 2. Study the WINDOWS API FUNCTIONS to see how we call Windows
//! 3. Pay attention to the `unsafe` blocks and why they're needed
//!
//! **If you're here to understand GUI development:**
//! 1. Jump to the `impl eframe::App` section
//! 2. Read the `update` method - this is where all UI happens
//! 3. Note how egui's immediate mode works (rebuild UI every frame)
//!
//! ## Comment Conventions Used
//!
//! Throughout this file, you'll see several types of comments:
//!
//! - `//!` at the top = Module-level documentation (what you're reading now)
//! - `// ====` separator lines = Major section boundaries
//! - `// ----` separator lines = Minor section boundaries within UI
//! - `//` above code = Explains WHAT the next line(s) do and WHY
//! - `// inline` = Brief clarification of that specific line
//!
//! ---
//!
//! # QUICK REFERENCE CHEAT SHEET
//!
//! ## File Structure at a Glance
//!
//! | Line Range | Section | Purpose |
//! |------------|---------|---------|
//! | 1-230 | Module docs & imports | Documentation you're reading + dependencies |
//! | 231-330 | Windows FFI | Talking to Windows C APIs |
//! | 331-430 | Error Types | Custom error handling |
//! | 431-530 | Data Types | Core data structures |
//! | 531-730 | Helper & API Functions | Windows interaction logic |
//! | 731-930 | Enums & State | Application configuration types |
//! | 931-1200 | App Methods | Core application logic |
//! | 1201-1800 | UI (update method) | All the GUI code |
//! | 1801-1950 | Entry Point & Icons | main() and icon generation |
//! | 1951-end | Tests | Unit and integration tests |
//!
//! ## Key Functions - What They Do
//!
//! | Function | One-Line Description |
//! |----------|----------------------|
//! | `enumerate_files()` | Ask Windows for all locked files |
//! | `release_lock(id)` | Force-close a specific file lock |
//! | `is_admin()` | Check if running with admin rights |
//! | `extract_filename(path)` | Get "file.txt" from "C:\path\file.txt" |
//! | `from_wide_ptr(ptr)` | Convert Windows string to Rust string |
//! | `refresh_locks()` | Reload the lock list from Windows |
//! | `get_filtered_sorted_locks()` | Apply search filter and sorting |
//! | `handle_sort(column)` | Process column header clicks |
//! | `release_selected_locks()` | Release all selected locks |
//! | `select_all_visible()` | Select all filtered locks (Ctrl+A) |
//! | `show_message(text, type)` | Display status message to user |
//! | `apply_theme(ctx)` | Switch between dark/light mode |
//!
//! ## Key Data Structures
//!
//! ```text
//! FileLockInfo {           LockSmithApp {
//!   file_id: u32             all_locks: Vec<FileLockInfo>
//!   file_path: String        selected_ids: HashSet<u32>
//!   file_name: String        search_query: String
//!   username: String         sort_column: Option<SortColumn>
//!   num_locks: u32           sort_direction: SortDirection
//! }                          dark_mode: bool
//!                            status_message: Option<StatusMessage>
//!                            is_loading: bool
//!                            is_admin: bool
//!                            show_release_confirm: bool
//!                          }
//! ```
//!
//! ## Windows APIs Used
//!
//! | API Function | What It Does | When We Call It |
//! |--------------|--------------|-----------------|
//! | `NetFileEnum` | Lists all open files | On refresh |
//! | `NetFileClose` | Forces a file closed | When releasing locks |
//! | `NetApiBufferFree` | Frees Windows memory | After NetFileEnum |
//!
//! ## UI Layout (Top to Bottom)
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │           🔓 LockSmith                  │  ← Title
//! ├─────────────────────────────────────────┤
//! │ ⚠️ Not running as Administrator...     │  ← Warning (if needed)
//! ├─────────────────────────────────────────┤
//! │ Search: [____________] [🔄 Refresh]    │  ← Search bar
//! ├─────────────────────────────────────────┤
//! │ FILE NAME │ PATH │ USER │ LOCKS        │  ← Table header
//! │───────────┼──────┼──────┼──────────────│
//! │ doc.txt   │ C:\  │ john │ 1            │  ← Data rows
//! │ report.xls│ D:\  │ jane │ 2            │
//! ├─────────────────────────────────────────┤
//! │    [Release Selected Lock(s) (Delete)] │  ← Action button
//! ├─────────────────────────────────────────┤
//! │ ✓ Loaded 2 file lock(s)                │  ← Status message
//! └─────────────────────────────────────────┘
//!                                       [☀]  ← Theme toggle
//! ```
//!
//! ## Keyboard Shortcuts
//!
//! | Key | Action |
//! |-----|--------|
//! | F5 | Refresh lock list |
//! | Delete | Release selected (with confirmation) |
//! | Ctrl+A | Select all visible locks |
//!
//! ## Mouse Interactions
//!
//! | Action | Result |
//! |--------|--------|
//! | Click row | Select only that row |
//! | Ctrl+Click | Toggle row selection |
//! | Shift+Click | Select range |
//! | Double-click | Quick release (with confirmation) |
//! | Click header | Sort by that column |
//!
//! ## The Big Picture
//!
//! Here's what happens when you run LockSmith:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                         STARTUP                                 │
//! │  main() → Create window → Load preferences → Check admin       │
//! │         → Enumerate locks → Show UI                            │
//! └─────────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                      MAIN LOOP (runs every frame)              │
//! │                                                                 │
//! │  1. Check keyboard input (F5, Delete, Ctrl+A)                  │
//! │  2. Update message timers (auto-hide after 5 seconds)          │
//! │  3. Draw confirmation dialog if open                           │
//! │  4. Draw main UI:                                              │
//! │     - Title                                                    │
//! │     - Admin warning (if needed)                                │
//! │     - Search bar + Refresh button                              │
//! │     - Data table (with filtering, sorting, selection)          │
//! │     - Release button                                           │
//! │     - Status message                                           │
//! │  5. Draw theme toggle button                                   │
//! │  6. Request repaint if animations active                       │
//! │                                                                 │
//! └─────────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                      USER ACTIONS                              │
//! │                                                                 │
//! │  Click Refresh → enumerate_files() → Update all_locks         │
//! │  Type in search → Filter updates instantly                     │
//! │  Click column → Sort by that column                           │
//! │  Click row → Select that row                                  │
//! │  Click Release → Show confirmation → release_lock()           │
//! │  Click theme → Toggle dark_mode → Save preference             │
//! │                                                                 │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Key Rust Concepts Demonstrated
//!
//! This code demonstrates many important Rust concepts:
//!
//! | Concept | Where to Find It |
//! |---------|------------------|
//! | Structs | `FileLockInfo`, `LockSmithApp`, `StatusMessage` |
//! | Enums | `SortColumn`, `SortDirection`, `MessageType`, errors |
//! | Traits | `Display` for errors, `Default`, `App` for eframe |
//! | Option | `sort_column`, `status_message`, `last_selected_index` |
//! | Result | Return types of `enumerate_files()`, `release_lock()` |
//! | Vec | `all_locks`, `pending_release_ids` |
//! | HashSet | `selected_ids` |
//! | Closures | Filter/map operations, UI callbacks |
//! | Pattern matching | `match` statements throughout |
//! | Unsafe code | FFI calls to Windows |
//! | Conditional compilation | `#[cfg(windows)]` |
//! | Derive macros | `#[derive(Debug, Clone, Serialize, ...)]` |
//!
//! ---
//!
//! ## How This Program Is Organized
//!
//! The code is organized into several major sections:
//!
//! 1. **Windows FFI Declarations** - The "bridge" code that lets Rust talk to Windows
//! 2. **Error Types** - Custom error types for handling things that can go wrong
//! 3. **Data Types** - The structures that hold our application's data
//! 4. **Helper Functions** - Small utility functions used throughout the program
//! 5. **Windows API Functions** - The actual code that talks to Windows
//! 6. **Application State** - The main application structure and its logic
//! 7. **User Interface** - The GUI code that draws everything on screen
//! 8. **Entry Point** - Where the program starts
//! 9. **Tests** - Code that verifies everything works correctly

// =============================================================================
// COMPILER CONFIGURATION
// =============================================================================
//
// This special instruction tells the Rust compiler: "When building a release
// version of this program (not a debug version), don't create a console window."
//
// Without this, when you double-click the app, you'd see an ugly black console
// window pop up behind your nice GUI. This line prevents that.
//
// The cfg_attr means "only apply this attribute if the condition is met"
// The condition is: not(debug_assertions) - meaning "this is a release build"
// The attribute is: windows_subsystem = "windows" - meaning "this is a GUI app"
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// =============================================================================
// IMPORTS (USE STATEMENTS)
// =============================================================================
//
// These lines bring in code from external libraries (called "crates" in Rust)
// and from Rust's standard library. Think of them like #include in C/C++ or
// import in Python/Java.

// eframe::egui - This is our GUI library. "egui" stands for "easy GUI" and it's
// a simple, fast GUI library written entirely in Rust.
// - Color32: Represents colors as 32-bit values (8 bits each for R, G, B, A)
// - RichText: Text with formatting (bold, color, size, etc.)
// - Stroke: Defines how to draw lines/borders (color and thickness)
// - Vec2: A 2D vector (x, y) - used for sizes and positions
use eframe::egui::{self, Color32, RichText, Stroke, Vec2};

// serde - Short for "SERialize/DEserialize". This library converts Rust data
// structures to/from formats like JSON. We use it to save user preferences.
// - Deserialize: Lets us read data FROM a file/string into a Rust struct
// - Serialize: Lets us write data FROM a Rust struct TO a file/string
use serde::{Deserialize, Serialize};

// std::collections::HashSet - A collection that stores unique values.
// We use it to track which file locks the user has selected (no duplicates!).
use std::collections::HashSet;

// std::fmt - Formatting utilities. We use this to implement Display trait
// so our error types can be printed as human-readable strings.
use std::fmt;

// std::ptr - Raw pointer utilities. Windows APIs often work with raw pointers
// (memory addresses), so we need this to interact with them.
use std::ptr;

// =============================================================================
// WINDOWS FFI DECLARATIONS
// =============================================================================
//
// FFI stands for "Foreign Function Interface". This section defines how Rust
// can talk to Windows' C-based APIs.
//
// Windows doesn't "speak Rust" - its APIs are designed for C/C++. So we need
// to define the exact memory layout of the data structures Windows uses,
// and declare the functions we want to call.
//
// This is like creating a translation dictionary between Rust and Windows.

// The #[cfg(windows)] attribute means: "Only include this code when compiling
// for Windows." If someone tries to compile this on Mac or Linux, this section
// is completely ignored.
#[cfg(windows)]
// #[repr(C)] tells Rust: "Lay out this struct's memory EXACTLY like a C compiler
// would." This is crucial because Windows expects C-style memory layout.
#[repr(C)]
// We allow non-camel-case here because FILE_INFO_3 is the official Windows API name.
// Using the same name makes it easier to look up documentation.
#[allow(non_camel_case_types)]
// FILE_INFO_3 is a Windows structure that contains information about an open file.
// The "3" refers to the "information level" - Windows has different levels that
// return different amounts of detail. Level 3 gives us what we need.
struct FILE_INFO_3 {
    // fi3_id: A unique number that identifies this particular file lock.
    // Think of it like a ticket number - we use this ID to release the lock later.
    fi3_id: u32,

    // fi3_permissions: What kind of access the file was opened with.
    // For example: read-only, write, read-write, etc.
    fi3_permissions: u32,

    // fi3_num_locks: How many "locks" are on this file.
    // A program might have multiple locks on the same file (e.g., locking
    // different sections of the file for different operations).
    fi3_num_locks: u32,

    // fi3_pathname: A POINTER to the file path string.
    // Windows stores strings as "wide" characters (16-bit UTF-16), not the
    // 8-bit characters Rust normally uses. The *mut u16 means "a mutable
    // pointer to a sequence of 16-bit integers".
    fi3_pathname: *mut u16,

    // fi3_username: A POINTER to the username of whoever has this file open.
    // Also a wide string (UTF-16).
    fi3_username: *mut u16,
}

// This block declares the Windows functions we want to call.
// #[link(name = "netapi32")] tells the linker to connect to netapi32.dll,
// which is the Windows library containing network management functions.
#[cfg(windows)]
#[link(name = "netapi32")]
// extern "system" means "use the Windows calling convention" - this affects
// how function arguments are passed at the assembly level.
extern "system" {
    // NetFileEnum: "Network File Enumerate"
    // This function asks Windows: "Give me a list of all files that are
    // currently open on this server."
    //
    // Parameters:
    // - servername: Which server to query (null = local computer)
    // - basepath: Only return files under this path (null = all files)
    // - username: Only return files opened by this user (null = all users)
    // - level: What information to return (we use 3 for FILE_INFO_3)
    // - bufptr: Windows will allocate memory and put the address here
    // - prefmaxlen: Maximum buffer size (0xFFFFFFFF = "as much as needed")
    // - entriesread: Windows puts the count of returned entries here
    // - totalentries: Windows puts the total count available here
    // - resume_handle: For paginating through large result sets
    //
    // Returns: 0 for success, or an error code
    fn NetFileEnum(
        servername: *const u16,
        basepath: *const u16,
        username: *const u16,
        level: u32,
        bufptr: *mut *mut u8,
        prefmaxlen: u32,
        entriesread: *mut u32,
        totalentries: *mut u32,
        resume_handle: *mut u32,
    ) -> u32;

    // NetFileClose: "Network File Close"
    // This function forcibly closes a file that someone else has open.
    // It's like telling Windows: "I don't care who has this file - close it NOW."
    //
    // WARNING: This is dangerous! If a program is in the middle of writing
    // to a file and you force-close it, you could corrupt the data!
    //
    // Parameters:
    // - servername: Which server (null = local computer)
    // - fileid: The ID of the file to close (from fi3_id in FILE_INFO_3)
    //
    // Returns: 0 for success, or an error code
    fn NetFileClose(servername: *const u16, fileid: u32) -> u32;

    // NetApiBufferFree: "Network API Buffer Free"
    // Windows allocated memory for us in NetFileEnum - this function frees it.
    // It's like returning borrowed items when you're done with them.
    //
    // Parameters:
    // - buffer: The memory address to free
    //
    // Returns: 0 for success, or an error code
    fn NetApiBufferFree(buffer: *mut u8) -> u32;
}

// =============================================================================
// WINDOWS API CONSTANTS
// =============================================================================
//
// These are "magic numbers" that Windows uses to communicate status and options.
// We give them meaningful names so our code is readable.

// MAX_PREFERRED_LENGTH: When calling NetFileEnum, this value means
// "allocate as much memory as needed to return all results".
// 0xFFFFFFFF is the maximum value of a 32-bit unsigned integer.
#[cfg(windows)]
const MAX_PREFERRED_LENGTH: u32 = 0xFFFFFFFF;

// NERR_SUCCESS: "Network Error - Success" (yes, success is "error 0")
// This means "the operation completed successfully".
#[cfg(windows)]
const NERR_SUCCESS: u32 = 0;

// ERROR_MORE_DATA: There's more data available than fit in the buffer.
// With MAX_PREFERRED_LENGTH we shouldn't see this, but it's defined for
// completeness.
#[cfg(windows)]
const ERROR_MORE_DATA: u32 = 234;

// ERROR_ACCESS_DENIED: You don't have permission to do this.
// Usually means you need to run the program as Administrator.
#[cfg(windows)]
const ERROR_ACCESS_DENIED: u32 = 5;

// ERROR_NOT_ENOUGH_MEMORY: Windows couldn't allocate the memory it needed.
// Very rare, but possible on systems with extreme memory pressure.
#[cfg(windows)]
const ERROR_NOT_ENOUGH_MEMORY: u32 = 8;

// =============================================================================
// ERROR TYPES
// =============================================================================
//
// In Rust, we define custom error types to handle things that can go wrong.
// This is much better than just returning -1 or throwing exceptions everywhere!
//
// Each error type is an "enum" - a type that can be one of several variants.
// This lets us handle different kinds of errors in different ways.

// FileEnumError: Things that can go wrong when listing file locks
//
// #[derive(...)] automatically generates implementations of common traits:
// - Debug: Lets us print the error with {:?} format
// - Clone: Lets us make copies of the error
#[derive(Debug, Clone)]
enum FileEnumError {
    // AccessDenied: The user doesn't have permission to list file locks.
    // This usually means they need to run the program as Administrator.
    AccessDenied,

    // NotEnoughMemory: Windows couldn't allocate memory for the results.
    // Very rare, but we handle it gracefully.
    NotEnoughMemory,

    // WindowsApiError: Some other Windows error we didn't specifically handle.
    // The u32 inside holds the actual error code for debugging.
    WindowsApiError(u32),
}

// Implement the Display trait for FileEnumError.
// This defines how the error appears when we print it with {} format.
// Without this, we could only print it with {:?} (debug format).
impl fmt::Display for FileEnumError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Match on self to handle each variant differently
        match self {
            // For AccessDenied, give the user helpful instructions
            FileEnumError::AccessDenied => {
                write!(f, "Access denied. Please run as administrator.")
            }
            // For NotEnoughMemory, explain what happened
            FileEnumError::NotEnoughMemory => {
                write!(f, "Not enough memory to complete the operation")
            }
            // For generic Windows errors, show the error code
            FileEnumError::WindowsApiError(code) => {
                write!(f, "Windows API error: code {}", code)
            }
        }
    }
}

// Mark FileEnumError as a standard Rust Error.
// This is a "marker trait" - we don't need to implement anything,
// but it lets us use this error type with the ? operator and error handling.
impl std::error::Error for FileEnumError {}

// ReleaseLockError: Things that can go wrong when releasing a file lock
#[derive(Debug, Clone)]
enum ReleaseLockError {
    // AccessDenied: Same as above - need Administrator privileges
    AccessDenied,

    // FileNotFound: The file lock no longer exists.
    // Maybe the program that had it open closed normally.
    FileNotFound,

    // WindowsApiError: Some other Windows error
    WindowsApiError(u32),
}

// Display implementation for ReleaseLockError
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

// =============================================================================
// DATA TYPES
// =============================================================================
//
// These are the "nouns" of our program - the things we work with.
// In object-oriented terms, these would be our model classes.

// FileLockInfo: All the information about a single file lock
//
// This is what we display in each row of our table.
//
// The Serialize and Deserialize derives let us convert this to/from JSON,
// which we use for the test code (and could be used for exporting data).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileLockInfo {
    // file_id: The unique identifier for this lock.
    // We use this when we want to release the lock.
    file_id: u32,

    // server_name: Which server this lock is on.
    // For our purposes, this is always "LOCAL" since we only
    // look at the local computer.
    server_name: String,

    // file_path: The full path to the locked file.
    // Example: "C:\Users\John\Documents\report.docx"
    file_path: String,

    // file_name: Just the filename part (extracted from file_path).
    // Example: "report.docx"
    // We store this separately for faster searching and display.
    file_name: String,

    // username: Who has this file locked.
    // Example: "DOMAIN\john" or just "john"
    username: String,

    // permissions: The access mode (read, write, etc.)
    // We don't currently display this, but we keep it available.
    permissions: u32,

    // num_locks: How many locks are on this file.
    // Usually 1, but can be more for complex scenarios.
    num_locks: u32,
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================
//
// These are small utility functions that do one specific thing.
// They make the rest of the code cleaner and more readable.

// extract_filename: Gets just the filename from a full path
//
// Input:  "C:\Users\Admin\Documents\report.docx"
// Output: "report.docx"
//
// We use rsplit('\\') which splits the string from the RIGHT side
// on backslash characters. The first element is everything after
// the last backslash (i.e., the filename).
fn extract_filename(path: &str) -> String {
    path.rsplit('\\') // Split from right on backslashes
        .next() // Get the first piece (the filename)
        .unwrap_or(path) // If no backslash found, use the whole path
        .to_string() // Convert to an owned String
}

// from_wide_ptr: Converts a Windows "wide string" to a Rust String
//
// Windows uses UTF-16 encoding (16 bits per character), but Rust uses UTF-8
// (variable width, usually 8 bits for ASCII). This function converts between them.
//
// The `unsafe` keyword is required because we're working with raw pointers -
// Rust can't guarantee this memory is valid, so we have to be careful!
#[cfg(windows)]
unsafe fn from_wide_ptr(ptr: *mut u16) -> String {
    // If the pointer is null (address 0), return an empty string
    // This is defensive programming - we don't crash on bad input
    if ptr.is_null() {
        return String::new();
    }

    // Find the length of the string by counting characters until we hit
    // a null terminator (0). This is how C strings work.
    // We use .take_while() to stop when we find a zero.
    let len = (0..).take_while(|&i| *ptr.offset(i) != 0).count();

    // Create a slice (a view) of the u16 values
    let slice = std::slice::from_raw_parts(ptr, len);

    // Convert from UTF-16 to a Rust String
    // "lossy" means invalid characters get replaced with � instead of crashing
    String::from_utf16_lossy(slice)
}

// map_windows_error: Converts a Windows error code to our FileEnumError type
//
// This function takes a raw number from Windows and gives us a nice
// structured error type that's easier to work with.
#[cfg(windows)]
fn map_windows_error(code: u32) -> FileEnumError {
    match code {
        ERROR_ACCESS_DENIED => FileEnumError::AccessDenied,
        ERROR_NOT_ENOUGH_MEMORY => FileEnumError::NotEnoughMemory,
        _ => FileEnumError::WindowsApiError(code), // Anything else
    }
}

// map_release_error: Converts a Windows error code to ReleaseLockError
//
// Similar to above, but for lock release operations.
// Note: error code 2 is ERROR_FILE_NOT_FOUND in Windows.
#[cfg(windows)]
fn map_release_error(code: u32) -> ReleaseLockError {
    match code {
        ERROR_ACCESS_DENIED => ReleaseLockError::AccessDenied,
        2 => ReleaseLockError::FileNotFound, // ERROR_FILE_NOT_FOUND = 2
        _ => ReleaseLockError::WindowsApiError(code),
    }
}

// =============================================================================
// WINDOWS API FUNCTIONS
// =============================================================================
//
// These functions are the core of our application - they actually talk to
// Windows to get file lock information and release locks.

// enumerate_files: Gets a list of all file locks on the system
//
// This is the main "read" operation - it queries Windows for all open files
// and returns them as a Vec (vector/list) of FileLockInfo structures.
//
// Returns: Ok(Vec<FileLockInfo>) on success, or Err(FileEnumError) on failure
#[cfg(windows)]
fn enumerate_files() -> Result<Vec<FileLockInfo>, FileEnumError> {
    // Create an empty vector to hold our results
    let mut locks = Vec::new();

    // The `unsafe` block is required because we're calling Windows APIs
    // that use raw pointers. We're taking responsibility for memory safety.
    unsafe {
        // These variables will be filled in by Windows:

        // buffer: Where Windows will put the results
        // It starts as a null pointer; Windows will allocate memory and update this
        let mut buffer: *mut u8 = ptr::null_mut();

        // entries_read: How many file locks Windows returned
        let mut entries_read: u32 = 0;

        // total_entries: Total number of file locks available
        // (might be more than entries_read if we hit a size limit)
        let mut total_entries: u32 = 0;

        // resume_handle: For continuing if there are too many results
        // We set this to 0 to start from the beginning
        let mut resume_handle: u32 = 0;

        // Call the Windows API to enumerate files!
        //
        // We pass:
        // - ptr::null() for server (local machine)
        // - ptr::null() for basepath (all paths)
        // - ptr::null() for username (all users)
        // - 3 for level (we want FILE_INFO_3 structures)
        // - Pointers to our variables so Windows can fill them in
        let status = NetFileEnum(
            ptr::null(),          // server = local
            ptr::null(),          // basepath = all
            ptr::null(),          // username = all
            3,                    // level 3 = FILE_INFO_3
            &mut buffer,          // Windows puts results here
            MAX_PREFERRED_LENGTH, // Get all results at once
            &mut entries_read,    // How many we got
            &mut total_entries,   // How many exist total
            &mut resume_handle,   // For pagination (not used)
        );

        // Check if the call succeeded
        // NERR_SUCCESS (0) means success
        // ERROR_MORE_DATA means success but there's more data (we should have all of it though)
        if status != NERR_SUCCESS && status != ERROR_MORE_DATA {
            // Something went wrong - convert to our error type and return
            return Err(map_windows_error(status));
        }

        // If we got results, process them
        if !buffer.is_null() && entries_read > 0 {
            // Cast the raw byte buffer to an array of FILE_INFO_3 structures
            // This is where the "magic" happens - we interpret the raw memory
            // as our defined structure type
            let files =
                std::slice::from_raw_parts(buffer as *const FILE_INFO_3, entries_read as usize);

            // Loop through each file and convert to our FileLockInfo type
            for file in files {
                // Convert the Windows wide strings to Rust strings
                let file_path = from_wide_ptr(file.fi3_pathname);
                let username = from_wide_ptr(file.fi3_username);

                // Extract just the filename for easier display/searching
                let file_name = extract_filename(&file_path);

                // Create our FileLockInfo and add it to the results
                locks.push(FileLockInfo {
                    file_id: file.fi3_id,
                    server_name: "LOCAL".to_string(),
                    file_path,
                    file_name,
                    username,
                    permissions: file.fi3_permissions,
                    num_locks: file.fi3_num_locks,
                });
            }

            // IMPORTANT: Free the memory that Windows allocated!
            // If we don't do this, we have a memory leak.
            NetApiBufferFree(buffer);
        }
    }

    // Return the list of locks (wrapped in Ok because this is a Result)
    Ok(locks)
}

// release_lock: Forces a file lock to be released
//
// This is the "write" operation - it tells Windows to close a file handle
// that belongs to another process.
//
// Parameters:
// - file_id: The ID of the lock to release (from FileLockInfo.file_id)
//
// Returns: Ok(()) on success, or Err(ReleaseLockError) on failure
#[cfg(windows)]
fn release_lock(file_id: u32) -> Result<(), ReleaseLockError> {
    unsafe {
        // Call Windows to close the file
        // First parameter is null (local server)
        // Second parameter is the file ID we want to close
        let status = NetFileClose(ptr::null(), file_id);

        // Check if it worked
        if status != NERR_SUCCESS {
            return Err(map_release_error(status));
        }

        // Success! Return Ok with the "unit" type () which means "nothing"
        Ok(())
    }
}

// is_admin: Checks if we're running with Administrator privileges
//
// We detect this by trying to call NetFileEnum with a zero-size buffer.
// If we get "access denied", we're not admin. Otherwise, we are.
//
// This is a clever trick - we're not actually trying to enumerate files,
// just checking if we COULD if we wanted to.
#[cfg(windows)]
fn is_admin() -> bool {
    unsafe {
        let mut buffer: *mut u8 = ptr::null_mut();
        let mut entries_read: u32 = 0;
        let mut total_entries: u32 = 0;
        let mut resume_handle: u32 = 0;

        // Call NetFileEnum with buffer size 0 - this is just a permission check
        let status = NetFileEnum(
            ptr::null(),
            ptr::null(),
            ptr::null(),
            3,
            &mut buffer,
            0, // Zero buffer size - we just want to check permissions
            &mut entries_read,
            &mut total_entries,
            &mut resume_handle,
        );

        // Clean up if Windows allocated anything
        if !buffer.is_null() {
            NetApiBufferFree(buffer);
        }

        // If we got ACCESS_DENIED, we're not admin
        // Any other result (including success) means we have admin rights
        status != ERROR_ACCESS_DENIED
    }
}

// =============================================================================
// NON-WINDOWS STUBS
// =============================================================================
//
// These are "stub" implementations for non-Windows platforms.
// They allow the code to compile on Mac/Linux (for development purposes)
// but they don't actually work - they just return errors.
//
// This is useful for testing the UI code without needing Windows.

#[cfg(not(windows))]
fn enumerate_files() -> Result<Vec<FileLockInfo>, FileEnumError> {
    // Return an error on non-Windows platforms
    Err(FileEnumError::WindowsApiError(0))
}

#[cfg(not(windows))]
fn release_lock(_file_id: u32) -> Result<(), ReleaseLockError> {
    Err(ReleaseLockError::WindowsApiError(0))
}

#[cfg(not(windows))]
fn is_admin() -> bool {
    false // Never admin on non-Windows
}

// =============================================================================
// APPLICATION STATE - ENUMS
// =============================================================================
//
// These enums define the "options" and "modes" our application can be in.

// SortColumn: Which column the user wants to sort by
//
// When you click a column header in a table, you expect it to sort by that column.
// This enum tracks which column is currently the "active" sort column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SortColumn {
    FileName, // Sort by file name (alphabetically)
    FilePath, // Sort by full path (alphabetically)
    Username, // Sort by who has the lock (alphabetically)
    NumLocks, // Sort by number of locks (numerically)
}

// SortDirection: Ascending (A→Z, 0→9) or Descending (Z→A, 9→0)
//
// Users expect to be able to reverse the sort order by clicking
// the same column header again.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SortDirection {
    Ascending,  // A, B, C... or 1, 2, 3...
    Descending, // Z, Y, X... or 9, 8, 7...
}

impl SortDirection {
    // toggle: Flips between ascending and descending
    //
    // This is called when the user clicks the same column header twice.
    fn toggle(&self) -> Self {
        match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        }
    }

    // arrow: Returns an arrow character to show in the UI
    //
    // ↑ means ascending (small to big, going "up")
    // ↓ means descending (big to small, going "down")
    fn arrow(&self) -> &'static str {
        match self {
            SortDirection::Ascending => " ↑",
            SortDirection::Descending => " ↓",
        }
    }
}

// MessageType: The "severity" of a status message
//
// Different message types get different colors in the UI.
#[derive(Clone)]
#[allow(dead_code)] // Tell compiler it's OK if we don't use all variants
enum MessageType {
    Info,    // Neutral information (blue/gray)
    Success, // Something good happened (green)
    Warning, // Be careful! (yellow/orange)
    Error,   // Something went wrong (red)
}

// StatusMessage: A message to display to the user
//
// These appear at the bottom of the screen to give feedback
// about what just happened (or went wrong).
#[derive(Clone)]
struct StatusMessage {
    // text: The message content
    text: String,

    // message_type: What kind of message (determines color/icon)
    message_type: MessageType,

    // timestamp: When the message was created
    // We use this to auto-hide messages after a few seconds
    timestamp: std::time::Instant,
}

// =============================================================================
// PERSISTENCE (SAVING SETTINGS)
// =============================================================================
//
// We want to remember the user's preferences (like dark/light mode)
// when they close and reopen the app. This struct holds what we save.

// AppPersistence: Data that gets saved between application sessions
//
// The serde derives let us serialize this to/from storage.
// #[serde(default)] means: if loading fails or fields are missing,
// use the Default implementation instead of crashing.
#[derive(Serialize, Deserialize)]
#[serde(default)]
struct AppPersistence {
    // dark_mode: Whether the user prefers dark theme (true) or light (false)
    dark_mode: bool,
}

// Default implementation for AppPersistence
//
// This defines what values we use when there's no saved data.
impl Default for AppPersistence {
    fn default() -> Self {
        Self {
            dark_mode: true, // Dark mode by default (easier on the eyes!)
        }
    }
}

// =============================================================================
// APPLICATION STATE - MAIN STRUCT
// =============================================================================
//
// This is the "brain" of our application. It holds all the data and
// provides methods to manipulate it.
//
// In GUI programming, we separate "state" (data) from "view" (how it looks).
// This struct is the state; the `update` method (below) is the view.

struct LockSmithApp {
    // -------------------------------------------------------------------------
    // DATA: The actual file lock information
    // -------------------------------------------------------------------------

    // all_locks: Every file lock we know about (from the last refresh)
    all_locks: Vec<FileLockInfo>,

    // selected_ids: Which locks the user has selected (by file_id)
    // Using a HashSet because:
    // 1. Fast lookup (O(1) to check if an ID is selected)
    // 2. No duplicates (can't select the same lock twice)
    selected_ids: HashSet<u32>,

    // last_selected_index: The index of the last lock the user clicked
    // We need this for Shift+Click range selection
    // (select everything between last click and current click)
    last_selected_index: Option<usize>,

    // -------------------------------------------------------------------------
    // UI STATE: How the user interface is configured
    // -------------------------------------------------------------------------

    // search_query: What the user has typed in the search box
    // We filter the lock list to only show matching items
    search_query: String,

    // sort_column: Which column to sort by (or None for original order)
    sort_column: Option<SortColumn>,

    // sort_direction: Ascending or descending sort order
    sort_direction: SortDirection,

    // dark_mode: True for dark theme, false for light theme
    dark_mode: bool,

    // -------------------------------------------------------------------------
    // STATUS: Application state flags
    // -------------------------------------------------------------------------

    // status_message: A message to show the user (or None)
    status_message: Option<StatusMessage>,

    // is_loading: True while we're fetching data from Windows
    // Shows a spinner in the UI
    is_loading: bool,

    // is_admin: True if we're running as Administrator
    // Determines whether operations will succeed
    is_admin: bool,

    // -------------------------------------------------------------------------
    // DIALOG STATE: For the confirmation dialog
    // -------------------------------------------------------------------------

    // show_release_confirm: True if the confirmation dialog is visible
    show_release_confirm: bool,

    // pending_release_ids: IDs of locks waiting to be released
    // (after user confirms the dialog)
    pending_release_ids: Vec<u32>,
}

// Default implementation for LockSmithApp
//
// This creates a "blank" application state. We use this as a starting point
// and then customize it in the `new` method.
impl Default for LockSmithApp {
    fn default() -> Self {
        Self {
            all_locks: Vec::new(),        // No locks loaded yet
            selected_ids: HashSet::new(), // Nothing selected
            last_selected_index: None,    // No last selection
            search_query: String::new(),  // Empty search
            sort_column: None,            // No sorting
            sort_direction: SortDirection::Ascending,
            dark_mode: true,                 // Default to dark mode
            status_message: None,            // No message
            is_loading: false,               // Not loading
            is_admin: false,                 // Assume not admin until we check
            show_release_confirm: false,     // Dialog hidden
            pending_release_ids: Vec::new(), // No pending releases
        }
    }
}

// =============================================================================
// APPLICATION STATE - METHODS
// =============================================================================
//
// These are the "verbs" of our application - the actions it can perform.

impl LockSmithApp {
    // new: Creates a new LockSmithApp instance
    //
    // This is called once when the application starts.
    // It loads saved preferences, applies the theme, and fetches initial data.
    //
    // Parameters:
    // - cc: The "creation context" from eframe, which gives us access to
    //       storage (for loading preferences) and the GUI context
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Try to load saved preferences from storage
        // If loading fails, use the default values
        let persistence: AppPersistence = cc
            .storage // Get the storage (might be None)
            .and_then(|s|                         // If we have storage...
                eframe::get_value(s, eframe::APP_KEY)) // Try to read our data
            .unwrap_or_default(); // If anything fails, use default

        // Create the app with default values, but override dark_mode
        // from the saved preferences
        let mut app = Self {
            dark_mode: persistence.dark_mode,
            ..Default::default() // Everything else gets default values
        };

        // Apply the theme (dark or light) to the GUI
        app.apply_theme(cc.egui_ctx.clone());

        // Check if we're running as Administrator
        app.is_admin = is_admin();

        // Load the initial list of file locks
        app.refresh_locks();

        app
    }

    // apply_theme: Sets the visual theme of the application
    //
    // egui has built-in dark and light themes. We just tell it which one to use.
    fn apply_theme(&self, ctx: egui::Context) {
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
    }

    // show_message: Displays a status message to the user
    //
    // This updates the status bar at the bottom of the window.
    // The message will be shown until another message replaces it,
    // or it auto-hides (for success/info messages).
    fn show_message(&mut self, text: impl Into<String>, message_type: MessageType) {
        self.status_message = Some(StatusMessage {
            text: text.into(), // Convert to String (allows &str or String input)
            message_type,
            timestamp: std::time::Instant::now(), // Record when it was shown
        });
    }

    // refresh_locks: Fetches the current list of file locks from Windows
    //
    // This is the main "load data" function. It calls the Windows API,
    // updates our state, and shows a status message.
    fn refresh_locks(&mut self) {
        // Show loading state
        self.is_loading = true;

        // Clear any existing selection (the IDs might not exist anymore)
        self.selected_ids.clear();
        self.last_selected_index = None;

        // Try to enumerate files from Windows
        match enumerate_files() {
            // Success! We got a list of locks.
            Ok(locks) => {
                let count = locks.len();
                self.all_locks = locks;
                self.is_loading = false;

                // Show appropriate message based on count
                if count > 0 {
                    self.show_message(
                        format!("Loaded {} file lock(s)", count),
                        MessageType::Success,
                    );
                } else {
                    self.show_message("No file locks found. System is ready.", MessageType::Info);
                }
            }
            // Error! Something went wrong.
            Err(e) => {
                self.all_locks.clear(); // Clear any stale data
                self.is_loading = false;
                self.show_message(e.to_string(), MessageType::Error);
            }
        }
    }

    // get_filtered_sorted_locks: Returns the locks that match the search and are sorted
    //
    // This is the main "query" function - it takes all_locks and returns
    // just the ones that should be displayed, in the correct order.
    //
    // We don't modify the original data; we create a new filtered/sorted copy.
    fn get_filtered_sorted_locks(&self) -> Vec<FileLockInfo> {
        // Convert search query to lowercase for case-insensitive matching
        let query = self.search_query.to_lowercase();

        // Step 1: Filter
        // Keep only locks that match the search query
        let mut filtered: Vec<FileLockInfo> = self
            .all_locks
            .iter() // Iterate over all locks
            .filter(|lock| {
                // If search is empty, include everything
                if query.is_empty() {
                    true
                } else {
                    // Check if the query appears in filename, path, or username
                    // (all converted to lowercase for case-insensitive matching)
                    lock.file_name.to_lowercase().contains(&query)
                        || lock.file_path.to_lowercase().contains(&query)
                        || lock.username.to_lowercase().contains(&query)
                }
            })
            .cloned() // Make copies (we need owned values, not references)
            .collect(); // Collect into a Vec

        // Step 2: Sort (if a sort column is selected)
        if let Some(column) = &self.sort_column {
            filtered.sort_by(|a, b| {
                // Compare two locks based on the selected column
                let cmp = match column {
                    SortColumn::FileName => {
                        // Compare filenames alphabetically (case-insensitive)
                        a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase())
                    }
                    SortColumn::FilePath => {
                        // Compare paths alphabetically (case-insensitive)
                        a.file_path.to_lowercase().cmp(&b.file_path.to_lowercase())
                    }
                    SortColumn::Username => {
                        // Compare usernames alphabetically (case-insensitive)
                        a.username.to_lowercase().cmp(&b.username.to_lowercase())
                    }
                    SortColumn::NumLocks => {
                        // Compare lock counts numerically
                        a.num_locks.cmp(&b.num_locks)
                    }
                };

                // If descending, reverse the comparison result
                match self.sort_direction {
                    SortDirection::Ascending => cmp,
                    SortDirection::Descending => cmp.reverse(),
                }
            });
        }

        filtered
    }

    // handle_sort: Called when a column header is clicked
    //
    // If clicking the same column, toggle ascending/descending.
    // If clicking a different column, start sorting by that column (ascending).
    fn handle_sort(&mut self, column: SortColumn) {
        if self.sort_column == Some(column) {
            // Same column - toggle direction
            self.sort_direction = self.sort_direction.toggle();
        } else {
            // Different column - start fresh as ascending
            self.sort_column = Some(column);
            self.sort_direction = SortDirection::Ascending;
        }
    }

    // release_selected_locks: Releases all selected file locks
    //
    // This is called after the user confirms the release dialog.
    // It attempts to release each lock, tracks successes/failures,
    // and refreshes the list afterward.
    fn release_selected_locks(&mut self) {
        // Convert HashSet to Vec for iteration
        let ids: Vec<u32> = self.selected_ids.iter().cloned().collect();
        if ids.is_empty() {
            return; // Nothing to do
        }

        let mut success_count = 0;
        let mut errors: Vec<String> = Vec::new();

        // Try to release each lock
        for id in &ids {
            match release_lock(*id) {
                Ok(()) => success_count += 1,
                Err(e) => {
                    // Find the lock info to get the filename for the error message
                    if let Some(lock) = self.all_locks.iter().find(|l| l.file_id == *id) {
                        errors.push(format!("{}: {}", lock.file_name, e));
                    } else {
                        errors.push(format!("ID {}: {}", id, e));
                    }
                }
            }
        }

        // Show success message if any succeeded
        if success_count > 0 {
            self.show_message(
                format!("Successfully released {} lock(s)", success_count),
                MessageType::Success,
            );
        }

        // Show error message if any failed
        if !errors.is_empty() {
            // Truncate the error list if it's too long
            let error_text = if errors.len() > 3 {
                format!("{} failed: {}, ...", errors.len(), errors[..3].join(", "))
            } else {
                format!("{} failed: {}", errors.len(), errors.join(", "))
            };
            self.show_message(error_text, MessageType::Error);
        }

        // Refresh the list to show current state
        self.refresh_locks();
    }

    // select_all_visible: Selects all currently visible (filtered) locks
    //
    // This is called when the user presses Ctrl+A.
    // Note: only selects VISIBLE locks (those matching the current filter).
    fn select_all_visible(&mut self) {
        // Get IDs of all visible locks
        let ids: Vec<u32> = self
            .get_filtered_sorted_locks()
            .iter()
            .map(|lock| lock.file_id)
            .collect();

        // Add all IDs to the selection
        for id in ids {
            self.selected_ids.insert(id);
        }
    }
}

// =============================================================================
// EFRAME APP TRAIT IMPLEMENTATION
// =============================================================================
//
// eframe requires us to implement the `App` trait for our application.
// This trait has two main methods:
// - save: Called when the app needs to save its state
// - update: Called every frame to draw the UI and handle input
//
// =============================================================================
// UNDERSTANDING IMMEDIATE MODE GUI
// =============================================================================
//
// egui uses an "immediate mode" GUI paradigm. This is VERY different from
// traditional "retained mode" GUIs (like Windows Forms, Qt, or web DOM).
//
// ## Retained Mode (Traditional) - How most GUIs work:
//
// ```
// // Create widgets ONCE at startup
// let button = Button::new("Click me");
// window.add(button);
//
// // Later, update the button's state
// button.set_text("New text");
// button.set_enabled(false);
// ```
//
// The GUI framework "retains" (remembers) all your widgets. You create them
// once and then modify their properties later.
//
// ## Immediate Mode (egui) - How THIS code works:
//
// ```
// // Every frame, describe what the UI should look like RIGHT NOW
// fn update(&mut self, ctx: &Context) {
//     if ui.button("Click me").clicked() {
//         // Handle click
//     }
// }
// ```
//
// There are NO persistent widget objects! Every frame (60+ times per second),
// we describe the entire UI from scratch. The framework figures out what
// changed and handles rendering efficiently.
//
// ## Why Immediate Mode?
//
// Pros:
// - Simpler mental model (no widget state to manage)
// - UI always reflects current data (no sync issues)
// - Easy to add/remove UI elements dynamically
// - Less boilerplate code
//
// Cons:
// - Slightly more CPU usage (rebuilding UI each frame)
// - Different way of thinking (takes adjustment)
//
// ## How to Read the update() Method:
//
// Think of it as answering: "What should the screen look like RIGHT NOW?"
//
// The code reads almost like a description:
// - "Show a panel"
// - "Inside it, show a heading"
// - "Then show a search box"
// - "Then show a table with these rows"
// - "If this row is selected, color it blue"
// - "If the user clicked this button, do this action"
//
// Everything is conditional on current state. If `self.is_loading` is true,
// show a spinner. If `self.all_locks` is empty, show "No locks". Etc.
//
// =============================================================================
// - save: Called when the app needs to save its state
// - update: Called every frame to draw the UI and handle input

impl eframe::App for LockSmithApp {
    // save: Saves application state to persistent storage
    //
    // This is called by eframe when the application closes,
    // or periodically during runtime.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let persistence = AppPersistence {
            dark_mode: self.dark_mode,
        };
        eframe::set_value(storage, eframe::APP_KEY, &persistence);
    }

    // update: The main UI function - called every frame
    //
    // This function:
    // 1. Handles keyboard input
    // 2. Updates state (like auto-hiding messages)
    // 3. Draws all the UI elements
    //
    // egui uses an "immediate mode" paradigm - we rebuild the entire UI
    // every frame. This is simpler than retained-mode GUIs where you
    // create widgets once and update them.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // =====================================================================
        // KEYBOARD INPUT HANDLING
        // =====================================================================
        //
        // We check for keyboard shortcuts at the start of each frame.

        ctx.input(|i| {
            // F5: Refresh the lock list (like in Windows Explorer)
            if i.key_pressed(egui::Key::F5) {
                self.refresh_locks();
            }

            // Delete: Release selected locks (with confirmation)
            if i.key_pressed(egui::Key::Delete) && !self.selected_ids.is_empty() {
                // Don't immediately release - show confirmation dialog first
                self.pending_release_ids = self.selected_ids.iter().cloned().collect();
                self.show_release_confirm = true;
            }

            // Ctrl+A: Select all visible locks
            if i.modifiers.ctrl && i.key_pressed(egui::Key::A) {
                self.select_all_visible();
            }
        });

        // =====================================================================
        // AUTO-HIDE STATUS MESSAGES
        // =====================================================================
        //
        // Success and info messages automatically disappear after 5 seconds.
        // Error and warning messages stay until dismissed.

        if let Some(ref msg) = self.status_message {
            match msg.message_type {
                MessageType::Success | MessageType::Info => {
                    // Check if 5 seconds have passed
                    if msg.timestamp.elapsed().as_secs() >= 5 {
                        self.status_message = None;
                    }
                }
                _ => {} // Errors/warnings stay visible
            }
        }

        // =====================================================================
        // CONFIRMATION DIALOG
        // =====================================================================
        //
        // This modal dialog appears when the user wants to release locks.
        // It warns about the dangers and requires explicit confirmation.

        if self.show_release_confirm {
            egui::Window::new("⚠️ Confirm Release")
                .collapsible(false) // Can't collapse this window
                .resizable(false) // Fixed size
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO) // Centered on screen
                .show(ctx, |ui| {
                    // Add some padding at the top
                    ui.add_space(10.0);

                    // Main message - how many locks will be released
                    ui.label(
                        RichText::new(format!(
                            "Release {} file lock(s)?",
                            self.pending_release_ids.len()
                        ))
                        .size(16.0)
                        .strong(),
                    );

                    ui.add_space(10.0);

                    // Warning about consequences
                    ui.label("This will force-close the file handle(s) and may cause:");
                    ui.label("• Data loss if unsaved changes exist");
                    ui.label("• Application crashes");
                    ui.label("• File corruption");

                    ui.add_space(10.0);

                    // Extra emphasis on the danger
                    ui.label(
                        RichText::new("Only proceed if you understand the risks.")
                            .color(Color32::from_rgb(245, 158, 11)), // Orange warning color
                    );

                    ui.add_space(15.0);

                    // Buttons: Cancel and Release
                    ui.horizontal(|ui| {
                        // Cancel button - safe option, comes first
                        if ui.button("Cancel").clicked() {
                            self.show_release_confirm = false;
                            self.pending_release_ids.clear();
                        }

                        ui.add_space(10.0);

                        // Release button - dangerous, styled in red
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("Release Locks").color(Color32::WHITE),
                                )
                                .fill(Color32::from_rgb(239, 68, 68)), // Red background
                            )
                            .clicked()
                        {
                            // User confirmed - do the release
                            self.show_release_confirm = false;
                            self.selected_ids = self.pending_release_ids.drain(..).collect();
                            self.release_selected_locks();
                        }
                    });
                });
        }

        // =====================================================================
        // MAIN UI PANEL
        // =====================================================================
        //
        // This is where we draw the main content of the application.
        // CentralPanel fills all available space.

        egui::CentralPanel::default().show(ctx, |ui| {
            // Get the width so we can center things
            let panel_width = ui.available_width();

            // Padding at top
            ui.add_space(20.0);

            // -----------------------------------------------------------------
            // TITLE
            // -----------------------------------------------------------------
            // The app name with a lock icon, centered

            ui.vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    // Center the title by adding space on the left
                    ui.add_space((ui.available_width() - 180.0) / 2.0);
                    ui.heading(RichText::new("🔓").size(28.0));       // Lock emoji
                    ui.heading(RichText::new("LockSmith").size(28.0).strong());  // App name
                });
            });

            ui.add_space(15.0);

            // -----------------------------------------------------------------
            // ADMIN WARNING BANNER
            // -----------------------------------------------------------------
            // If not running as admin, show a prominent warning

            if !self.is_admin {
                ui.vertical_centered(|ui| {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(239, 68, 68))  // Red background
                        .rounding(5.0)
                        .inner_margin(10.0)
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(
                                    "⚠️ Not running as Administrator. File lock operations may fail.",
                                )
                                .color(Color32::WHITE),
                            );
                        });
                });
                ui.add_space(10.0);
            }

            // -----------------------------------------------------------------
            // SEARCH BAR AND REFRESH BUTTON
            // -----------------------------------------------------------------

            ui.vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    let total_width = panel_width - 60.0;
                    ui.add_space(30.0);

                    // Search label
                    ui.label("Search:");

                    // Search input box
                    let search_width = total_width - 200.0;
                    let _search_response = ui.add_sized(
                        Vec2::new(search_width, 30.0),
                        egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text("Filter by filename, path, or username..."),
                    );

                    ui.add_space(10.0);

                    // Refresh button
                    if ui
                        .add_sized(Vec2::new(130.0, 30.0), egui::Button::new("🔄 Refresh (F5)"))
                        .clicked()
                    {
                        self.refresh_locks();
                    }

                    ui.add_space(30.0);
                });
            });

            ui.add_space(10.0);

            // -----------------------------------------------------------------
            // DATA TABLE
            // -----------------------------------------------------------------
            //
            // This is the main content area showing the list of file locks.
            // We handle three states:
            // 1. Loading - show a spinner
            // 2. Empty - show a friendly message
            // 3. Has data - show the table

            // Get filtered locks BEFORE entering UI closures
            // (Rust's borrow checker requires this)
            let filtered_locks = self.get_filtered_sorted_locks();
            let row_height = 28.0;
            let available_height = ui.available_height() - 120.0;

            // Calculate table width for centering
            let table_width = panel_width - 40.0;

            // Table container with border
            ui.vertical_centered(|ui| {
                ui.set_max_width(table_width);

                egui::Frame::none()
                    .stroke(Stroke::new(
                        1.0,
                        ui.visuals().widgets.noninteractive.bg_stroke.color,
                    ))
                    .rounding(8.0)
                    .inner_margin(0.0)
                    .show(ui, |ui| {
                        // CASE 1: Loading state
                        if self.is_loading {
                            ui.set_min_height(available_height);
                            ui.vertical_centered(|ui| {
                                ui.add_space(available_height / 2.0 - 40.0);
                                ui.spinner();  // Animated loading spinner
                                ui.add_space(10.0);
                                ui.label("Loading file locks...");
                            });
                        }
                        // CASE 2: No data to display
                        else if filtered_locks.is_empty() {
                            ui.set_min_height(available_height);
                            ui.vertical_centered(|ui| {
                                ui.add_space(available_height / 2.0 - 60.0);
                                ui.label(RichText::new("🔓").size(48.0));  // Big lock icon
                                ui.add_space(10.0);
                                // Different message depending on whether it's filtered
                                if self.all_locks.is_empty() {
                                    ui.label(RichText::new("No Locks").size(20.0).strong());
                                    ui.add_space(5.0);
                                    ui.label("No files are currently locked on this system.");
                                } else {
                                    ui.label(RichText::new("No matching locks").size(20.0).strong());
                                    ui.add_space(5.0);
                                    ui.label("No locks match your search criteria.");
                                }
                            });
                        }
                        // CASE 3: We have data - show the table!
                        else {
                            // Scrollable area in case there are many locks
                            egui::ScrollArea::vertical()
                                .max_height(available_height)
                                .show(ui, |ui| {
                                    ui.set_min_width(table_width - 20.0);

                                    // Grid layout for the table
                                    egui::Grid::new("locks_table")
                                        .num_columns(4)
                                        .striped(true)  // Alternating row colors
                                        .min_col_width(80.0)
                                        .show(ui, |ui| {
                                            // =================================
                                            // TABLE HEADER ROW
                                            // =================================

                                            // Header background color
                                            let header_bg = if self.dark_mode {
                                                Color32::from_rgb(51, 65, 85)   // Dark blue-gray
                                            } else {
                                                Color32::from_rgb(226, 232, 240) // Light gray
                                            };

                                            ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                                                header_bg;
                                            ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
                                                header_bg;

                                            // File Name column header
                                            // Shows arrow if sorted by this column
                                            let file_name_text = format!(
                                                "FILE NAME{}",
                                                if self.sort_column == Some(SortColumn::FileName) {
                                                    self.sort_direction.arrow()
                                                } else {
                                                    ""
                                                }
                                            );
                                            if ui
                                                .add_sized(
                                                    Vec2::new(200.0, row_height),
                                                    egui::Button::new(
                                                        RichText::new(file_name_text).strong(),
                                                    )
                                                    .frame(false),
                                                )
                                                .clicked()
                                            {
                                                self.handle_sort(SortColumn::FileName);
                                            }

                                            // Path column header
                                            let path_text = format!(
                                                "PATH{}",
                                                if self.sort_column == Some(SortColumn::FilePath) {
                                                    self.sort_direction.arrow()
                                                } else {
                                                    ""
                                                }
                                            );
                                            if ui
                                                .add_sized(
                                                    Vec2::new(350.0, row_height),
                                                    egui::Button::new(RichText::new(path_text).strong())
                                                        .frame(false),
                                                )
                                                .clicked()
                                            {
                                                self.handle_sort(SortColumn::FilePath);
                                            }

                                            // User column header
                                            let user_text = format!(
                                                "USER{}",
                                                if self.sort_column == Some(SortColumn::Username) {
                                                    self.sort_direction.arrow()
                                                } else {
                                                    ""
                                                }
                                            );
                                            if ui
                                                .add_sized(
                                                    Vec2::new(150.0, row_height),
                                                    egui::Button::new(RichText::new(user_text).strong())
                                                        .frame(false),
                                                )
                                                .clicked()
                                            {
                                                self.handle_sort(SortColumn::Username);
                                            }

                                            // Locks column header
                                            let locks_text = format!(
                                                "LOCKS{}",
                                                if self.sort_column == Some(SortColumn::NumLocks) {
                                                    self.sort_direction.arrow()
                                                } else {
                                                    ""
                                                }
                                            );
                                            if ui
                                                .add_sized(
                                                    Vec2::new(80.0, row_height),
                                                    egui::Button::new(RichText::new(locks_text).strong())
                                                        .frame(false),
                                                )
                                                .clicked()
                                            {
                                                self.handle_sort(SortColumn::NumLocks);
                                            }

                                            ui.end_row();

                                            // Reset style for data rows
                                            ui.reset_style();

                                            // =================================
                                            // TABLE DATA ROWS
                                            // =================================
                                            //
                                            // Loop through each lock and create a row

                                            for (index, lock) in filtered_locks.iter().enumerate() {
                                                // Check if this row is selected
                                                let is_selected =
                                                    self.selected_ids.contains(&lock.file_id);

                                                // Determine row background color:
                                                // - Selected: bright blue
                                                // - Odd row: slightly tinted for striping
                                                // - Even row: transparent
                                                let row_color = if is_selected {
                                                    Color32::from_rgb(96, 165, 250)  // Selection blue
                                                } else if self.dark_mode && index % 2 == 1 {
                                                    Color32::from_rgba_unmultiplied(255, 255, 255, 5)
                                                } else if !self.dark_mode && index % 2 == 1 {
                                                    Color32::from_rgba_unmultiplied(0, 0, 0, 10)
                                                } else {
                                                    Color32::TRANSPARENT
                                                };

                                                // Text color (white on selected, normal otherwise)
                                                let text_color = if is_selected {
                                                    Color32::WHITE
                                                } else if self.dark_mode {
                                                    Color32::from_rgb(248, 250, 252)
                                                } else {
                                                    Color32::from_rgb(15, 23, 42)
                                                };

                                                // Create each cell as a button (so it's clickable)

                                                // File name cell
                                                let file_name_response = ui.add_sized(
                                                    Vec2::new(200.0, row_height),
                                                    egui::Button::new(
                                                        RichText::new(&lock.file_name).color(text_color),
                                                    )
                                                    .fill(row_color)
                                                    .frame(false),
                                                );

                                                // Path cell
                                                let path_response = ui.add_sized(
                                                    Vec2::new(350.0, row_height),
                                                    egui::Button::new(
                                                        RichText::new(&lock.file_path).color(text_color),
                                                    )
                                                    .fill(row_color)
                                                    .frame(false),
                                                );

                                                // Username cell
                                                let user_response = ui.add_sized(
                                                    Vec2::new(150.0, row_height),
                                                    egui::Button::new(
                                                        RichText::new(&lock.username).color(text_color),
                                                    )
                                                    .fill(row_color)
                                                    .frame(false),
                                                );

                                                // Locks count cell
                                                let locks_response = ui.add_sized(
                                                    Vec2::new(80.0, row_height),
                                                    egui::Button::new(
                                                        RichText::new(lock.num_locks.to_string())
                                                            .color(text_color),
                                                    )
                                                    .fill(row_color)
                                                    .frame(false),
                                                );

                                                // =================================
                                                // CLICK HANDLING
                                                // =================================
                                                //
                                                // We handle clicks on any cell in the row.
                                                // Different modifiers change the behavior:
                                                // - Normal click: Select only this row
                                                // - Ctrl+click: Toggle this row (add/remove from selection)
                                                // - Shift+click: Select range from last click to here
                                                // - Double-click: Open release confirmation

                                                // Check if ANY cell was clicked
                                                let clicked = file_name_response.clicked()
                                                    || path_response.clicked()
                                                    || user_response.clicked()
                                                    || locks_response.clicked();

                                                // Check if ANY cell was double-clicked
                                                let double_clicked = file_name_response.double_clicked()
                                                    || path_response.double_clicked()
                                                    || user_response.double_clicked()
                                                    || locks_response.double_clicked();

                                                if double_clicked {
                                                    // Double-click: Quick release (with confirmation)
                                                    self.pending_release_ids = vec![lock.file_id];
                                                    self.show_release_confirm = true;
                                                } else if clicked {
                                                    // Get current keyboard modifiers
                                                    let modifiers = ui.input(|i| i.modifiers);

                                                    if modifiers.ctrl || modifiers.command {
                                                        // Ctrl+Click (or Cmd+Click on Mac):
                                                        // Toggle this item in selection
                                                        if self.selected_ids.contains(&lock.file_id) {
                                                            self.selected_ids.remove(&lock.file_id);
                                                        } else {
                                                            self.selected_ids.insert(lock.file_id);
                                                        }
                                                        self.last_selected_index = Some(index);
                                                    } else if modifiers.shift {
                                                        // Shift+Click: Range selection
                                                        // Select everything from last click to here
                                                        if let Some(last_index) =
                                                            self.last_selected_index
                                                        {
                                                            let start = last_index.min(index);
                                                            let end = last_index.max(index);
                                                            for i in start..=end {
                                                                if let Some(l) = filtered_locks.get(i) {
                                                                    self.selected_ids.insert(l.file_id);
                                                                }
                                                            }
                                                        } else {
                                                            // No previous selection - just select this one
                                                            self.selected_ids.clear();
                                                            self.selected_ids.insert(lock.file_id);
                                                            self.last_selected_index = Some(index);
                                                        }
                                                    } else {
                                                        // Normal click: Single selection
                                                        // Clear previous selection and select just this one
                                                        self.selected_ids.clear();
                                                        self.selected_ids.insert(lock.file_id);
                                                        self.last_selected_index = Some(index);
                                                    }
                                                }

                                                // End this row and move to the next
                                                ui.end_row();
                                            }
                                        });
                                });
                        }
                    });
            });

            ui.add_space(15.0);

            // -----------------------------------------------------------------
            // RELEASE BUTTON
            // -----------------------------------------------------------------
            //
            // Big red button to release selected locks.
            // Disabled when nothing is selected.

            ui.vertical_centered(|ui| {
                // Dynamic button text showing count
                let button_text = if self.selected_ids.is_empty() {
                    "Release Selected Lock(s) (Delete)".to_string()
                } else {
                    format!(
                        "Release Selected Lock(s) ({}) (Delete)",
                        self.selected_ids.len()
                    )
                };

                // Button color depends on whether it's enabled
                let button = egui::Button::new(RichText::new(button_text).color(Color32::WHITE))
                    .fill(if self.selected_ids.is_empty() {
                        Color32::from_rgb(127, 127, 127)  // Gray when disabled
                    } else {
                        Color32::from_rgb(239, 68, 68)    // Red when enabled
                    })
                    .min_size(Vec2::new(300.0, 40.0));

                // add_enabled creates a disabled button when condition is false
                if ui
                    .add_enabled(!self.selected_ids.is_empty(), button)
                    .clicked()
                {
                    // Show confirmation dialog instead of releasing immediately
                    self.pending_release_ids = self.selected_ids.iter().cloned().collect();
                    self.show_release_confirm = true;
                }
            });

            ui.add_space(10.0);

            // -----------------------------------------------------------------
            // STATUS MESSAGE BAR
            // -----------------------------------------------------------------
            //
            // Shows feedback messages at the bottom of the window.
            // Colors indicate the type of message.

            if let Some(ref msg) = self.status_message {
                // Choose colors based on message type
                let (bg_color, text_color) = match msg.message_type {
                    MessageType::Success => (Color32::from_rgb(16, 185, 129), Color32::WHITE),  // Green
                    MessageType::Error => (Color32::from_rgb(239, 68, 68), Color32::WHITE),     // Red
                    MessageType::Warning => (Color32::from_rgb(245, 158, 11), Color32::WHITE),  // Orange
                    MessageType::Info => {
                        // Info adapts to theme
                        if self.dark_mode {
                            (
                                Color32::from_rgb(51, 65, 85),
                                Color32::from_rgb(248, 250, 252),
                            )
                        } else {
                            (
                                Color32::from_rgb(226, 232, 240),
                                Color32::from_rgb(15, 23, 42),
                            )
                        }
                    }
                };

                // Choose icon based on message type
                let icon = match msg.message_type {
                    MessageType::Success => "✓",
                    MessageType::Error => "✕",
                    MessageType::Warning => "⚠",
                    MessageType::Info => "ℹ",
                };

                let msg_text = msg.text.clone();
                let full_text = format!("{} {}", icon, msg_text);

                // Display as a non-clickable button (for proper centering)
                ui.vertical_centered(|ui| {
                    let button = egui::Button::new(
                        RichText::new(&full_text).color(text_color).size(14.0)
                    )
                    .fill(bg_color)
                    .stroke(egui::Stroke::NONE)
                    .rounding(5.0)
                    .sense(egui::Sense::hover()); // Hover only, not clickable

                    ui.add(button);
                });
            }
        });

        // =====================================================================
        // THEME TOGGLE BUTTON
        // =====================================================================
        //
        // A floating button in the bottom-right corner to toggle dark/light mode.
        // Uses egui::Area for absolute positioning.

        egui::Area::new(egui::Id::new("theme_toggle"))
            .fixed_pos(egui::pos2(
                ctx.screen_rect().max.x - 50.0, // 50 pixels from right edge
                ctx.screen_rect().max.y - 50.0, // 50 pixels from bottom edge
            ))
            .show(ctx, |ui| {
                // Show sun in dark mode (click to go light)
                // Show moon in light mode (click to go dark)
                let icon = if self.dark_mode { "☀" } else { "🌙" };

                if ui
                    .add_sized(
                        Vec2::new(36.0, 36.0),
                        egui::Button::new(RichText::new(icon).size(18.0)).rounding(18.0),
                    )
                    .on_hover_text("Toggle theme")
                    .clicked()
                {
                    self.dark_mode = !self.dark_mode;
                    self.apply_theme(ctx.clone());
                }
            });

        // =====================================================================
        // REPAINT REQUEST
        // =====================================================================
        //
        // In immediate-mode GUI, we need to tell egui when to redraw.
        // If we're loading or have a message that might auto-hide,
        // we request a repaint so the UI stays updated.

        if self.is_loading || self.status_message.is_some() {
            ctx.request_repaint();
        }
    }
}

// =============================================================================
// ENTRY POINT
// =============================================================================
//
// This is where the program actually starts executing.
// The main function sets up the application and starts the event loop.

fn main() -> eframe::Result<()> {
    // Print a startup banner to the console (useful for debugging)
    println!("===========================================");
    println!("  LockSmith v{}", env!("CARGO_PKG_VERSION"));
    println!("  Windows File Lock Management Utility");
    println!("===========================================");

    // Check admin status and warn if not running as admin
    #[cfg(windows)]
    {
        if !is_admin() {
            eprintln!("WARNING: Not running with administrator privileges!");
            eprintln!("Some features may not work correctly.");
        } else {
            println!("✓ Running with administrator privileges");
        }
    }

    println!();

    // Configure the native window
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("LockSmith - Local File Locks") // Window title
            .with_inner_size(Vec2::new(1000.0, 660.0)) // Starting size
            .with_min_inner_size(Vec2::new(800.0, 400.0)) // Minimum size
            .with_icon(load_icon()), // Window icon
        ..Default::default()
    };

    // Start the application!
    // eframe::run_native takes:
    // - App name (for storage/identification)
    // - Options (window configuration)
    // - A closure that creates the App instance
    eframe::run_native(
        "LockSmith",
        options,
        Box::new(|cc| Ok(Box::new(LockSmithApp::new(cc)))),
    )
}

// =============================================================================
// ICON LOADING
// =============================================================================
//
// These functions handle loading or generating the application icon.

// load_icon: Tries to load an icon from file, falls back to generated icon
//
// We try multiple paths because the icon might be in different locations
// depending on how the app is run (from source vs installed).
fn load_icon() -> egui::IconData {
    // Try to load the icon from the icons directory
    if let Ok(image) = image::open("icons/128x128.png") {
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();
        egui::IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        }
    } else if let Ok(image) = image::open("src/icons/128x128.png") {
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();
        egui::IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        }
    } else {
        // Generate a proper padlock icon as fallback
        create_padlock_icon()
    }
}

// create_padlock_icon: Generates a padlock icon programmatically
//
// This is a fallback when no icon file is found. It draws a simple
// open padlock shape using basic pixel operations.
//
// The icon is 64x64 pixels with RGBA color (4 bytes per pixel).
fn create_padlock_icon() -> egui::IconData {
    let size: usize = 64;
    // Allocate buffer for all pixels (width * height * 4 bytes per pixel)
    let mut rgba = vec![0u8; size * size * 4];

    // Define our colors
    let bg_color = [30, 41, 59, 255u8]; // Dark blue background
    let lock_color = [96, 165, 250, 255u8]; // Light blue for padlock

    // Fill circular background
    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) * 4;

            // Calculate distance from center for circular shape
            let cx = size as f32 / 2.0;
            let cy = size as f32 / 2.0;
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();

            // If within circle radius, fill with background color
            if dist < (size as f32 / 2.0) - 2.0 {
                rgba[idx] = bg_color[0];
                rgba[idx + 1] = bg_color[1];
                rgba[idx + 2] = bg_color[2];
                rgba[idx + 3] = bg_color[3];
            }
        }
    }

    // Draw padlock body (rectangle in lower portion)
    let body_left = size / 4;
    let body_right = size * 3 / 4;
    let body_top = size / 2;
    let body_bottom = size * 7 / 8;

    for y in body_top..body_bottom {
        for x in body_left..body_right {
            let idx = (y * size + x) * 4;
            rgba[idx] = lock_color[0];
            rgba[idx + 1] = lock_color[1];
            rgba[idx + 2] = lock_color[2];
            rgba[idx + 3] = lock_color[3];
        }
    }

    // Draw shackle (the U-shaped part on top)
    // We draw it "open" (only left side) to represent an unlocked padlock
    let shackle_outer = size / 4;
    let shackle_inner = size / 6;
    let shackle_top = size / 5;
    let shackle_bottom = size / 2 + 4;
    let center_x = size / 2;

    for y in shackle_top..shackle_bottom {
        for x in (body_left - 2)..(body_right + 2) {
            let dx = (x as i32 - center_x as i32).abs() as usize;
            let in_outer = dx <= shackle_outer && dx >= shackle_inner;
            let at_top = y < shackle_top + 8 && dx <= shackle_outer;

            // Only draw left side of shackle (open lock)
            let is_left_side = (x as i32) < center_x as i32;

            if (in_outer || at_top) && (is_left_side || y < shackle_top + 8) {
                let idx = (y * size + x) * 4;
                rgba[idx] = lock_color[0];
                rgba[idx + 1] = lock_color[1];
                rgba[idx + 2] = lock_color[2];
                rgba[idx + 3] = lock_color[3];
            }
        }
    }

    // Draw keyhole (circle cutout in the lock body)
    let keyhole_x = size / 2;
    let keyhole_y = size * 5 / 8;
    let keyhole_radius = size / 12;

    for y in 0..size {
        for x in 0..size {
            let dx = x as i32 - keyhole_x as i32;
            let dy = y as i32 - keyhole_y as i32;
            let dist = ((dx * dx + dy * dy) as f32).sqrt();

            if dist < keyhole_radius as f32 {
                let idx = (y * size + x) * 4;
                rgba[idx] = bg_color[0];
                rgba[idx + 1] = bg_color[1];
                rgba[idx + 2] = bg_color[2];
                rgba[idx + 3] = bg_color[3];
            }
        }
    }

    egui::IconData {
        rgba,
        width: size as u32,
        height: size as u32,
    }
}

// =============================================================================
// TESTS
// =============================================================================
//
// Unit tests verify that individual pieces of our code work correctly.
// They're organized into modules by what they test.
//
// The #[cfg(test)] attribute means this entire section is only compiled
// when running tests (cargo test), not in the actual application.

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Helper Function Tests
    // ========================================================================
    //
    // Tests for the small utility functions

    mod extract_filename_tests {
        use super::*;

        #[test]
        fn test_extract_filename_windows_path() {
            // Standard Windows path
            assert_eq!(extract_filename(r"C:\Users\Admin\file.txt"), "file.txt");
        }

        #[test]
        fn test_extract_filename_unc_path() {
            // UNC network path
            assert_eq!(extract_filename(r"\\server\share\doc.pdf"), "doc.pdf");
        }

        #[test]
        fn test_extract_filename_simple() {
            // Just a filename with no path
            assert_eq!(extract_filename("simple.txt"), "simple.txt");
        }

        #[test]
        fn test_extract_filename_empty() {
            // Empty string edge case
            assert_eq!(extract_filename(""), "");
        }

        #[test]
        fn test_extract_filename_nested_path() {
            // Deeply nested path
            assert_eq!(
                extract_filename(r"C:\Users\Admin\Documents\Projects\LockSmith\src\main.rs"),
                "main.rs"
            );
        }

        #[test]
        fn test_extract_filename_no_extension() {
            // File without extension
            assert_eq!(extract_filename(r"C:\Users\Admin\Makefile"), "Makefile");
        }

        #[test]
        fn test_extract_filename_hidden_file() {
            // Hidden file (starts with dot)
            assert_eq!(extract_filename(r"C:\Users\Admin\.gitignore"), ".gitignore");
        }

        #[test]
        fn test_extract_filename_multiple_dots() {
            // Multiple dots in filename
            assert_eq!(
                extract_filename(r"C:\Users\Admin\file.backup.txt"),
                "file.backup.txt"
            );
        }

        #[test]
        fn test_extract_filename_spaces_in_name() {
            // Spaces in filename
            assert_eq!(
                extract_filename(r"C:\Users\Admin\My Document.docx"),
                "My Document.docx"
            );
        }

        #[test]
        fn test_extract_filename_unicode() {
            // Unicode characters in filename
            assert_eq!(extract_filename(r"C:\Users\Admin\文件.txt"), "文件.txt");
        }

        #[test]
        fn test_extract_filename_trailing_backslash() {
            // Edge case: path ends with backslash
            assert_eq!(extract_filename(r"C:\Users\Admin\"), "");
        }

        #[test]
        fn test_extract_filename_only_backslashes() {
            assert_eq!(extract_filename(r"\\\"), "");
        }

        #[test]
        fn test_extract_filename_root_path() {
            assert_eq!(extract_filename(r"C:\"), "");
        }

        #[test]
        fn test_extract_filename_drive_only() {
            assert_eq!(extract_filename(r"C:"), "C:");
        }
    }

    // ========================================================================
    // SortDirection Tests
    // ========================================================================

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
            // Toggling twice should return to original
            let dir = SortDirection::Ascending;
            assert_eq!(dir.toggle().toggle(), SortDirection::Ascending);

            let dir2 = SortDirection::Descending;
            assert_eq!(dir2.toggle().toggle(), SortDirection::Descending);
        }

        #[test]
        fn test_sort_direction_arrow_ascending() {
            assert_eq!(SortDirection::Ascending.arrow(), " ↑");
        }

        #[test]
        fn test_sort_direction_arrow_descending() {
            assert_eq!(SortDirection::Descending.arrow(), " ↓");
        }

        #[test]
        fn test_sort_direction_equality() {
            assert_eq!(SortDirection::Ascending, SortDirection::Ascending);
            assert_eq!(SortDirection::Descending, SortDirection::Descending);
            assert_ne!(SortDirection::Ascending, SortDirection::Descending);
        }

        #[test]
        fn test_sort_direction_debug_impl() {
            let debug_str = format!("{:?}", SortDirection::Ascending);
            assert!(debug_str.contains("Ascending"));

            let debug_str = format!("{:?}", SortDirection::Descending);
            assert!(debug_str.contains("Descending"));
        }
    }

    // ========================================================================
    // SortColumn Tests
    // ========================================================================

    mod sort_column_tests {
        use super::*;

        #[test]
        fn test_sort_column_equality() {
            assert_eq!(SortColumn::FileName, SortColumn::FileName);
            assert_eq!(SortColumn::FilePath, SortColumn::FilePath);
            assert_eq!(SortColumn::Username, SortColumn::Username);
            assert_eq!(SortColumn::NumLocks, SortColumn::NumLocks);
        }

        #[test]
        fn test_sort_column_inequality() {
            assert_ne!(SortColumn::FileName, SortColumn::FilePath);
            assert_ne!(SortColumn::FileName, SortColumn::Username);
            assert_ne!(SortColumn::FileName, SortColumn::NumLocks);
        }

        #[test]
        fn test_sort_column_copy() {
            let col = SortColumn::FileName;
            let col_copy = col;
            assert_eq!(col, col_copy);
        }
    }

    // ========================================================================
    // Error Type Tests
    // ========================================================================

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
        fn test_file_enum_error_not_enough_memory_display() {
            let err = FileEnumError::NotEnoughMemory;
            let display = format!("{}", err);
            assert!(display.contains("Not enough memory"));
        }

        #[test]
        fn test_file_enum_error_windows_api_error_display() {
            let err = FileEnumError::WindowsApiError(1234);
            let display = format!("{}", err);
            assert!(display.contains("Windows API error"));
            assert!(display.contains("1234"));
        }

        #[test]
        fn test_file_enum_error_clone() {
            let err = FileEnumError::WindowsApiError(5);
            let err_clone = err.clone();
            assert!(matches!(err_clone, FileEnumError::WindowsApiError(5)));
        }

        #[test]
        fn test_file_enum_error_is_std_error() {
            let err: Box<dyn std::error::Error> = Box::new(FileEnumError::AccessDenied);
            assert!(!err.to_string().is_empty());
        }
    }

    mod release_lock_error_tests {
        use super::*;

        #[test]
        fn test_release_lock_error_access_denied_display() {
            let err = ReleaseLockError::AccessDenied;
            let display = format!("{}", err);
            assert!(display.contains("Access denied"));
        }

        #[test]
        fn test_release_lock_error_file_not_found_display() {
            let err = ReleaseLockError::FileNotFound;
            let display = format!("{}", err);
            assert!(display.contains("File not found") || display.contains("already closed"));
        }

        #[test]
        fn test_release_lock_error_windows_api_error_display() {
            let err = ReleaseLockError::WindowsApiError(9999);
            let display = format!("{}", err);
            assert!(display.contains("9999"));
        }
    }

    // ========================================================================
    // Error Mapping Tests (Windows-specific)
    // ========================================================================

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

        #[test]
        fn test_map_windows_error_unknown() {
            let err = map_windows_error(12345);
            assert!(matches!(err, FileEnumError::WindowsApiError(12345)));
        }

        #[test]
        fn test_map_release_error_file_not_found() {
            let err = map_release_error(2);
            assert!(matches!(err, ReleaseLockError::FileNotFound));
        }
    }

    // ========================================================================
    // FileLockInfo Tests
    // ========================================================================

    mod file_lock_info_tests {
        use super::*;

        fn create_test_lock() -> FileLockInfo {
            FileLockInfo {
                file_id: 1234,
                server_name: "LOCAL".to_string(),
                file_path: r"C:\Users\Admin\test.txt".to_string(),
                file_name: "test.txt".to_string(),
                username: "DOMAIN\\user".to_string(),
                permissions: 3,
                num_locks: 1,
            }
        }

        #[test]
        fn test_file_lock_info_creation() {
            let lock = create_test_lock();
            assert_eq!(lock.file_id, 1234);
            assert_eq!(lock.file_name, "test.txt");
        }

        #[test]
        fn test_file_lock_info_clone() {
            let lock = create_test_lock();
            let lock_clone = lock.clone();
            assert_eq!(lock.file_id, lock_clone.file_id);
        }

        #[test]
        fn test_file_lock_info_serialize() {
            let lock = create_test_lock();
            let serialized = serde_json::to_string(&lock).unwrap();
            assert!(serialized.contains("file_id"));
            assert!(serialized.contains("1234"));
        }

        #[test]
        fn test_file_lock_info_deserialize() {
            let json = r#"{
                "file_id": 5678,
                "server_name": "REMOTE",
                "file_path": "D:\\data\\file.doc",
                "file_name": "file.doc",
                "username": "admin",
                "permissions": 1,
                "num_locks": 5
            }"#;
            let lock: FileLockInfo = serde_json::from_str(json).unwrap();
            assert_eq!(lock.file_id, 5678);
            assert_eq!(lock.num_locks, 5);
        }
    }

    // ========================================================================
    // AppPersistence Tests
    // ========================================================================

    mod app_persistence_tests {
        use super::*;

        #[test]
        fn test_app_persistence_default() {
            let persistence = AppPersistence::default();
            assert!(persistence.dark_mode); // Default should be dark mode
        }

        #[test]
        fn test_app_persistence_serialize() {
            let persistence = AppPersistence { dark_mode: false };
            let serialized = serde_json::to_string(&persistence).unwrap();
            assert!(serialized.contains("false"));
        }

        #[test]
        fn test_app_persistence_deserialize_missing_field_uses_default() {
            let json = r#"{}"#;
            let persistence: AppPersistence = serde_json::from_str(json).unwrap();
            assert!(persistence.dark_mode); // Should default to true
        }
    }

    // ========================================================================
    // LockSmithApp State Tests
    // ========================================================================

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
                    file_path: r"C:\Users\Admin\alpha.txt".to_string(),
                    file_name: "alpha.txt".to_string(),
                    username: "user1".to_string(),
                    permissions: 1,
                    num_locks: 3,
                },
                FileLockInfo {
                    file_id: 2,
                    server_name: "LOCAL".to_string(),
                    file_path: r"C:\Users\Admin\beta.doc".to_string(),
                    file_name: "beta.doc".to_string(),
                    username: "user2".to_string(),
                    permissions: 2,
                    num_locks: 1,
                },
                FileLockInfo {
                    file_id: 3,
                    server_name: "LOCAL".to_string(),
                    file_path: r"C:\Data\gamma.pdf".to_string(),
                    file_name: "gamma.pdf".to_string(),
                    username: "admin".to_string(),
                    permissions: 3,
                    num_locks: 5,
                },
                FileLockInfo {
                    file_id: 4,
                    server_name: "LOCAL".to_string(),
                    file_path: r"D:\Projects\Alpha\config.json".to_string(),
                    file_name: "config.json".to_string(),
                    username: "user1".to_string(),
                    permissions: 1,
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
            assert!(app.dark_mode);
        }

        #[test]
        fn test_app_show_message() {
            let mut app = create_test_app();
            app.show_message("Test info message", MessageType::Info);
            assert!(app.status_message.is_some());
        }

        // Filtering Tests

        #[test]
        fn test_get_filtered_sorted_locks_no_filter() {
            let mut app = create_test_app();
            app.all_locks = create_sample_locks();
            let filtered = app.get_filtered_sorted_locks();
            assert_eq!(filtered.len(), 4);
        }

        #[test]
        fn test_get_filtered_sorted_locks_filter_by_filename() {
            let mut app = create_test_app();
            app.all_locks = create_sample_locks();
            app.search_query = "alpha".to_string();
            let filtered = app.get_filtered_sorted_locks();
            assert_eq!(filtered.len(), 2); // alpha.txt and config.json in Alpha folder
        }

        #[test]
        fn test_get_filtered_sorted_locks_case_insensitive() {
            let mut app = create_test_app();
            app.all_locks = create_sample_locks();
            app.search_query = "ALPHA".to_string();
            let filtered = app.get_filtered_sorted_locks();
            assert_eq!(filtered.len(), 2);
        }

        // Sorting Tests

        #[test]
        fn test_get_filtered_sorted_locks_sort_by_filename_ascending() {
            let mut app = create_test_app();
            app.all_locks = create_sample_locks();
            app.sort_column = Some(SortColumn::FileName);
            app.sort_direction = SortDirection::Ascending;
            let filtered = app.get_filtered_sorted_locks();
            assert_eq!(filtered[0].file_name, "alpha.txt");
            assert_eq!(filtered[3].file_name, "gamma.pdf");
        }

        #[test]
        fn test_get_filtered_sorted_locks_sort_by_num_locks() {
            let mut app = create_test_app();
            app.all_locks = create_sample_locks();
            app.sort_column = Some(SortColumn::NumLocks);
            app.sort_direction = SortDirection::Descending;
            let filtered = app.get_filtered_sorted_locks();
            assert_eq!(filtered[0].num_locks, 5);
        }

        // Handle Sort Tests

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

        // Selection Tests

        #[test]
        fn test_select_all_visible() {
            let mut app = create_test_app();
            app.all_locks = create_sample_locks();
            app.select_all_visible();
            assert_eq!(app.selected_ids.len(), 4);
        }

        #[test]
        fn test_select_all_visible_with_filter() {
            let mut app = create_test_app();
            app.all_locks = create_sample_locks();
            app.search_query = "user1".to_string();
            app.select_all_visible();
            assert_eq!(app.selected_ids.len(), 2);
        }
    }

    // ========================================================================
    // Icon Generation Tests
    // ========================================================================

    mod icon_tests {
        use super::*;

        #[test]
        fn test_create_padlock_icon_dimensions() {
            let icon = create_padlock_icon();
            assert_eq!(icon.width, 64);
            assert_eq!(icon.height, 64);
        }

        #[test]
        fn test_create_padlock_icon_data_size() {
            let icon = create_padlock_icon();
            // RGBA = 4 bytes per pixel
            assert_eq!(icon.rgba.len(), 64 * 64 * 4);
        }

        #[test]
        fn test_create_padlock_icon_not_empty() {
            let icon = create_padlock_icon();
            let has_visible_pixels = icon.rgba.chunks(4).any(|pixel| pixel[3] > 0);
            assert!(has_visible_pixels);
        }
    }

    // ========================================================================
    // Non-Windows Stub Tests
    // ========================================================================

    #[cfg(not(windows))]
    mod non_windows_stub_tests {
        use super::*;

        #[test]
        fn test_enumerate_files_returns_error_on_non_windows() {
            let result = enumerate_files();
            assert!(result.is_err());
        }

        #[test]
        fn test_release_lock_returns_error_on_non_windows() {
            let result = release_lock(12345);
            assert!(result.is_err());
        }

        #[test]
        fn test_is_admin_returns_false_on_non_windows() {
            assert!(!is_admin());
        }
    }
}
