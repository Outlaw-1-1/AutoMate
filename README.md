# AutoMate BAS Studio

A Rust desktop prototype for data-driven BAS estimating and submittal workflows.

## Features

- Glass-inspired, transparent full-screen interface with configurable accent color.
- Custom title bar with drag, minimize, restore, and close controls.
- Top tool tabs: Project Settings, Hours Estimator, Drawings Overlay.
- Three-pane workspace:
  - Left: Collapsible BAS object tree (Buildings → Controllers → Equipment → Points).
  - Center: Active tool view.
  - Right: Property editor with collapsible groups.
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
