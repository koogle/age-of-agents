# Villager billboard sprites

`villager.png` and `villager.json`: one villager in the illustrated ink-and-wash style of the UI icon kit, for camera-facing billboards. Not wired into the client.

## Layout (`villager.json`)

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
   - Scale each strip to the idle_front reference: by head-to-feet height for idle/walk/carry; by height × 0.62 for the forage crouch; for the tool strips, by the geometric mean of the height estimate and the √(blue scarf area) estimate, because raised tools inflate height.
   - Put the feet on y = 240, using each strip's median ground line, and centre on the feet.
5. **Review sheet** (`tools/spr_contact.py`): `villager_contact.jpg` shows every frame at in-game size (figure about 56 px) on the meadow, prairie and beach textures.

## Verdict

- **Identity:** consistent. The face, curly dark hair, cream tunic, belt, sandals and blue scarf match in all 34 frames, and the style matches the icon kit.
- **Motion:** walk_front, walk_back and carry_front were regenerated (revision 2) with an explicit low, natural stride (feet near the ground, no knee lift, sack held in every frame). The earlier knee-lift "hop" and carry_front's static stride are gone. Tool strips now scale by the geometric mean described above, which removed the chop strip's 10% oversize.
- **Facings:** front and back are hard to tell apart at 56 px; the face and scarf knot are the only cues.

## Cost

Revision 2 (two walk strips, two carry_front tries, three cutouts): about $0.23.

About $0.75: three masters, thirteen strips including two redos, and thirteen BiRefNet runs. The per-request ledger is `tools/ledger.jsonl`.

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
- **Weak spots:**
  - Red berries are barely visible at about 60 px per cell; the full bush mostly reads as darker. Berry stages 2 and 3 are close.
  - Stone stage 3 is tiny rubble that nearly disappears.
  - Gold's full stage is an oddly cubic block.
  - Fiber's full stage had a small baked shadow, mostly removed by the cutout.

## Cost

About $0.39: eight nano-banana/edit strips including the berry redo, and seven BiRefNet runs.
