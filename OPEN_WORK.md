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

## Cost-scaled construction (2026-10-03)

`BuildingKind::build_seconds()` replaces the flat 4 s `BUILD_SECONDS`: total cost × 0.3 s (house 4.5, town center 6, granary 7.5, watchtower/dock 9). All are at least 4 s, so saved foundations stay valid. Client progress bars and construction stages use the per-kind time; legacy frontend divides by 6 (town center only). 94 tests, native/WASM clippy pass; web/pkg rebuilt; `/play` loads without console errors.

## New-island villager freeze (fixed 2026-10-03)

On the hosted game, villagers on a freshly reset island looked frozen when ordered to move. The new world restarts at tick 0 but reuses `villager-1`/`villager-2`, and the async `POST /reset` let old-world snapshots refill the view first, so stale high-tick samples stayed "newest" forever and the camera framed the old base. `WorldView::sync` now starts over (and returns true) when the tick goes backwards; the app then re-frames the new town center. Regression test in view.rs; 96 tests, native/WASM clippy pass; WASM rebuilt; headless browser reset plus WebSocket move verified (camera on new island, villager walks).

## Villager direction hold (merged 2026-10-03)

Walking villagers no longer flicker between sprite directions on zigzag grid routes: the heading is smoothed over about a quarter second, the drawn direction flips only about 17 degrees past a boundary and at most every 0.5 s, and a stop must last 0.2 s before the standing or working pose shows (the stride frame freezes meanwhile, so there is no walking in place). Stride-paced walk frames and raw velocity are unchanged. Client tests, clippy and the WASM rebuild pass; not yet eyeballed in a browser.

## Current building-placement work

- Houses retain the approved smaller size (0.72 plot fill) for walls/roofs, completion and placement ghosts. Initial foundations now use full plot fill (1.0), independently of house size. Authoritative claims and exact cobblestone paving are unchanged.
- Completed granary retains its wheat emblem, wheat sheaves, sacks and basket. Eight non-HQ roof/complete frames use targeted curved-tile line cleanup edits in `building_sources/clean_roofs/`. Original sources retained; HQ untouched. Source corners and generation provenance updated.
- Loading now shares lossless 512 px gameplay atlases, with high-quality canvas sampling and DPR sizing. Old 256 px loading atlas retained as historical, unused art.
- Integrated concurrent master f2bbf32 (villager direction hold and footprint-based sprite depth/silhouettes) and rebuilt WASM. 94 workspace tests, native/WASM clippy, formatting and atlas checks pass. New regression ensures initial house foundations reach plot bounds while completed houses keep 0.72 fill. Desktop gameplay and desktop/DPR-2 phone loading checks pass with no page errors; DPR-2 phone ghost drag/release and snapped foundation placement also pass without page errors, and the full-size foundation was visually checked against the grid.
- Thermonuclear review passed: existing packer/scale logic and shared loading assets, no extra renderer transformations, dependencies or game rules. All client files remain under 1,000 lines.
- Next: push verified correction to master, which triggers production deployment.

## Illustrated paving follow-up

- Replaced the grey, heavy-grout cobblestone with warm cream limestone, sandy joints and delicate brown contours matched to the original house art. Source/provenance in `assets/terrain/cobblestone.provenance.json`.
- FAL nano-banana/edit request 01a10209-fef2-7533-bc0f-2d70b3d3985b returned HTTP 422 content-check rejection, no image. Used OpenAI image_gen fallback. Texture is opaque 1254px RGB PNG.
- Ground shader mirrors the swatch in world space and clamps sampling within each mip level using the existing sampler, joining identical edge pixels without processing/duplicating the ink lines. Geometry, footprints, buildings and placement rules unchanged.
- Integrated remote 782beab first. 96 workspace tests, native/WASM lint, formatting and WebAssembly build pass. Repeated 3×3 texture preview and desktop gameplay reviewed; selection and grid checks pass without page errors. The first browser run caught WebGL's single-sampler-per-texture restriction; manual clamping within the selected mip resolves it while retaining minification filtering. DPR-2 phone ghost drag/release and snapped placement pass with no page errors.
- Thermonuclear review: one texture replacement and localized sampling logic, no new dependencies or rendering paths. Next: push verified correction to master for automatic deployment.
