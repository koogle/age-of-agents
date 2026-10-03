# Villager billboard sprites

Three villagers in the illustrated ink-and-wash style of the UI icon kit, rendered as camera-facing billboards in the shared Rust client. All villager actions, military actions/facings and base resource stages now use lossless 512×512 cells.

| Sheet | Who |
| --- | --- |
| `villager.png` / `.json` | Young man, curly dark hair, knee-length cream tunic |
| `villager_woman.png` / `.json` | Young woman, terracotta headscarf over long dark hair, ankle-length cream dress |
| `villager_elder.png` / `.json` | Older man, grey hair and beard, oatmeal tunic |

All three share the same layout, anchor, frame order and royal-blue team scarf, so a client can pick a sheet per unit and reuse one set of rects.

## Layout (identical in all three `.json` files)

- 512×512 cells on an 8-column grid (4096×2560 atlas). Each animation and facing is a list of `[x, y, w, h]` cell rects in playback order.
- **Anchor `[256, 480]`:** the feet's bottom-centre in every cell. Head-to-feet height is about 352 px for standing frames. Raised tools reach toward the top of the cell; nothing is clipped.
- **Facings:** `front` is three-quarter front (body turned toward viewer-left); `back` is three-quarter back (turned toward viewer-right). Mirror them for the other two diagonals.
- **Animations and frame counts:** idle 2 (front, back), walk 4 (front, back), carry 4 (front, back; brown sack on the shoulder), chop 3, mine 3, forage 2 (crouched, basket), dig 3, build 3 (mallet). The gather and build animations are front only.
- **`fps`:** suggested playback rates per animation.
- **Team colour:** a saturated royal-blue scarf and chest band, which reads on grass and sand at about 56 px (see `villager_contact.jpg`). No shadow is baked in.

## HD recovery and refinement (2026-10-03)

The original FAL/BiRefNet strips are retained in `hd_sources/`, with request IDs,
URLs, dimensions and SHA-256 hashes in `hd_sources/provenance.json`. Before this
migration, repacking those sources at 256 px reproduced all five shipped atlases
pixel-for-pixel. No source regeneration was needed. The originals preserve the
approved identities and animation poses, recovering detail discarded by packing.

OpenAI image_gen refined the guard action strip (consistent spear direction,
separated silhouettes) and cart motion/action strips (fixed camera per facing,
consistent pennants, slender bolts). Reviewed generated sources and BiRefNet
cutouts are retained in `hd_sources/refined/`. White-background intermediate refinements removed colored matte fringes before the final cutout; that directory's ledger records
six cutout requests (including the rejected fringe pass), estimated at $0.006 total. Image-tool usage is separate
from that FAL estimate. Cart idle uses two poses from the matching refined motion
row, so its chassis and pennant stay consistent across idle/walk.

Reproduce all five HD atlases with Pillow, NumPy and SciPy installed:

```bash
python3 scripts/pack_hd_sprites.py
python3 scripts/check_sprite_resolution.py
```

The packer uses original pixels, caps source scaling at 1, retains a common feet
baseline and scale per strip, and fits raised tools/long weapons inside their
cells. The woman's raised pickaxe needed a slightly smaller shared strip scale
to avoid clipping. Villager world height and normalized anchors are unchanged;
resource `unitsPerPixel` maintains world size. No low-resolution atlas is enlarged.
All 264 frames in the strict audit now pass; it also checks decoded PNG dimensions,
RGBA mode, nonempty frames and rectangle bounds. CI runs this gate.

The dedicated idle HD sheet remains in use. Resource variants/scenery and the
legacy combined activity sheets are separate assets outside this audit. Existing
variant carry-back poses remain near-profile; no new directions or playable
combat/healing animations are introduced.

## Original generation pipeline

