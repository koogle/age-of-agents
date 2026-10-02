# Generated 3D models

## `villager.glb` (275,516 bytes)

A rigged, animated villager generated with fal.ai on 2026-10-01/02. Loaded only with `?villager=glb` (`frontend/models.js`); the procedural villager is the default and the fallback if loading fails.

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
- In-game, headless Chromium (SwiftShader), at the default camera distance and the closest zoom, desktop 1280×800 and phone 390×844: select → gather wood → walk → hammer swing → deposit, with no page errors. Without `?villager=glb` the loaders and GLB are never fetched.

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
