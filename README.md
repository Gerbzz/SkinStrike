# SkinStrike

**A native Rust texture-finishing sketchpad for CS2 creators.** SkinStrike explores a small, repeatable workflow: inspect a source PNG, stack named color washes, save the editable choices, and export a new PNG while keeping the source alpha. It is a focused prototype for finish iteration, not a tool that produces a game-ready weapon skin.

> **Current stage:** The PNG inspection, color-layer document, export path, CLI, and native desktop window are implemented and covered by headless tests. Valve-compatible material and texture round trips have not been verified.

## A finish from source to export

1. Open a PNG texture. SkinStrike reads its dimensions, alpha distribution, and SHA-256 fingerprint.
2. Add named RGB color layers with 0–255 opacity. Layers are applied in order; the document also stores whether each is enabled.
3. Save a versioned JSON finish document. It records the base PNG path and hash, so a changed source cannot silently replace the image behind an existing edit.
4. Export a new PNG. The color channels receive the ordered washes; the original alpha channel is preserved.

The desktop UI exposes source inspection, a finish project, layer controls, and export paths. The CLI offers the same narrow file workflow for repeatable experiments.

| File | Role |
| --- | --- |
| Source <code>.png</code> | Original RGBA texture; kept as a separate file |
| Finish project <code>.json</code> | Versioned layer settings plus base-path and hash reference |
| Inspection report <code>.json</code> | Dimensions, alpha counts, and source fingerprint |
| Export <code>.png</code> | New image; never an in-place replacement of the source |

The project prefers a path relative to its folder when the source lives beneath that folder. Keep the source PNG with the project when moving it. Output creation refuses to overwrite an existing export.

## What the current UI helps you inspect

The desktop window shows the opened texture's dimensions and alpha breakdown, the selected project and export paths, layer inputs, and operation errors. A source fingerprint belongs to the finish document, so reopening after an accidental source swap fails clearly instead of exporting a different texture under the old settings.

This is intentionally a color-pass model: each enabled layer blends a single RGB value over every pixel in sequence. The export keeps each pixel's alpha untouched. It is useful for testing color direction and file handoff, while finer artwork and game-format checks remain outside the implemented slice.

## Build and try it

Use Windows and a Rust toolchain with edition 2024 support. The first-party desktop app uses egui/eframe; image decoding and PNG writing use Rust libraries.

~~~powershell
cargo test --locked --features native-ui
cargo build --locked --features native-ui --bins
cargo run --locked --features native-ui --bin skinstrike-ui
~~~

For the CLI, start with a source PNG you own and choose new output filenames:

~~~powershell
cargo run --locked --bin skinstrike -- inspect <source.png> <new-report.json>
cargo run --locked --bin skinstrike -- new <source.png> <new-project.json>
cargo run --locked --bin skinstrike -- add-color <project.json> <name> <r> <g> <b> <opacity>
cargo run --locked --bin skinstrike -- export <project.json> <new-output.png>
~~~

Color channels and opacity are integers from 0 to 255. The project must still resolve to the unchanged base PNG when it is reopened or exported.

## Scope and next steps

SkinStrike currently applies full-image color washes. It does not yet author masks, wear patterns, stickers, Valve materials, or a verified Counter-Strike asset bundle. The native window has compiled and passed headless logic checks, but visual workflow review and clean-machine packaging are still pending.

Future work can add localized layer controls, richer finish compositing, preview and asset validation, and an end-to-end Valve-compatible round trip. Those are goals rather than shipped capabilities.

This repository grants no public license for its project source. Third-party crates retain their own licenses.
