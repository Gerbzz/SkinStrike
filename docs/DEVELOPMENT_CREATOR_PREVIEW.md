# SkinStrike creator development preview

These images preview the isolated [draft PR #1](https://github.com/Gerbzz/SkinStrike/pull/1). This main-branch update adds documentation and screenshots only; it does not merge painting or library code.

The [hosted Windows run 38712703316](https://github.com/Gerbzz/SkinStrike/actions/runs/38712703316) passed its project and native UI tests and built `skinstrike-ui.exe` from source commit `91f558d94e4455fede5eb63a27fc5c63332b006d`. The downloaded 22,282,240-byte executable matched its hosted SHA-256 manifest: `C165B151F9E2F19A0E9C619F853DF52698EF37EE59EB0A28E8A3360451FC244B`.

The executable produced both 1440 × 900 PNGs through its own egui screenshot event on the Office PC. They were visually inspected at original resolution. Their PNG chunks contain only IHDR, IDAT, and IEND, without embedded text metadata.

| Capture | SHA-256 | Git blob |
| --- | --- | --- |
| [Visual library with eight covers](../assets/screenshots/skinstrike-creator-gallery-current.png) | `89BCF75C36719577786FCF9F24828782E0B6727F827FFE7144F3C82FA11D9DB5` | `a3b499fd5932da0be9a6c4e0567c89f900143f86` |
| [Editor with saved paint layer](../assets/screenshots/skinstrike-paint-editor-current.png) | `D98AA1D5F4349637C740D44A0D1FFF874AC81DEBD4A6CE8F9D8A985F082C0327` | `20b1c555ce19f96571efd6c167a0ef346e6a2a6f` |

The eight local project files were made with original procedural gradients and stroke coordinates and loaded by the draft executable. No personal, client, Valve, or third-party media was used. Only the two screenshots were added to main. The captures demonstrate displayed native states, not a recorded pointer gesture or verified CS2 compatibility. The [draft acceptance checklist](https://github.com/Gerbzz/SkinStrike/blob/feature/project-library-2026-10-15/docs/CREATOR_ACCEPTANCE.md) tracks additional interaction and game-review work.
