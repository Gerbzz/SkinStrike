# Active 3D screenshot provenance

The README image is an unedited native screenshot, not a mockup or an old 2D editor capture.

- Capture workflow: [run 40231707117](https://github.com/Gerbzz/SkinStrike/actions/runs/40231707117), successful on 2026-10-25.
- Root source: `06d2ae8bb299bfaa569ae450337bf421aab79f57`.
- Selected foundation: `85d35ed6ad74f98aa79f5c2d931ded104ebc05df`.
- Binary SHA-256: `ecd5878d04ff18586b2b180301ee823715ef78c24e1180a2dcc910ae23623004`.
- PNG SHA-256: `0f84fd1769267a80f8b942efb9a661caa326bd0c577256881c077f2ffa70b3c1` (1600 × 940).
- Downloaded artifact ZIP SHA-256: `b7d294a4e4f6a173a66b1d5f28f13cfa4acba58173a860ab8b2657d5abcdf056`, matching the hosted artifact digest.
- Original [machine provenance](verification/active-3d-capture-2026-10-25/provenance.json) and [runtime log](verification/active-3d-capture-2026-10-25/capture.log) are retained unchanged. Their pending-review field describes the upload-time state, before the inspection below.

## Capture and visual inspection

The existing native `subcraft --demo --screenshot` hook was used with the
`skinstrike-candidate` feature. The app ran under Xvfb on the standard hosted Ubuntu
runner using Mesa llvmpipe software Vulkan. Its data/library locations were isolated
inside the runner; no user project was read or published. The capture script and
workflow are in this repository. No foundation source, UI layout or project was
modified to create the image.

After download, the artifact and PNG hashes were verified and the PNG visually
inspected: the 3D viewport displays the sample crate with wood/moss materials and
paint strokes; the Parts, History, Layers, Properties and Shelf panels are present.
The screenshot retains the upstream SubCraft name visible in the current selected
candidate. No pixels were edited, cropped, composited or renamed to impersonate a
finished SkinStrike UI.

This is a built-in procedural demonstration, **not** CS2 game assets or verified
game-ready output. It establishes that this exact source built, launched and
rendered this scene in hosted software Vulkan. It does not establish interactive
painting, save/reopen/export round trips, hardware GPU performance, Windows UI
acceptance, packaging or completed migration. The separate local Application
Control 4551 block remains recorded and was not bypassed or retried here.
