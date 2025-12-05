# Chapter 10: Build Configuration

This chapter covers everything about building LockSmith for distribution: release optimizations, the Windows manifest, application icons, and build scripts.

## Debug vs Release Builds

Rust has two main build profiles:

| Aspect | Debug | Release |
|--------|-------|---------|
| Compile time | Fast | Slow |
| Runtime speed | Slow | Fast |
| Binary size | Large | Small |
| Debug info | Full | Stripped |
| Optimizations | None | Maximum |

### Building in Debug Mode

```
$ cargo build
```

Output: `target/debug/locksmith.exe`

Debug builds:
- Compile quickly (faster iteration)
- Include debug symbols (for debugging)
- Have bounds checking and overflow detection
- Run slower (no optimizations)

### Building in Release Mode

```
$ cargo build --release
```

Output: `target/release/locksmith.exe`

Release builds:
- Take longer to compile
- Run much faster
- Produce smaller binaries
- Strip debug information

## Release Profile Configuration

In `Cargo.toml`, we configure the release profile:

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
panic = "abort"
```

### Understanding Each Option

#### opt-level

```toml
opt-level = 3
```

Controls optimization intensity:

| Value | Meaning |
|-------|---------|
| 0 | No optimization (debug default) |
| 1 | Basic optimizations |
| 2 | Most optimizations |
| 3 | All optimizations (release default) |
| "s" | Optimize for size |
| "z" | Optimize for size, more aggressively |

Level 3 produces the fastest code at the cost of compile time.

#### lto (Link-Time Optimization)

```toml
lto = true
```

LTO performs optimizations across crate boundaries during linking:

| Value | Meaning |
|-------|---------|
| `false` | No LTO |
| `true` or `"fat"` | Full LTO (slowest compile, best optimization) |
| `"thin"` | Thin LTO (faster compile, good optimization) |

LTO can:
- Inline functions across crates
- Remove dead code more effectively
- Improve overall performance by 10-20%

Trade-off: Significantly longer compile times.

#### codegen-units

```toml
codegen-units = 1
```

Rust can compile crates in parallel using multiple "codegen units":

| Value | Effect |
|-------|--------|
| 1 | Single unit (slowest compile, best optimization) |
| 16 | Default for release (balanced) |
| 256 | Default for debug (fastest compile) |

With one unit, the compiler has full visibility for optimization but can't parallelize.

#### strip

```toml
strip = true
```

Removes debug symbols and other metadata from the binary:

| Value | Effect |
|-------|--------|
| `"none"` | Keep everything |
| `"debuginfo"` | Strip debug info only |
| `"symbols"` or `true` | Strip all symbols |

This can reduce binary size by 50% or more.

#### panic

```toml
panic = "abort"
```

Controls what happens when the program panics:

| Value | Effect |
|-------|--------|
| `"unwind"` | Unwind the stack, run destructors (default) |
| `"abort"` | Immediately abort the process |

`"abort"` produces smaller binaries because:
- No unwinding code is generated
- No panic handler runtime is needed

Trade-off: No graceful cleanup on panic. For a GUI app like LockSmith, this is acceptable—a panic indicates a bug, and immediate termination is fine.

## The Build Script (build.rs)

Rust supports **build scripts**—code that runs before your crate compiles. Create `build.rs` in your project root:

```rust
fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        
        // Set application icon
        res.set_icon("icons/icon.ico");
        
        // Set manifest based on build type
        #[cfg(not(debug_assertions))]
        res.set_manifest(r#"
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
    <assemblyIdentity
        version="1.0.0.0"
        processorArchitecture="*"
        name="LockSmith"
        type="win32"
    />
    <description>LockSmith - Windows File Lock Management Utility</description>
    <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
        <security>
            <requestedPrivileges>
                <requestedExecutionLevel level="requireAdministrator" uiAccess="false"/>
            </requestedPrivileges>
        </security>
    </trustInfo>
    <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
        <application>
            <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
            <supportedOS Id="{1f676c76-80e1-4239-95bb-83d0f6d0da78}"/>
            <supportedOS Id="{4a2f28e3-53b9-4441-ba9c-d69d4a4a6e38}"/>
            <supportedOS Id="{35138b9a-5d96-4fbd-8e2d-a2440225f93a}"/>
        </application>
    </compatibility>
    <application xmlns="urn:schemas-microsoft-com:asm.v3">
        <windowsSettings>
            <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
            <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness>
        </windowsSettings>
    </application>
</assembly>
"#);

        #[cfg(debug_assertions)]
        res.set_manifest(r#"
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
    <assemblyIdentity
        version="1.0.0.0"
        processorArchitecture="*"
        name="LockSmith"
        type="win32"
    />
    <description>LockSmith - Windows File Lock Management Utility (Debug)</description>
    <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
        <security>
            <requestedPrivileges>
                <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
            </requestedPrivileges>
        </security>
    </trustInfo>
</assembly>
"#);

        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile Windows resources: {}", e);
        }
    }
}
```

### How Build Scripts Work

1. Cargo compiles `build.rs` first
2. Cargo runs the compiled build script
3. Build script output goes to a temporary directory
4. Cargo compiles your main code with build script outputs

### The winres Crate

```toml
[target.'cfg(windows)'.build-dependencies]
winres = "0.1"
```

`winres` embeds Windows resources into your executable:
- Application icon
- Version information
- Application manifest

### Setting the Icon

```rust
res.set_icon("icons/icon.ico");
```

The `.ico` file contains the application icon shown in:
- Windows Explorer
- Taskbar
- Alt+Tab switcher
- Title bar

ICO files can contain multiple sizes (16x16, 32x32, 48x48, 256x256). Windows picks the appropriate size.

## The Windows Manifest

The manifest is an XML file embedded in the executable that tells Windows about the application.

### Administrator Privileges

```xml
<requestedExecutionLevel level="requireAdministrator" uiAccess="false"/>
```

Execution levels:

| Level | Meaning |
|-------|---------|
| `asInvoker` | Run with current user privileges |
| `highestAvailable` | Request highest available (admin if user is admin) |
| `requireAdministrator` | Always require admin, show UAC prompt |

For LockSmith:
- Release builds use `requireAdministrator` (needed for API access)
- Debug builds use `asInvoker` (easier development)

### Windows Version Compatibility

```xml
<compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
        <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
    </application>
