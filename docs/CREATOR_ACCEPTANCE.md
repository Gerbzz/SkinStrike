# Creator workflow acceptance

This checklist tracks the isolated draft branch. A checked item needs current test or native-runtime evidence; source code alone does not complete it.

## Implemented and tested

- [x] Hand-authored paint appears in full rendering and bounded preview.
- [x] Paint survives project save/reopen; undo and invalid/locked strokes preserve state.
- [x] Collection folders and saved projects persist and reopen in the editor.
- [x] Gallery covers come from the saved project's rendered pixels.
- [x] Invalid paths, missing folders, and corrupt projects retain the current work and show errors.
- [ ] Rapid repeated UI actions and an interrupted large UI save do not duplicate writes or apply stale results. A staged-cancellation model test and repeated CLI save/export checks passed; the native interaction check remains open.
- [x] Hosted native editor builds and its actual editor and gallery windows are visually inspected.

## Implemented but unverified

- Existing color/artwork layers, alpha/UV controls, portable project format, and PNG export passed the branch's automated regression suite; their interactive use still needs hands-on review.
- Gallery and file workers use generation/revision checks and a cancel signal. Repeated save to the same collection path and dirty switch/close guards have tests. The [core persistence review](CORE_PERSISTENCE_ACCEPTANCE.md) records the tested cancellation point, 14 exact-binary CLI checks, and native error captures. An actual interrupted large UI save and rapid pointer actions still need native review.

## Missing from this slice

- CS2 Workshop material generation and verified weapon-model preview.
- Production installer, broad clean-machine compatibility, and real-game round trip.
