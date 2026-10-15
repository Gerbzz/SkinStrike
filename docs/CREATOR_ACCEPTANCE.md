# Creator workflow acceptance

This checklist tracks the isolated draft branch. A checked item needs current test or native-runtime evidence; source code alone does not complete it.

## Implemented and tested

- [x] Hand-authored paint appears in full rendering and bounded preview.
- [x] Paint survives project save/reopen; undo and invalid/locked strokes preserve state.
- [x] Collection folders and saved projects persist and reopen in the editor.
- [x] Gallery covers come from the saved project's rendered pixels.
- [x] Invalid paths, missing folders, and corrupt projects retain the current work and show errors.
- [ ] Repeated or interrupted actions do not duplicate writes or apply stale results.
- [x] Hosted native editor builds and its actual editor and gallery windows are visually inspected.

## Implemented but unverified

- Existing color/artwork layers, alpha/UV controls, portable project format, and PNG export passed the branch's automated regression suite; their interactive use still needs hands-on review.
- Gallery and file workers use generation/revision checks and a cancel signal. Repeated save to the same collection path and the dirty-navigation guard have tests; interruption during a large save and rapid repeated UI actions still need native review.

## Missing from this slice

- CS2 Workshop material generation and verified weapon-model preview.
- Production installer, broad clean-machine compatibility, and real-game round trip.
