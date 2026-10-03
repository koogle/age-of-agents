# Open Work

Updated 2026-10-03. Branch master, including native-fixes commit 97a04d7 (deployed successfully). User authorized pushing the missing recent native fixes. Recovery commit da820bb on codex/native-sprite-rendering preserves all older local fixes, docs, fonts and generated WASM. Historical root and gukaet edits/saves remain untouched.

## Integrated

Shift-click group selection; centered speed labels; rising ivory/brown italic deposit feedback with Alegreya/OFL; movement-paced two-picture walk/carry with immediate velocity and neutral idle; authored sprite color preservation. Feedback covers all current unloading paths and uses current terrain heights. Ghost opacity, current calibrated buildings/plots, terrain occlusion and cloud camera/input/domain behavior preserved. Small window module keeps client files below 1,000 lines. Two equivalent old occupancy lint simplifications included.

## Verification

92 workspace tests pass; fmt, strict native/WASM Clippy, release native/WASM builds and diff check pass. Browser WebGL2 verified Shift-group/shared gathering/deposits, movement, navigation, time labels and rejected translucent ghost without console errors. Mac locked, so new native window inspection unavailable; native binary builds. Review: docs/NATIVE_FIXES_REVIEW.md. Rebuilt tracked web/pkg matches integrated source.

Prior native-fix session used /tmp/age-of-agents-integration-20261003.db. This art session uses /tmp/aoa-placement.db on :8000. Prior dedicated fa6cd43 DB preserved. Native binary target/release/age-of-agents-client rebuilt; bundle not refreshed because running-app inspection was blocked by lock.

## Remaining / next action

Push the building-art correction after combined verification; production push triggers Modal deploy. Additive touch, box selection and DOM accessibility remain gaps. Explicit blocking/non-blocking task classification remains absent (valid replacement orders already work); remote direction-hold branch remains separate. Older expanded prototype is preserved separately, not part of this push.

## Current building-placement work

- Fixed orthographic camera, rectangular claims, Grid/G toggle, placement ghosts and exact cobblestone pads remain in place. Touching plots share one level foundation height.
- Recreated house and granary artwork (all four construction stages) with consistent upright walls and symmetric bases. Source sheet: `assets/sprites/building_sources/orthogonal_house_granary.png`; generation provenance and reviewed structural corners are checked in. Original cutouts are retained. The existing packer crops the new sheet and packs it alongside the unchanged tower/dock frames, without geometric warping. No renderer or game-rule changes.
- Verification: 92 workspace tests after integrating 97a04d7, native/WASM clippy and formatting pass. Desktop WebGL selection and grid checks also pass on the combined version; visually reviewed adjacent completed houses against independent footprint overlays. New art has clean visible edges at gameplay scale. DPR-2 mobile Grid, ghost drag/release and snapped foundation placement pass with no page errors.
- Thermonuclear review: asset-only rendering correction, existing packer extended directly, no new dependencies or runtime transforms. Geometry and plot-containment tests continue to pass; docs reflect the actual replacement scope (house/granary).
- Deployment: 97a04d7 succeeded; this verified art correction is ready for default-branch master push and production workflow.
