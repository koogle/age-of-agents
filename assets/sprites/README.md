# Villager billboard sprites

Three villagers in the illustrated ink-and-wash style of the UI icon kit, for camera-facing billboards. Not wired into the client.

| Sheet | Who |
| --- | --- |
| `villager.png` / `.json` | Young man, curly dark hair, knee-length cream tunic |
| `villager_woman.png` / `.json` | Young woman, terracotta headscarf over long dark hair, ankle-length cream dress |
| `villager_elder.png` / `.json` | Older man, grey hair and beard, oatmeal tunic |

All three share the same layout, anchor, frame order and royal-blue team scarf, so a client can pick a sheet per unit and reuse one set of rects.

## Layout (identical in all three `.json` files)

- 256×256 cells on an 8-column grid. Each animation and facing is a list of `[x, y, w, h]` cell rects in playback order.
- **Anchor `[128, 240]`:** the feet's bottom-centre in every cell. Head-to-feet height is about 176 px for standing frames. Raised tools reach toward the top of the cell; nothing is clipped.
- **Facings:** `front` is three-quarter front (body turned toward viewer-left); `back` is three-quarter back (turned toward viewer-right). Mirror them for the other two diagonals.
- **Animations and frame counts:** idle 2 (front, back), walk 4 (front, back), carry 4 (front, back; brown sack on the shoulder), chop 3, mine 3, forage 2 (crouched, basket), dig 3, build 3 (mallet). The gather and build animations are front only.
- **`fps`:** suggested playback rates per animation.
- **Team colour:** a saturated royal-blue scarf and chest band, which reads on grass and sand at about 56 px (see `villager_contact.jpg`). No shadow is baked in.

## Pipeline

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
