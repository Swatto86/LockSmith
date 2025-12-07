# Chapter 13: The Plain English Specification

## The Problem We're Solving

**Have you ever seen this message?**

> "The action can't be completed because the file is open in another program."

This is one of the most frustrating experiences on Windows. You want to delete, rename, move, or modify a file, but Windows won't let you because *something* has the file locked. But what? Windows doesn't tell you.

**The pain points:**

1. **You can't identify the culprit.** Windows tells you the file is in use, but not by whom or what program.

2. **You can't easily fix it.** Your options are:
   - Close every program one by one until it works (tedious)
   - Restart your computer (disruptive)
   - Use Task Manager to kill processes blindly (risky)
   - Give up and try again later (frustrating)

3. **Sometimes nothing seems to have it open.** A program crashed, or closed improperly, but its file handle is still held by the operating system. The file appears locked by a ghost.

4. **Existing solutions are complicated.** Tools like Process Explorer or Handle.exe exist but require technical knowledge and command-line expertise.

**Our solution: LockSmith**

We're building a simple, visual tool that:
- Shows you **exactly which files are locked** on your system
- Tells you **who has them locked** (the username)
- Lets you **force-release the lock** with a click (when you understand the risks)
- Works through a **clean, intuitive GUI** - no command line needed
- Runs as a **small, fast, native application** - no browser, no runtime dependencies

Think of it as a locksmith for your file system. When you're locked out of your own files, this tool helps you get back in.

---

## Introduction

This chapter serves a unique purpose: it describes the entire LockSmith application in plain English, as if you were writing a specification document or a prompt for an AI assistant to build the application from scratch.

If you gave this chapter to a skilled developer or an AI coding assistant, they should be able to recreate LockSmith with all its features. Think of it as the "blueprint in words" for the entire project.

---

## The Complete Specification

### Project Overview

**Name:** LockSmith

**Purpose:** A Windows desktop application that helps users identify and release file locks. When Windows says "The file is in use by another program," this tool shows which files are locked and lets the user force-release those locks.

**Platform:** Windows 10 or later (requires Windows-specific APIs)

**Technology Stack:**
- Language: Rust
- GUI Framework: egui (via eframe)
- Build System: Cargo

**Key Dependencies:**
- `eframe` - For windowing and the egui integration
- `egui` - Immediate-mode GUI library
- `serde` and `serde_json` - For serializing/deserializing settings
- `image` - For loading icon files

---

### Core Functionality

#### 1. Enumerate File Locks

The application must query Windows for all currently open file handles on the local machine. This is done using the Windows Network Management API function `NetFileEnum`.

**What the function needs to do:**
1. Call `NetFileEnum` with level 3 to get `FILE_INFO_3` structures
2. For each file returned, extract:
   - File ID (a unique identifier for this lock)
   - File path (the full path to the locked file)
   - Username (who has the file locked)
   - Number of locks on the file
   - Permission flags
3. Convert Windows wide strings (UTF-16) to Rust strings (UTF-8)
4. Extract just the filename from the full path for display purposes
5. Return a list of all file locks

**Error handling:**
- If access is denied (error code 5), tell the user to run as Administrator
- If there's not enough memory (error code 8), report that error
- For any other error, report the Windows error code

#### 2. Release File Locks

The application must be able to forcibly close file handles using the Windows `NetFileClose` function.

**What the function needs to do:**
1. Take a file ID (from the enumeration)
2. Call `NetFileClose` with that ID
3. Return success or an appropriate error

**Important warnings to convey to users:**
- Force-closing a file can cause data loss
- The application holding the file might crash
- The file could become corrupted
- Only do this if you understand the risks

#### 3. Check Administrator Status

The application should detect whether it's running with Administrator privileges, because the Windows APIs require admin rights.

**Implementation approach:**
- Try calling `NetFileEnum` with a zero-size buffer
- If it returns "access denied," we're not admin
- Display a warning banner if not running as admin

---

### User Interface Design

#### Window Configuration

- **Title:** "LockSmith - Local File Locks"
- **Default size:** 1000 × 660 pixels
- **Minimum size:** 800 × 400 pixels
- **Icon:** A padlock icon (load from file or generate programmatically)

#### Layout (Top to Bottom)

