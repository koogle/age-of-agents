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
- **Motion:** weaker.
  - Walk frame 2 (front and back) is an exaggerated knee lift that reads like a hop.
  - carry_front frames 2–4 barely change stride.
  - The chop strip is about 10% larger than idle; its scale comes from the scarf landmark.
- **Facings:** front and back are hard to tell apart at 56 px; the face and scarf knot are the only cues.

## Cost

About **$2.68** in total:
- Base villager: about $0.75 (three masters, thirteen strips including two redos, thirteen BiRefNet runs).
- Two variants: about $1.93 (four masters including the back views, 22 strips, eleven back-view redos, 33 BiRefNet runs). The per-request ledger is `tools/ledger.jsonl`.

The other images in this folder (`agent_*.png`, `sprite_*.png`, `tile_*.png`, `building_towncenter.png`, `test_bg_removed.png`) are older experiments and are not part of this sheet.
