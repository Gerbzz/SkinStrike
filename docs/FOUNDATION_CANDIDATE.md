# SkinStrike 3D foundation integration candidate

This is the explicit **source candidate** for the general Rust 3D texture
authoring app, starting with CS2 requirements. It preserves the full SubCraft
0.1.7 tree and Git ancestry instead of rebuilding its mesh/paint/render stack.
It has not been built or launched locally. No existing executable was replaced.

## Exact entry points

| Entry | Source and behavior |
| --- | --- |
| `./SkinStrike-3D.ps1 -Action Info` | Default/read-only candidate identity: `foundation/subcraft/Cargo.toml`, package/bin `subcraft`, feature `skinstrike-candidate`, target `x86_64-pc-windows-msvc`. Does not build or launch. |
| `./SkinStrike-3D.ps1 -Action Build -BuildSlotConfirmed` | Only after CAD releases the heavy-build slot. Builds the exact submodule manifest/features/target into `.candidate/target-subcraft`; single job, no incremental/debug symbols. Requires 40 GiB free for headroom over the 20 GiB reserve. |
| `./SkinStrike-3D.ps1 -Action Run -AppArgs @('--demo')` | Only once Windows policy permits native execution and a coordinated build succeeds. Requires a clean committed submodule matching the parent gitlink, the exact Cargo-reported binary and features, and matching source/entry/contract/binary hashes in a build receipt. Never chooses another executable or retries a failed launch. |
| Root `cargo run --features native-ui --bin skinstrike-ui` | Preserved **legacy 2D SkinStrike**, not the 3D foundation. Root Cargo was not silently redirected. Existing projects remain supported here. |

The candidate binary remains named `subcraft.exe` in its separate target folder.
Its candidate Cargo feature identifies the native caption as
`SkinStrike 3D candidate — SubCraft 0.1.7 foundation`, including per-frame title
updates, while retaining project/dirty state. The compact upstream header,
upstream logo/icon/font assets and contributor notices remain intact.

PowerShell 7 is required for the entry script. Build/Run fail closed if source
or artifact identity does not match. Cargo JSON must report the actual expected
compiler artifact; an unrelated old executable cannot earn a fresh receipt.
An inherited Cargo build-target setting cannot redirect the pinned target.
There is **no build receipt or candidate executable from this session**.

## Source ancestry and local handoff

`foundation/subcraft` is a full-source Git submodule rooted at
`https://github.com/Aalzard/subcraft.git`, baseline
`71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5` / 0.1.7. The baseline contains 478
tracked files. The local branch is
`skinstrike/foundation-candidate-2026-10-21`; local repair/integration commits
descend from that exact baseline. The parent repository pins the reviewed local
candidate commit, and `foundation/candidate.json` records the build contract.

This is local-only work. The upstream public URL does **not** contain our local
descendant commits. A fresh recursive clone cannot fetch that pointer until an
approved fork publication; use the delivered Git bundles for offline handoff.
Do not work around this by resetting the submodule to upstream and presenting
that as the reviewed candidate. No push, PR, merge or release is authorized.

## Retained feature inventory

All rows are source-present, **not locally runtime verified**:

| Workflow | Preserved implementation |
| --- | --- |
| Mesh ingestion and parts | OBJ, glTF/GLB, desktop FBX; transforms, slots/object names, BVH, UV rasterization and part picking. |
| Surface authoring | 3D brush/eraser, symmetry, tips, stencils, pen paths, polygon fill, depth/backface handling and seam-aware projection. |
| Materials and layers | PBR channels, image/procedural fills, paint/fill/folder layers, blend modes, masks, content/mask effects and copy-on-write history. |
| Preview | GPU PBR viewport, individual channel views, studio/HDRI lighting, 2D UV inspection and background compositing. Upstream renderer API fallback within the same app is retained; the launcher never switches executables. |
| Baking | CPU map baking, optional high-poly input, progress/cancellation. Existing undo/session-lifetime gaps remain gates. |
| Projects and resources | Native `.subcraft` schema 1–5, embedded mesh/images/sparse rasters, autosave/recovery and user library. |
| General exports | Existing channel packing, normal conventions, color encoding, padding, engine templates and browser ZIP/VFS path. Native directory replacement becomes explicit rejection for safety. |

The app was already Rust. This work performs no language migration and claims
neither Substance Painter parity nor verified game-ready CS2 output.

## Candidate isolation

Only the candidate feature changes launch configuration. The entry point sets
process-local `SKINSTRIKE_CANDIDATE_HOME`, `SUBCRAFT_DATA` and
`SUBCRAFT_LIBRARY` under `.candidate/skinstrike-3d-candidate`. The binary
validates them **before logging/recovery or updater actions**. UI persistence
uses its explicit `ui-state.ron` path, leaving upstream/legacy saved settings
alone. The wrapper restores inherited process environment on exit.

The updater object is never constructed in candidate mode, so automatic/manual
upstream update UI paths are inert; update installation/resume/check CLI flags
are rejected. Direct candidate invocation without isolated paths fails closed.
The normal upstream feature-off build retains its original behavior.

