# SkinStrike 3D foundation recheck — 2026-10-21

## Recommendation

Preserve the complete **Aalzard/SubCraft 0.1.7 app and engine** as the leading
3D foundation candidate, then repair its concrete data-safety and CS2 gaps.
Keep current SkinStrike and all existing projects intact. Do not start a
custom importer/viewport implementation before evaluating this integrated app.
No migration, integration, rebrand, release or replacement was performed here.

The previous v0.1.5 selective-reuse recommendation was too strong for the full
3D product the user wants. Its listed gaps are repairable and do not establish
that rebuilding the integrated workflow costs less. This recommendation follows
the newly approved whole-app comparison; it supersedes that earlier direction
for the 3D product. The older 2D foundation decision remains historical evidence.

## Exact baseline and evidence boundary

- Repository: [Aalzard/subcraft](https://github.com/Aalzard/subcraft), not
  `storytold`. Current main checked through the connected GitHub read API:
  [`71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5`](https://github.com/Aalzard/subcraft/tree/71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5).
  It merges release 0.1.7 on October 18, 2026; the workspace declares 0.1.7.
- Local, clean, separate reference checkout: `../SubCraft-reference` relative
  to SkinStrike, pinned to that exact commit. Sparse population includes source,
  text documentation and licenses; it is **not a complete runnable package**.
- [Release v0.1.7](https://github.com/Aalzard/subcraft/releases/tag/v0.1.7)
  targets the same SHA, published October 18 at 21:34:15 UTC. GitHub lists a
  25,179,889-byte Windows ZIP, 23,362,573-byte installer and 26,783,649-byte Linux
  archive. None was downloaded or executed.
- GitHub reports [CI run 39167586738](https://github.com/Aalzard/subcraft/actions/runs/39167586738)
  successful for this SHA. Its workflow tests the workspace on Linux with
  software Vulkan and builds the workspace/runs engine tests on Windows.
  This is upstream-reported CI evidence, not a local execution claim.
- An independent source reviewer agreed on the integrated-app recommendation.
  No SubCraft build, test, GPU run, package execution or new screenshot occurred.
  Existing upstream captures/performance notes were not treated as our own
  runtime evidence. No new screenshot Library ID exists.

## Why the full app is a materially stronger starting point

Source links below are pinned to the reviewed SHA.

| Existing workflow | Evidence and practical value |
| --- | --- |
| Model ingestion | [`mesh/src/import.rs`](https://github.com/Aalzard/subcraft/blob/71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5/crates/mesh/src/import.rs#L31) supports OBJ, glTF/GLB and desktop FBX, retaining scene transforms, material slots and object names. |
| Picking and projection paint | [`app/src/session.rs`](https://github.com/Aalzard/subcraft/blob/71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5/crates/app/src/session.rs#L439), [`paint/src/stroke.rs`](https://github.com/Aalzard/subcraft/blob/71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5/crates/paint/src/stroke.rs#L28) and [`paint/src/kernel.rs`](https://github.com/Aalzard/subcraft/blob/71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5/crates/paint/src/kernel.rs#L1) connect BVH hits, depth/backface checks, symmetry, stencils and seam-aware UV rasterization. |
| Authoring and history | [`doc/src/layer.rs`](https://github.com/Aalzard/subcraft/blob/71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5/crates/doc/src/layer.rs#L34) and [`doc/src/history.rs`](https://github.com/Aalzard/subcraft/blob/71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5/crates/doc/src/history.rs#L1) provide paint/fill/folder layers, masks, content effects, editable paths and copy-on-write document history. |
| PBR preview/compositing | [`render/src/lib.rs`](https://github.com/Aalzard/subcraft/blob/71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5/crates/render/src/lib.rs#L86) and [`app/src/session.rs`](https://github.com/Aalzard/subcraft/blob/71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5/crates/app/src/session.rs#L1071) implement GPU material viewing/channel views and background compositing; export forces current composites. |
| Baking | [`bake/src/lib.rs`](https://github.com/Aalzard/subcraft/blob/71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5/crates/bake/src/lib.rs#L1) and [`app/src/bake_job.rs`](https://github.com/Aalzard/subcraft/blob/71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5/crates/app/src/bake_job.rs#L19) include CPU mesh-map baking with optional high-poly input and cancellation/progress. |
| Portable 3D projects/export | [`project/src/lib.rs`](https://github.com/Aalzard/subcraft/blob/71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5/crates/project/src/lib.rs#L1) embeds mesh, resources and sparse rasters. Current schema is 5 with earlier schemas supported. [`export/src/template.rs`](https://github.com/Aalzard/subcraft/blob/71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5/crates/export/src/template.rs#L63) already packs channels for several general game engines. |

These features connect across geometry, document/history, tiled images,
materials, compositor, paint, render, bake and project crates. Extracting enough
for actual 3D painting would still inherit much of that system and require a
bridge to SkinStrike's distinct data model. SubCraft uses egui/eframe 0.36 and
declares Rust 1.95 as its minimum; SkinStrike uses eframe 0.30 and now declares Rust 1.89. The renderer
exposes egui-wgpu types, making a mixed UI-version integration more involved
than copying a renderer module. This is an architectural inference from the
reviewed dependencies, not a measured project-duration estimate.

Selective integration remains sensible for a deliberately narrower feature:
preview a SkinStrike-exported texture on a model. That would be less than the
requested integrated Substance Painter-like 3D authoring workflow.

## Targeted repairs still required

| Priority | Verified source concern | Bounded repair/verification target |
| --- | --- | --- |
| Data safety | [`project/src/lib.rs:281`](https://github.com/Aalzard/subcraft/blob/71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5/crates/project/src/lib.rs#L281) stages a predictable sibling `.saving` file. [`export/src/lib.rs:212`](https://github.com/Aalzard/subcraft/blob/71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5/crates/export/src/lib.rs#L212) writes directory outputs directly; sanitized set names may collide. The export dialog has no existing-output confirmation. | Unique staging, explicit no-clobber/replace policy, plan collision detection, cancellation/failure and simultaneous-save tests. Reuse SkinStrike's specified safety requirements and test cases; do not claim its patch has been ported. |
| Load budgets | [`project/src/lib.rs:321`](https://github.com/Aalzard/subcraft/blob/71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5/crates/project/src/lib.rs#L321) allocates from ZIP entry size and reads to completion; manifest reads lack a specific byte ceiling. | Enforce compressed/uncompressed aggregate and per-entry limits before allocations, plus resource/mesh/layer/decoded-memory budgets. Existing texture-set resolution checks and raster dimension caps mean saying “no limits anywhere” would be inaccurate. |
| Bake undo | [`app/src/session.rs:98`](https://github.com/Aalzard/subcraft/blob/71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5/crates/app/src/session.rs#L98) stores mesh maps outside document history; [`set_mesh_maps:299`](https://github.com/Aalzard/subcraft/blob/71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5/crates/app/src/session.rs#L299) changes maps/revision without a history snapshot. | Make bake installation a reversible transaction; verify undo/redo restores maps and dependent generators. |
| Bake lifetime | [`ui/src/state.rs:251`](https://github.com/Aalzard/subcraft/blob/71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5/crates/ui/src/state.rs#L251) replaces sessions without clearing `bake.job`; [`ui/src/dialogs.rs:288`](https://github.com/Aalzard/subcraft/blob/71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5/crates/ui/src/dialogs.rs#L288) polls it into the current session. Messages carry set IDs/maps without an originating-session token. | Reproduce then scope results to originating session/resolution or cancel/drop them on replacement. This is an unexecuted source finding, not a reproduced runtime incident. |
| CS2 data alpha | [`core/src/channel.rs`](https://github.com/Aalzard/subcraft/blob/71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5/crates/core/src/channel.rs#L11) has Opacity and User0–3 channels, but defaults omit them; current viewport output is opaque and built-in BaseColor exports are RGB. | Preserve a dedicated linear data channel and explicitly pack it into export alpha according to a verified CS2 finish-style contract. Do not equate this payload with transparency or silently gamma-transform it. Test exact alpha bytes and RGB independence. |

These are reasons for staged repairs and acceptance gates, not evidence that
the complete app is a worse foundation than rebuilding its workflow.

## Preserve current SkinStrike and avoid silent regressions

Current SkinStrike's portable v2 JSON is image-sized, with transformed artwork,
retained brush points, independent alpha data and an optional UV guide. SubCraft
uses square material-slot texture sets and its own ZIP schema. There is no
SkinStrike adapter in the reviewed source. Flattening a PNG would lose editable
structure and must not be presented as a project migration. Keep the legacy
editor/projects available until a non-destructive adapter or separate legacy
workspace is verified.

SubCraft's UV panel currently pans on primary drag except for Pen, where it
dispatches pen editing ([`uv_view.rs:45`](https://github.com/Aalzard/subcraft/blob/71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5/crates/ui/src/panels/uv_view.rs#L45)).
It does not yet preserve SkinStrike's 2D brush-canvas behavior. Also, glTF import
discards imported images/material appearance while retaining geometry and slot
names; imported PBR appearance must not be promised. UVs must already exist;
UDIM offsets are reserved rather than a complete multi-tile workflow. No CS2
material generation or verified Valve-compatible round trip was found.

## License, attribution and isolation

Code declares [MIT OR Apache-2.0](https://github.com/Aalzard/subcraft/blob/71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5/Cargo.toml#L9).
Retain the upstream license/notice text and contributor attribution; README
credits CalangoTech adaptations. Bundled Inter is SIL OFL 1.1 and Phosphor is MIT.
The pinned [starter-pack credits](https://github.com/Aalzard/subcraft/blob/71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5/assets/starter-pack/CREDITS-CC0.txt)
were separately retrieved through GitHub: 60 maps supplied by Aalzard are
dedicated under CC0 1.0. The artwork itself and all transitive dependency licenses
were not independently audited. Relevant native surfaces include ufbx's C
importer, image codecs, ZIP, GPU/UI dependencies and mimalloc. No code/assets
were copied into SkinStrike in this review.

A derived distribution needs its own deliberate update/storage configuration:
[`updater/mod.rs:11`](https://github.com/Aalzard/subcraft/blob/71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5/crates/app/src/updater/mod.rs#L11)
points to Aalzard's releases; recovery/library paths use SubCraft's namespaces.
For evaluation, prevent automatic updates and isolate config, library and
recovery directories. Do not let a fork update into the upstream product or
write into the user's existing libraries. No namespace/updater/branding changes
were made here, and no installer or security prompt was bypassed.

## Practical next gate

The recommended default is the preserved full-app baseline, not another custom
mesh parser. Once managed Windows policy permits execution and build coordination is settled, verify
the pinned baseline with a licensed synthetic GLB: import, orbit/pick, paint
across a UV seam, layers/masks, save/reopen with original sources removed,
undo/redo, material-channel export and cancellation. Use isolated evaluation
storage and disable upstream updating before a native run. Then repair the
specific safety and CS2 requirements while retaining upstream 3D features.

Room PC has more than 2,000 GiB free; the 20 GiB reserve is intact. No heavy
build was started. Windows Application Control separately blocked the newly
compiled SkinStrike test executable with error 4551 before execution; the exact
target/status are in [save verification notes](SAVE_CONFLICT_ACCEPTANCE_2026-10-21.md).
No policy workaround was attempted. Existing SkinStrike source and user projects remain untouched by
this upstream evaluation except for its separately reviewed save-safety patch.
