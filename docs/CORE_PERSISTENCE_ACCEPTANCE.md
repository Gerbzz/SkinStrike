# Core persistence review - 15 October 2026 UTC

This review covers the isolated [draft PR #1](https://github.com/Gerbzz/SkinStrike/pull/1). It uses original synthetic inputs, with no game or client assets. It is evidence for file behavior and selected native error states, not a complete interactive UI acceptance test.

## Exact executable and hosted checks

- [Windows CI run 38714651535](https://github.com/Gerbzz/SkinStrike/actions/runs/38714651535) succeeded at source `e16803030603e71d695d47f5bc204370c57bdc5b`. Project and native UI tests, native UI and CLI builds, SHA manifest creation, size caps, and one-day artifact upload all passed. The later feature merge added main-branch documentation and screenshots, with no source changes.
- Downloaded `skinstrike.exe`: 2,427,392 bytes; SHA-256 `4179EBB64BE74EED7F5E6EFC73B18A6A40F29275361492E81DF0E35B4F05F2B4`.
- Downloaded `skinstrike-ui.exe`: 22,282,240 bytes; SHA-256 `AE6454DE67893A9663A1A84B5E26AE1548371C4BA1152E36A0833BC918C53CA2`.
- Both local hashes matched the hosted manifest. No local Rust build was run.

The hosted regression test `cancellation_after_staging_preserves_previous_project_and_repeated_save_is_stable` cancels after temporary-file staging and before persistence. It checks that the prior project bytes remain unchanged, the staged file is removed, and a later save/reopen/repeat save remains stable. The existing native UI suite also covers the unsaved close guard and library-card switch guard. These tests exercise their model paths; they do not emulate an OS power loss or a rapid human gesture sequence.

## Synthetic CLI checks on the exact hosted binary

Fourteen checks passed locally with a generated 256 x 256 source image and a hand-authored paint project. A newly created project accepted two successive named color-layer edits, reopened, and exported decoded pixels different from the base. The paint project reopened and exported more than 100 expected accent pixels. Invalid requests returned failure without creating output or changing protected source bytes.

| Area | Verified result |
| --- | --- |
| Repeated project creation | Existing project destination refused; original SHA-256 unchanged. |
| Repeated layer name | Duplicate name refused; saved project bytes unchanged. |
| Reopen and export | Reopened edited project exported a valid 256 x 256 PNG with altered pixels. |
| Repeated export | Existing export refused; its SHA-256 unchanged. |
| Protected destinations | Non-PNG destination and imported source PNG both refused; source SHA-256 unchanged. |
| Painted project | Portable synthetic project reopened and exported a decoded PNG containing its saved strokes. |
| Invalid inputs | Corrupt and missing project, missing source PNG, and corrupt PNG were refused without output. |

The local receipt and synthetic input files remain outside this repository because the receipt contains Office PC paths. The painted export SHA-256 was `89880B2B0D43209F5D95EB2EF07A73C13617C6AF0F14D5923E10219F52A70770`. The created project's byte hash is run-specific because the new command assigns a project identifier.

## Native error-state checks

The same hosted UI executable rendered a 1440 x 900 local library with one corrupt synthetic project. Visual inspection confirmed a project card with a visible JSON error. That diagnostic screenshot stays local. The first-party capture entry point returned exit code 1 for a corrupt project, a missing project, and a missing library folder; each case printed a specific error and created no screenshot output.

Native pointer drawing, Save dialog behavior, collection switching under rapid input, interrupted large UI saves, OS-level abrupt termination, and clean-machine installation remain unverified. The [native creator captures](NATIVE_CREATOR_ACCEPTANCE.md) document the visually inspected valid gallery and editor states.