1. **Identity master** (`villager_master.jpg`): `fal-ai/nano-banana/edit` with the `command_train` icon (style) and the earlier villager concept (character) attached. The red sash becomes a royal-blue scarf and band. Three candidates; the one with the clearest scarf was kept.
2. **One strip per animation and facing:** nano-banana/edit with the master attached as the only reference. The prompt asks for exactly N frames of the same villager side by side on a shared ground line (`tools/strips.py` has the prompts). Generating a whole strip in one image is what keeps identity within an animation; the master keeps it across animations. carry_front (frame 1 lost the sack) and forage (a bush was drawn in) were regenerated once with stricter prompts.
3. **Cutout:** `fal-ai/birefnet/v2` (Heavy, 2048) on each strip.
4. **Pack** (`tools/pack.py`):
   - Drop specks (berries, dirt, ground lines) by connected-component area.
   - Split frames by empty columns.
   - Scale each strip to its own sheet's idle_front:
     - idle, walk and carry: by head-to-feet height.
     - Forage crouch: by height × 0.62.
     - Tool strips: by the geometric mean of the height estimate and the √(blue scarf area) estimate, because raised tools inflate height and poses can hide scarf area. The woman's sheet uses height only (`TOOL_SCALE=height`), since her headscarf covers most of the blue scarf.
   - Put the feet on y = 240, using each strip's median ground line, and centre on the feet.
5. **Review sheet** (`tools/spr_contact.py`): `villager_contact.jpg` shows all three sheets side by side, every frame at in-game size (figure about 56 px) on the meadow texture.

### Variants

- **Masters** (`villager_master_{woman,elder}.jpg`): nano-banana/edit from the base master. The prompt keeps the drawing style, costume family and blue scarf, and asks for a different person of the same village.
- **Strips:** made with the same prompts (`VARIANT=woman|elder python3 tools/strips.py`).
- **Back views:** from front masters alone, the model kept drifting the back strips into side or front views. Back masters (`villager_master_{woman,elder}_back.jpg`) were made first, and the back strips were redone with them and then "rotated" once more by an edit pass that took the strip plus the back master.
- **Rejected approach:** attaching the base villager's back strip as a pose reference made the variant turn into the base villager mid-strip, so it was dropped.

## Verdict

- **Identity:** consistent within each sheet, and the three people read as distinct at about 56 px (headscarf, grey hair). The base villager's face, curly hair, tunic, belt, sandals and blue scarf match in all 34 frames, and the style matches the icon kit.
- **Variant back views:** the woman's and elder's idle and walk backs show the back. Their **carry_back** strips stayed near-profile (walking right, face hidden or in profile) after three tries, so they read as walking away sideways rather than a true three-quarter back.
- **Motion:** the base villager's walk_front, walk_back and carry_front were regenerated (revision 2) with an explicit low, natural stride (feet near the ground, no knee lift, sack held in every frame). The earlier knee-lift "hop" and carry_front's static stride are gone. Revision 3 gave the woman and elder the same low-stride walk_front, walk_back and carry_front, except the elder's walk_back: its regeneration drifted into a side profile, so his earlier back-turned walk_back (which already had a normal stride) was kept.
- **Facings:** front and back are hard to tell apart at 56 px; the face and scarf knot are the only cues.

## Cost

About **$2.68** in total:
- Base villager: about $0.75 (three masters, thirteen strips including two redos, thirteen BiRefNet runs).
- Two variants: about $1.93 (four masters including the back views, 22 strips, eleven back-view redos, 33 BiRefNet runs). The per-request ledger is `tools/ledger.jsonl`.
- Base villager revision 2 (two walk strips, two carry_front tries, three cutouts): about $0.23.
- Variant stride fix, revision 3 (six strips, five cutouts): about $0.29.

The other images in this folder (`agent_*.png`, `sprite_*.png`, `tile_*.png`, `building_towncenter.png`, `test_bg_removed.png`) are older experiments and are not part of this sheet.

# Resource and tree billboards

`resources.png` (2048×768 RGBA, 256 px cells, 8 columns) and `resources.json`: generated billboards for the gatherable nodes and trees, in the same illustrated style, three-quarter view and upper-left light as the villager. Fine internal pen lines only, with no heavy outer stroke, because the game adds its own one-pixel ink pass. Not wired into the client.

## Layout (`resources.json`)

- `nodes.<name>.stages`: cell rects running from full to nearly empty.
  - Three stages: berry, stone, gold, iron, clay, fiber.
  - One stage: cypress (a pair), olive, stump. The stump is the depleted tree.
- `anchor [128, 248]`: the base bottom-centre in every cell.
- `unitsPerPixel`: world units per sprite pixel, shared by all stages of a node so they shrink honestly. Multiply by 256 for the billboard's world size. The suggested full-stage size is `worldSize`, measured along `measured` (height for the trees and fiber, width for the rest; one cell = 1 unit, villager = 0.78).

## Pipeline

