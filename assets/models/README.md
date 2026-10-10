# Generated 3D building models (2026-10-10 spike)

`towncenter`, `house`, `watchtower` and `monument` are `fal-ai/trellis` meshes made
from the painted sprite frames (request ids and the fit procedure are in
[the shadow spike](../../docs/verification/shadow-spike/README.md)). Each `.bin`
holds `u32 vertex_count, u32 index_count`, then interleaved `f32` position,
normal and uv per vertex, then `u32` indices; `.png` is the building's own sprite
frame, and each vertex's UV is where it lands in that frame at the game view, so
the visible surfaces show the painting exactly (`tools/convert_trellis.py`);
`models.json` records extents and the mesh azimuth that matches the sprite. The client draws them on the plot in place of the completed
building's sprite (`crates/client/src/render/models.rs`), with the mesh also
flattened along the shadow sun for its cast shadow. Meshes are unsimplified
(15k to 80k triangles). This is a spike for
review, not an accepted direction.

# Archived 3D model experiments

The Three.js client, runtime GLBs, and their optimization scripts were retired on 2026-10-03. The current Rust renderer uses painted sprite sheets under `assets/sprites/`. Concept images, the generation ledger, and the historical notes below are retained as provenance; commands and client references below describe the retired implementation.

# Generated 3D models

The client picks generated models with `?glb=` (`frontend/models.js`): a comma-separated subset of `villager,towncenter,cypress`, `all`, or `none`. **Default: `towncenter` only**, the one model that reads better than its procedural counterpart at gameplay zoom. Loaders (`frontend/vendor/GLTFLoader.js`, `meshopt_decoder.js`, `SkeletonUtils.js`) and GLBs are fetched only for the selected models. If loading fails, the client warns once and uses the procedural models.

| Model | Bytes | Triangles | Default | Verdict at gameplay zoom |
| --- | --- | --- | --- | --- |
| `towncenter.glb` | 126,484 | 5,483 | on | Clearly better: reads instantly as a marble temple and matches the diorama reference |
| `cypress.glb` | 21,940 | 1,444 | off | About equal; nicer clumped foliage up close |
| `villager.glb` | 275,516 | 5,094 | off | Less clear: slimmer, cream tunic on sand, no per-villager colour or team marker |

## `villager.glb` (275,516 bytes)

A rigged, animated villager generated with fal.ai on 2026-10-01/02. Loaded only with `?glb=villager` (or `all`); the procedural villager is the default. In the diorama look it uses the shared `paint()` matte material with its texture and no outline.

| Property | Value |
| --- | --- |
| Triangles | 5,094 (simplified from 7,280) |
| Skeleton | 24 joints (Meshy humanoid: Hips, Spine02/01/Spine, neck, Head, arms, legs) |
| Texture | one 512×512 JPEG base colour (q86) |
| Clips | `idle` (Meshy 0 Idle), `walk` (613 Casual_Walk_inplace), `hammer` (128 Heavy_Hammer_Swing), `forage` (278 Female_Stand_Pick_Fruit_Basket), `dig` (283 Pull_Radish) |
| Extensions | `EXT_meshopt_compression`, `KHR_mesh_quantization` |
| Orientation | Faces +Z, feet at the origin after the client's floor offset; the client scales it to 0.78 world units tall |

The game maps activities to clips as idle→idle, walk→walk, chop/mine/build→hammer, dig→dig, forage→forage. Carrying uses the plain walk because cargo rides on the back. The procedural tools attach to the `RightHand` bone and the cargo to the `Spine` bone.

## Pipeline

