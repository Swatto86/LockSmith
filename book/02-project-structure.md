# Chapter 2: Project Structure and Cargo

Now that our development environment is ready, let's create the LockSmith project and understand how Rust projects are organized. This chapter covers project structure, dependencies, and the build system.

## Creating the LockSmith Project

Let's create our project from scratch. Open a terminal and navigate to where you want the project:

```
$ cd C:\Users\YourName\Projects
$ cargo new locksmith
     Created binary (application) `locksmith` package
$ cd locksmith
```

Cargo creates this basic structure:

```
locksmith/
├── Cargo.toml
└── src/
    └── main.rs
```

For LockSmith, we'll expand this to a more complete structure. Let's look at what the final project looks like:

```
LockSmith/
├── src/                          # Rust project root
│   ├── src/
│   │   └── main.rs              # Main application code
│   ├── tests/                   # Integration tests
│   │   ├── integration_tests.rs
│   │   └── test_utils.rs
│   ├── benches/                 # Performance benchmarks
│   │   └── benchmarks.rs
│   ├── icons/                   # Application icons
│   │   ├── icon.ico
│   │   └── *.png
│   ├── Cargo.toml               # Project manifest
│   ├── Cargo.lock               # Locked dependency versions
│   └── build.rs                 # Build script
├── .github/
│   └── workflows/
│       └── ci.yml               # Continuous integration
├── build.bat                    # Windows build script
├── README.md
├── LICENSE
└── .gitignore
```

### Why the Nested `src/` Directories?

You might notice there are two `src` folders—one containing the Cargo project, and another inside for the actual source code. This is a organizational choice that separates:

- **Top-level `LockSmith/`** — Repository root with documentation, CI config, etc.
- **`LockSmith/src/`** — The actual Rust project (where `Cargo.toml` lives)
- **`LockSmith/src/src/`** — The Rust source code

This keeps the repository organized when you have documentation, scripts, and other non-Rust files at the top level.

## Understanding Cargo.toml

The `Cargo.toml` file is the heart of your Rust project. Let's build it piece by piece.

### Package Metadata

```toml
[package]
name = "locksmith"
version = "1.0.0"
description = "A Windows utility for managing file locks"
authors = ["LockSmith Contributors"]
license = "MIT"
repository = ""
edition = "2021"
```

Let's understand each field:

| Field | Purpose |
|-------|---------|
| `name` | Package and executable name. Must be lowercase, hyphens or underscores |
| `version` | Version number following semantic versioning |
| `description` | Short description, shown on crates.io if published |
| `authors` | List of author names/emails |
| `license` | License identifier (MIT, Apache-2.0, GPL-3.0, etc.) |
| `repository` | URL to source code repository |
| `edition` | Rust edition (2015, 2018, 2021). Affects language features available |

#### What's a Rust Edition?

Rust editions (2015, 2018, 2021) are "snapshots" of the language that introduce new features or syntax changes. Edition 2021 is current and recommended. Your code can use libraries from different editions—they interoperate seamlessly.

### Dependencies

Dependencies are external libraries (called **crates** in Rust) that your project uses. Here's what LockSmith needs:

```toml
[dependencies]
# GUI framework
eframe = { version = "0.29", default-features = false, features = [
    "default_fonts",
    "glow",
    "persistence",
] }
egui_extras = { version = "0.29", features = ["all_loaders"] }

# Serialization for persistence
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Image loading for icons
image = { version = "0.25", default-features = false, features = ["png"] }
```

Let's break down this syntax:

#### Simple Dependencies

```toml
serde_json = "1"
```

This means "use serde_json version 1.x.x, where x.x is the latest compatible version." This is called a **caret requirement**—Cargo will use `1.0.0` through `1.999.999` but not `2.0.0`.

#### Dependencies with Features

```toml
serde = { version = "1", features = ["derive"] }
```

Many crates have optional **features** that enable extra functionality. The `derive` feature for serde lets us use `#[derive(Serialize, Deserialize)]` on our types.

#### Dependencies with Specific Feature Control

```toml
eframe = { version = "0.29", default-features = false, features = [
    "default_fonts",
    "glow",
    "persistence",
] }
```

- `default-features = false` — Don't include features that are on by default
- `features = [...]` — Only include these specific features

This is useful for:
1. **Reducing compile time** — Fewer features = less code to compile
2. **Reducing binary size** — Only include what you need
3. **Avoiding unwanted dependencies** — Some features pull in extra crates

### Platform-Specific Dependencies

Some dependencies are only needed on certain platforms:

```toml
[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = ["shellapi", "winuser"] }
```

The `[target.'cfg(windows)'.dependencies]` section means "only include these when compiling for Windows."

