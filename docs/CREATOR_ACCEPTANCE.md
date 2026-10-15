# Creator workflow acceptance

This checklist tracks the isolated draft branch. A checked item needs current test or native-runtime evidence; source code alone does not complete it.

## Implemented and tested

- [ ] Hand-authored paint appears in full export and bounded preview.
- [ ] Paint survives project save/reopen; undo and invalid/locked strokes preserve state.
- [ ] Collection folders and saved projects persist and reopen in the editor.
- [ ] Gallery covers come from the saved project's rendered pixels.
- [ ] Invalid paths, missing folders, and corrupt projects retain the current work and show errors.
- [ ] Repeated or interrupted actions do not duplicate writes or apply stale results.
- [ ] Hosted native editor builds and its actual window is visually inspected.

## Implemented but unverified

- Existing color/artwork layers, alpha/UV controls, portable project format, and PNG export have prior repository tests and one native screenshot. They need fresh draft-branch regression and interaction checks after this change.

## Missing from this slice

- CS2 Workshop material generation and verified weapon-model preview.
- Production installer, broad clean-machine compatibility, and real-game round trip.
