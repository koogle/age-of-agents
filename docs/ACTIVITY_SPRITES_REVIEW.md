# Gathering activity sprite regression

The farm-field change (`9ff0087`) suppressed work whenever cargo was present. Gathering creates partial cargo on the first collection tick, so chopping, mining, digging and foraging then held the carry frame for the rest of the load.

The shared native/WebGL renderer now permits the work pose while the authoritative phase is `Gathering`. Movement, stopped cargo and unloading still use the carry pose; construction and cultivation still wait for unloading. Existing animation sheets, facing holds, frame rates, terrain depth and simulation behavior remain unchanged.

## Structural review

- One existing pose-selection condition changed; no new production helper, dependency, asset or rendering layer.
- Simulation, commands, costs, occupancy and persistence are untouched; idle villagers acquire no behavior.
- Desktop and touch use the same renderer. Client source files remain under 1,000 lines; focused render tests live in their own module.
- Tests inspect drawn atlas rectangles and advancing frames for eight resource kinds, three villager variants, empty and partial loads, plus stopped/returning/depositing cargo, construction and cultivation.
- Browser verification feeds controlled snapshots to the actual WebGL2 client and inspects the uploaded villager instance UVs. The original bundle holds a single carry frame; the repaired bundle advances authored work frames. This isolates rendering without mutating a production save.

Verification passed: 142 workspace tests, formatting, strict native and WASM Clippy, frontend syntax, asset checks, rebuilt release WASM, and all eight resource poses on desktop and DPR-2 phone without page errors.
