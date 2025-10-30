# Icon Generation System

This directory contains the complete icon generation pipeline for the Speculative Execution Engine application.

## Overview

The system generates all platform-specific icons (Windows, macOS, Linux, web) from a single SVG source file, while also managing UI icons from Heroicons.

## Directory Structure

```
icons/
├── branding/
│   └── s_e_e.svg              # Source logo (64x64 viewBox, transparent)
├── scripts/
│   ├── build.mjs              # Main icon generation script
│   └── verify.mjs             # Validation script
├── dist/                      # Generated icons (gitignored)
│   ├── png/{size}/see.png     # PNG icons in various sizes
│   ├── windows/see.ico        # Windows icon file
│   ├── macos/see.icns         # macOS icon file
│   ├── linux/hicolor/         # Linux XDG icon hierarchy
│   └── web/                   # Web assets (favicon, OG image)
├── copy-icons.js              # Copies icons to GUI assets
└── package.json               # Dependencies and scripts
```

## Generated Outputs

### PNG Sizes
16, 22, 24, 32, 48, 64, 96, 128, 256, 512, 1024 pixels

### Platform Icons
- **Windows**: `see.ico` (mix of BMP and PNG for optimal compatibility)
- **macOS**: `see.icns` (complete resolution set with Retina support)
- **Linux**: XDG hicolor directory structure (hardlinked to PNGs)
- **Web**: `favicon.ico`, `favicon-16.png`, `favicon-32.png`, `og-1200x630.png`

## Usage

### Build All Icons

```bash
npm install
npm run build
```

This will:
1. Copy Heroicons to `../gui/assets/icons/`
2. Generate all platform icons from `branding/s_e_e.svg`
3. Copy branding icons to `../gui/assets/branding/`

### Verify Output

```bash
npm run verify
```

Validates that all expected icon files were generated.

### Clean Generated Files

```bash
npm run clean
```

## Dependencies

- **sharp**: High-performance SVG to PNG conversion
- **@ctjs/png2icons**: Creates ICNS and ICO from PNG
- **heroicons**: UI icon library

## Icon Integration

### Dioxus GUI (Rust)

The logo is available in the Icon component:

```rust
Icon {
    name: "logo".to_string(),
    // ...
}
```

Platform icons are configured in:
- `gui/Cargo.toml`: Bundle metadata
- `dioxus.toml`: Platform-specific icon paths

### Heroicons

UI icons are copied to `gui/assets/icons/` and included via `include_str!` in `gui/src/icons.rs`.

## Technical Details

### SVG Requirements
- Square viewBox (current: 64x64)
- Transparent background
- No external dependencies (fonts, images)

### Optimization
- PNGs use maximum compression (level 9)
- Linux hicolor uses hardlinks (saves ~5MB)
- ICO files use BMP for small sizes, PNG for large (compatibility + size)
- ICNS includes all required resolutions for Retina displays

### Platform Compatibility
- **Windows**: ICO format optimized for Windows 7-11
- **macOS**: ICNS tested on 10.10-14.x
- **Linux**: XDG hicolor specification compliant

## Maintenance

### Updating the Logo

1. Replace `branding/s_e_e.svg` with new logo (ensure square viewBox)
2. Run `npm run build`
3. Run `npm run verify`
4. Commit both the SVG and regenerated icons

### Adding New Heroicons

1. Add mapping to `iconMap` in `copy-icons.js`
2. Add case to `get_icon_svg()` in `gui/src/icons.rs`
3. Run `npm run build:heroicons`

## Troubleshooting

### Sharp Installation Issues (ARM64/M1/M2)
Use `npm ci` instead of `npm i` if prebuilt binaries fail.

### Hardlink Errors on Windows
The script automatically falls back to copying files if hardlinks fail.

### Icon Not Showing in App
1. Ensure `npm run build` completed successfully
2. Check that icon files exist in `dist/` directory
3. Verify `gui/assets/branding/logo.svg` exists
4. Rebuild Rust application: `cargo build --manifest-path=gui/Cargo.toml`