The `cfg(windows)` syntax is called a **configuration predicate**. Common predicates include:
- `cfg(windows)` — Windows
- `cfg(unix)` — Linux, macOS, etc.
- `cfg(target_os = "macos")` — macOS specifically
- `cfg(target_arch = "x86_64")` — 64-bit x86 architecture

### Build Dependencies

```toml
[target.'cfg(windows)'.build-dependencies]
winres = "0.1"
```

**Build dependencies** are crates used during compilation but not included in the final binary. Here, `winres` is used by our build script to embed Windows resources (like the application icon and manifest).

### Features Section

```toml
[features]
default = []
```

This defines your crate's own features. We have none for LockSmith, so it's empty.

### Release Profile

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
panic = "abort"
```

This configures how release builds are compiled. Let's understand each option:

| Option | Value | Effect |
|--------|-------|--------|
| `opt-level` | `3` | Maximum optimization (slower compile, faster runtime) |
| `lto` | `true` | Link-Time Optimization — optimizes across crate boundaries |
| `codegen-units` | `1` | Uses one codegen unit (slower compile, better optimization) |
| `strip` | `true` | Remove debug symbols from binary (smaller file size) |
| `panic` | `"abort"` | On panic, abort immediately instead of unwinding |

These settings prioritize small binary size and runtime performance over compile speed.

### Benchmark Configuration

```toml
[[bench]]
name = "benchmarks"
harness = false

[dev-dependencies]
criterion = "0.5"
```

- `[[bench]]` — Defines a benchmark target (note the double brackets for arrays)
- `harness = false` — Use our own test harness (criterion) instead of Rust's built-in
- `[dev-dependencies]` — Dependencies only needed for testing/benchmarking

## Understanding Cargo.lock

When you run `cargo build`, Cargo creates `Cargo.lock`. This file records the exact versions of every dependency (and their dependencies). 

```toml
[[package]]
name = "serde"
version = "1.0.193"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "25dd9975e68d0cb5aa1120c288333fc98731bd1dd12f561e468ea4728c042b89"
dependencies = [
 "serde_derive",
]
```

### Should You Commit Cargo.lock?

**For applications (like LockSmith): Yes!**

Committing `Cargo.lock` ensures everyone building the project uses the exact same dependency versions, making builds reproducible.

**For libraries: Usually no.**

Libraries should let the final application decide dependency versions.

## The Build Script (build.rs)

Rust supports **build scripts**—Rust code that runs before your main code compiles. Create `build.rs` in the project root (next to `Cargo.toml`):

```rust
fn main() {
    // Windows-specific build configuration
    #[cfg(windows)]
    {
        // Use winres to embed Windows resources
        let mut res = winres::WindowsResource::new();

        // Set the application icon
        res.set_icon("icons/icon.ico");

        // Set the application manifest
        #[cfg(not(debug_assertions))]
        res.set_manifest(r#"
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
    <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
        <security>
            <requestedPrivileges>
                <requestedExecutionLevel level="requireAdministrator" uiAccess="false"/>
            </requestedPrivileges>
        </security>
    </trustInfo>
</assembly>
"#);

        // Compile the resources
        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile Windows resources: {}", e);
        }
    }
}
```

### What Does This Build Script Do?

1. **Embeds an application icon** — The icon shown in Windows Explorer and the taskbar
2. **Sets a Windows manifest** — Tells Windows this app requires administrator privileges

### Understanding the Attributes

```rust
#[cfg(windows)]
```

This is a **conditional compilation attribute**. The code inside only compiles on Windows. On other platforms, it's as if the code doesn't exist.

```rust
#[cfg(not(debug_assertions))]
```

This compiles only in release mode. `debug_assertions` is true in debug builds. So `not(debug_assertions)` means "release mode only."

We use this because:
- **Release builds** require admin privileges (for real file lock operations)
- **Debug builds** run without elevation (easier for development and testing)

### Understanding Raw Strings

```rust
r#"
<?xml version="1.0"?>
...
"#
```

The `r#"..."#` syntax creates a **raw string literal**. Inside raw strings:
- Backslashes are literal (no escaping needed)
- Quotes can be included
- Newlines are preserved

This is perfect for embedding XML, JSON, or regex patterns.

## The .gitignore File

The `.gitignore` file tells Git which files to exclude from version control:

```gitignore
# Rust
/target
**/*.rs.bk
*.pdb

# Build artifacts
/src/target/

# Logs
logs
*.log

# Editor directories and files
.vscode/*
!.vscode/extensions.json
.idea
.DS_Store
*.suo
*.ntvs*
*.njsproj
*.sln
*.sw?

# OS
Thumbs.db
.DS_Store
```

Key exclusions:
- `target/` — Compiled output (can be huge, easily regenerated)
- `*.pdb` — Windows debug database files
- Editor settings — Personal preferences shouldn't be shared

## Creating the Minimal Application

Let's create a minimal working application to verify our setup. In `src/main.rs`:

