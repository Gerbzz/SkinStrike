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

No executable installation, credentials, upstream remote, public release or user
project files were modified by this evaluation.

## Approved candidate branding — 2026-10-25

![Approved SkinStrike logo](assets/branding/SkinStrike.png)

The `skinstrike-candidate` feature embeds the exact approved 512px SkinStrike PNG
for its native window and full-canvas ICO derivatives for Windows resources.
Feature-off SubCraft artwork and behavior are unchanged.
Source SHA-256: `e1523b9145d862eeb35dd566afe6d697984cf404f30cfd5c8f7d3b1ecefddff4`.
Run `python scripts/check_skinstrike_branding.py` (Pillow) for source, binding and
frame-pixel contracts; `python scripts/generate_skinstrike_icons.py` regenerates
only the candidate ICO. No cropping, recoloring or padding changes are applied.
Local asset contracts pass; native candidate compilation/UI verification remains
a separate gate. Upstream version and source ancestry remain unchanged.