1. **Concept** (`villager_concept.jpg`): `fal-ai/nano-banana`, 3:4. Prompt:
   > Full-body character turnaround-style concept art for a low-poly 3D game model: one young Mediterranean villager, short dark-brown hair, warm olive skin, knee-length cream linen tunic with a simple terracotta-red cloth band across the chest, brown leather belt, bare legs, brown leather sandals. Standing straight in a relaxed A-pose, arms held down and angled about 30 degrees away from the body, hands open and empty, feet slightly apart. Front three-quarter view, the entire body visible from head to feet, centered. Simple sturdy readable shapes, slightly stylised adult proportions (about 6 heads tall), drawn as a thin-line European comic in the style of Moebius crossed with Studio Ghibli: fine dark-brown ink outlines, flat cel colors, even soft frontal lighting, no strong shadows. Plain flat light-grey background, no ground, no props, no text.

   Two alternatives were rejected: a nano-banana/edit variant that read as a child, and a nano-banana-pro variant whose drawing was stiffer.
2. **Background removal**: `fal-ai/birefnet/v2`, then cropped and padded to a white square.
3. **Image to 3D**: `tripo3d/p2/image-to-3d` with `face_limit: 7000`, `texture: true`, `pbr: false`, `texture_quality: standard`, `model_seed: 7`, `texture_seed: 7`. Output: 7,280 triangles, 2048² texture, 1.1 MB (`villager_tripo_preview.png` is Tripo's render). The Meshy v7.1 comparison was not run: `model_type: lowpoly` is rejected for Meshy 7 (HTTP 422, unbilled), and Tripo's result was already good.
4. **Re-orient** (`tools/face_z.mjs`): Tripo exports the character facing +X. Rigging that mesh as-is put the hip joints at belt height and ran the legs diagonally, so every clip was badly distorted. Baking a −90° Y rotation (face +Z) before rigging fixed it.
5. **Rigging**: `fal-ai/meshy/rigging/multi-animation`, `height_meters: 1.7`, `animation_action_ids: [0, 613, 611, 128, 237, 278, 283]`, all in **one** call. Separate rigging calls produce different skeletons, so clips from different calls cannot be merged. 611 (carry) and 237 (Charged_Axe_Chop) were generated but left out: the carry clip holds both arms forward for an object in front, and the chop clip mostly holds the axe out in a charge pose that doesn't read as chopping.
6. **Merge and optimize** (`tools/build_villager.mjs`, gltf-transform 4 + meshoptimizer): merge clips by bone name onto the idle file's rig; pin the hips' horizontal translation to the idle rest pose (the hammer clip lunged ~35 cm); drop channels that hold the node's rest value for the whole clip (scale and most bone translations, lossless); resize the texture to 512; `dedup`, `prune`, `weld`, `simplify` (ratio 0.7, error 0.002), `resample` (tolerance 5e-4), `sparse`, `quantize` (position 14, normal 8, texcoord 12, weight 8 bits), `meshopt` (high).
   `node build_villager.mjs <dir with anim_*.glb> villager.glb 0.7 512` with `RESAMPLE_TOL=5e-4`.

## Verification

- `gltf-transform validate` on the raw Meshy output: no errors.
- Skeleton overlay on the bind-pose mesh: joints sit at hips, knees, ankles, shoulders, and elbows. A manual 45–60° knee and elbow bend from rest deforms cleanly.
- All clips share identical inverse bind matrices.
- In-game, headless Chromium (SwiftShader), at the default camera distance and the closest zoom, desktop 1280×800 and phone 390×844: select → gather wood → walk → hammer swing → deposit, with no page errors. With `?glb=none` the loaders and GLBs are never fetched.

## Cost

About $1.33 for this model: concepts $0.23 (two nano-banana, one nano-banana-pro), BiRefNet <$0.01, Tripo P2 about $0.40 (billed in credits; the per-call amount couldn't be read with this API key), four Meshy rigging calls $0.32 (two were wasted on the mis-oriented mesh and one on a separate work-clip rig before the single-call rule was found).

## Verdict and limitations

It fits the Mediterranean ink-comic direction far better than the block villager and holds identity across all clips. At gameplay zoom, though, it reads **less clearly** than the procedural villager: the cream tunic is low-contrast on sandy ground, the figure is slimmer, and the thin skinned outline is lighter than the block villager's ink. So it stays opt-in.

- One texture: no per-villager tunic/skin/hair variety, and no team-colour marker (the procedural villager has a blue scarf).
- The idle clip has a ~40° contrapposto hip turn and a weight shift that can look like a half step.
- The hammer swing twists the torso by up to ~90° mid-swing.
- The dig clip (Pull_Radish) is a crouch-and-pull, so the shovel points into the ground.
- The outline follows the bind-pose normals, so it thins slightly on strongly bent joints.
- Tripo's knee-length tunic is skinned to the thighs, so it stretches a little in wide strides.

## `towncenter.glb` and `cypress.glb` (diorama look)

Made on 2026-10-02 for the soft matte tilt-shift diorama direction (`assets/reference/diorama_primary.webp`).

1. **Concepts** (`towncenter_concept.jpg`, `cypress_concept.jpg`): `fal-ai/nano-banana/edit` with the diorama reference attached (768²) as a style reference, two of each; picked the temple with the fully visible stepped base and the cleaner cypress. The prompts start with "Use the attached image only as a STYLE reference: soft matte 3D miniature tabletop diorama look, rounded slightly chunky shapes, smooth matte painted surfaces, warm sunlight, no outlines." and then ask for, respectively:
   - Temple: "a small ancient Greek temple-style town hall, square footprint, white marble colonnade of six columns on each side around a cella, stepped marble base, warm terracotta tiled gable roof with pediments, simple and readable, low detail".
   - Cypress: "a single tall slender Mediterranean cypress, dark green flame-shaped crown made of soft rounded foliage clumps, short brown trunk, simple and readable, low detail".

   Both on a plain light-grey background, no ground, no text.
2. **Cutout**: `fal-ai/birefnet/v2`, then padded onto a white square.
3. **Image to 3D**: `tripo3d/p2/image-to-3d`, `texture_quality: standard`, `pbr: false`, seeds 7. `face_limit` was 6000 for the temple and 1500 for the cypress.
4. **Optimize** (`tools/build_static.mjs`: `node build_static.mjs in.glb out.glb <simplify ratio> <texture px> <turn deg> <remap>`): bake an optional Y turn, downsize the texture, `dedup`, `prune`, `weld`, `simplify`, `quantize`, `meshopt`.
   - Temple: ratio 1, 512², turn 0 (the pediment entrance already faces +Z), remap `marble`.
   - Cypress: ratio 0.5, 256², turn 0, remap `foliage`.
   - **Remap:** Tripo bakes dull colours (marble around RGB 149/134/123, foliage around 73/85/20). The remap puts each texel onto the procedural palette and keeps its relative shading: neutral stone becomes `#F6F1E4`, green leaves become `#34743E`. Terracotta gets a 1.15× lift and the trunk is left as is.

Integration (`frontend/models.js`):
- **Temple:** scaled to 1.8 world units across and grounded. While under construction, the procedural plinth, rising walls, scaffold and roof still show the stages; the temple appears when the building completes. The procedural team flag tower stays; its red cap peeks over the ridge like an acroterion. The yard props (barrels, crate) are dropped because they sank into the temple's steps. The build ghost shows the temple.
- **Cypress:** replaces the conifer branch of the procedural `tree()` (wood nodes outside the `heath` biome), scaled to 1.1 world units tall.

Costs: about $0.97 (four concepts $0.16, two BiRefNet runs, two Tripo P2 runs at about $0.40 each).

Limitations:
- The temple has no lit-window "working" cue (the procedural windows are hidden once it completes); chimney smoke still rises from the roof.
- All cypresses share one silhouette apart from the random rotation and scale variation.

## Total FAL spend

About $3.32 for this branch (UI kit, coin buttons, and the three models), estimated from fal's published unit prices. The per-request ledger is `tools/ledger.jsonl`.
