# LockSmith

A lightweight Windows utility for viewing and managing file locks. Built with pure Rust and egui — no WebView2 or runtime dependencies required.

![Version](https://img.shields.io/badge/version-1.0.0-blue)
![Platform](https://img.shields.io/badge/platform-Windows-blue)
![License](https://img.shields.io/badge/license-MIT-green)

## Overview

LockSmith helps you identify and release file locks on Windows, solving those frustrating "file is in use" errors. It provides a clean, modern interface to view all open files on your system and forcibly close file handles when needed.

## Features

- 🔍 **View open files** — See all files currently locked on your system
- 🔓 **Release locks** — Force-close file handles to resolve "file in use" errors
- 🔎 **Search & filter** — Quickly find files by name, path, or username
- 🔀 **Sortable columns** — Click headers to sort ascending/descending
- 🎯 **Multi-select** — Ctrl+Click and Shift+Click support
- 🎨 **Theme support** — Light and dark modes with persistent preference
- ⌨️ **Keyboard shortcuts** — F5 to refresh, Delete to release, Ctrl+A to select all
- 💨 **Lightweight** — ~4 MB binary, ~20 MB memory usage
- 🔒 **Secure** — No network access, no telemetry, fully offline

## Requirements

- Windows 10 or Windows 11
- Administrator privileges (required for file lock operations)

## Installation

### Download

Download the latest `locksmith.exe` from the [Releases](https://github.com/yourusername/LockSmith/releases) page.

### Build from Source

```bash
git clone https://github.com/yourusername/LockSmith.git
cd LockSmith/src-tauri
cargo build --release
```

The executable will be at `target/release/locksmith.exe`.

## Usage

1. **Run as Administrator** — Right-click `locksmith.exe` and select "Run as administrator"
2. **View locks** — The application displays all currently locked files
3. **Search** — Type in the search box to filter by filename, path, or username
4. **Sort** — Click column headers to sort
5. **Select** — Click rows to select, use Ctrl+Click for multiple or Shift+Click for range
6. **Release** — Click "Release Selected Lock(s)" or press Delete

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| F5 | Refresh file list |
| Delete | Release selected locks |
| Ctrl+A | Select all visible files |
| Ctrl+Click | Toggle individual selection |
| Shift+Click | Select range |
| Double-Click | Release single lock (with confirmation) |

## ⚠️ Safety Warning

**Releasing file locks forcibly closes file handles. This can cause:**

- Data loss if the application hasn't saved recent changes
- Application crashes as handles become invalid
- File corruption if writes are interrupted

**Only use when:**
- The owning application has crashed or is unresponsive
- You're certain no unsaved work will be lost
- Normal methods (closing the application) have failed

## Project Structure

```
LockSmith/
├── src-tauri/
│   ├── src/
│   │   └── main.rs         # Application source code
│   ├── icons/              # Application icons
│   ├── Cargo.toml          # Rust dependencies
│   └── build.rs            # Build script (Windows manifest)
├── .github/
│   └── workflows/
│       └── ci.yml          # GitHub Actions CI
├── build.bat               # Windows build script
├── icon.svg                # Source icon (vector)
├── README.md
└── LICENSE
```

## Development

### Prerequisites

- [Rust](https://rustup.rs/) 1.70 or later
- Windows 10/11 with Visual Studio Build Tools

### Commands

```bash
cd src-tauri

# Run in debug mode
cargo run

# Build release binary
cargo build --release

# Run tests (requires admin)
cargo test

# Check formatting
cargo fmt --check

# Run linter
cargo clippy
```

## Technical Details

### Windows APIs

LockSmith uses the Windows Network Management API:

- `NetFileEnum` — Enumerate open files on the local machine
- `NetFileClose` — Forcibly close a file handle
- `NetApiBufferFree` — Free API-allocated memory

These APIs require administrator privileges and are accessed via direct FFI bindings to `netapi32.dll`.

### GUI Framework

Built with [egui](https://github.com/emilk/egui) and [eframe](https://github.com/emilk/egui/tree/master/crates/eframe):

- Pure Rust immediate-mode GUI
- Native OpenGL rendering via glow
- No web technologies or external runtimes
- Cross-platform compatible (Windows-only due to API usage)

### Why Not Tauri?

The original version used Tauri with WebView2. This pure Rust rewrite offers:

| Aspect | egui | Tauri/WebView2 |
|--------|------|----------------|
| Binary size | ~4 MB | ~11 MB |
| Memory usage | ~20 MB | ~40 MB |
| Dependencies | None | WebView2 Runtime |
| Startup time | ~0.5s | ~1.5s |

## Known Limitations

- **Windows only** — Uses Windows-specific APIs
- **Administrator required** — Cannot function without elevation
- **Network shares only** — Shows files opened via SMB/network shares hosted by this machine; local file handles may not appear

## Troubleshooting

### "Access Denied" error

Run the application as Administrator (right-click → "Run as administrator").

### No files shown

This is normal if:
- No files are currently open via network shares
- The Windows Server service is not running
- No network shares are active on this machine

### Build errors

Ensure you have:
- Rust toolchain installed (`rustup`)
- Visual Studio Build Tools with C++ workload
- Windows SDK

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Run `cargo fmt` and `cargo clippy`
4. Commit your changes
5. Push and open a Pull Request

## License

MIT License — see [LICENSE](LICENSE) for details.

## Acknowledgments

- [egui](https://github.com/emilk/egui) — Pure Rust GUI framework
- [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) — egui integration framework
- Inspired by [SysInternals Handle](https://docs.microsoft.com/en-us/sysinternals/downloads/handle)