```rust
fn main() {
    println!("LockSmith starting...");
}
```

Now build and run:

```
$ cargo run
   Compiling locksmith v1.0.0 (C:\...\locksmith)
    Finished dev [unoptimized + debuginfo] target(s) in 0.50s
     Running `target\debug\locksmith.exe`
LockSmith starting...
```

It works! But it's not very exciting yet. Let's add the GUI framework.

## Adding egui: Our First Real Dependency

Update `Cargo.toml` to add egui:

```toml
[package]
name = "locksmith"
version = "1.0.0"
edition = "2021"

[dependencies]
eframe = { version = "0.29", default-features = false, features = [
    "default_fonts",
    "glow",
] }
```

Now update `src/main.rs` to show a window:

```rust
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("LockSmith")
            .with_inner_size(egui::Vec2::new(800.0, 600.0)),
        ..Default::default()
    };

    eframe::run_native(
        "LockSmith",
        options,
        Box::new(|_cc| Ok(Box::new(LockSmithApp::default()))),
    )
}

#[derive(Default)]
struct LockSmithApp;

impl eframe::App for LockSmithApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🔓 LockSmith");
            ui.label("Coming soon...");
        });
    }
}
```

Run it:

```
$ cargo run
```

A window should appear! Let's understand what we just wrote.

### Breaking Down the Code

#### Use Statement

```rust
use eframe::egui;
```

The `use` statement brings items from other modules into scope. Now we can write `egui::Something` instead of `eframe::egui::Something`.

#### The main Function

```rust
fn main() -> eframe::Result<()> {
```

This `main` function returns a `Result`. The `()` inside means "on success, return nothing." If something goes wrong, it returns an error.

#### NativeOptions

```rust
let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
        .with_title("LockSmith")
        .with_inner_size(egui::Vec2::new(800.0, 600.0)),
    ..Default::default()
};
```

We're creating a struct with specific fields. The `..Default::default()` syntax means "fill in all other fields with their default values."

`with_title` and `with_inner_size` use the **builder pattern**—methods that return the modified object, allowing chaining.

#### run_native

```rust
eframe::run_native(
    "LockSmith",
    options,
    Box::new(|_cc| Ok(Box::new(LockSmithApp::default()))),
)
```

This starts the application. The third argument is a **closure** (anonymous function) that creates our app. We'll cover closures in detail later.

#### The App Struct

```rust
#[derive(Default)]
struct LockSmithApp;
```

`#[derive(Default)]` automatically implements the `Default` trait, which provides a `default()` function that creates a default instance.

#### The App Trait

```rust
impl eframe::App for LockSmithApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ...
    }
}
```

`impl TraitName for TypeName` implements a **trait** (like an interface) for our type. The `App` trait requires an `update` method that's called every frame.

The underscore prefix (`_frame`) tells Rust we intentionally aren't using that parameter.

## The Complete Cargo.toml

Here's the complete `Cargo.toml` we'll use for LockSmith:

```toml
[package]
name = "locksmith"
version = "1.0.0"
description = "A Windows utility for managing file locks"
authors = ["LockSmith Contributors"]
license = "MIT"
repository = ""
edition = "2021"

[dependencies]
# GUI framework
eframe = { version = "0.29", default-features = false, features = [
    "default_fonts",
    "glow",
    "persistence",
] }
egui_extras = { version = "0.29", features = ["all_loaders"] }

# Serialization for persistence
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Image loading for icons
image = { version = "0.25", default-features = false, features = ["png"] }

# Windows-specific for admin check and manifest
[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = ["shellapi", "winuser"] }

[target.'cfg(windows)'.build-dependencies]
winres = "0.1"

[features]
default = []

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
panic = "abort"

[[bench]]
name = "benchmarks"
harness = false

[dev-dependencies]
criterion = "0.5"
```

## What We've Accomplished

You now have:

✅ Created the LockSmith project structure  
✅ Understood `Cargo.toml` and dependency management  
✅ Learned about build scripts and conditional compilation  
✅ Created a minimal working window  

## Key Takeaways

1. **Cargo.toml is the project manifest** — It defines metadata, dependencies, and build configuration
2. **Features let you customize dependencies** — Enable only what you need
3. **Build scripts run before compilation** — Useful for embedding resources
4. **Conditional compilation with `#[cfg(...)]`** — Include/exclude code based on platform or build type
5. **egui uses an immediate-mode paradigm** — The UI is rebuilt every frame in the `update` method

## Looking Ahead

In the next chapter, we'll define LockSmith's core data types—the structures that represent file locks and application state. We'll learn about Rust's struct, enum, and derive system.

---

**Previous Chapter:** [← Setting Up the Development Environment](01-setup.md)  
**Next Chapter:** [Core Data Types →](03-data-types.md)