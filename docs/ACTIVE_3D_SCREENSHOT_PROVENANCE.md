# Active 3D screenshot provenance

The README image is an unedited native screenshot, not a mockup or an old 2D editor capture.

- Capture workflow: [run 40234535679](https://github.com/Gerbzz/SkinStrike/actions/runs/40234535679), successful on 2026-10-25.
- Root source: `6c1fd6d6b1bd53e4e53e4ee7149838f2a21180aa`.
- Selected foundation: `c7d7a1a633e32a5fa90d8cd2ec8deaa405ca5b1c`.
- Binary SHA-256: `203add380ad87416b4706e62f25259e8f58a8fa83124fb9a010822e92b1246f7`.
- PNG SHA-256: `d4b88b4ba9b8e116485f443caf706c581d6fe78085d8048f3b449acae15496a7` (1600 × 940).
- Downloaded artifact ZIP SHA-256: `37828770143088b78d7da155f1df4480d09e9b68392954754abd6a11727f4728`, matching the hosted artifact digest.
- Original [machine provenance](verification/active-3d-capture-c7d7a1a6/provenance.json) is retained unchanged; its pending-review field describes the upload-time state, before the inspection below. A [path-free capture summary](verification/active-3d-capture-c7d7a1a6/capture-summary.txt) records the rendering result; raw runner paths are not republished.

## Capture and visual inspection

The existing native `subcraft --demo --screenshot` hook was used with the
`skinstrike-candidate` feature. The app ran under Xvfb on the standard hosted Ubuntu
runner using Mesa llvmpipe software Vulkan. Its data/library locations were isolated
inside the runner; no user project was read or published. The capture script and
workflow are in this repository. The only candidate source changes are the documented approved artwork/name
bindings and DPI-aware full-canvas header minification; no UI layout or demo project
was redesigned to create this image.

After download, the artifact and PNG hashes were verified and the PNG visually
inspected: the 3D viewport displays the sample crate with wood/moss materials and
paint strokes; the Parts, History, Layers, Properties and Shelf panels are present.
The screenshot shows the SkinStrike header and title, with the approved chameleon
artwork minified at the existing 18-point size; the earlier sampling noise is absent.
Upstream source and legal credits remain intact. No screenshot pixels were edited,
cropped or composited. This is still the integration candidate, not a completed migration.

This is a built-in procedural demonstration, **not** CS2 game assets or verified
game-ready output. It establishes that this exact source built, launched and
rendered this scene in hosted software Vulkan. It does not establish interactive
painting, save/reopen/export round trips, hardware GPU performance, Windows UI
acceptance, packaging or completed migration. The separate local Application
Control 4551 block remains recorded and was not bypassed or retried here.
