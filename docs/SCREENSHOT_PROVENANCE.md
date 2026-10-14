# SkinStrike native editor screenshot provenance

`assets/screenshots/skinstrike-editor.png` is a genuine 1440 x 900 image returned by SkinStrike's egui/eframe Glow renderer on 14 October 2026 UTC. The editor opened a local 512 x 512 procedural PNG of blue checker squares, cyan grid lines, and orange diagonals. It shows the current canvas, tools, layer controls, alpha options, material intent, and UV-reference entry. The document is unsaved and has zero artwork layers; the fixture was not uploaded. No Valve, user, or client assets were used.

The first-party native entry gained `skinstrike-ui --capture FILE.png TEXTURE.png`. Capture mode opens that texture, requests an `egui::ViewportCommand::Screenshot` after the editor paints, saves the returned `egui::Event::Screenshot`, permits its synthetic unsaved document to close, and exits with status 0. Normal `skinstrike-ui` launch remains available. The successful capture PNG was visually inspected and has the same bytes as the earlier rendered attempt; only the capture-close lifecycle was fixed before publication.

| Item | SHA-256 |
| --- | --- |
| Local synthetic input PNG (not published) | `7F6FB70D4C720658821A1661F71299B14244E8435B4B4ED66B92E3DAD3D54E43` |
| Local capture executable (not published) | `714097BF110038E48EC3F8C56C14ED8F6FC80C8A95AD56C7187EA6E7DA186E4E` |
| Published screenshot PNG | `C22EDC03493D5F12886093CCC36EC9C64CFDBDD7FB44D9CD4D3A9D6B65263492` |

The executable was built from the exact public package source plus the two capture changes with `cargo build --locked --offline -j 1 -p skinstrike --features native-ui --bin skinstrike-ui`, using an existing cached target. The final one-job build finished successfully with C: above 9 GiB free and below the 750 MiB target-growth cap. No dependency download, clean, installer, release, or license grant was made.

This image verifies a rendered native editor state and displayed PNG. It does not verify layer editing, save/reopen, export, alpha/UV interactions, Counter-Strike compatibility, accessibility, or a release package. The previously documented headless tests were not rerun after the capture-only source change.
