# HD sprite recovery and repack

The resolution audit found 183 undersized frames: 102 villager frames, 60 military
frames and 21 base-resource frames. All are now packed in 512×512 lossless RGBA
cells; the strict audit covers 268 frames and reports zero below minimum.

## Sources and refinement

Recovered 60 original FAL/BiRefNet strips using the checked-in request ledger.
Before changing resolution, the recovered strips reproduced the decoded pixels
of all five shipped atlases exactly. Originals, request IDs, dimensions, hashes
and refinement provenance are retained under `assets/sprites/hd_sources/`.

OpenAI image_gen refined three sources: the overlapping/reversed guard spear
sequence, inconsistent cart front/back motion, and malformed cart projectiles.
The cart keeps its pennant and camera orientation across each motion row; idle
uses the same reviewed chassis. A second image-tool pass used white intermediate backgrounds to remove colored matte fringes; BiRefNet restored final transparency. New FAL
usage was six cutout calls, including the rejected fringe pass (estimated $0.006); image-tool usage is separate.

`python3 scripts/pack_hd_sprites.py` reproduces the five atlases offline with
Pillow, NumPy and SciPy. Existing packers retain frame ordering, normalized feet
anchors and a shared scale per strip. Isolated neighboring weapon fragments are removed after splitting frames. Source scaling is capped at 1; this does
not enlarge old atlases. A shared fit bound prevents raised tools and spears from
clipping. Resource pixel-to-world scale preserves node size.

## Integration and review

- The Rust renderer and legacy client read villager atlas dimensions from the
  manifest instead of assuming 2048×1280. Military/resource dimensions were
  already manifest-driven. The tracked WASM is rebuilt.
- The audit now checks decoded PNG dimensions, RGBA mode, nonempty cells and
  rectangle bounds, and runs in deployment CI.
- No simulation, save, command, occupancy or autonomous behavior changes. No new
  runtime dependencies, rendering passes or generalized asset framework. The
  existing packers and typed atlas structures remain the integration points.
- Retained original sources make the pipeline reproducible without another paid
  generation call or expiring remote artifacts. Runtime downloads remain the
  atlases; source strips are not in the client asset manifest.
- Villager sheets are 4096×2560, units 4096×4096 and resources 4096×1536. These
  increase texture memory; browser verification must exercise actual 4K uploads
  on desktop and DPR-2 phone before release.

## Verification

All 152 workspace tests, strict native/WASM Clippy and formatting passed. Existing
activity-frame regressions use manifest dimensions; all villager variants and
resource work poses remain covered. The strict 268-frame audit and existing
resource-activity, depleted-tree and icon checks pass. All 183 repacked frames
have clear cell edges after the raised-pickaxe correction.

Browser verification evidence and drivers are under `/workspace/scratch/hd-sprites/`.
Maximum-zoom desktop and DPR-2 phone checks pass for three villager identities (idle, front/back walking, carry, chop, mine, dig and forage) and all four military types (idle and front/back walking). Actual WebGL2 uploads include 4096×2560 and 4096×4096 textures, with no runtime errors. Reviewed screenshots show clean silhouettes and aligned feet. Final military checks repeat after the fringe cleanup and integration of master `aba4c6d` (selection-ring fixes). A separate actual UI selection → gather flow reaches the partial-cargo gathering phase successfully. The integrated client passes all 44 tests and strict native/WASM lint; no domain code changed during integration.

## Limits

Military action art is packed and reviewed but combat/healing playback remains
unimplemented. Variant villager carry-back poses remain near-profile. The separate
resource-variant/scenery sheets and legacy combined activity assets are outside
this audit. Ship-state art remains dependent on transport requirements. A native
window and physical phone are not available here; browser phone checks use DPR-2
Chromium emulation.

The final integration also preserves custom-field art from master `4255568`. Its four new 512 px field stages bring the combined strict audit to 268 frames; the 183-frame HD migration scope is unchanged. All 152 combined workspace tests and strict native/WASM lint pass; the combined desktop/phone 4K-texture smoke checks also pass.
