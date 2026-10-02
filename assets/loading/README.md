# Loading-screen art

Art for the `/play` loading screen (`web/index.html`): a painted title and buildings that rise in four stages while the game downloads. The two runtime files total **306 KB** (`title.webp` 90 KB, `buildings.webp` 216 KB). Nothing here is wired in yet.

| File | Size | Use |
| --- | --- | --- |
| `title.webp` | 1600×279, lossy WebP with alpha | The "Age of Agents" wordmark: carved cream marble letters with a terracotta bevel, fine ink outlines, and an olive sprig under "of". Transparent background. Display it about 500–760 px wide. |
| `buildings.webp` | 1024×1280, lossy WebP with alpha | Five buildings × four stages, in 256 px cells, for display at about 240 px tall. |
| `buildings.json` | | The layout, below. |
| `contact_sheet.jpg` | | The title and every frame on the loading gradient (#f4f0e2 → #cfe5f2) at 240 px. |
| `tools/` | | Generation and processing scripts, plus the FAL ledger. |

## `buildings.json`

```json
{
  "image": "buildings.webp", "size": [1024, 1280], "cell": [256, 256], "anchor": [128, 248],
  "stages": ["foundation", "walls", "roof", "complete"],
  "frames": { "<building>": [[x, y, w, h], [..], [..], [..]] }
}
```

- **Buildings, in row order:**
  - `towncenter`: the existing temple stages: foundation, build33, build66, complete.
  - `house`: whitewashed, terracotta roof, blue door.
  - `granary`: limestone, amphorae and sacks.
  - `watchtower`: round, wooden gallery, blue pennant.
  - `dock`: jetty, boathouse, fishing boat.
- **Anchor:** `anchor` is each building's bottom-centre in every cell. Draw every stage of a building with the anchor on the same point and it grows in place.
- **Scale:** one scale per building, so its stages keep their relative size. The largest stage fills about 236 px of the cell height or 244 px of its width.
- **Intended use:** the progress bar picks the stage (`floor(progress * 4)`), and the screen cycles through the buildings.

## How it was made

- **Title (text route, no compositing):**
  - Generation: `fal-ai/ideogram/v3` (`DESIGN`, `QUALITY`, 1536×512, seeds 41 and 42), asking for exactly "Age of Agents" in carved marble letters with a terracotta edge, ink outlines and an olive sprig. Both renders spelled it correctly. Seed 41 added a stray "™" mark, so seed 42 was used. Two nano-banana/edit candidates in serif type were also made and rejected as less in style.
  - Cutout: `fal-ai/birefnet/v2` turned some near-white marble highlights partly transparent, so they showed as dark smudges on dark backgrounds. `tools/title_fix.py` fills those holes, keeps the real letter counters (the "O" of "of") clear by classifying each hole by colour and alpha, and takes colour from the original render with edges un-composited from the paper background.
  - Final: `fal-ai/esrgan` (2×, anime model) for resolution. A second cutout at 2× is bounded by the repaired 1× alpha (`tools/title_final.py`), which removed ghost leaves the 2× cutout had picked up. Then resized to 1600 wide.
- **Buildings:**
  - Complete stage: `fal-ai/nano-banana/edit` with the temple (`complete`) as the style and camera reference.
  - Earlier stages: edits of each complete image, asking for the same footprint, camera and scale (`tools/bld_gen.py`).
  - Two rerolls: the first house had a baked teal ground shadow, and the first watchtower foundation was a square plate under a round tower; it is now a round stone footing.
  - Then BiRefNet cutouts and `tools/bld_pack.py`: speck removal, a shared per-building scale, and bottom-centre seating. Exported as lossy WebP at quality 82 with alpha quality 90.
- **No text** is in any building image.

## Cost

41 FAL calls, about **$1.09**: 20 nano-banana/edit, 2 Ideogram v3, 18 BiRefNet and 1 ESRGAN. Budget was about $5.

## Limitations

- House and granary foundations are drawn as a dirt patch wider than the finished building, so the footprint shrinks slightly at the walls stage.
- The watchtower foundation ring sits a little left of the cell centre, because the stones and planks beside it widen its bounding box.
- The dock's foundation posts spread wider than the finished jetty.
- The title's marble is slightly smoother than the original render after the 2× upscale.
