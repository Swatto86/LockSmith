# Chapter 0: Introduction

## Welcome to Building LockSmith

Welcome! This book will guide you through building **LockSmith**, a Windows desktop application for viewing and managing file locks. By the end, you'll have created a complete, working application from scratch—and more importantly, you'll understand *why* every piece of code exists and *how* it all fits together.

## What Is LockSmith?

Have you ever tried to delete, move, or rename a file on Windows only to see this frustrating message?

> "The action can't be completed because the file is open in another program."

LockSmith solves this problem. It's a utility that:

- **Shows you which files are currently locked** on your system
- **Reveals who/what is locking them** (the username holding the lock)
- **Lets you forcibly release locks** when applications crash or become unresponsive

Think of it as a locksmith for your file system—hence the name!

## What You'll Learn

This book covers much more than just "how to build this app." You'll learn:

### Rust Programming Fundamentals
- How Rust's ownership system works in practice
- Structs, enums, and pattern matching
- Error handling with `Result` and custom error types
- Traits and implementations
- Modules and code organization

### Windows System Programming
- How to call Windows APIs from Rust (Foreign Function Interface / FFI)
- Working with the Windows Network Management API
- Understanding file handles and locks at the OS level
- Creating Windows application manifests for admin privileges

### GUI Development with egui
- Building immediate-mode graphical interfaces
- Creating responsive layouts with panels, grids, and scroll areas
- Handling user input (clicks, keyboard shortcuts, selections)
- Implementing themes (dark/light mode)
- Persisting user preferences

### Professional Development Practices
- Project structure and organization
- Writing unit tests and integration tests
- Performance benchmarking
- Continuous integration with GitHub Actions
- Build scripts and release optimization

## Prerequisites

This book assumes you understand basic programming concepts:

- **Variables and data types** (integers, strings, booleans)
- **Control flow** (if/else statements, loops, switch/match statements)
- **Functions** (parameters, return values)
- **Basic data structures** (arrays, lists)

You don't need prior Rust experience—we'll explain everything as we go. However, if you've never programmed before, I recommend starting with a beginner programming course first.

## The Technology Stack

Here's what we're using and why:

| Technology | Purpose | Why This Choice? |
|------------|---------|------------------|
| **Rust** | Programming language | Memory safety, performance, excellent Windows support |
| **egui** | GUI framework | Pure Rust, no external dependencies, lightweight |
| **eframe** | Application framework | Integrates egui with native windows |
| **Windows APIs** | File lock operations | Direct access to OS-level functionality |
| **serde** | Serialization | Save/load user preferences |
| **Cargo** | Build system | Rust's standard package manager |

### Why Rust?

Rust gives us:

1. **Memory safety without garbage collection** — The program won't crash from null pointer bugs or memory leaks
2. **Performance** — Code runs as fast as C/C++ 
3. **Excellent tooling** — Cargo handles dependencies, building, testing, and more
4. **Cross-platform potential** — Though LockSmith is Windows-only (due to the APIs we use), Rust works everywhere

### Why egui Instead of Web Technologies?

Many modern desktop apps use web technologies (Electron, Tauri with WebView2). We chose egui because:

| Aspect | egui | WebView2-based |
|--------|------|----------------|
| Binary size | ~4 MB | ~11 MB |
| Memory usage | ~20 MB | ~40 MB |
| Startup time | ~0.5 seconds | ~1.5 seconds |
| Runtime dependencies | None | Requires WebView2 |

For a simple utility like this, a lightweight native solution makes more sense.

## How This Book Is Organized

Each chapter builds on the previous one. Here's the roadmap:

1. **Setting Up the Development Environment** — Installing Rust, configuring your editor
2. **Project Structure and Cargo Basics** — Understanding Rust projects
3. **Core Data Types** — Building our data model
4. **Windows API Integration** — Calling Windows functions from Rust
5. **The Application State** — Managing our app's data
6. **Building the User Interface** — Creating the GUI with egui
7. **User Interactions** — Handling clicks, selections, and keyboard shortcuts
8. **Persistence** — Saving user preferences
9. **Polish and UX** — Themes, status messages, confirmation dialogs
10. **Testing** — Unit tests, integration tests, and test organization
11. **Benchmarking** — Measuring and optimizing performance
12. **Build Configuration** — Release optimization, Windows manifests, icons
13. **Continuous Integration** — Automated testing with GitHub Actions
14. **Final Assembly** — Putting it all together

## How to Use This Book

### If You're Learning

Read chapters in order. Each one introduces new concepts and builds on what came before. Type out the code yourself rather than copying—muscle memory helps!

### If You're Referencing

Use the table of contents to jump to specific topics. Each chapter is self-contained enough to be useful on its own.

### Code Conventions

Throughout this book:

- Code that you should type appears in `code blocks like this`
- New or important terms appear in **bold** the first time they're introduced
- File paths are shown as `src/main.rs` or similar
- Terminal commands are prefixed with `$` (don't type the `$`):
  ```
  $ cargo build
  ```

### A Note on Errors

When learning, errors are your friends! Rust's compiler produces incredibly helpful error messages. If something doesn't work:

1. Read the error message carefully
2. Look at the line number it mentions
3. The compiler often suggests exactly how to fix the problem

## The Final Product

When we're done, LockSmith will have:

- A clean, professional interface
- Real-time file lock enumeration
- Search and filtering capabilities
- Sortable columns
- Multi-select with Ctrl+Click and Shift+Click
- Keyboard shortcuts (F5 to refresh, Delete to release, Ctrl+A to select all)
- Dark and light themes
- Persistent preferences
- Confirmation dialogs for dangerous operations
- A ~4 MB executable with no runtime dependencies

## Let's Begin!

In the next chapter, we'll set up your development environment and create the project skeleton. By the end of Chapter 2, you'll have a window appearing on your screen.

Ready? Let's build something useful!

---

**Next Chapter:** [Setting Up the Development Environment →](01-setup.md)