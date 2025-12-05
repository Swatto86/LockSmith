# Chapter 1: Setting Up the Development Environment

Before we write any code, we need to set up our development environment. This chapter walks you through installing everything you need to build LockSmith.

## What You'll Need

1. **Windows 10 or Windows 11** — LockSmith uses Windows-specific APIs
2. **Rust toolchain** — The compiler and package manager
3. **Visual Studio Build Tools** — Required for compiling Rust on Windows
4. **A code editor** — We recommend VS Code with the rust-analyzer extension

## Step 1: Install Visual Studio Build Tools

Rust on Windows needs the Microsoft C++ build tools. Here's how to get them:

1. Go to [Visual Studio Downloads](https://visualstudio.microsoft.com/downloads/)
2. Scroll down to "Tools for Visual Studio"
3. Download "Build Tools for Visual Studio 2022" (or later)
4. Run the installer
5. Select **"Desktop development with C++"** workload
6. Click Install

This installs the MSVC compiler and Windows SDK, which Rust needs to link programs.

### Why Do We Need C++ Tools for Rust?

Rust compiles to native code, but it still needs a **linker** to combine all the pieces into an executable. On Windows, Rust uses Microsoft's linker. Additionally, when we call Windows APIs, we link against Windows system libraries—the build tools provide these.

## Step 2: Install Rust

The official way to install Rust is through **rustup**, a command-line tool that manages Rust versions.

1. Go to [rustup.rs](https://rustup.rs/)
2. Download and run `rustup-init.exe`
3. Follow the prompts (the defaults are fine)
4. When it finishes, **restart your terminal** (or open a new one)

To verify the installation, open a terminal (PowerShell or Command Prompt) and run:

```
$ rustc --version
rustc 1.75.0 (82e1608df 2023-12-21)
```

Your version number will likely be different—that's fine! LockSmith works with Rust 1.70 or later.

Also check that Cargo (Rust's package manager) is installed:

```
$ cargo --version
cargo 1.75.0 (1d8b05cdd 2023-11-20)
```

### What Did We Just Install?

The Rust toolchain includes several programs:

| Program | Purpose |
|---------|---------|
| `rustc` | The Rust compiler — turns `.rs` files into executables |
| `cargo` | Package manager and build system — handles dependencies, building, testing |
| `rustup` | Toolchain manager — updates Rust, switches between versions |
| `rustfmt` | Code formatter — keeps your code style consistent |
| `clippy` | Linter — finds common mistakes and suggests improvements |

We'll use all of these throughout this book.

## Step 3: Set Up Your Editor

You can use any text editor for Rust, but **Visual Studio Code** with the **rust-analyzer** extension provides an excellent experience.

### Installing VS Code

1. Download VS Code from [code.visualstudio.com](https://code.visualstudio.com/)
2. Run the installer

### Installing rust-analyzer

1. Open VS Code
2. Press `Ctrl+Shift+X` to open the Extensions panel
3. Search for "rust-analyzer"
4. Click Install on the extension by "rust-lang"

**rust-analyzer** provides:
- Syntax highlighting
- Code completion (IntelliSense)
- Inline error messages
- Go to definition
- Find references
- Automatic imports
- And much more!

### Recommended VS Code Settings for Rust

Open your settings (`Ctrl+,`) and consider enabling:

```json
{
    "editor.formatOnSave": true,
    "rust-analyzer.checkOnSave.command": "clippy"
}
```

- **formatOnSave** — Automatically formats your code when you save (uses rustfmt)
- **checkOnSave.command: clippy** — Runs the linter instead of just the compiler, catching more issues

## Step 4: Create a Test Project

Let's verify everything works by creating a simple "Hello, World!" project.

Open a terminal and navigate to where you want to create projects:

```
$ cd C:\Users\YourName\Projects
```

Create a new Rust project:

```
$ cargo new hello_test
     Created binary (application) `hello_test` package
```

Navigate into the project:

```
$ cd hello_test
```

Look at what Cargo created:

```
hello_test/
├── Cargo.toml    # Project configuration and dependencies
└── src/
    └── main.rs   # Your source code
```

Let's examine each file:

### Cargo.toml

```toml
[package]
name = "hello_test"
version = "0.1.0"
edition = "2021"

[dependencies]
```

This is the **manifest file**. It describes your project:

- `name` — The project name (also the executable name)
- `version` — Following semantic versioning (major.minor.patch)
- `edition` — Which Rust edition to use (2021 is current)
- `[dependencies]` — External libraries your project uses (empty for now)

### src/main.rs

```rust
fn main() {
    println!("Hello, world!");
}
```

This is the entry point of your program. Let's break it down:

- `fn main()` — Defines a function named `main`. Every Rust program starts executing here.
- `{ }` — Curly braces contain the function body
- `println!` — A **macro** (the `!` indicates it's a macro, not a regular function) that prints text
- `"Hello, world!"` — A string literal
- `;` — Statements end with semicolons

### Running the Project

Build and run with one command:

```
$ cargo run
   Compiling hello_test v0.1.0 (C:\Users\YourName\Projects\hello_test)
    Finished dev [unoptimized + debuginfo] target(s) in 0.47s
     Running `target\debug\hello_test.exe`
Hello, world!
```

Cargo:
1. Downloaded and compiled any dependencies (none in this case)
2. Compiled your code in **debug mode** (unoptimized, for faster compilation)
3. Ran the resulting executable

The compiled program is at `target/debug/hello_test.exe`.

### Building Without Running

To just compile without running:

```
$ cargo build
```

For a release build (optimized, slower to compile, faster to run):

```
$ cargo build --release
```

Release builds go to `target/release/hello_test.exe`.

## Understanding Cargo Commands

Here are the commands we'll use throughout this book:

| Command | Purpose |
|---------|---------|
| `cargo new <name>` | Create a new project |
| `cargo build` | Compile the project |
| `cargo build --release` | Compile with optimizations |
| `cargo run` | Build and run |
| `cargo test` | Run tests |
| `cargo bench` | Run benchmarks |
| `cargo fmt` | Format code |
| `cargo clippy` | Run the linter |
| `cargo check` | Fast syntax/type checking without full compilation |
| `cargo clean` | Delete compiled files |

### cargo check vs cargo build

`cargo check` is faster than `cargo build` because it doesn't generate an executable—it just checks that your code compiles. During development, you'll often run `cargo check` to catch errors quickly, then `cargo build` when you want to run the program.

## Cleaning Up

Once you've verified everything works, you can delete the test project:

```
$ cd ..
$ rmdir /s hello_test
```

Or keep it around for experimentation!

## Troubleshooting Common Issues

### "rustc is not recognized as an internal or external command"

Your PATH wasn't updated. Either:
- Restart your terminal/computer
- Or manually add `%USERPROFILE%\.cargo\bin` to your PATH

### "linker 'link.exe' not found"

Visual Studio Build Tools aren't installed correctly. Reinstall them, making sure to select the "Desktop development with C++" workload.

### "error: toolchain 'stable-x86_64-pc-windows-msvc' is not installed"

Run:
```
$ rustup toolchain install stable-x86_64-pc-windows-msvc
```

### VS Code rust-analyzer shows "rust-analyzer failed to start"

1. Make sure Rust is installed and in your PATH
2. Restart VS Code
3. Check the Output panel (Ctrl+Shift+U) and select "rust-analyzer" for error details

## What We've Accomplished

You now have:

✅ Visual Studio Build Tools installed  
✅ Rust toolchain installed and working  
✅ A code editor configured for Rust development  
✅ Verified everything works with a test project  

## Key Takeaways

1. **Rust uses MSVC on Windows** — We need Visual Studio Build Tools for the linker
2. **rustup manages Rust versions** — You can update with `rustup update`
3. **Cargo is central to Rust development** — It handles building, dependencies, testing, and more
4. **rust-analyzer makes coding easier** — Autocomplete, error highlighting, and refactoring support

## Looking Ahead

In the next chapter, we'll create the actual LockSmith project and understand how a real Rust project is structured. We'll set up our dependencies and create the basic project skeleton.

---

**Previous Chapter:** [← Introduction](00-introduction.md)  
**Next Chapter:** [Project Structure and Cargo →](02-project-structure.md)