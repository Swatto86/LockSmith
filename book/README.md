# Building LockSmith: A Complete Guide

A comprehensive tutorial on building a Windows file lock management utility with Rust and egui.

## About This Book

This book guides you through building **LockSmith** from scratch—a native Windows application for viewing and managing file locks. You'll learn Rust programming, Windows API integration, GUI development with egui, testing, benchmarking, and professional development practices.

## Prerequisites

- Basic programming knowledge (variables, loops, conditionals, functions)
- Windows 10 or Windows 11
- No prior Rust experience required

## Table of Contents

### Part I: Foundations

| Chapter | Title | Description |
|---------|-------|-------------|
| [0](00-introduction.md) | **Introduction** | What we're building and why |
| [1](01-setup.md) | **Setting Up the Development Environment** | Installing Rust, VS Code, and tools |
| [2](02-project-structure.md) | **Project Structure and Cargo** | Understanding Rust projects and dependencies |

### Part II: Core Implementation

| Chapter | Title | Description |
|---------|-------|-------------|
| [3](03-data-types.md) | **Core Data Types** | Structs, enums, and the type system |
| [4](04-windows-api.md) | **Windows API Integration** | FFI, calling C functions, unsafe code |
| [5](05-application-logic.md) | **Application Logic** | State management, filtering, sorting |

### Part III: User Interface

| Chapter | Title | Description |
|---------|-------|-------------|
| [6](06-user-interface.md) | **Building the User Interface** | egui basics, layouts, widgets |
| [7](07-polish.md) | **Polish and Finishing Touches** | Icons, persistence, themes |

### Part IV: Quality Assurance

| Chapter | Title | Description |
|---------|-------|-------------|
| [8](08-testing.md) | **Testing** | Unit tests, integration tests, test organization |
| [9](09-benchmarking.md) | **Benchmarking** | Measuring performance with Criterion |

### Part V: Production

| Chapter | Title | Description |
|---------|-------|-------------|
| [10](10-build-configuration.md) | **Build Configuration** | Release optimization, Windows manifests |
| [11](11-continuous-integration.md) | **Continuous Integration** | GitHub Actions, automated testing |
| [12](12-final-assembly.md) | **Final Assembly** | Putting it all together |

## Key Topics Covered

### Rust Language
- Ownership and borrowing
- Structs, enums, and pattern matching
- Traits and implementations
- Error handling with Result
- Iterators and closures
- Modules and visibility

### Windows Programming
- Foreign Function Interface (FFI)
- Calling Windows APIs from Rust
- Raw pointers and unsafe code
- Application manifests
- Administrator privileges

### GUI Development
- Immediate mode GUI with egui
- Layouts and responsive design
- Keyboard shortcuts
- Dark/light themes
- Data grids and selection

### Professional Practices
- Project organization
- Comprehensive testing
- Performance benchmarking
- Build optimization
- Continuous integration

## The Final Result

By completing this book, you'll have built an application that:

- 🔍 Views all file locks on a Windows system
- 🔓 Releases locks to resolve "file in use" errors
- 🔎 Searches and filters by filename, path, or username
- 🔀 Sorts by any column
- 🎯 Supports multi-select (Ctrl+Click, Shift+Click)
- 🎨 Offers light and dark themes
- ⌨️ Provides keyboard shortcuts
- 💨 Runs fast with a ~4 MB binary

## How to Use This Book

### Reading Order

The chapters are designed to be read in order. Each builds on concepts from previous chapters.

### Hands-On Learning

For best results:
1. Read each chapter completely
2. Type out the code yourself (don't just copy-paste)
3. Run the code and experiment with changes
4. Complete the exercises if any are provided

### Reference Use

Once you've read the book, use it as a reference:
- Jump to specific chapters for particular topics
- Search within chapters for specific concepts
- Use the code examples as templates

## Source Code

The complete LockSmith source code is available in this repository alongside this book. Compare your implementation with the reference code as you progress.

## Getting Help

If you get stuck:
1. Re-read the relevant section carefully
2. Check Rust's excellent error messages
3. Compare with the source code
4. Consult the [Rust Book](https://doc.rust-lang.org/book/) for language details
5. Check the [egui documentation](https://docs.rs/egui)

## License

This book and the LockSmith application are released under the MIT License.

---

Ready to start? Begin with [Chapter 0: Introduction →](00-introduction.md)