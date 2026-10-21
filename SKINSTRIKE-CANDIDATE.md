# Local SkinStrike foundation candidate

Derived from Aalzard/SubCraft 0.1.7, upstream commit
`71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5`. Full upstream source and assets
remain present. Upstream MIT/Apache license text, contributor attribution,
Inter/Phosphor notices and CC0 starter-pack credits are retained unchanged.

Local modifications:

- Native texture directory exports reject collisions and existing outputs,
  stage complete encoded maps uniquely, then publish without replacement.
  Browser VFS behavior and rendering/channel math remain available.
- The opt-in `skinstrike-candidate` feature provides explicit native candidate
  identity, isolated data/library/UI state, no upstream updater, and rejected
  update CLI operations. Feature-off launch behavior remains upstream's.
- Regression test source covers the export and candidate boundary changes.

This is an altered local evaluation version, not an upstream release or an
upstream endorsement. No local build or native execution has verified these
changes. Rust parsing and locked no-dependency Cargo metadata passed only.
CAD retains the heavy-build slot; previously blocked legacy SkinStrike tests
were not retried or bypassed. New tests remain uncompiled and unexecuted.

Use the parent SkinStrike repository's `SkinStrike-3D.ps1` entry and build
receipt. Root Cargo there still builds the preserved legacy 2D editor; existing
SkinStrike JSON has no destructive conversion path here. This candidate uses
native `.subcraft` projects. CS2-specific output remains an acceptance gate.

No branding assets, executable installation, credentials, upstream remote,
public release or user project files were modified by this evaluation.