1. **Title Area**
   - Centered heading with lock emoji (🔓) and "LockSmith" text
   - Large, bold font (28pt)

2. **Admin Warning Banner** (conditional)
   - Only shown if NOT running as Administrator
   - Red background with white text
   - Message: "⚠️ Not running as Administrator. File lock operations may fail."

3. **Search/Filter Bar**
   - Label: "Search:"
   - Text input field that filters the lock list
   - Placeholder text: "Filter by filename, path, or username..."
   - Refresh button: "🔄 Refresh (F5)"

4. **Data Table**
   - Four columns: FILE NAME, PATH, USER, LOCKS
   - Column headers are clickable for sorting
   - Show sort direction arrows (↑ or ↓) on the active sort column
   - Rows are clickable for selection
   - Selected rows highlight in blue with white text
   - Alternating row colors (subtle striping)
   - Scrollable if content exceeds available height

5. **Empty States**
   - If loading: Show spinner with "Loading file locks..."
   - If no locks exist: Show lock emoji with "No Locks" and "No files are currently locked on this system."
   - If no matches for search: Show "No matching locks" and "No locks match your search criteria."

6. **Release Button**
   - Centered below the table
   - Text: "Release Selected Lock(s) (Delete)" or "Release Selected Lock(s) (N) (Delete)" where N is the count
   - Red background when enabled, gray when disabled
   - Disabled when nothing is selected
   - Minimum size: 300 × 40 pixels

7. **Status Message Bar**
   - Appears at the bottom when there's a message to show
   - Color-coded by message type:
     - Success: Green background (✓ icon)
     - Error: Red background (✕ icon)
     - Warning: Orange background (⚠ icon)
     - Info: Theme-appropriate gray (ℹ icon)
   - Success and Info messages auto-hide after 5 seconds

8. **Theme Toggle Button**
   - Fixed position in bottom-right corner
   - Circular button (36 × 36 pixels)
   - Shows ☀ (sun) in dark mode, 🌙 (moon) in light mode
   - Tooltip: "Toggle theme"

#### Confirmation Dialog

When the user tries to release locks, show a modal dialog:

- **Title:** "⚠️ Confirm Release"
- **Content:**
  - "Release N file lock(s)?" (where N is the count)
  - Warning list:
    - "This will force-close the file handle(s) and may cause:"
    - "• Data loss if unsaved changes exist"
    - "• Application crashes"
    - "• File corruption"
  - "Only proceed if you understand the risks." (in orange)
- **Buttons:**
  - "Cancel" (normal style)
  - "Release Locks" (red background, white text)

---

### Interaction Behaviors

#### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| F5 | Refresh the file lock list |
| Delete | Release selected locks (shows confirmation) |
| Ctrl+A | Select all visible (filtered) locks |

#### Mouse Interactions

| Action | Behavior |
|--------|----------|
| Click row | Select only that row (clear other selections) |
| Ctrl+Click row | Toggle that row's selection (add/remove) |
| Shift+Click row | Select range from last click to current row |
| Double-click row | Open release confirmation for that single lock |
| Click column header | Sort by that column (ascending) |
| Click same column header again | Toggle sort direction |
| Click different column header | Sort by new column (ascending) |

#### Search/Filter Behavior

- Filter is case-insensitive
- Matches against: filename, full path, and username
- Partial matches work (e.g., "doc" matches "document.txt")
- Filter updates immediately as user types
- Empty search shows all locks

#### Sorting Behavior

- Default: No sorting (order returned by Windows)
- Click column header to sort ascending
- Click again to sort descending
- Click different column resets to ascending
- Sorting is case-insensitive for text columns
- Numeric sorting for lock count column

---

### Data Persistence

#### What to Save

- Dark mode preference (boolean)

#### Storage Method

- Use eframe's built-in storage mechanism
- Store under the app key "LockSmith"
- Use serde for serialization

#### Default Values

- Dark mode: true (default to dark theme)

---

### Data Structures

#### FileLockInfo

Represents a single file lock:

```
FileLockInfo {
    file_id: u32        // Unique identifier for this lock
    server_name: String // Always "LOCAL" for our purposes
    file_path: String   // Full path, e.g., "C:\Users\Admin\doc.txt"
    file_name: String   // Just filename, e.g., "doc.txt"
    username: String    // Who has it locked, e.g., "DOMAIN\user"
    permissions: u32    // Access flags (not displayed but stored)
    num_locks: u32      // Number of locks on this file
}
```

#### Application State

```
LockSmithApp {
    // Data
    all_locks: Vec<FileLockInfo>    // All locks from last refresh
    selected_ids: HashSet<u32>       // IDs of selected locks
    last_selected_index: Option<usize> // For shift+click range selection
    
    // UI State
    search_query: String             // Current filter text
    sort_column: Option<SortColumn>  // Which column to sort by
    sort_direction: SortDirection    // Ascending or Descending
    dark_mode: bool                  // Theme preference
    
    // Status
    status_message: Option<StatusMessage> // Current feedback message
    is_loading: bool                 // True while fetching data
    is_admin: bool                   // True if running as admin
    
    // Dialog State
    show_release_confirm: bool       // True if dialog is open
    pending_release_ids: Vec<u32>    // IDs waiting to be released
}
```

---

### Error Types

#### FileEnumError

For errors when listing file locks:
- `AccessDenied` - Need admin privileges
- `NotEnoughMemory` - System memory issue
- `WindowsApiError(u32)` - Other Windows error with code

#### ReleaseLockError

For errors when releasing locks:
- `AccessDenied` - Need admin privileges
- `FileNotFound` - Lock no longer exists
- `WindowsApiError(u32)` - Other Windows error with code

---

### Windows API Integration

#### Required Windows Functions

From `netapi32.dll`:

1. **NetFileEnum**
   - Purpose: List all open files
   - Level: 3 (FILE_INFO_3)
   - Server: NULL (local machine)
   - Returns: Array of FILE_INFO_3 structures

2. **NetFileClose**
   - Purpose: Force-close a file handle
   - Parameters: Server (NULL for local), File ID
   - Returns: Success or error code

3. **NetApiBufferFree**
   - Purpose: Free memory allocated by NetFileEnum
   - Must be called after processing NetFileEnum results

#### FILE_INFO_3 Structure (C Layout)

```
struct FILE_INFO_3 {
    fi3_id: u32           // File ID
    fi3_permissions: u32  // Access permissions
    fi3_num_locks: u32    // Lock count
    fi3_pathname: *u16    // Wide string pointer to path
    fi3_username: *u16    // Wide string pointer to username
}
```

#### String Conversion

Windows uses UTF-16 ("wide") strings. Convert to Rust UTF-8 strings by:
1. Finding the null terminator (0)
2. Creating a slice of u16 values
3. Using `String::from_utf16_lossy()` for safe conversion

---

### Visual Design

#### Colors (Dark Mode)

| Element | Color |
|---------|-------|
| Background | Default egui dark |
| Selected row | RGB(96, 165, 250) - Light blue |
| Selected text | White |
| Table header | RGB(51, 65, 85) - Slate gray |
| Normal text | RGB(248, 250, 252) - Near white |
| Striped row | RGBA(255, 255, 255, 5) - Very subtle white |

#### Colors (Light Mode)

| Element | Color |
|---------|-------|
| Background | Default egui light |
| Selected row | RGB(96, 165, 250) - Light blue |
| Selected text | White |
| Table header | RGB(226, 232, 240) - Light gray |
| Normal text | RGB(15, 23, 42) - Near black |
| Striped row | RGBA(0, 0, 0, 10) - Very subtle black |

#### Message Colors (Both Modes)

| Type | Background |
|------|------------|
| Success | RGB(16, 185, 129) - Green |
| Error | RGB(239, 68, 68) - Red |
| Warning | RGB(245, 158, 11) - Orange |
| Release button (enabled) | RGB(239, 68, 68) - Red |
| Release button (disabled) | RGB(127, 127, 127) - Gray |

---

### Build Configuration

#### Cargo.toml Dependencies

```toml
[dependencies]
eframe = "0.28"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
image = { version = "0.25", default-features = false, features = ["png"] }

[target.'cfg(windows)'.dependencies]
# Windows-specific if needed
```

#### Release Optimizations

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

#### Windows Manifest

