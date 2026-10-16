# SkinStrike authoring and library design

## Intent

Let a creator make visible changes by hand, save the editable finish, and find and reopen it from a visual local library. The work stays on an isolated draft branch. The gallery shows real rendered project pixels. It never substitutes generated artwork, public sample skins, or an unrelated mockup for a user's finish.

## Authoring

Keep the existing portable version 2 project format and layer controls. Add a paint layer containing bounded vector strokes: canvas coordinates, RGB color, brush radius, and point samples. A stroke is committed only when a pointer gesture ends; Undo restores the previous layer state. The same compositor rasterizes paint for preview and full PNG export, so the gallery and export reflect the saved content. Reject non-finite coordinates, out-of-range radius, and oversized stroke data before save or render. Existing projects without paint layers still load.

The native editor exposes a Brush tool, a new paint-layer action, clear color and size controls, and a live stroke overlay while drawing. Locked or hidden paint layers cannot be edited. Save/reopen must preserve the stroke and rendered pixels. The editor continues to protect unsaved changes when switching projects.

## Library

A creator chooses a root folder. Named collection subfolders and their portable `.skinstrike.json` projects are the persistent store; the app creates no private cloud account or database. Saving a project into a collection uses its stable project identity and a file name bounded to the selected root. Reopening a card uses the existing project loader and unsaved-change guard. A bad project leaves the current document untouched and appears as an error card rather than a fabricated cover.

The gallery scans only the selected root and direct collection folders. It indexes paths and modification times, then generates visible page covers in a worker by loading and rendering each actual project at thumbnail resolution. Search filters titles, collection chips limit scope, and paging bounds memory and startup work. Folder choice persists in app-local settings so the same library can return after restart; missing folders show a repair action. Symlinks and non-project files are skipped.

## Visual bar

The current native screenshot is a functional but sparse editor. The next UI should give the work a prominent canvas, a legible tool strip, a clear properties/layers panel, and a gallery with image-first cards, useful labels, empty/error states, and an obvious route back into the editor. This is a quality direction, not a claim of matching another application's feature set.

The approved SkinStrike logo and name anchor the startup/library view and any real loading state. A restrained PWDS publisher credit belongs there and in About. The active editor needs useful navigation and the native window title, not a permanent logo/name block. No artificial loading delay is added.

## Verification and limits

Use real save/reopen/export tests for paint, collection creation and project persistence, content-derived thumbnails, corrupt-file handling, missing roots, and traversal rejection. Run hosted Windows build/tests before any native capture. Capture the genuine executable with synthetic creator-owned inputs and check the image visually. No Office PC dependency builds while free space is near 20 GiB. Do not merge the draft into main or publish an installer during this pass.
