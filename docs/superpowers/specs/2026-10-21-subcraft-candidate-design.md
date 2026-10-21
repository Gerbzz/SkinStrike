# Reversible SubCraft foundation candidate

The user approved proceeding from the pinned whole-app comparison. Preserve
full SubCraft 0.1.7 (`71e49fd`), current SkinStrike source/projects/executable,
and upstream notices. This is a source integration candidate, not a release,
Rust migration, verified CS2 tool, or permission to run a heavy build.

Use a Git submodule at `foundation/subcraft` with local descendant commits.
`SkinStrike-3D.ps1` is the explicit candidate entry point. Root Cargo remains
the legacy 2D app and is labeled accordingly. Candidate build/run must never
fall back to its executable. Track upstream ancestry and an immutable source,
feature, executable hash receipt. Keep target output separate from legacy.

A candidate-only Cargo feature retains upstream's full authoring engine while
validating isolated runtime paths before logging/recovery, using isolated UI
persistence, disabling upstream update creation/CLI actions, and identifying
the window as a candidate. Original branding assets and attribution stay intact.

First repair: directory exports reject all existing output paths and unsafe or
colliding portable names; native file publication uses unique synced staging
and no-clobber commit. Browser VFS behavior stays available. Multi-file export
is not promised transactional. Write regression source and mark it unexecuted.

No conversion of SkinStrike v2 JSON: existing editable projects remain in the
legacy editor. Native `.subcraft` remains schema 1–5. No flattening masquerades
as migration. CS2 alpha semantics, baking history/lifetime, load budgets and
legacy editing compatibility remain explicit acceptance gates.

Constraints: no push/PR/merge/release; no executable replacement; no heavy build
while CAD owns the slot; no execution/retry/bypass of the Application Control
4551-blocked SkinStrike test; maintain at least 20 GiB free. Static metadata,
parser checks, source review and local commits are allowed. Actual tests/build
and native capture remain pending policy clearance and build coordination.
