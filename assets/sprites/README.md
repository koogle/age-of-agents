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
