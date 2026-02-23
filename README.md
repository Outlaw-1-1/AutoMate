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
- **Drawings overlay:** PDF/image-backed canvas with drag/drop tokens, route lines, undo/redo, and object-linked nodes.

## Crate adoption opportunities

High-value public crates that can be integrated next:

- `uuid` for globally unique IDs across merged project files.
- `tracing` + `tracing-subscriber` for structured diagnostics and performance telemetry.
- `anyhow` / `thiserror` for cleaner error propagation and typed app errors.
- `directories` for OS-native app config, autosave, and cache locations.
- `rayon` for parallel heavy operations (PDF rasterization, large template expansions).
- `strum` for enum iteration/labels to reduce UI boilerplate around object and point types.
- `schemars` to generate JSON schema for project/template files and improve compatibility checks.

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
