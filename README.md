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

## Feature inventory (current)

- **Studio shell + UX:** splash/login/studio flow, custom title bar, toolbar menus, glass panels, adaptive UI scale.
- **BAS object modeling:** hierarchical building/controller/equipment/point tree, reparenting, duplication, context actions.
- **Template-driven engineering:** equipment templates with point generation and dual hour modes (static / points-based).
- **Estimating:** calibrated labor model with complexity/renovation/integration factors and QA/PM/risk overhead.
- **Project data lifecycle:** save/load obfuscated `.m8` bundles (ZIP + JSON + assets), autosave, markdown proposal export.
- **Data interoperability:** one-click object CSV export, JSON Schema export for the `.m8` model, and in-app health diagnostics.
- **Drawings overlay:** PDF/image-backed canvas with drag/drop tokens, route lines, undo/redo, and object-linked nodes.

## Crates integrated in this iteration

- `chrono`: readable local timestamps in proposal exports.
- `itertools`: concise join logic for object-mix summaries and duplicate-ID diagnostics.
- `uuid`: stable project identity (`project_uuid`) for autosave naming and cross-file traceability.
- `directories`: OS-native autosave fallback directory for unsaved projects.
- `thiserror`: typed I/O/serialization/archive errors for cleaner save/load flow.
- `tracing` + `tracing-subscriber`: structured telemetry for app launch and export operations.
- `anyhow`: context-rich, composable error handling for schema/CSV exports.
- `rayon`: parallelized object health analysis for large project graphs.
- `strum`: enum iteration to simplify workspace tab rendering.
- `schemars`: generated JSON Schema export for project contracts.
- `once_cell`: lightweight static app metadata and one-time runtime initialization.
- `csv`: native object inventory export for downstream reporting workflows.

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