Preexisting symlink/reparse redirects in controlled evaluation state/output
trees are rejected by the wrapper; candidate startup rejects redirected path
ancestors. This is a check of existing paths, not a security sandbox against
hostile concurrent filesystem changes. Do not repurpose the evaluation folder
as an alias of an existing SubCraft/SkinStrike library.

## First targeted safety repair

Native directory export validates the complete planned output names before
rendering: unsafe portable filenames, sanitized duplicates and conservative
case aliases are rejected. Existing files, folders and dangling directory
entries in the output plan are preserved. Each map stages in a unique sibling
temporary file, writes/syncs its complete encoded bytes, then uses tempfile
`persist_noclobber`; a destination appearing after preflight is not replaced.

This is per-destination no-clobber publication, **not** an all-or-nothing
multi-map transaction. Earlier new outputs can remain if a later map fails.
Tempfile's filesystem fallback can use link/unlink and may leave an extra
temporary link after a crash; universal atomic rename/durability is not claimed.
Name comparison is conservative, not complete Unicode normalization or a
cross-process directory transaction. Browser directory VFS behavior is retained;
ZIP name validation shares the same plan checks. Rendering/math is unchanged.

Ten export regression functions were added, including a native publication-race
case using real staged/destination files and an archive collision case. Existing
pixel tests that intentionally reused names now use separate variant folders.
Candidate isolation/update/title test source was also added (five cases,
including a Unix symlink case). On 2026-10-21 the native export suite compiled
and executed: **18 passed, 0 failed**, including nine of the ten new regressions
(the remaining case is Unix-only). The candidate UI test build was blocked by
Windows Application Control 4551 at a dependency build script, before the UI
test binary compiled or ran. It was not retried. See the [immutable source,
commands, executable hashes and logs](verification/2026-10-21-subcraft-native-gates.md).

## Non-destructive data boundary

Existing SkinStrike v2 JSON remains editable in the preserved legacy editor.
No automatic conversion, schema bump, flattening, asset deletion or source
replacement occurs. The 3D candidate reads native `.subcraft` and supported mesh
formats. Legacy JSON is not a native project; the entry point rejects a direct
legacy-JSON argument, and upstream otherwise treats unsupported input as a
failed import rather than converting it. Save As writes native `.subcraft`.

A future adapter must preserve original JSON, artwork, vector stroke source,
independent alpha payload and UV guide. A flattened texture is a preview input,
not a migration. Square material-slot texture sets differ from image-sized
legacy canvases. SubCraft's UV view does not yet preserve legacy 2D brush input.

## Acceptance gates and smallest next check

Completed: full source/asset presence and ancestry inspection; Cargo locked,
offline, `--no-deps` metadata feature resolution; Rust syntax parsing; PowerShell
parsing; read-only entry Info; refusal to build without releasing CAD's slot;
independent source review. These do not establish type correctness or runtime
behavior. Prior upstream CI success applies to the pinned **unmodified** source,
not our local changes. The old SkinStrike 37-test pass does not validate these
changes or its later `8e85a9b` save implementation.

The single-job native export gate completed successfully after CAD released the
build slot:

```powershell
cargo test --locked --manifest-path foundation/subcraft/Cargo.toml -p subcraft-export --lib --target x86_64-pc-windows-msvc --target-dir .candidate/target-subcraft -j 1 -- --test-threads=1
```

The initial offline attempt lacked a locked dependency index entry; normal
locked downloads enabled the passing run. No lockfile changed. The following
candidate UI feature test gate stopped at the first Windows policy denial:

```powershell
cargo test --locked --manifest-path foundation/subcraft/Cargo.toml -p subcraft-ui --features skinstrike-candidate --lib candidate::tests:: --target x86_64-pc-windows-msvc --target-dir .candidate/target-subcraft -j 1 -- --test-threads=1
```

Its exact denied `serde_core` build script and hash are in the verification
record. Candidate UI type/tests remain pending Windows execution approval by
the responsible policy owner; do not retry while denied. The heavy-build slot
is released, with no compiler processes remaining and 2,056.92 GiB free.
Any later build needs fresh slot coordination, no incremental/debug symbols,
and the 20 GiB reserve. A desktop build through the explicit entry point follows
the boundary gate. Native acceptance uses a licensed synthetic GLB: orbit/pick, paint
across a seam, layers/masks, undo/redo, portable save/reopen, export, cancellation
and isolated state inspection. Only an actual successful native capture can be
delivered as a screenshot, with its Library ID and exact build evidence.

Before treating it as a user replacement: repair project-save staging/conflicts,
aggregate project/mesh/decoded resource ceilings, bake undo and stale-session
results; implement and verify style-specific CS2 data alpha without treating it
as transparency; preserve legacy authoring or build a non-destructive adapter.
Verify any Valve material/texture round trip separately. Those gates remain open.

The previously compiled legacy save test executable was blocked by Windows
Application Control (4551) before execution. It has not been retried, renamed,
relocated or bypassed. The scoped test builds above occurred; no native app run,
new screenshot, installer, credential change or publication occurred. Existing branding assets remain
unchanged. MIT/Apache upstream notices, contributor attribution, Inter OFL,
Phosphor MIT and starter-pack CC0 credits are retained in the full submodule.
