<p align="center">
  <img src="assets/branding/SkinStrike.png" alt="Approved SkinStrike chameleon logo" width="160" height="160">
</p>

<h1 align="center">SkinStrike</h1>

<p align="center">
  <a href="https://prowebdevsolutions.com"><picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/branding/PWDS-logo-orange.svg">
    <img src="assets/branding/PWDS-logo.png" alt="PWDS company logo" width="120">
  </picture></a>
</p>

**Explore a finish in layers.**

## Rust direction and migration status

- **Target:** A Rust-native 3D texture-authoring application built on the selected SubCraft foundation, starting with CS2 requirements; the legacy 2D editor is not the final selected application.
- **Current implementation:** The [pinned 3D foundation candidate](docs/FOUNDATION_CANDIDATE.md) is already Rust and has its own explicit entry point. The separate root [Cargo package](Cargo.toml) and [library](src/lib.rs) remain the preserved Rust 2D editor, not an accepted replacement for the 3D target.
- **Remaining migration:** Complete 3D candidate integration and native workflow/asset acceptance before adoption. This is an unfinished application transition, not a new language port; no automatic legacy-project conversion, completed 3D migration or game-ready output is claimed.

## 3D foundation candidate

The primary 3D development candidate is now the preserved full SubCraft 0.1.7 app at `foundation/subcraft`, with explicit entry point `./SkinStrike-3D.ps1`. Run `./SkinStrike-3D.ps1 -Action Info` to inspect its exact source, feature and executable path without building or launching. See [candidate provenance, retained features and acceptance gates](docs/FOUNDATION_CANDIDATE.md).

**Evaluation candidate:** its native export suite compiled and executed with **18 passing tests** on `7958dc6`. Windows Application Control then blocked a dependency build script for the candidate UI tests (4551); those tests did not compile or execute, and were not retried. The desktop app has not been built or launched locally. See the [exact verification and remaining gate](docs/verification/2026-10-21-subcraft-native-gates.md). Existing root Cargo commands, source, projects and screenshots below belong to the preserved **legacy 2D editor**. No executable or editable project was replaced or converted. The 3D entry point never falls back to that editor.

SkinStrike lets CS2 creators compose a finish from a base PNG, color, positioned artwork, and hand-painted strokes. Keep an editable project in a local collection, then export a full-resolution PNG for review.

## Highlights

- Combine color washes and artwork layers.
- Paint round-brush strokes on an editable layer, with undo and saved stroke data.
- Adjust placement, visibility, and opacity.
- Find saved finishes by collection and title, with covers rendered from each project.
- Keep an editable source and export a PNG.

## Creator preview

![SkinStrike native project library with eight rendered synthetic finish covers](assets/screenshots/skinstrike-creator-gallery-current.png)

![SkinStrike native editor displaying a saved synthetic paint layer](assets/screenshots/skinstrike-paint-editor-current.png)

These [genuine native captures](docs/NATIVE_CREATOR_ACCEPTANCE.md) came from tested source `91f558d94e4455fede5eb63a27fc5c63332b006d`. Eight original synthetic local projects supplied the rendered covers; only the screenshots were published. They first appeared on main as [development previews](docs/DEVELOPMENT_CREATOR_PREVIEW.md). The implementation and review are in [PR #1](https://github.com/Gerbzz/SkinStrike/pull/1). The [earlier editor capture](docs/SCREENSHOT_PROVENANCE.md) shows the narrower canvas before this work.

## Current status

Image-authoring prototype. Core layer, alpha, project, CLI, and desktop paths have automated coverage. The creator work adds bounded brush strokes and a local visual project library; its hosted Windows tests pass and its two captured native states were visually inspected. The [core persistence review](docs/CORE_PERSISTENCE_ACCEPTANCE.md) records exact-binary save, reopen, export, and error checks. The [acceptance checklist](docs/CREATOR_ACCEPTANCE.md) tracks remaining interaction checks. Valve-compatible material and texture round trips remain unverified; this does not produce a game-ready skin.

## A finish from source to export

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

## What the current UI helps you inspect

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

## Scope and next steps

SkinStrike offers a simple round brush, without advanced mask painting or pressure-sensitive brush dynamics. It has no rotatable verified weapon preview, Valve material generation, or verified Counter-Strike asset bundle. The tested hosted editor compiled, passed automated tests, and rendered the documented synthetic projects in real native windows. Hands-on interaction review and clean-machine packaging are still pending.

Limits: 128 layers, 80 characters per layer name, 4096 pixels per PNG side, 32 MiB per PNG file, 64 megapixels across unique referenced decoded assets, and 256 MiB per project JSON. Preview is capped at 1024 pixels; final export uses the full canvas. Existing PNG replacements are limited to 32 MiB. A Valve-compatible round trip remains a future verification gate.

The [focused foundation review](docs/FOUNDATION_CHOICE_2026-10-15.md) explains why the creator work retains its Rust document and renderer instead of importing a larger photo or 3D painting engine. No code or artwork was copied from those references.

This repository grants no public license for its project source. Third-party crates retain their own licenses.

## PWDS brand artwork

The company mark in this masthead is the existing [official PWDS logo](https://github.com/Gerbzz/PWDS-Website/blob/main/public/prowebdevsolutions-logo.png), with the [official orange display asset](https://prowebdevsolutions.com/images/prowebdevsolutions-logo-orange.svg) for dark backgrounds. Any app logo above it remains the original Strike suite artwork.
