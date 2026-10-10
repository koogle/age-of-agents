# Shadow spike: 3D casters and image-model shadows (2026-10-10)

Experiment for cast shadows that follow roofs, towers and statues, run on four
buildings: town center, house, watchtower and monument. Nothing here is integrated
into the client; the runtime still uses footprint boxes and sprite cards.

## Method

- **3D caster:** each 512 px sprite frame went to `fal-ai/trellis` (single image to
  mesh). `project_mesh_shadow.py` finds the camera azimuth whose orthographic
  silhouette best matches the sprite alpha at the game pitch, projects the mesh
  along the game sun (0.15 toward camera, 0.9 up, 1.0 left) onto the ground plane,
  and composites the mask under the original sprite. Output: `*-mesh.png`.
- **Image model:** the sprite, padded onto a 1024 px white canvas, went to
  `fal-ai/nano-banana/edit` asking for only the cast shadow with the building
  unchanged. A blue-grey mask was extracted and composited under the original
  sprite. Output: `*-image.png`; `img_raw_sheet.png` holds the raw model outputs.
- `comparison.png` shows both methods side by side. `generate.py` is the FAL driver;
  both scripts hold scratch paths from the session and need a path edit to rerun.

## Results

| Building | Mesh silhouette fit (IoU) | Notes |
| --- | --- | --- |
| town center | 0.84 | gable and colonnade read in the mesh shadow |
| house | 0.95 | clean box-like shadow with roof pitch |
| watchtower | 0.85 | tall tower silhouette, roof cone visible |
| monument | 0.71 | statue and plinth silhouette; azimuth fit weakest |

The image model also produced convincing shadows for all four, including the
statue, but picks its own sun (lower, longer shadows) and redraws the building
slightly, so masks need alignment and the direction is not exactly controllable.

## Provenance

- `shadow-spike-3d-house`: request `01a12784-d610-7500-986f-c3d26e6a83cc`
- `shadow-spike-3d-monument`: request `01a12784-d523-7b73-ac29-fd571940349c`
- `shadow-spike-3d-towncenter`: request `01a12784-d55e-7622-98e2-8c7bbee0699c`
- `shadow-spike-3d-watchtower`: request `01a12784-d53a-7152-979f-f40ae2c2dc8e`
- `shadow-spike-img-house`: request `01a12786-2b98-7a82-8ba5-456ce7b99770`
- `shadow-spike-img-monument`: request `01a12786-2b63-77d0-9911-ed456546af39`
- `shadow-spike-img-towncenter`: request `01a12786-2b96-7fc3-8dba-437790ea2087`
- `shadow-spike-img-watchtower`: request `01a12786-2a5e-7910-9629-dca67873d7ec`

Estimated spend: $1.72 (4 Trellis runs, 8 edits; the first 4 edits used
unpadded frames and clipped the shadows). Meshes (0.9 to 2.4 MB GLB each) are not
stored in the repository; the FAL result URLs are in the ledger entries.

## Integration path (proposed, not implemented)

Because the sun is fixed, either source can be baked offline into one ground-plane
shadow mask per building, drawn by the existing shadow pass as a draped decal
aligned to the footprint. No mesh loader or runtime 3D is needed.
