# SubCraft candidate native verification — 2026-10-21

The Windows export suite **compiled and executed: 18 passed, 0 failed**.
The candidate UI boundary suite **did not compile or execute**: Windows
Application Control blocked a dependency build script before execution (4551).
No retry, relocation, alternative execution route or policy change was attempted.
No desktop candidate executable was built or launched; no screenshot exists.

## Source and environment

- Room PC: `DESKTOP-A4P8DV3`; target `x86_64-pc-windows-msvc`.
- Parent at test start: `e5c7e4ed5884e8cea92d0745e4240ebe5a595c7e`.
- Parent gitlink, `foundation/candidate.json` and actual candidate HEAD all
  matched `7958dc640982f014e56a4f9df81c2ffc0f1f0aee`; both checkouts were clean.
- Upstream ancestor: Aalzard/SubCraft 0.1.7,
  `71e49fde7e4e1dba188a6589c9e5a139d1e3d9d5`; ancestry check passed.
- `rustc 1.96.1 (31fca3adb 2026-06-26)`;
  `cargo 1.96.1 (356927216 2026-06-26)`.
- CAD explicitly released the heavy-build slot. Each invocation used one Cargo
  job, one test thread, `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`,
  `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_BUILD_JOBS=1`, `RAYON_NUM_THREADS=1`.
- Build artifacts were confined to `.candidate/target-subcraft`.
  Locked normal dependency downloads were allowed; no lockfile changed.

## Executed export gate

```powershell
cargo test --locked --manifest-path foundation/subcraft/Cargo.toml -p subcraft-export --lib --target x86_64-pc-windows-msvc --target-dir .candidate/target-subcraft -j 1 -- --test-threads=1
```

Exit 0. Compilation: 5m 12s. Execution: 18 tests in 0.39s; 0 failed,
0 ignored, 0 filtered. Finished before 08:00:39 UTC.

The executed suite covers existing sentinel files, case aliases, destination
directories, sanitized/duplicate/unsafe names, unique staging, complete-byte
publication, an exact destination created after preflight, ZIP name checks,
archive/directory pixel equivalence, packed channels, normal/height conversion,
padding and baked AO. The Unix dangling-symlink regression is excluded on Windows.
There are ten new regression functions in source; nine ran on Windows.

Test executable:
`.candidate/target-subcraft/x86_64-pc-windows-msvc/debug/deps/subcraft_export-86e1b21a65e51f5e.exe`

SHA-256:
`597F547325122F5EEFF032757A1F936BDA2B53452BF7E46BF934BF290B10FCFE`

The initial `--offline` attempt stopped before compilation because the local
crates.io index lacked locked workspace dependency `mimalloc`. Removing only
`--offline` allowed normal locked dependency retrieval and the successful run.
The current compiler also reports a future-incompatibility advisory for upstream
`nalgebra 0.26.2`; it is not a failure of this run.

## Blocked candidate boundary gate

```powershell
cargo test --locked --manifest-path foundation/subcraft/Cargo.toml -p subcraft-ui --features skinstrike-candidate --lib candidate::tests:: --target x86_64-pc-windows-msvc --target-dir .candidate/target-subcraft -j 1 -- --test-threads=1
```

Exit 101. Some dependencies compiled, but neither the `subcraft-ui` test binary
nor its four Windows candidate tests was compiled or executed. The fifth
candidate test is Unix-only. Cargo reported:

```text
failed to run custom build command for `serde_core v1.0.229`
could not execute process .../debug/build/serde_core-524f7cebb8054fba/build-script-build (never executed)
An Application Control policy has blocked this file. (os error 4551)
```

Exact blocked file:
`C:\Users\Neil's Room\Documents\Codex\2026-10-21\task-5\SkinStrike\.candidate\target-subcraft\debug\build\serde_core-524f7cebb8054fba\build-script-build.exe`

SHA-256:
`79C4EDAF118313B7C41EB1A20E70BC39B999D40F51FBF77F84EA9E1A2A8D57DB`

Command approval succeeded; Windows execution policy denied this new build
script. The build stopped at that denial. The older blocked legacy
`project_save_conflicts-03558817f42cf99d.exe` was never retried.

## Logs, release and next gate

Full local logs are retained under ignored `.candidate/verification/`:

| File | SHA-256 |
| --- | --- |
| `export-tests-7958dc6-online.log` | `AC89CFDB63AC9E45034C216F96D7E9D52D401636B1026CC12BE21ED4459E00CF` |
| `export-tests-7958dc6.log` (offline prerequisite failure) | `8923E115982DE33F2463BBE4380475B2C44AE614A58E0B6DACEB525199952DD6` |
| `candidate-boundary-tests-7958dc6.log` | `07A9E98B57F83E1BE9DFD7E8CBF1910F2A610F3802EA438F8B931B3AF7249A34` |

At **08:01:32 UTC**, an elevated read-only host check found no Cargo, rustc,
link or cl processes and **2,056.92 GiB free**. The heavy-build slot was released.

The smallest remaining verification gate is the same candidate boundary suite,
after the responsible Windows policy owner authorizes the blocked build script
through its normal approval process and a build slot is coordinated again.
Do not retry while it remains denied. No alternate path, repackaging, policy
disablement or legacy-binary fallback is authorized.

These results validate the scoped native export behavior, not 3D painting,
candidate launch isolation at runtime, CS2-ready output, or revised legacy save
behavior. Project-save conflict/staging repairs, aggregate load ceilings, bake
history/session lifetime, CS2 data alpha, legacy authoring compatibility and
game-tool round trips remain open acceptance gates. Original projects, source,
branding, upstream licenses and candidate update disablement were preserved.
The earlier source-only notes inside the pinned submodule describe its commit-time
status; this record supersedes those verification statements without changing
the tested candidate commit.