</compatibility>
```

The GUIDs identify Windows versions:

| GUID | Windows Version |
|------|----------------|
| `8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a` | Windows 10/11 |
| `1f676c76-80e1-4239-95bb-83d0f6d0da78` | Windows 8.1 |
| `4a2f28e3-53b9-4441-ba9c-d69d4a4a6e38` | Windows 8 |
| `35138b9a-5d96-4fbd-8e2d-a2440225f93a` | Windows 7 |

Declaring compatibility:
- Enables new Windows features
- Prevents compatibility shims
- Shows Windows we've tested on these versions

### DPI Awareness

```xml
<windowsSettings>
    <dpiAware>true/pm</dpiAware>
    <dpiAwareness>PerMonitorV2</dpiAwareness>
</windowsSettings>
```

DPI (Dots Per Inch) awareness levels:

| Level | Behavior |
|-------|----------|
| Unaware | Windows scales the app (blurry) |
| System aware | App handles system DPI |
| Per-monitor | App handles each monitor's DPI |
| Per-monitor V2 | Best: handles DPI changes dynamically |

`PerMonitorV2` ensures:
- Sharp rendering on high-DPI displays
- Correct scaling when moving between monitors
- Dynamic response to DPI setting changes

## Application Icons

### Icon File Structure

A proper Windows `.ico` file contains multiple sizes:

| Size | Used For |
|------|----------|
| 16x16 | Title bar, small list view |
| 32x32 | Standard icons, taskbar |
| 48x48 | Large icons |
| 256x256 | Extra large icons, high-DPI |

### Creating Icons

You can create `.ico` files with:
- Image editors (GIMP, Photoshop)
- Online converters
- Command-line tools (ImageMagick)

From an SVG or PNG:
```
$ magick convert icon.png -define icon:auto-resize=256,48,32,16 icon.ico
```

### Icon Files in LockSmith

```
icons/
├── icon.ico          # Multi-resolution Windows icon
├── icon.png          # Source PNG
├── 32x32.png         # Individual sizes
├── 64x64.png
├── 128x128.png
└── 128x128@2x.png    # Retina/high-DPI variant
```

## The Build Script (build.bat)

For convenience, we provide a Windows batch script:

```batch
@echo off
echo ========================================
echo   LockSmith Build Script
echo ========================================
echo.

