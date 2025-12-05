# Chapter 4: Windows API Integration

This is where LockSmith gets its superpowers. We'll learn how to call Windows operating system functions directly from Rust to enumerate file locks and force-close them when needed.

## What Is FFI?

**FFI** stands for **Foreign Function Interface**. It's a mechanism that allows code written in one programming language to call functions written in another language.

In our case:
- **Rust** (our language) → calling → **C functions** (Windows API)

The Windows API is written in C/C++ and exposed as DLLs (Dynamic Link Libraries). Rust can call these functions, but we need to tell Rust:
1. What functions exist
2. What parameters they take
3. What they return
4. What data structures they use

This "telling Rust about external functions" is called creating **FFI bindings**.

## The Windows Network Management API

LockSmith uses three functions from `netapi32.dll`:

| Function | Purpose |
|----------|---------|
| `NetFileEnum` | List all open files (file locks) |
| `NetFileClose` | Force-close a file handle |
| `NetApiBufferFree` | Free memory allocated by the API |

These are part of the **Network Management API**, which manages network resources including files shared over the network (SMB shares).

### A Note on Scope

These APIs show files opened via network shares. If a file is opened locally (not through a share), it won't appear. This is a limitation of this particular API—true system-wide handle enumeration would require kernel-mode access or other techniques.

## Declaring External Functions

Here's how we declare the Windows functions we'll call:

```rust
#[cfg(windows)]
#[link(name = "netapi32")]
extern "system" {
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

    fn NetFileClose(servername: *const u16, fileid: u32) -> u32;

    fn NetApiBufferFree(buffer: *mut u8) -> u32;
}
```

### Understanding Each Part

#### Conditional Compilation

```rust
#[cfg(windows)]
```

