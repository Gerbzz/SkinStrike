# Project save conflict guard — 2026-10-21

Isolated Room PC work based on verified main
`edb993641c74315025deb2463f847b5ea126f1c6` (PR #2 merge).

## Behavior and guarantees

Previously, `save_document_with_cancel` checked project identity before staging,
then unconditionally replaced an existing destination. Data placed there during
encoding could be lost. The initial patch added a second parsed-document check;
independent review then identified its remaining publication race and its failure
to notice unknown JSON fields added by another writer.

The latest source:

- Holds an OS file lock on a stable sibling
  `.<project-filename>.skinstrike-save.lock`, acquired before reading the initial
  destination and kept through publication. A competing participating save to
  the same normalized pathname fails closed. Lock errors also fail closed.
- Never truncates or unlinks this sidecar. Unlinking would allow different
  processes to lock different files at the same name. The suffix is reserved:
  a project cannot be saved over a lock sidecar. Existing sidecar links and
  nonregular files are rejected. The parent directory is canonicalized.
- Parses project identity and hashes the **same initial byte snapshot**. The
  final check hashes bytes, so unknown fields and whitespace changes cause a
  conflict too. Reads use the opened file's metadata and a bounded reader; file
  growth cannot turn the 256 MiB project ceiling into an unbounded read.
- Keeps unique temporary-file encoding, sync and same-directory publication.
  An initially absent destination always uses `persist_noclobber`, even when
  overwrite was requested. Changed, invalid, oversized or deleted destinations
  detected at the final check abort. A final cancellation checkpoint follows
  fingerprint verification. Temporary staging files are dropped on failure.

The sidecar closes the check-to-replacement race **between cooperating saves
using that pathname** without introducing a missing-destination window. It does
not make arbitrary external writes into a compare-and-swap transaction.

The UI's existing worker error path keeps the current edits and displays the
failure. No document schema, artwork, renderer, branding or dependency changed.
The declared minimum Rust version is now **1.89**, when `File::try_lock` became
stable; compilation here used Rust 1.96.1. Native UI interaction was not tested.

## Platform boundary

Local tempfile 3.27.0 uses `MoveFileExW(MOVEFILE_REPLACE_EXISTING)` for replacement
on Windows and rename on Unix. Those operations have no expected-byte/hash
precondition. Denying delete sharing on the Windows destination would block our
own rename; permitting it leaves path replacement possible. Moving the original
aside before creation would sacrifice atomic publication. No such fallback was
introduced. See [MoveFileExW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw)
and [Rust file locking](https://doc.rust-lang.org/stable/std/fs/struct.File.html#method.try_lock).

The protocol assumes all participating writers honor and retain the same
sidecar. Other editors, older SkinStrike versions, or programs replacing/removing
the sidecar do not participate. Leaf aliases such as Windows short names and
symlink/hard-link aliases are not promised to share a lock. Byte changes after
the final fingerprint check can still be overwritten by publication. The test
`characterizes_external_writes_after_last_check_as_outside_the_lock_protocol`
deliberately records that limitation rather than claiming it is fixed.

There is also no last-loaded revision token. A stale in-memory document can
overwrite edits completed before the initial destination snapshot, including
edits made while input validation was running. Sequential saves retain existing
last-writer behavior. Atomic publication is not a power-loss durability promise
for directory metadata on every filesystem.

## Verification status — latest source is not runtime verified

Historical executed evidence:

- All five initial conflict tests failed against unchanged main with the
  expected destructive-save assertions.
- The first parsed-check patch then passed those five tests and the full
  default suite: **37 passed, 0 failed** (29 unit, 3 library, 5 conflict tests).
  This result predates the sidecar, exact-byte comparison and extra checkpoint.

Latest source evidence:

- `cargo check --locked --offline --tests -j 1` passed with one job, incremental
  compilation and debug symbols disabled. This compiles the default library,
  CLI and test code; it executes no test binary and excludes the native UI.
- Changed Rust files passed `rustfmt --edition 2024 --check`; `git diff --check`
  passed. No dependencies were added or updated.
- Eleven conflict integration test cases are present: the original five, an
  unknown-field edit, an overlapping participating save, cancellation after final
  verification/retry, stable sidecar contents and separate OS handles, reserved
  lock names, and the remaining nonparticipating-writer race. Existing staging
  cancellation/repeated-save coverage was adjusted for the persistent sidecar.
  **These cases have not been executed against the latest implementation.**
- Independent read-only review identified the initial weaknesses and reviewed
  the revised production logic and tests. This is source review, not test proof.

Runtime blocker (known not executed): approved command
`cargo test --locked --offline --test project_save_conflicts -j 1` compiled its
test target but Windows refused to execute
`C:\Users\Neil's Room\Documents\Codex\2026-10-21\task-5\SkinStrike\target\debug\deps\project_save_conflicts-03558817f42cf99d.exe`:
**“An Application Control policy has blocked this file. (os error 4551)”**.
The tool explicitly reported **“never executed.”** No retry, alternate binary
location, policy change or bypass was attempted. Required next step is the
authorized managed Application Control approval path before test execution.

An earlier restricted-sandbox setup failure involved Windows tempfile rename;
the historical successful test runs used approved escalation on synthetic
temporary fixtures. That is separate from the newer Application Control block.

No heavy/native UI build, package run, screenshot, merge, release or deployment
occurred. No screenshot Library ID exists. Disk free remains above 2,000 GiB,
well above the 20 GiB reserve. Current SkinStrike projects and branding stay in
place. The work may be preserved in an explicitly unverified **local commit**;
no push or integration is implied.

The save fix is data integrity work, not a claim of 3D painting progress or
game-ready output.

## Publication verification

The preceding results record the Room PC checks before publication. Hosted CI
checks out and verifies the exact PR head. Windows runs the full default and
native-UI suites plus executable builds; Linux additionally executes the save
conflict regressions before its existing native capture. Hosted results will be
recorded on the PR and do not clear the Room Application Control denial above.
