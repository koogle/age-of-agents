# Open Work

Updated 2026-10-03. Branch master, including native-fixes commit 97a04d7 (deployed successfully). User authorized pushing the missing recent native fixes. Recovery commit da820bb on codex/native-sprite-rendering preserves all older local fixes, docs, fonts and generated WASM. Historical root and gukaet edits/saves remain untouched.

## Integrated

Shift-click group selection; centered speed labels; rising ivory/brown italic deposit feedback with Alegreya/OFL; movement-paced two-picture walk/carry with immediate velocity and neutral idle; authored sprite color preservation. Feedback covers all current unloading paths and uses current terrain heights. Ghost opacity, current calibrated buildings/plots, terrain occlusion and cloud camera/input/domain behavior preserved. Small window module keeps client files below 1,000 lines. Two equivalent old occupancy lint simplifications included.

- Occlusion: building sprites write per-column depth from the footprint's front edge (`Sprite::footprint`, `building_depth` in billboard.wgsl); a silhouette pass draws hidden villager parts in team blue (depth greater, no depth write).

## Verification

92 workspace tests pass; fmt, strict native/WASM Clippy, release native/WASM builds and diff check pass. Browser WebGL2 verified Shift-group/shared gathering/deposits, movement, navigation, time labels and rejected translucent ghost without console errors. Mac locked, so new native window inspection unavailable; native binary builds. Review: docs/NATIVE_FIXES_REVIEW.md. Rebuilt tracked web/pkg matches integrated source.

Prior native-fix session used /tmp/age-of-agents-integration-20261003.db. This art session uses /tmp/aoa-placement.db on :8000. Prior dedicated fa6cd43 DB preserved. Native binary target/release/age-of-agents-client rebuilt; bundle not refreshed because running-app inspection was blocked by lock.

## Remaining / next action

Push the building-art correction after combined verification; production push triggers Modal deploy. Additive touch, box selection and DOM accessibility remain gaps. Explicit blocking/non-blocking task classification remains absent (valid replacement orders already work). Older expanded prototype is preserved separately, not part of this push.

## Villager direction hold (merged 2026-10-03)

Walking villagers no longer flicker between sprite directions on zigzag grid routes: the heading is smoothed over about a quarter second, the drawn direction flips only about 17 degrees past a boundary and at most every 0.5 s, and a stop must last 0.2 s before the standing or working pose shows (the stride frame freezes meanwhile, so there is no walking in place). Stride-paced walk frames and raw velocity are unchanged. Client tests, clippy and the WASM rebuild pass; not yet eyeballed in a browser.

## Current building-placement work

- Latest direction: make building types easier to recognize while keeping the original illustrated style. Houses render at 0.72 plot fill instead of 0.94 (about 23% smaller), including all construction stages and ghosts. Authoritative 3×3 claims and full cobblestone pads are unchanged.
- Completed granary uses a targeted edit of the original architecture: visible wheat sheaves, open grain sacks, grain basket and wheat emblem above the door. Original cutout retained; new source `granary_grain_complete.png` and its structural base corners/provenance are checked in. Construction art remains original. Town center, tower and dock are unchanged.
- Verification: 92 workspace tests, native/WASM clippy and formatting pass; WebAssembly rebuilt. Desktop Grid/selection/visual review and DPR-2 phone ghost drag/release with exact snapped placement pass without page errors in isolated `/tmp/aoa-identity.db`, with a completed granary beside houses.
- Thermonuclear review: one per-kind visual scale and one completed-stage asset replacement in the existing packer; no dependencies or gameplay changes. Existing containment and stable-camera tests cover every stage.
- Next: commit/push to master after browser review; production workflow deploys automatically.
