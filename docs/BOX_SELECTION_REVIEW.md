# Shift-drag box selection review — 2026-10-03

Reviewed against `docs/THERMONUCLEAR_REVIEW.md`. No blocking findings.

- Input remains shared by native and WebGL2. Shift is captured at mouse-down, only on empty ground with the build flow closed. Touch explicitly starts without Shift; HUD clicks and placement retain their existing paths. Ordinary drag pans; Shift-click retains its toggle behavior. Focus loss and two-finger gestures cancel the pointer.
- Box release adds enclosed projected ground anchors to the current selection, without toggling or duplicates. Empty boxes preserve selection. The viewport clips the selection bounds and sorted IDs make the result deterministic. Only units already in the visible snapshot are eligible; buildings and resources cannot enter unit selection.
- A translucent gold rectangle uses the existing HUD quad renderer and has no hit region. No new dependency, renderer path, domain rule, command, cost, reservation, persistence behavior or autonomous unit action.
- Gesture code and selection code were moved to small dedicated modules so no client file exceeds 1,000 lines. The alternative of extending the nearly full entry-point/view files would reduce readability.
- Focused Rust tests cover projected feet, reversed bounds, offscreen exclusion, additive selection, empty boxes and duplicates. Existing Shift-click tests remain active. All 98 workspace tests pass; formatting and strict native/WASM Clippy pass; native release and WebAssembly rebuilt.

Browser evidence under `artifacts/box-selection/`: desktop DPR 1 and DPR 2 exercise box selection of both starting villagers, reversed additive selection, Shift-click toggling, Escape and ordinary drag pan, with no page errors. Native uses the same gesture and selection code. No generated artwork; tracked `web/pkg` is rebuilt WASM and bindings.

Remaining prior gaps: additive touch selection and accessible DOM controls in the wgpu client. This PR does not claim either is implemented.

Deployment prerequisite: the previous master workflow failed after deployment because its verification still expected 60×40/2,400 cells. Updated the existing check to the current 120×80/9,600 cells; no runtime or save changes.