This entire block only compiles on Windows. On other platforms, this code doesn't exist—it's as if we never wrote it. This is important for:
- Cross-platform compilation (even if we don't run on other platforms)
- Cleaner code organization

#### Linking to the DLL

```rust
#[link(name = "netapi32")]
```

This tells the linker: "Link against `netapi32.dll` (or `netapi32.lib` at compile time)."

When you run the compiled program, Windows will load `netapi32.dll` and connect our function calls to the actual implementations.

#### The extern Block

```rust
extern "system" {
    // function declarations
}
```

`extern` declares external functions (not defined in Rust). The `"system"` part specifies the **calling convention**.

#### Calling Conventions

A **calling convention** defines how functions receive parameters and return values at the machine level:
- How parameters are passed (registers vs. stack)
- Who cleans up the stack (caller vs. callee)
- How the return value is passed back

Common calling conventions:
| Convention | Rust Syntax | Used By |
|------------|-------------|---------|
| C | `extern "C"` | Most C libraries, Unix |
| System | `extern "system"` | Windows API |
| Rust | (default) | Rust functions |

On Windows, `"system"` is the `stdcall` convention used by most Windows APIs.

### Function Signatures

Let's understand `NetFileEnum`:

```rust
fn NetFileEnum(
    servername: *const u16,    // Server name (null for local)
    basepath: *const u16,      // Base path filter (null for all)
    username: *const u16,      // Username filter (null for all)
    level: u32,                // Information level (we use 3)
    bufptr: *mut *mut u8,      // OUT: pointer to buffer pointer
    prefmaxlen: u32,           // Preferred buffer size
    entriesread: *mut u32,     // OUT: entries returned
    totalentries: *mut u32,    // OUT: total entries available
    resume_handle: *mut u32,   // IN/OUT: for pagination
) -> u32;                      // Return code
```

#### Pointer Types

The `*const` and `*mut` are **raw pointers**:

| Type | Meaning |
|------|---------|
| `*const T` | Immutable pointer to T (we can read but not write) |
| `*mut T` | Mutable pointer to T (we can read and write) |

These are "unsafe" in Rust's terminology—the compiler can't verify they're valid. We must use them carefully inside `unsafe` blocks.

#### Why u16 for Strings?

```rust
servername: *const u16,
```

Windows APIs use **UTF-16** encoding for strings. Each character is 16 bits (a `u16`). This is different from Rust's `&str`, which uses UTF-8.

`*const u16` is a pointer to null-terminated UTF-16 string data.

#### Double Pointer

```rust
bufptr: *mut *mut u8,
```

This is a "pointer to a pointer." The API allocates memory internally and gives us back a pointer to that memory. So we pass in the address of our pointer variable, and the API writes the buffer address there.

```
Before: bufptr -> [our_variable = null]
After:  bufptr -> [our_variable = address_of_buffer] -> [buffer data]
```

## Defining Data Structures

The `NetFileEnum` function returns data in a specific format. We need to define a matching Rust struct:

```rust
#[cfg(windows)]
#[repr(C)]
struct FILE_INFO_3 {
    fi3_id: u32,
    fi3_permissions: u32,
    fi3_num_locks: u32,
    fi3_pathname: *mut u16,
    fi3_username: *mut u16,
}
```

### The repr(C) Attribute

```rust
#[repr(C)]
```

This is crucial! It tells Rust to lay out the struct in memory the same way a C compiler would.

Without `#[repr(C)]`, Rust might:
- Reorder fields for better alignment
- Add different amounts of padding
- Use a completely different memory layout

With `#[repr(C)]`, we get predictable C-compatible layout, essential for FFI.

### Field Types

| Field | Type | Meaning |
|-------|------|---------|
| `fi3_id` | `u32` | Unique file identifier |
| `fi3_permissions` | `u32` | Permission flags |
| `fi3_num_locks` | `u32` | Number of locks on this file |
| `fi3_pathname` | `*mut u16` | Pointer to file path (UTF-16) |
| `fi3_username` | `*mut u16` | Pointer to username (UTF-16) |

The "3" in `FILE_INFO_3` refers to the information level—level 3 gives us these fields.

## Windows API Constants

The API uses numeric codes for success, errors, and configuration:

```rust
#[cfg(windows)]
const MAX_PREFERRED_LENGTH: u32 = 0xFFFFFFFF;

#[cfg(windows)]
const NERR_SUCCESS: u32 = 0;

#[cfg(windows)]
const ERROR_MORE_DATA: u32 = 234;

#[cfg(windows)]
const ERROR_ACCESS_DENIED: u32 = 5;

#[cfg(windows)]
const ERROR_NOT_ENOUGH_MEMORY: u32 = 8;
```

| Constant | Meaning |
|----------|---------|
| `MAX_PREFERRED_LENGTH` | "Give me as much data as possible" |
| `NERR_SUCCESS` | Operation succeeded |
| `ERROR_MORE_DATA` | More data available (call again for next page) |
| `ERROR_ACCESS_DENIED` | Need admin privileges |
| `ERROR_NOT_ENOUGH_MEMORY` | System is out of memory |

## Working with Raw Pointers

Before we implement the main functions, let's understand how to work with raw pointers safely.

### Converting UTF-16 to String

```rust
#[cfg(windows)]
unsafe fn from_wide_ptr(ptr: *mut u16) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let len = (0..).take_while(|&i| *ptr.offset(i) != 0).count();
    let slice = std::slice::from_raw_parts(ptr, len);
    String::from_utf16_lossy(slice)
}
```

Let's break this down:

#### The unsafe Keyword

```rust
unsafe fn from_wide_ptr(ptr: *mut u16) -> String {
```

The `unsafe` keyword means "this function contains operations the compiler can't verify are safe." Callers must also use `unsafe` to call it.

Why unsafe?
- We're dereferencing a raw pointer (`*ptr.offset(i)`)
- We're creating a slice from a raw pointer (`from_raw_parts`)
- The compiler can't verify the pointer is valid or the length is correct

#### Null Check

```rust
if ptr.is_null() {
    return String::new();
}
```

Always check for null pointers! Dereferencing null is undefined behavior (crashes, security vulnerabilities, etc.).

#### Finding String Length

```rust
let len = (0..).take_while(|&i| *ptr.offset(i) != 0).count();
```

UTF-16 strings are null-terminated (end with a 0 value). This finds the length:

1. `(0..)` — An infinite iterator: 0, 1, 2, 3, ...
2. `.take_while(|&i| *ptr.offset(i) != 0)` — Keep taking while the character at offset `i` is not null
3. `.count()` — Count how many we took

So for the string `"Hello\0"`, this would count 5 characters.

#### Creating a Slice

```rust
let slice = std::slice::from_raw_parts(ptr, len);
```

`from_raw_parts` creates a slice (`&[u16]`) from a raw pointer and length. This is unsafe because:
- The pointer might not be valid
- The length might be wrong
- The memory might be modified by another thread

We're trusting the Windows API to give us valid data.

#### Converting to String

```rust
String::from_utf16_lossy(slice)
```

Converts UTF-16 to UTF-8 (Rust's `String` format). The `lossy` part means invalid UTF-16 is replaced with the replacement character (�) instead of returning an error.

## Error Mapping Functions

Let's create helper functions to convert Windows error codes to our error types:

```rust
#[cfg(windows)]
fn map_windows_error(code: u32) -> FileEnumError {
    match code {
        ERROR_ACCESS_DENIED => FileEnumError::AccessDenied,
        ERROR_NOT_ENOUGH_MEMORY => FileEnumError::NotEnoughMemory,
        _ => FileEnumError::WindowsApiError(code),
    }
}

#[cfg(windows)]
fn map_release_error(code: u32) -> ReleaseLockError {
    match code {
        ERROR_ACCESS_DENIED => ReleaseLockError::AccessDenied,
        2 => ReleaseLockError::FileNotFound,  // ERROR_FILE_NOT_FOUND
        _ => ReleaseLockError::WindowsApiError(code),
    }
}
```

These convert numeric codes to our descriptive enums, making error handling cleaner.

## Implementing enumerate_files

Now for the main function—enumerating all file locks:

```rust
#[cfg(windows)]
fn enumerate_files() -> Result<Vec<FileLockInfo>, FileEnumError> {
    let mut locks = Vec::new();

    unsafe {
        let mut buffer: *mut u8 = ptr::null_mut();
        let mut entries_read: u32 = 0;
        let mut total_entries: u32 = 0;
        let mut resume_handle: u32 = 0;

        let status = NetFileEnum(
            ptr::null(),          // Local server
            ptr::null(),          // All paths
            ptr::null(),          // All users
            3,                    // Info level 3
            &mut buffer,          // Output buffer
            MAX_PREFERRED_LENGTH, // Get all data
            &mut entries_read,    // Output: count
            &mut total_entries,   // Output: total
            &mut resume_handle,   // For pagination
        );

        if status != NERR_SUCCESS && status != ERROR_MORE_DATA {
            return Err(map_windows_error(status));
        }

        if !buffer.is_null() && entries_read > 0 {
            let files = std::slice::from_raw_parts(
                buffer as *const FILE_INFO_3,
                entries_read as usize,
            );

            for file in files {
                let file_path = from_wide_ptr(file.fi3_pathname);
                let username = from_wide_ptr(file.fi3_username);
                let file_name = extract_filename(&file_path);

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

            NetApiBufferFree(buffer);
        }
    }

    Ok(locks)
}
```

### Understanding the Implementation

#### The Return Type

```rust
fn enumerate_files() -> Result<Vec<FileLockInfo>, FileEnumError>
```

`Result<T, E>` is Rust's standard way to represent operations that can fail:
- `Ok(value)` — Success, containing the result
- `Err(error)` — Failure, containing error information

Here we return either a vector of locks or an error.

#### Variable Initialization

```rust
let mut buffer: *mut u8 = ptr::null_mut();
let mut entries_read: u32 = 0;
let mut total_entries: u32 = 0;
let mut resume_handle: u32 = 0;
```

These are output parameters—the Windows API will write values to them. We initialize:
- `buffer` to null (the API will allocate and set this)
- Counters to 0
- `resume_handle` to 0 (for first call; subsequent calls use the returned value)

`ptr::null_mut()` creates a null mutable pointer.

#### Calling the API

```rust
let status = NetFileEnum(
    ptr::null(),          // servername (null = local)
    ptr::null(),          // basepath (null = all)
    ptr::null(),          // username (null = all)
    3,                    // level (FILE_INFO_3)
    &mut buffer,          // &mut converts to *mut
    MAX_PREFERRED_LENGTH,
    &mut entries_read,
    &mut total_entries,
    &mut resume_handle,
);
```

`&mut variable` takes a mutable reference, which is automatically converted to `*mut` when passed to an FFI function.

#### Error Checking

```rust
if status != NERR_SUCCESS && status != ERROR_MORE_DATA {
    return Err(map_windows_error(status));
}
```

We check for errors. Note that `ERROR_MORE_DATA` is not an error in our case—it means "here's some data, and there's more if you call again." We're getting all data at once, so we treat it as success.

#### Processing Results

```rust
if !buffer.is_null() && entries_read > 0 {
    let files = std::slice::from_raw_parts(
        buffer as *const FILE_INFO_3,
        entries_read as usize,
    );
```

We cast the buffer pointer to our struct type and create a slice. `as` performs type casting.

#### The Loop

```rust
for file in files {
    let file_path = from_wide_ptr(file.fi3_pathname);
    // ...
    locks.push(FileLockInfo { /* ... */ });
}
```

For each `FILE_INFO_3`, we:
1. Convert UTF-16 strings to Rust strings
2. Extract the filename from the path
3. Create our `FileLockInfo` and add it to the vector

#### Cleanup

```rust
NetApiBufferFree(buffer);
```

**Critical!** The Windows API allocated memory for us. We must free it when done, or we have a memory leak. This is manual memory management—Rust can't help us here.

## Implementing release_lock

Releasing a lock is simpler:

```rust
#[cfg(windows)]
fn release_lock(file_id: u32) -> Result<(), ReleaseLockError> {
    unsafe {
        let status = NetFileClose(ptr::null(), file_id);

        if status != NERR_SUCCESS {
            return Err(map_release_error(status));
        }

        Ok(())
    }
}
```

### Understanding the Return Type

```rust
Result<(), ReleaseLockError>
```

The `()` is the **unit type**—it has only one value: `()`. It means "nothing" or "void." So this returns either `Ok(())` (success, no data) or `Err(...)`.

## Checking Admin Status

We can check if we have admin privileges by attempting an API call:

```rust
#[cfg(windows)]
fn is_admin() -> bool {
    unsafe {
        let mut buffer: *mut u8 = ptr::null_mut();
        let mut entries_read: u32 = 0;
        let mut total_entries: u32 = 0;
        let mut resume_handle: u32 = 0;

        let status = NetFileEnum(
            ptr::null(),
            ptr::null(),
            ptr::null(),
            3,
            &mut buffer,
            0,  // Request 0 bytes
            &mut entries_read,
            &mut total_entries,
            &mut resume_handle,
        );

        if !buffer.is_null() {
            NetApiBufferFree(buffer);
        }

        status != ERROR_ACCESS_DENIED
    }
}
```

We request 0 bytes of data. If we get `ERROR_ACCESS_DENIED`, we don't have admin rights. Any other result (including no data found) means we do have access.

## Non-Windows Stubs

For cross-platform compilation, we provide stub implementations for non-Windows:

```rust
#[cfg(not(windows))]
fn enumerate_files() -> Result<Vec<FileLockInfo>, FileEnumError> {
    Err(FileEnumError::WindowsApiError(0))
}

#[cfg(not(windows))]
fn release_lock(_file_id: u32) -> Result<(), ReleaseLockError> {
    Err(ReleaseLockError::WindowsApiError(0))
}

#[cfg(not(windows))]
fn is_admin() -> bool {
    false
}
```

The underscore in `_file_id` tells Rust we're intentionally not using this parameter.

These let the code compile on Linux/macOS (for development/CI), but the functions just return errors.

## The Complete Windows API Code

Here's everything together:

```rust
use std::ptr;

// ============================================================================
// Windows FFI Declarations
// ============================================================================

#[cfg(windows)]
#[repr(C)]
struct FILE_INFO_3 {
    fi3_id: u32,
    fi3_permissions: u32,
    fi3_num_locks: u32,
    fi3_pathname: *mut u16,
    fi3_username: *mut u16,
}

#[cfg(windows)]
#[link(name = "netapi32")]
extern "system" {
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

    fn NetFileClose(servername: *const u16, fileid: u32) -> u32;

    fn NetApiBufferFree(buffer: *mut u8) -> u32;
}

// Windows API constants
#[cfg(windows)]
const MAX_PREFERRED_LENGTH: u32 = 0xFFFFFFFF;
#[cfg(windows)]
const NERR_SUCCESS: u32 = 0;
#[cfg(windows)]
const ERROR_MORE_DATA: u32 = 234;
#[cfg(windows)]
const ERROR_ACCESS_DENIED: u32 = 5;
#[cfg(windows)]
const ERROR_NOT_ENOUGH_MEMORY: u32 = 8;

// ============================================================================
// Helper Functions
// ============================================================================

#[cfg(windows)]
unsafe fn from_wide_ptr(ptr: *mut u16) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let len = (0..).take_while(|&i| *ptr.offset(i) != 0).count();
    let slice = std::slice::from_raw_parts(ptr, len);
    String::from_utf16_lossy(slice)
}

#[cfg(windows)]
fn map_windows_error(code: u32) -> FileEnumError {
    match code {
        ERROR_ACCESS_DENIED => FileEnumError::AccessDenied,
        ERROR_NOT_ENOUGH_MEMORY => FileEnumError::NotEnoughMemory,
        _ => FileEnumError::WindowsApiError(code),
    }
}

#[cfg(windows)]
fn map_release_error(code: u32) -> ReleaseLockError {
    match code {
        ERROR_ACCESS_DENIED => ReleaseLockError::AccessDenied,
        2 => ReleaseLockError::FileNotFound,
        _ => ReleaseLockError::WindowsApiError(code),
    }
}

// ============================================================================
// Windows API Functions
// ============================================================================

#[cfg(windows)]
fn enumerate_files() -> Result<Vec<FileLockInfo>, FileEnumError> {
    let mut locks = Vec::new();

    unsafe {
        let mut buffer: *mut u8 = ptr::null_mut();
        let mut entries_read: u32 = 0;
        let mut total_entries: u32 = 0;
        let mut resume_handle: u32 = 0;

        let status = NetFileEnum(
            ptr::null(),
            ptr::null(),
            ptr::null(),
            3,
            &mut buffer,
            MAX_PREFERRED_LENGTH,
            &mut entries_read,
            &mut total_entries,
            &mut resume_handle,
        );

        if status != NERR_SUCCESS && status != ERROR_MORE_DATA {
            return Err(map_windows_error(status));
        }

        if !buffer.is_null() && entries_read > 0 {
            let files = std::slice::from_raw_parts(
                buffer as *const FILE_INFO_3,
                entries_read as usize,
            );

            for file in files {
                let file_path = from_wide_ptr(file.fi3_pathname);
                let username = from_wide_ptr(file.fi3_username);
                let file_name = extract_filename(&file_path);

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

            NetApiBufferFree(buffer);
        }
    }

    Ok(locks)
}

#[cfg(windows)]
fn release_lock(file_id: u32) -> Result<(), ReleaseLockError> {
    unsafe {
        let status = NetFileClose(ptr::null(), file_id);

        if status != NERR_SUCCESS {
            return Err(map_release_error(status));
        }

        Ok(())
    }
}

#[cfg(windows)]
fn is_admin() -> bool {
    unsafe {
        let mut buffer: *mut u8 = ptr::null_mut();
        let mut entries_read: u32 = 0;
        let mut total_entries: u32 = 0;
        let mut resume_handle: u32 = 0;

        let status = NetFileEnum(
            ptr::null(),
            ptr::null(),
            ptr::null(),
            3,
            &mut buffer,
            0,
            &mut entries_read,
            &mut total_entries,
            &mut resume_handle,
        );

        if !buffer.is_null() {
            NetApiBufferFree(buffer);
        }

        status != ERROR_ACCESS_DENIED
    }
}

// Non-Windows stubs
#[cfg(not(windows))]
fn enumerate_files() -> Result<Vec<FileLockInfo>, FileEnumError> {
    Err(FileEnumError::WindowsApiError(0))
}

#[cfg(not(windows))]
fn release_lock(_file_id: u32) -> Result<(), ReleaseLockError> {
    Err(ReleaseLockError::WindowsApiError(0))
}

#[cfg(not(windows))]
fn is_admin() -> bool {
    false
}
```

## Safety and Best Practices

Working with FFI is inherently unsafe. Here are the rules we follow:

### 1. Minimize Unsafe Code

Keep `unsafe` blocks as small as possible. Do all validation before entering unsafe code.

### 2. Check All Pointers

Never dereference a pointer without checking for null first.

### 3. Free All Allocated Memory

If the API allocates memory, we must free it. Create clear ownership—know who's responsible for freeing what.

### 4. Validate Lengths

Never trust lengths from external sources without bounds checking.

### 5. Handle All Error Cases

Check return codes. Don't assume success.

### 6. Document Assumptions

Comment on why something is safe when using `unsafe`.

## What We've Accomplished

You now understand:

✅ FFI — Calling C functions from Rust  
✅ extern blocks — Declaring foreign functions  
✅ repr(C) — C-compatible struct layout  
✅ Raw pointers — *const T and *mut T  
✅ unsafe — When and why it's needed  
✅ Windows API patterns — Buffers, error codes, memory management  

## Key Takeaways

1. **FFI bridges languages** — We can call any C library from Rust
2. **unsafe is a contract** — You're promising the compiler you've verified safety
3. **Memory management is manual** — Allocated memory must be freed
4. **Error codes need translation** — Convert to Rust idioms (Result, enums)
5. **Conditional compilation keeps code clean** — Platform-specific code stays isolated

## Looking Ahead

In the next chapter, we'll implement the application logic—the methods that tie our data types and Windows API together into a working system.

---

**Previous Chapter:** [← Core Data Types](03-data-types.md)  
**Next Chapter:** [Application Logic →](05-application-logic.md)