cd /d "%~dp0src"

echo Building release binary...
cargo build --release
if errorlevel 1 (
    echo.
    echo ERROR: Build failed!
    pause
    exit /b 1
)

echo.
echo ========================================
echo   Build successful!
echo ========================================
echo.
echo Output: src\target\release\locksmith.exe
echo.
echo NOTE: Right-click and "Run as administrator" to use.
echo.
pause
```

### Understanding the Script

```batch
@echo off
```
Disable command echoing (cleaner output).

```batch
cd /d "%~dp0src"
```
Change to the script's directory + `src` subfolder:
- `%~dp0` — Script's directory with trailing backslash
- `/d` — Change drive if necessary

```batch
cargo build --release
if errorlevel 1 (
```
Build and check for errors. `errorlevel 1` means "exit code >= 1" (failure).

```batch
pause
```
Wait for keypress so users can read the output.

## Binary Size Analysis

Let's understand what contributes to binary size:

### Checking Size

```
$ dir target\release\locksmith.exe
    4,231,168 bytes
```

About 4 MB for LockSmith.

### What's in the Binary?

| Component | Approximate Size |
|-----------|-----------------|
| egui/eframe | ~2.5 MB |
| OpenGL renderer | ~0.5 MB |
| Windows API bindings | ~0.3 MB |
| Our code | ~0.1 MB |
| Standard library | ~0.5 MB |
| Miscellaneous | ~0.3 MB |

### Reducing Size Further

If size is critical, additional options:

```toml
[profile.release]
opt-level = "z"        # Optimize for size instead of speed
```

Or use `cargo-bloat` to analyze:

```
$ cargo install cargo-bloat
$ cargo bloat --release -n 20
```

## Distribution Checklist

Before distributing LockSmith:

### 1. Build in Release Mode

```
$ cargo build --release
```

### 2. Test the Release Build

Release builds can behave differently:
- Different optimization levels might expose bugs
- No debug assertions
- Different timing (race conditions)

### 3. Verify Administrator Requirements

Right-click the exe and check:
- UAC prompt appears
- Shield icon on executable
- App functions correctly

### 4. Test on Clean System

Ideally, test on a VM or different machine:
- No Rust toolchain installed
- No Visual C++ redistributables (egui includes what it needs)
- Different Windows version

### 5. Consider Code Signing

For professional distribution:
- Obtain a code signing certificate
- Sign the executable with `signtool`
- Prevents "Unknown Publisher" warnings

## Continuous Integration

Automate builds with GitHub Actions (covered in Chapter 11).

The CI workflow:
1. Checks code formatting
2. Runs clippy (linter)
3. Builds debug binary
4. Runs tests
5. Builds release binary
6. Archives artifacts

## Troubleshooting Build Issues

### "linker not found"

Install Visual Studio Build Tools with C++ workload.

### "icon.ico not found"

Ensure the path in `build.rs` is correct relative to `Cargo.toml`.

### "manifest parse error"

Check XML syntax in the manifest string. Common issues:
- Unclosed tags
- Invalid characters
- Encoding issues

### Build is slow

LTO and `codegen-units = 1` are slow. For development:
```
$ cargo build  # Debug build (faster)
```

Only use `--release` for final builds.

## What We've Accomplished

You now understand:

✅ Debug vs Release builds — When to use each  
✅ Release profile options — LTO, codegen-units, strip, panic  
✅ Build scripts — Custom compilation steps with build.rs  
✅ Windows manifests — Admin privileges, DPI awareness, compatibility  
✅ Application icons — Multi-resolution ICO files  
✅ Distribution preparation — Testing and verification  

## Key Takeaways

1. **Release builds are essential for distribution** — Debug builds are for development only
2. **LTO improves performance** — At the cost of compile time
3. **Manifests configure Windows behavior** — Admin rights, DPI, compatibility
4. **Icons matter for polish** — Include multiple sizes
5. **Test release builds** — They can behave differently

## Looking Ahead

In the next chapter, we'll set up continuous integration with GitHub Actions to automate testing and building.

---

**Previous Chapter:** [← Benchmarking](09-benchmarking.md)  
**Next Chapter:** [Continuous Integration →](11-continuous-integration.md)