# Shadow spike: 3D casters and image-model shadows (2026-10-10)

Experiment for cast shadows that follow roofs, towers and statues, run on four
buildings: town center, house, watchtower and monument. Nothing here is integrated
into the client; the runtime still uses footprint boxes and sprite cards.

## Method

- **3D caster:** each 512 px sprite frame went to  (single image to
  mesh).  finds the camera azimuth whose orthographic
  silhouette best matches the sprite alpha at the game pitch, projects the mesh
  along the game sun (0.15 toward camera, 0.9 up, 1.0 left) onto the ground plane,
  and composites the mask under the original sprite. .
- **Image model:** the sprite, padded onto a 1024 px white canvas, went to
   asking for only the cast shadow with the building
  unchanged. A blue-grey mask was extracted and composited under the original
  sprite. ;  holds the raw model outputs.
-  shows both methods side by side.  is the FAL driver.

## Results

| Building | Mesh silhouette fit (IoU) | Notes |
| --- | --- | --- |
| town center | 0.84 | gable and colonnade read in the mesh shadow |
| house | 0.95 | clean box-like shadow with roof pitch |
| watchtower | 0.85 | tall tower silhouette, roof cone visible |
| monument | 0.71 | statue and plinth silhouette; azimuth fit weakest |

The image model also produced convincing shadows for all four, including the
statue, but picks its own sun (lower, longer shadows) and redraws the building
slightly, so masks need alignment and the direction is not controllable exactly.

## Provenance

- : request 
- : request 
- : request 
- : request 
- : request 
- : request 
- : request 
- : request 

Estimated spend: $1.72 (4 Trellis runs, 8 edits; the first 4 edits used
unpadded frames and clipped the shadows). Meshes (0.9 to 2.4 MB GLB each) are not
stored in the repository; the FAL URLs are in the ledger entries.

## Integration path (proposed, not implemented)

Because the sun is fixed, either source can be baked offline into one ground-plane
shadow mask per building, drawn by the existing shadow pass as a draped decal
aligned to the footprint. No mesh loader or runtime 3D is needed.
