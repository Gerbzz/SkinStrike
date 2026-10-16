# Native creator draft review — 15 October 2026 UTC

This is evidence for the isolated `feature/project-library-2026-10-15` branch, not a release or a game compatibility claim.

## Build and test source

- Exact tested source: `91f558d94e4455fede5eb63a27fc5c63332b006d`.
- [Hosted Windows run 38712703316](https://github.com/Gerbzz/SkinStrike/actions/runs/38712703316) completed successfully: `cargo test --locked`, `cargo test --locked --features native-ui`, and `cargo build --locked --features native-ui --bin skinstrike-ui`. Its executable and SHA manifest passed the 120 MiB cap and were uploaded as a one-day artifact.
- Downloaded native executable: 22,282,240 bytes, SHA-256 `C165B151F9E2F19A0E9C619F853DF52698EF37EE59EB0A28E8A3360451FC244B`, matching the hosted SHA manifest. No local Rust build was run.

## Genuine native window captures

The executable was run on the Office PC with the first-party `--capture-project` and `--capture-library` modes. Both exited successfully after receiving an egui/eframe screenshot event. The views were inspected at their original 1440 × 900 resolution. Their PNGs contain only IHDR, IDAT, and IEND chunks, with no embedded text metadata.

| View | File | PNG SHA-256 |
| --- | --- | --- |
| Library with eight content-derived covers | [`skinstrike-creator-gallery-current.png`](../assets/screenshots/skinstrike-creator-gallery-current.png) | `89BCF75C36719577786FCF9F24828782E0B6727F827FFE7144F3C82FA11D9DB5` |
| Editor with one saved paint layer | [`skinstrike-paint-editor-current.png`](../assets/screenshots/skinstrike-paint-editor-current.png) | `D98AA1D5F4349637C740D44A0D1FFF874AC81DEBD4A6CE8F9D8A985F082C0327` |

The eight local source projects were made for this review with original procedural gradients and stroke coordinates. They are synthetic `.skinstrike.json` projects in two local collections, loaded and rendered by the executable. No Valve, personal, client, or external media was used or uploaded. The fixture project files, helper script, and executable remain outside this repository. The screenshot demonstrates saved stroke rendering and the gallery presentation; it does not show a recorded human pointer gesture.

## Limits

Hosted tests cover paint pixels in preview and full rendering, portable save/reopen, undo and invalid/locked edits, collection persistence, real cover pixels, path rejection, corrupt files, and a dirty-navigation guard. Native pointer drawing, save-dialog behavior, gallery pagination with many projects, accessibility, clean-machine installation, and CS2 Workshop appearance still need hands-on review. The screenshots prove the two displayed native states only.