1. **One strip per node** (`tools/res_strips.py`): `fal-ai/nano-banana/edit` draws all stages, or the three tree types, in one image, so they share scale, light and style. References are the villager master (style) and a `mediterranean_4.webp` crop (palette). The prompt asks for fine internal ink lines, two-tone fills, warm light from the upper left and cool teal shadows, with no outer outline, ground line or shadows. The berry strip was regenerated once (the first try drew a tree, and its middle stage didn't look picked).
2. **Cutout:** `fal-ai/birefnet/v2`.
3. **Pack** (`tools/res_pack.py`):
   - Remove drawn ground lines with a vertical-element morphological opening, which keeps thin upright stalks.
   - Drop specks.
   - Split stages by empty columns.
   - Scale each node by one shared factor so its largest stage fits 232 px.
   - Seat each stage on y = 248, centred.
4. **Review** (`tools/res_contact.py`, `resources_contact.jpg`): every stage at its suggested world size (60 px per cell) beside an idle villager, on meadow and prairie.

## Verdict

- **Style:** consistent with the villager and the Mediterranean palette. Nodes are distinct and their silhouettes read at game size; depletion stages shrink visibly for stone, gold, iron, clay and fiber.
- **Revision 2:**
  - Berry was regenerated with large, bold berries and explicit counts per stage (many, three, none), so berries read as red dots at about 60 px per cell and the three stages are clearly different.
  - Gold was regenerated as an irregular, lumpy boulder instead of a cube; its `unitsPerPixel` changed accordingly.
  - Rects are unchanged.
- **Weak spots:**
  - Stone stage 3 is tiny rubble that nearly disappears (kept on purpose).
  - Gold's middle stage shows only a little gold.
  - Fiber's full stage had a small baked shadow, mostly removed by the cutout.

## Cost

About $0.39: eight nano-banana/edit strips including the berry redo, and seven BiRefNet runs. Revision 2 (three berry tries, one gold strip, two cutouts): about $0.18.

# Town center billboard

`towncenter.png` (2560×512 RGBA, five 512 px cells in a row) and `towncenter.json`: the marble temple town center in the same illustrated style, view and light as the villager and resource sheets. Fine internal pen lines, no heavy outer stroke, no baked ground shadow, and a small blue team pennant on the roof ridge.

## Layout (`towncenter.json`)

- **`frames`:** `foundation`, `build33`, `build66`, `complete`, `working`, each a cell rect. `constructionStages` suggests the construction-progress threshold at which each building frame starts (0, 0.15, 0.5); `complete` shows when construction ends, and `working` while the job slot runs.
- **`anchor [256, 344]`:** the footprint centre on the ground, measured as the middle of the flat foundation platform. All five frames share it, so the building grows in place.
- **`baseBottom [256, 470]`:** the lowest point of the front step, if a client prefers bottom anchoring.
- **`unitsPerPixel`:** world units per sprite pixel, the same for every frame. It is set so the complete temple's full sprite width is 1.9 units (2×2 footprint; villager 0.78 tall); the cell is about 2.09 units.

## Pipeline

1. **Complete temple** (`tools/tc_gen.py`): `fal-ai/nano-banana/edit` with the villager master and the stone resource strip as style references, 1024². The prompt asks for a white marble temple on a three-step base with fluted columns, a terracotta gable roof with pediments, a blue pennant, a 45° three-quarter view, and no ground or shadow.
2. **Other stages:** nano-banana/edit of the complete image, asking for the same footprint, camera, scale and position. Foundation is the lowest step as an outline with blocks and planks. build33 is knee-high column stumps on the finished base, with no beams or roof. build66 is full columns, a partial roof frame and scaffolding with ladders. working is the finished temple plus a doorway glow and smoke. build33 and working were each regenerated once: the first build33 already had full columns and beams, and the first working lost the pennant and shrank.
3. **Cutout:** `fal-ai/birefnet/v2`.
4. **Pack** (`tools/tc_pack.py`): the edits keep the complete image's framing (build33, complete and working share the same bounding box to the pixel), so all frames are placed in shared raw coordinates with one scale. The foundation, which the model drew about 9% larger, is rescaled about its bottom centre to the complete temple's width.
5. **Review** (`tools/tc_contact.py`, `towncenter_contact.jpg`): every frame at game size (60 px per cell) with a villager beside it, on meadow and prairie.

## Verdict

- The five stages read clearly at game size and line up on one footprint. The temple matches the villager and resource style and palette.
- **Weak spots:**
  - The working frame's doorway glow is subtle and barely visible at game size.
  - Most of its smoke wisp was lost in the cutout; a client particle or glow would read better.
  - The build66 scaffolding extends a little beyond the base.

## Cost

About $0.30: seven nano-banana/edit images (the complete temple, four stages, two redos) and five BiRefNet runs.

# Idle villagers in HD

`villager_idle_hd.png` (2048×1536 RGBA, 512 px cells) and `villager_idle_hd.json`: the idle frames of all three people at twice the resolution, for the poses on screen most of the time.

- **Layout:** rows are `villager`, `villager_woman`, `villager_elder`; columns are idle front 1, front 2, back 1, back 2. `people.<who>.{front,back}` gives the cell rects.
- **Scale:** `cell [512,512]`, `anchor [256,480]` and `figureHeight 352` are exactly 2× `villager.json`. Feet land on the same pixel after scaling, so the client can swap in the HD cell for idle with the same world size (draw the 512 px cell at the size it draws the 256 px cell).
- **Within limits:** 2048 px per side, the WebGL2 target limit.

## How it was made (no FAL spend)

No upscaler was needed: the original nano-banana strip generations are about 2.1× the HD figure height (figures about 740–760 px tall), so the HD frames are **downsampled from the original generation pixels**, not upscaled. That keeps the real pen lines, faces, hands, sandals and scarf edges, with no upscaler artifacts to clean up.

- `tools/pack.py` gained `HD_SCALE=2`: the same cutouts, speck removal, per-strip scale and feet anchoring as the shipped 1× sheets, at twice the cell size. Sizes and positions are snapped to exactly 2× the rounded 1× values; the 1× output stays byte-identical to the shipped sheets.
- `tools/hd_idle.py` collects the idle cells. As a check, each HD frame downsampled 2× matches the shipped 1× frame (mean alpha difference 0.03–0.04 levels, RGB about 0.3).
- Edge check on a dark background: 0.1% of semi-transparent edge pixels are bright, so there's no light matte fringe.

`villager_idle_hd_contact.jpg` compares the old and new frames at 60 px and 200 px tall on meadow, plus a 1:1 detail crop of the HD frame against the 1× frame stretched 2×.

**Historical note:** this section describes the earlier idle-only upgrade. The full HD recovery above now covers every villager action. If idle sources change, regenerate the dedicated idle sheet with `hd_idle.py` as well.

# Resource variants (round 3)

`resources_variants.png` (2048×1536 RGBA, 256 px cells, 8 columns) and `resources_variants.json`: extra individuals per node so the dense 60×40 clusters (woodlines, berry patches, 2×2 clumps, 3-cell patches) don't look stamped. Same conventions as `resources.json`: base anchor `[128, 248]`, stages from full to nearly empty, and `unitsPerPixel` sized like the base node (same `worldSize` along the same axis).

- `nodes.<name>` is a **list** of variants, each `{stages, unitsPerPixel}`: cypress ×4 (single trees), olive ×3, and berry, stone, gold, iron, clay and fiber ×2 each, with three stages.
- **Use:** per node, pick deterministically by id hash between the base entry in `resources.json` and these variants. Keep the client's per-node scale factor; the variants' `unitsPerPixel` already matches the base node's size convention.
- **Pipeline** (`tools/var_gen.py`, `tools/var_pack.py`):
  - nano-banana/edit with the node's existing strip as the reference, asking for a different individual (wider and lower, or narrower and taller) with the same style, scale and stages.
  - Trees: one strip of distinct cypresses and one of distinct olives.
  - BiRefNet cutout, then the same ground-line removal and stage splitting as `res_pack.py`.
  - One berry strip was regenerated once (it dropped a stage).
- **Limitations:**
  - A few stone, gold and fiber variants keep a faint teal ground tint at their base from the cool-shadow style.
  - One clay variant's full stage is a mound without the pit rim.

# Scenery and effects (round 3)

`scenery.png` (2048×654 RGBA, shelf-packed) and `scenery.json`: `sprites.<name>` gives `rect` [x,y,w,h], `anchor` in sprite pixels, `anchorKind` (`bottom` = base or waterline centre, `center`), a suggested `worldWidth`, and `unitsPerPixel` = worldWidth / rect width.

| Sprite | Use |
| --- | --- |
| `volcano` | Distant backdrop billboard, bottom-anchored on the horizon or far sea; suggested 24 units wide. Fade it with distance fog. |
| `ship_small`, `ship_merchant`, `ship_striped` | Waterline-anchored billboards drifting slowly on the deep sea (1.0–1.5 units long). Mirror them for heading. |
| `cloud_1` … `cloud_4` | Puffy cumulus cards (2.5–5.5 units) floating above the island or sea; centre-anchored. They can also feed the cloud-shadow pass. |
| `fx_wood_chips`, `fx_stone_dust`, `fx_berry_leaves`, `fx_smoke_puff` | Small work particles (0.25–0.4 units): spawn at the work point, scale, rise and fade. The smoke puff is for the working town center's chimney. |

- **Pipeline** (`tools/scen_gen.py`, `tools/scen_pack.py`): nano-banana/edit with `res/trees.png` (style) and `diorama_primary.webp` (palette) as references, one strip per group. BiRefNet cutout, part splitting, a stem trimmed off the smoke puff, then shelf packing.
- **Limitations:** the volcano's summit smoke wisp was lost in the cutout; the cloud cards have a faint ink line along their flat bottoms.

`variety_scenery_contact.jpg` shows mixed clusters on the half-unit grid (base plus variants at about 0.45 units per node, 120 px per unit), the scenery as a composition preview (not to scale), and the effects.

Round 3 cost about $1.02 for 40 FAL calls; the stop icon is included. The per-request ledger is `tools/ledger.jsonl`.

## Roof cleanup, revision 2 (2026-10-02)

The Rust client's completed and working frames now use a simpler terracotta
gable roof with straight eaves, one straight ridge and sparse straight seams.
The dense, irregular curved tile outlines were removed. The working frame keeps
an amber doorway glow and adds no smoke across the roof.

`towncenter_roof_complete.png` and `towncenter_roof_working.png` are the reviewed
source renders from the built-in image tool. FAL was unavailable because this
cloud environment's network policy blocks its host; no FAL generation spend was
incurred for this revision. An earlier atlas-wide candidate was rejected for
irregular tile seams and colored edge artifacts.

Repack the reviewed sources with Node.js and `sharp` available:

```bash
node assets/sprites/tools/tc_roof_pack.cjs \
  assets/sprites/towncenter_roof_complete.png \
  assets/sprites/towncenter_roof_working.png assets/sprites/towncenter.png
```

The packer fits both finished renders to the previous finished frame's bounds
and copies the complete frame's top 260 rows into the working frame, keeping its
roof and pennant identical. The three construction cells are preserved exactly.
The atlas size, frame rectangles, world scale and footprint anchor are unchanged;
`towncenter.json` requires no edit. The legacy client's `towncenter.glb` is unchanged.

Verified: decoded construction pixels unchanged, identical finished roof pixels,
RGBA PNG with transparent corners, gameplay-scale contact sheet, and Chromium
WebGL2 at 1280×800 and 390×844 with no page errors. Town-center selection still
shows the command medallions. Rust workspace tests, formatting and Clippy pass.

## Gameplay building atlas (2026-10-02)

`buildings_hd.png` and `buildings_hd.json` hold house, granary, watchtower, and dock foundation/walls/roof/completed frames in lossless 512 px cells. They are repacked from the original 1024 px FAL cutouts, recovering detail lost when those sources were reduced to the 256 px loading-screen sheet and compressed as lossy WebP. The town center continues to use its dedicated atlas.

Sources are checked in under `building_sources/`; `provenance.json` records the original request IDs. Recovery incurred no new generation spend. Reproduce with `python scripts/pack_building_sprites.py` (Pillow, NumPy and SciPy). One scale and baseline per building preserves construction-stage proportions.

# Catalog buildings and units (2026-10-03)

Sprites for every catalog entry that had no art yet, in the same illustrated style, camera and light as the existing buildings and villagers. Integrated into the shared Rust native/WebGL client through `view/catalog.rs`: buildings show their four construction stages; military units show idle/walk with occlusion silhouettes. Action frames are available for later combat/healing.

## Buildings: `buildings_{economy,crafts,civic}.{png,json}`

Three 2048×2048 lossless RGBA atlases in **exactly the `buildings_hd.json` format**: 512 px cells, `anchor [256, 496]`, `stages` foundation, walls, roof, complete, `frames.<kind>` (four cell rects) and `footprints.<kind>` (four corners per frame). Keys are the snake_case `BuildingKind` names, so a client can merge these maps into the `buildings_hd` lookup.

| Sheet | Buildings |
| --- | --- |
| `buildings_economy` | `mining_camp`, `farm`, `lumber_mill`, `smelter` |
| `buildings_crafts` | `kiln`, `weaver`, `kitchen`, `monument` |
| `buildings_civic` | `barracks`, `range`, `workshop`, `infirmary` |

- **Footprint corners** `[left, front, right, rear]` are estimated automatically from each frame's base outline (front = lowest opaque point, left and right = outermost points in the lower 40%, rear completes the parallelogram), not placed by hand like `building_sources/footprints.json`. Props that stick out (the mining cart, amphorae, the kitchen pergola) widen them a little.
- **Pipeline:**
  - `tools/cat_gen.py`: `fal-ai/nano-banana/edit` draws each completed building with `building_sources/clean_roofs/house_complete.png` and the temple as style and camera references. The three earlier stages are edits of that image (same footprint, camera and scale).
  - Five foundations were regenerated once because they came out already built: farm, lumber mill, kiln, range and monument.
  - Then BiRefNet cutouts and `tools/cat_pack.py`, with the same cell, baseline, fit and one-scale-per-building rule as `scripts/pack_building_sprites.py`.
  - The 1024 px sources (about 19 MB) are not checked in; the ledger lists every request.

## Units: `units.{png,json}`

One 4096×4096 RGBA sheet, 512 px cells, `anchor [256, 480]`, the same conventions as `villager.json`. `units.<kind>` has `figureHeight` (352), `fps` and `animations.<anim>.<facing>` cell rects:

| Unit | idle (front, back) | walk (front, back) | action (front) |
| --- | --- | --- | --- |
| `guard`: hoplite, bronze crested helmet, blue cloak, spear, blue shield | 2, 2 | 4, 4 | 3: spear thrust |
| `archer`: leather cap, quiver, bow, blue scarf | 2, 2 | 4, 4 | 3: nock, draw, release |
| `healer`: older woman, blue mantle, herb satchel, staff | 2, 2 | 4, 4 | 3: kneel, herbs glowing, staff raised glowing |
| `siege_cart`: two-wheeled ballista cart, blue panels and pennant | 2, 2 (parked) | 4, 4 (rolling) | 3: firing a bolt |

Front is three-quarter front, facing viewer-left; back is three-quarter back, facing viewer-right. Mirror for the other two diagonals, as with villagers. The siege cart is scaled to a 400 px-wide body instead of a figure height (bounded to preserve complete silhouettes).

- **Pipeline** (`tools/unit_gen.py`, `tools/unit_pack.py`):
  - A front master and a back master per unit (nano-banana/edit from the villager master), then one strip per animation from the matching master.
  - Rerolls: the guard's walk-back (side profile), the archer's front walk twice (it dropped the bow), the healer's walk-back (lost the staff), and the cart's idle and walk (divider lines, five carts). The guard's walk-back strip came with five frames, so the first four are used, and its drawn ground line is removed.
- **Limitations:**
  - Original guard thrust overlap and cart pennant/orientation defects were corrected in the HD refinement above; the original source strips remain for provenance.
  - Action animations are front-facing only.

`catalog_contact.jpg` shows each completed building next to the existing house, barracks and farm construction stages, and all units at game size (figure about 56 px) and enlarged, with a villager for scale.

**Not generated, because they already exist:** the town center, house, granary, watchtower and dock (`towncenter`, `buildings_hd`), the three villager appearances (`villager*`), and the three ships (`scenery.json`: `ship_small`, `ship_merchant`, `ship_striped`).

**Cost:** 155 FAL calls, about **$3.90** (87 nano-banana/edit, 68 BiRefNet). The ledger is `tools/ledger.jsonl`.

HD integration requirement: all actions/facings/stages must have at least 512×512 authored frame pixels from the high-resolution originals. All audited catalog, unit, villager and base-resource sheets now pass. Run `python3 scripts/check_sprite_resolution.py` (or `--report-only` to inventory migration gaps). Changing DPI metadata or enlarging low-resolution frames does not recover detail.
