# SubCraft candidate implementation plan

> **For agentic workers:** use native execution for candidate integration and independent review; export safety is delegated with separate file ownership.

**Goal:** Preserve and expose the full upstream 3D app as an explicit reversible local candidate.

**Architecture:** Pin a full-source submodule; add a candidate-only launch/isolation feature and root entry point; preserve legacy root Cargo. Repair export safety within the upstream model.

**Tech Stack:** Existing Rust 1.95 minimum, egui/eframe 0.36, PowerShell 7, Git submodule; no engine rewrite.

**Spec:** `docs/superpowers/specs/2026-10-21-subcraft-candidate-design.md`

## Global constraints

- No push/PR/merge/release, branding asset edits, destructive migration or executable replacement.
- No heavy build while CAD owns the slot. Keep 20 GiB reserve.
- No execution/retry/bypass of the Application Control 4551-blocked save test.
- Source/test additions do not count as executed verification.

## Review focus

- Wrong/root legacy binary: exact submodule manifest/features and parent gitlink; source/executable receipt comparison.
- Upstream state/update leakage: candidate paths validated before logging; no updater; isolated UI persistence.
- Export collisions: case/sanitized names and existing sentinels survive; new files use no-clobber publication.
- Data compatibility: native formats remain separate; no automatic conversion or loss of editing data.
- Failure handling: no runtime retry or old-executable fallback; per-file publication limits stated.

### Task 1: Candidate source and entry boundary

**Files:** `.gitmodules`, `foundation/candidate.json`, `SkinStrike-3D.ps1`, candidate feature in `foundation/subcraft/apps/subcraft` and `crates/ui`.

- [x] Preserve full upstream tree and pinned ancestry.
- [x] Add candidate path/update-validation unit test source, then isolation feature.
- [x] Add explicit Info/Build/Run entry with build-slot gate and source/hash receipt.
- [x] Verify PowerShell parser, Info selection, no-build refusal and Cargo metadata only.
- [x] Independent source review and local commit; build/tests remain deferred.

### Task 2: Export safety

**Files:** `foundation/subcraft/crates/export/**`, relevant locked dependency entry.

- [x] Add sentinel, portable-name/case collision, unique staging and publication-race regression source.
- [x] Implement full-plan preflight and native staged no-clobber publication; preserve wasm VFS.
- [x] Preserve existing pixel tests with separate output directories for repeated export variants.
- [x] Rust parser/metadata checks and independent diff review; runtime tests deferred.
- [x] Local descendant commit, record pointer and offline Git bundle for handoff.

### Task 3: Honest acceptance record

**Files:** `docs/FOUNDATION_CANDIDATE.md`, root README, provenance record.

- [x] Inventory retained upstream features and distinguish source-present from locally verified.
- [x] Record project-format boundary, pending CS2/safety gates and exact next verification command.
- [x] Confirm old assets/source and all upstream license/asset blobs remain unchanged.
- [x] Commit root submodule pin and deliver source bundles; no publication.
