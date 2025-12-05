# LockSmith Icon

## Overview

LockSmith uses an open padlock icon to represent its file lock management functionality. The icon features a blue color scheme that works well on both light and dark backgrounds.

## Design

- **Symbol**: Open padlock (representing unlocking/releasing file locks)
- **Colors**: Blue gradient (#60a5fa to #3b82f6) on dark navy background
- **Shape**: Circular with the padlock centered

## Source Files

- `icon.svg` — Master vector file (512x512)
- `src-tauri/icons/icon.ico` — Windows icon (multi-resolution)
- `src-tauri/icons/icon.png` — 256x256 PNG
- `src-tauri/icons/128x128.png` — 128x128 PNG
- `src-tauri/icons/64x64.png` — 64x64 PNG
- `src-tauri/icons/32x32.png` — 32x32 PNG

## Regenerating Icons

If you modify `icon.svg`, regenerate the PNG files:

### Using ImageMagick

```bash
# Install ImageMagick first, then:
magick icon.svg -resize 256x256 src-tauri/icons/icon.png
magick icon.svg -resize 128x128 src-tauri/icons/128x128.png
magick icon.svg -resize 64x64 src-tauri/icons/64x64.png
magick icon.svg -resize 32x32 src-tauri/icons/32x32.png

# Create ICO file with multiple resolutions
magick icon.svg -define icon:auto-resize=256,128,64,48,32,16 src-tauri/icons/icon.ico
```

### Using Inkscape

```bash
inkscape icon.svg -w 256 -h 256 -o src-tauri/icons/icon.png
inkscape icon.svg -w 128 -h 128 -o src-tauri/icons/128x128.png
inkscape icon.svg -w 64 -h 64 -o src-tauri/icons/64x64.png
inkscape icon.svg -w 32 -h 32 -o src-tauri/icons/32x32.png
```

## Fallback Icon

The application includes a programmatically-generated fallback icon in case the PNG files cannot be loaded. This is defined in `main.rs` in the `create_padlock_icon()` function.

## Usage

The icon is used in two places:

1. **Window icon** — Loaded at runtime via `load_icon()` in `main.rs`
2. **Executable icon** — Embedded via `build.rs` using the `.ico` file