Create an application manifest that:
- Requests `requireAdministrator` or `highestAvailable` execution level
- Enables DPI awareness
- Sets Windows 10/11 compatibility

#### Resource Embedding

Use a build script (build.rs) to embed:
- Windows icon (.ico file)
- Application manifest

---

### Testing Requirements

#### Unit Tests

Test these functions in isolation:
- `extract_filename` - Various path formats, edge cases
- `SortDirection::toggle` - Verify toggling works
- `SortDirection::arrow` - Verify correct arrows returned
- Error type Display implementations
- FileLockInfo serialization/deserialization
- AppPersistence default values

#### Integration Tests

Test these workflows:
- Filtering: Various search terms, case insensitivity
- Sorting: Each column, both directions
- Selection: Single, Ctrl+click, range selection
- Combined filter + sort operations

#### Edge Cases to Test

- Empty lock list
- Very long file paths
- Unicode in filenames and usernames
- Special characters in search
- Maximum u32 values
- Empty strings

---

### Non-Windows Support

For development on non-Windows platforms:
- Provide stub implementations that return errors
- Allow UI development without Windows APIs
- `enumerate_files()` returns `Err(WindowsApiError(0))`
- `release_lock()` returns `Err(WindowsApiError(0))`
- `is_admin()` returns `false`

---

### Startup Sequence

1. Print banner to console (version info)
2. Check admin status and warn if not admin
3. Configure window options (title, size, icon)
4. Create app instance:
   a. Load saved preferences (dark mode)
   b. Apply theme
   c. Check admin status
   d. Perform initial lock enumeration
5. Start eframe event loop

---

### Icon Generation (Fallback)

If no icon file is found, generate a 64×64 padlock icon:

1. Create circular dark blue background
2. Draw rectangular lock body in lower half (light blue)
3. Draw U-shaped shackle on top (only left side, to show "unlocked")
4. Draw circular keyhole cutout in lock body

Colors:
- Background: RGB(30, 41, 59)
- Padlock: RGB(96, 165, 250)

---

## Using This Specification

### For Developers

This document contains everything you need to build LockSmith from scratch:

1. Start with the data structures
2. Implement the Windows FFI layer
3. Create the error types
4. Build the application state
5. Implement the UI following the layout description
6. Add keyboard shortcuts and mouse interactions
7. Implement persistence
8. Add the confirmation dialog
9. Polish with proper error handling and edge cases
10. Write tests

### For AI Assistants

If you're an AI being asked to create this application:

1. **Read the entire specification first** - Don't start coding until you understand the full picture
2. **Create the project structure** - Set up Cargo.toml with dependencies
3. **Implement in order** - FFI → Errors → Data Types → State → UI → Tests
4. **Match the colors and layout exactly** - The visual design is specified precisely
5. **Don't skip error handling** - Every error case is documented
6. **Test as you go** - The test requirements tell you what to verify

### Key Success Criteria

The implementation is correct if:

- [ ] Enumerates file locks on Windows (requires admin)
- [ ] Displays locks in a sortable, filterable table
- [ ] Allows multi-select with Ctrl/Shift+click
- [ ] Shows confirmation before releasing locks
- [ ] Successfully releases locks when confirmed
- [ ] Persists dark/light mode preference
- [ ] Handles all error cases gracefully
- [ ] Compiles on non-Windows (with stub implementations)
- [ ] All keyboard shortcuts work
- [ ] Theme toggle works
- [ ] Status messages appear and auto-hide appropriately

---

## Final Thoughts

This specification represents approximately 1,500 lines of Rust code. The ratio of specification to code is roughly 1:1 in terms of line count, but the specification is much easier to read and understand.

The value of writing specifications like this:

1. **Clarity** - Forces you to think through every detail
2. **Communication** - Anyone can understand what the app does
3. **Reproducibility** - Multiple developers (or AI) could implement this consistently
4. **Documentation** - Serves as both spec and user documentation
5. **Testing** - The behaviors described become test cases

Whether you're working with AI assistants or human teammates, clear specifications lead to better software.

---

**Next Chapter:** [Thinking in Rust →](14-thinking-in-rust.md) — Rust concepts explained for C programmers

**Previous Chapter:** [Final Assembly ←](12-final-assembly.md)

**Back to:** [Table of Contents](README.md)