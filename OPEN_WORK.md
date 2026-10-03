# Open Work

Updated 2026-10-03. Branch master, including native-fixes commit 97a04d7 (deployed successfully). User authorized pushing the missing recent native fixes. Recovery commit da820bb on codex/native-sprite-rendering preserves all older local fixes, docs, fonts and generated WASM. Historical root and gukaet edits/saves remain untouched.

## Integrated

Shift-click group selection; centered speed labels; rising ivory/brown italic deposit feedback with Alegreya/OFL; movement-paced two-picture walk/carry with immediate velocity and neutral idle; authored sprite color preservation. Feedback covers all current unloading paths and uses current terrain heights. Ghost opacity, current calibrated buildings/plots, terrain occlusion and cloud camera/input/domain behavior preserved. Small window module keeps client files below 1,000 lines. Two equivalent old occupancy lint simplifications included.

- Occlusion: building sprites write per-column depth from the footprint's front edge (`Sprite::footprint`, `building_depth` in billboard.wgsl); a silhouette pass draws hidden villager parts in team blue (depth greater, no depth write).

## Verification

92 workspace tests pass; fmt, strict native/WASM Clippy, release native/WASM builds and diff check pass. Browser WebGL2 verified Shift-group/shared gathering/deposits, movement, navigation, time labels and rejected translucent ghost without console errors. Mac locked, so new native window inspection unavailable; native binary builds. Review: docs/NATIVE_FIXES_REVIEW.md. Rebuilt tracked web/pkg matches integrated source.

Prior native-fix session used /tmp/age-of-agents-integration-20261003.db. This art session uses /tmp/aoa-placement.db on :8000. Prior dedicated fa6cd43 DB preserved. Native binary target/release/age-of-agents-client rebuilt; bundle not refreshed because running-app inspection was blocked by lock.

## Remaining / next action

Push the building-art correction after combined verification; production push triggers Modal deploy. Additive touch, box selection and DOM accessibility remain gaps. Explicit blocking/non-blocking task classification remains absent (valid replacement orders already work); remote direction-hold branch remains separate. Older expanded prototype is preserved separately, not part of this push.

## Current building-placement work

- User rejected the recreated house/granary style and explicitly requested the original houses and style. Restored the actual original FAL house/granary cutouts, including all four construction stages, and their original measured base corners. The packer again uses original individual cutouts for every building. Atlas and metadata match original-art commit ebe2019 byte-for-byte. The recreated sheet is retained but unused; the latest generated style-transfer attempt was rejected and never integrated.
- Fixed camera, uniform unskewed rendering, rectangular claims, Grid/G toggle, placement ghosts and cobblestone pads remain. Original painted perspective is retained: the exact grid claim is expressed by the paving, not a redraw or warp of the original architecture.
- Verification: 92 workspace tests, native/WASM lint and formatting pass. Desktop browser selection/grid checks pass with no page errors; restored original houses visually reviewed at gameplay zoom. DPR-2 phone Grid/menu/ghost drag and snapped foundation placement pass without page errors. No Rust/gameplay changes. Thermonuclear review: removed the experimental-sheet packing branch; no new dependencies or rendering machinery.
- Next: commit/push to master after verification; production workflow deploys the restored original art.
