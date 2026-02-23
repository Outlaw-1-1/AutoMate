# AutoMate BAS Studio

A Rust desktop prototype for data-driven BAS estimating and submittal workflows.

## Features

- Breathing full-screen gradient background (gray ↔ accent color) with configurable accent theme.
- Professional glass-elevated pane system with consistent padding and shadows.
- Custom title bar with drag, minimize, restore, and close controls.
- Dropdown-based top toolbar (`Tools`, `Project`, `View`, `Help`).
- Three-pane workspace:
  - Left: Collapsible BAS object tree (Buildings → Controllers → Equipment → Points).
  - Center: Active tool view.
  - Right: Property editor with collapsible groups.
- Icon-forward UI language for quick scanning and less text-heavy workflow.
- Engineering-forward typography setup with strong fallback stack.
- Local project save/load to JSON.
- Software settings + About dialog.
- Drawing overlay workflow:
  - Select PDF file for construction drawing reference.
  - Drag controller/equipment tokens onto overlay canvas.
  - Draw line segments on overlay for routing/markup.

## Run

```bash
cargo run
```

## Troubleshooting Windows build path issues

If you build from a synced directory (for example OneDrive) and see errors like:

- `output path is not a writable directory`
- panic in `autocfg` during build scripts (`num-traits`, `memoffset`, etc.)

this project forces Cargo output into a local repo folder via `.cargo/config.toml` (`.cargo-target`).

If you still hit the issue, run:

```bash
cargo clean
# optional: delete old target dir if present
# rmdir /s /q target   (Windows cmd)
cargo build
```

## Troubleshooting PDF overlay renderer

If you see this status in the app:

- `PDF renderer unavailable (...)`

AutoMate could not load the native PDFium library used to render drawing overlays.

You can fix it by either:

1. Placing the platform PDFium binary next to the executable (or in the current working
   directory), including common `bin/`, `lib/`, or `libs/` subfolders.
2. Setting `AUTOMATE_PDFIUM_LIB` to either:
   - the full path to the PDFium library file, or
   - a directory that contains that library.

Expected library names by platform:

- Windows: `pdfium.dll`
- macOS: `libpdfium.dylib`
- Linux: `libpdfium.so`
