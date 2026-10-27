# Historical SkinStrike 2D editor — not the selected 3D app

This page preserves the earlier 2D editor documentation and screenshot provenance. It is not the selected application or evidence of 3D workflow acceptance. Existing 2D projects have no automatic conversion to the SubCraft format.

The preserved **legacy 2D editor** lets CS2 creators compose a finish from a base PNG, color, positioned artwork, and hand-painted strokes. Keep an editable project in a local collection, then export a full-resolution PNG for review.

## Legacy 2D editor highlights

- Combine color washes and artwork layers.
- Paint round-brush strokes on an editable layer, with undo and saved stroke data.
- Adjust placement, visibility, and opacity.
- Find saved finishes by collection and title, with covers rendered from each project.
- Keep an editable source and export a PNG.

## Historical legacy 2D previews — not the selected 3D app

![Historical legacy 2D SkinStrike project library with eight rendered synthetic finish covers](../assets/screenshots/skinstrike-creator-gallery-current.png)

![Historical legacy 2D SkinStrike editor displaying a saved synthetic paint layer](../assets/screenshots/skinstrike-paint-editor-current.png)

These [genuine native captures](NATIVE_CREATOR_ACCEPTANCE.md) came from tested source `91f558d94e4455fede5eb63a27fc5c63332b006d`. Eight original synthetic local projects supplied the rendered covers; only the screenshots were published. They first appeared on main as [development previews](DEVELOPMENT_CREATOR_PREVIEW.md). The implementation and review are in [PR #1](https://github.com/Gerbzz/SkinStrike/pull/1). The [earlier editor capture](SCREENSHOT_PROVENANCE.md) shows the narrower canvas before this work.

## Legacy 2D status

Image-authoring prototype. Core layer, alpha, project, CLI, and desktop paths have automated coverage. The creator work adds bounded brush strokes and a local visual project library; its hosted Windows tests pass and its two captured native states were visually inspected. The [core persistence review](CORE_PERSISTENCE_ACCEPTANCE.md) records exact-binary save, reopen, export, and error checks. The [acceptance checklist](CREATOR_ACCEPTANCE.md) tracks remaining interaction checks. Valve-compatible material and texture round trips remain unverified; this does not produce a game-ready skin.

## Legacy 2D workflow: a finish from source to export

1. Open a base PNG. SkinStrike reads its dimensions, alpha distribution, and SHA-256 fingerprint.
2. Add named color washes, artwork PNG layers, or a paint layer. Choose Brush, color, and size to draw on an unlocked visible paint layer. Each stroke is stored with the project and can be undone. Position, scale, rotate, reorder, hide, and adjust layers. Lock a layer to prevent edits; copies start unlocked.
3. Choose how export alpha is produced: preserve base alpha, use composite coverage, or import a canvas-sized opaque grayscale PNG. Imported gray bytes become alpha bytes exactly, with optional inversion; RGB artwork remains separate.
4. Optionally import a canvas-sized UV reference PNG. The adjustable guide appears in RGB/RGBA editor views but stays out of the Alpha data view and the exported PNG. It is a 2D alignment aid, not a verified weapon preview.
5. Save a versioned JSON finish project that embeds the base, artwork, alpha data, UV guide, and paint strokes. Version 2 projects remain portable after their source files are removed; older version 2 projects open with defaults for new fields. Version 1 still needs its original hash-matched base PNG before resaving as version 2. Choose a local library folder, make collection folders, and save the current finish there to get a real rendered cover and reopenable project card.
6. Export a full-resolution PNG. The export path validates and protects project and source-image destinations. The desktop UI asks before replacing an existing PNG; the CLI requires a new output path.

The desktop UI exposes a project gallery, the finish editor, layer controls, and export paths. The CLI covers the original narrow file workflow; brush and gallery actions currently use the desktop UI or library API.

| File | Role |
| --- | --- |
| Source <code>.png</code> | Original input; embedded into a version 2 project on import |
| Finish project <code>.json</code> | Versioned layers, channel settings, embedded assets, and integrity hashes |
| Inspection report <code>.json</code> | Dimensions, alpha counts, and source fingerprint |
| Export <code>.png</code> | Full-resolution image; project and source-image paths are protected |

The editor supports undo/redo, background import and export work, cooperative cancellation, and stale-result checks. Gallery cards load in a worker and their covers use the saved project's compositor. An individual PNG decoder call may run until it returns. Imported assets are embedded for portable reopening; the originals are not silently overwritten.

## What the legacy 2D UI helps you inspect

The startup window shows the approved SkinStrike logo, a restrained PWDS credit, local collections, searchable project cards, actual rendered covers, and missing/corrupt project states. The editor shows the opened texture, paint and artwork tools, layers, color and alpha views, UV reference overlay, project and export paths, and operation errors. Alpha data view suppresses the colored UV overlay so grayscale values stay readable. The properties panel scrolls in compact windows.

Material intent records a target finish-style note for the artist; it does not change pixels or generate a CS2 material. Valve's [Weapon Finishes Guide](https://www.counter-strike.net/workshop/workshopfinishes/) describes style-dependent alpha meanings, and [Workshop Resources](https://www.counter-strike.net/workshop/workshopresources) lists official UV sheets and models. No Valve assets are bundled here. Check actual style, UV placement, wear, shading, and weapon appearance in CS2 Workshop Tools.

## Legacy 2D build and try it

Use Windows and Rust 1.89 or newer. The first-party desktop app uses egui/eframe; image decoding and PNG writing use Rust libraries. Project saves coordinate through a persistent sibling `.<project-filename>.skinstrike-save.lock` file; keep it in place so participating saves to the same path share an OS lock. Other editors do not participate in this protocol.

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

Color channels and opacity are integers from 0 to 255. To exercise the new channel and UV workflow with synthetic images, run `cargo run --locked --example channel_uv_smoke -- <new-output-directory>` from this folder. The example produces a decoded export and an offscreen editor render; the latter is not a native-window screenshot.

## Legacy 2D scope and next steps

SkinStrike offers a simple round brush, without advanced mask painting or pressure-sensitive brush dynamics. It has no rotatable verified weapon preview, Valve material generation, or verified Counter-Strike asset bundle. The tested hosted editor compiled, passed automated tests, and rendered the documented synthetic projects in real native windows. Hands-on interaction review and clean-machine packaging are still pending.

Limits: 128 layers, 80 characters per layer name, 4096 pixels per PNG side, 32 MiB per PNG file, 64 megapixels across unique referenced decoded assets, and 256 MiB per project JSON. Preview is capped at 1024 pixels; final export uses the full canvas. Existing PNG replacements are limited to 32 MiB. A Valve-compatible round trip remains a future verification gate.

The [historical foundation review](FOUNDATION_CHOICE_2026-10-15.md) explains the earlier 2D decision; the later selected direction is the separate 3D SubCraft candidate. No code or artwork was copied from those references.

This repository grants no public license for its project source. Third-party crates retain their own licenses.
