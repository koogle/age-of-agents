# Generated ground shadows

Status: draft for review. Requested by Jakob on 2026-10-11 after reviewing the
Trellis preview at `447dd32`. Target integration baseline: `master` at `8917c74`.
This PR is a design draft, with one rejected image-model calibration. Renderer
changes, usable shadow assets and production rollout are not implemented here.

## Result we want

Buildings, villagers, wildlife, trees, and other solid world objects cast soft,
recognizable shadows in one consistent direction. An image editing model authors
the silhouettes from the actual game appearance. Runtime rendering places those
precomputed masks on the ground; it does not call an image model.

The building or NPC artwork is preserved. The generated image is a separate
shadow-only asset, allowing ground colour and texture to show through. No shadow
is baked into the visible sprite or model texture.

The four Trellis buildings may retain their meshes: shadow generation is based
on a registered capture of the visible object, and both meshes and sprites use
the same ground-mask renderer. Whether to retain those meshes is a separate
visual choice; this proposal does not promote the Trellis experiment to master.

## Why the current preview falls short

The Trellis branch flattens mesh geometry along its light vector, with sharp,
noisy edges. Other casters use projected sprite cards or swept footprint boxes.
Those approximations do not consistently resemble the painted subjects.
`master` still uses small procedural contact blobs for villagers and has no
shared authored cast-shadow atlas. These are different rendering baselines.

The earlier image-editing spike produced promising building shadows but noted
that the model could choose its own light direction and subtly redraw the
building. Its README's "not integrated" statement predates the Trellis runtime
integration. The new pipeline treats registration and direction as acceptance
criteria, not something a prompt is assumed to guarantee.

## Fixed camera and lighting contract

- Source camera: orthographic, yaw 45 degrees, pitch 35.2643897 degrees, matching
  `crates/client/src/camera.rs`. Capture without camera rotation, perspective
  distortion, post-process blur, selection/UI, fog, or an existing cast shadow.
- Keep the object at its real gameplay proportions. Record the output size,
  pixels per world unit, ground origin, and building footprint corners or planted
  feet in the same pixel coordinate system. Capture a Trellis model with its
  actual material and transform; use the registered sprite frame for billboards.
- Choose one world-space sun vector and derive the image-space guide from that
  vector and the camera basis. The desired art direction is high sun from the
  screen's upper left, casting to the right and slightly away from the viewer.
  Current `master` uses world sun `normalize(-11, 15, 7)` in `lib.rs`, while
  the Trellis branch's `shadow_sun()` uses a separate camera-relative vector.
  Those are already inconsistent. Resolve the intended shared vector before
  bulk generation; do not claim an arbitrary prompt angle matches both.
  For height `h` and sun `L`, compute ground displacement
  `d = (-L.x/L.y, 0, -L.z/L.y) * h`, then guide pixel displacement
  `(dot(d, camera_right), -dot(d, camera_up)) * pixels_per_world_unit`.
  Master currently projects approximately `(0.8485, 0.1089) * h` (right/slightly
  down in image coordinates); the Trellis light projects slightly up/right.
  The calibration prompt used `(1, -0.18)` and therefore is not an accepted
  match to either measured contract.
- Supply a padded source capture, a ground-plane/anchor guide, and the approved
  shadow calibration as references. Padding is measured from the maximum
  projected height, leaving transparent gutters around the entire shadow.
- Store the camera/light values with the asset contract. A camera or sun change
  invalidates affected authored masks; it must not silently distort old masks.

## Offline asset pipeline

1. **Inventory.** Enumerate the frames the client actually draws from its manifests
   and runtime mappings. Record source hashes, character variants, orientation,
   animation/action, frame, construction/depletion state, and ground registration.
   Archived experiment sheets and UI icons are excluded.
2. **Capture.** Render or extract the exact source object at the fixed game view.
   Produce a clean colour reference plus the explicit anchor/ground guide. Keep
   scale and position identical between guide and colour reference. For mirrored
   sprites, mirror the subject before generation while retaining the same sun.
3. **Calibrate.** Start with town center, house, watchtower, one villager, cypress,
   and olive. These cover broad footprints, tall roofs, thin legs, and leaf gaps.
   Compare them on meadow, cobblestone, sand, and dark ground at gameplay size.
4. **Generate.** Ask the image editing model for the complete ground shadow only,
   including the portion hidden under the object. Require transparent background,
   no subject remnants, no floor, no text, and a single consistent light vector.
   A mask that is merely the upright sprite rotated sideways is rejected.
5. **Review.** Inspect contact, extent, silhouette, light angle, softness and opacity
   against the reference and nearby accepted shadows. Reject geometric drift,
   cropped tails, floating contact points, doubled legs/roofs, or baked scenery.
   Repair creative shape/lighting defects through the image model.
6. **Pack.** Deterministic tooling performs extraction, registration, padding,
   resampling and atlas packing only. It must not invent shadows, redraw edges,
   add procedural limbs/roof forms, or silently change the sun angle. Keep
   approved source output, prompt, references/hashes, model identifier and request
   ID when available. Record rejected requests and why they failed without
   retaining bulky rejected image sets.
7. **Validate.** Check file signatures, dimensions, transparent borders, nonempty
   masks, atlas bounds/gutters, source hashes, finite registration, and explicit
   coverage. Export one concise coverage report and fail on missing required
   mappings. A technical pass does not replace the visual review.
8. **Integrate and verify.** Load the atlas with other startup assets, select masks
   alongside the visible frame, draw on the terrain, then run the browser and
   animation checks below. No generation service or credentials enter the game.

A generation command must support an explicit family/frame selection, bounded
request count/cost estimate, and resumable request IDs. Resume completed outputs;
never silently resubmit a charged request after a polling timeout. Credentials
come from the existing environment and never enter provenance or logs.

## Calibration performed in this draft

A six-object 1536×1024 reference board was assembled mechanically from current
source frames: town center complete, house complete, watchtower complete,
villager front idle, cypress and olive. Each unchanged source frame was resized
into a 256×256 slot in a 512×512 tile to leave room for cast shadows.
ChatGPT's `image_gen` editing tool was called once with that reference and a
transparent-background request. The exact prompt, source/frame hashes and output
hash are in [calibration provenance](shadow-calibration.json); the tool did not
expose an underlying model version or provider request ID.

**Rejected:** the 1536×1024 RGBA output retained upright subject silhouettes,
introduced broad blur and changed the contact registration. It did have transparent
corners, demonstrating why an alpha/format check alone cannot establish usability.
No masks were accepted, packed or wired into the game. The failed image remains
outside the repository; the failure record is retained rather than adding it to
the approved art library.

The next calibration should process one object at a time with a measured light
and anchor guide, then review a composite under the unchanged subject. Only after
that succeeds should we attempt a multi-pose family. If direct shadow-only editing
keeps leaving subject remnants, test a two-stage edit (add the shadow, then remove
only the subject) with the same registration guide. Neither route is assumed to
work until the output passes the contact/direction checks.

## Asset contract

Use `assets/shadows/manifest.json`, one or more shadow atlas PNGs, retained
approved sources, and `provenance.json`. Each runtime entry is keyed by a stable
visual frame identity, not an entity ID or a filename substring.

Illustrative entry (values are placeholders until measured from the capture):

```json
{
  "schema": 1,
  "camera": {"yaw_degrees": 45, "pitch_degrees": 35.2643897},
  "light_contract": "fixed-sun-v1",
  "frames": {
    "towncenter/complete": {
      "atlas": "buildings.png",
      "rect_px": [0, 0, 512, 512],
      "anchor_px": [128, 320],
      "pixels_per_world_unit": 128,
      "source_sha256": "measured-source-hash",
      "status": "approved"
    }
  }
}
```

Atlas alpha stores authored coverage/softness. Runtime supplies one restrained
cool shadow colour and maximum strength, so image-model colour variation cannot
create a different lighting system for every building. Keep straight/premultiplied
alpha handling explicit and use linear-space compositing. Native and browser must
sample the same masks. Mips need transparent gutters to prevent neighbouring
shadows bleeding into each other; mask resolution is judged at maximum zoom.

Aliases are explicit and reviewed: e.g. an unchanged silhouette may reuse a mask
between idle frames. Do not infer reuse from equal frame counts, colour similarity,
or a shared creature family. Mirrored subject views need independently authored
or reviewed masks under the unchanged world-space sun.

## Coverage and animation

| Family | Required coverage and switching |
| --- | --- |
| Buildings | All 17 buildable kinds plus the sanctuary; every visible construction stage; working states where silhouette changes; all four dock facings. A footprint does not determine roof/tower silhouette. |
| Villagers | Man, woman, elder; front/back and both lateral orientations; idle, walk, carry, gather/build and field preparation poses. Combined worker/resource pictures require one combined shadow or a reviewed split with no doubled resource caster. |
| Trained units | Guard, archer, healer, siege cart; all runtime idle/walk/action frames and orientations. |
| Wildlife and raiders | Wolf, bear, boar, lioness, lion, python, sea serpent, viper, barbarian, chieftain and torchbearer; idle/walk/windup/strike and both runtime facings. Hidden snakes cast no revealing shadow. |
| Resources and scenery | Cypress, olive, stump, berry, stone, gold, iron, clay, fiber; full and depleted states actually drawn. Preserve shared anchors when swapping states. |
| Fields | Bare/preparation/growing/ripe stages: low or absent cast shadow where physically appropriate; do not cast the crop height for empty soil. |
| Water and transport | The floating water-resource jug is an interface representation, so do not give it a solid-object ground shadow. Ships need a separate reviewed water-contact treatment; do not project ground shadows across water or replace the wake with a building mask. |
| Placement/selection | Placement ghosts cast no solid shadow. Selection rings, hostile rings and placement outlines remain separate and keep their existing readability. |

For each animated object, choose sprite and shadow from the same resolved frame,
variant and facing. Use the existing interpolation/playout clock: no second timer,
no gameplay decisions and no simulation state change. Pause, Stop, state changes,
reconnect and 2x simulation must keep both layers synchronized.

Generate pose families together with the approved calibration and adjacent frames
as references, then review a looping composite at normal and slow speed. Normalize
only capture scale/registration; do not compensate for a bad pose by stretching a
shadow. If intermediate shadows visibly pop, regenerate the family. Crossfading
may be considered only after silhouette and anchor consistency are correct.

## Rendering integration

Keep the addition in presentation code. `crates/game`, save format, commands,
movement and picking are unchanged. The source object remains the picking target;
shadows never enlarge selectable bounds or become occupancy claims.

- `assets.rs`: add the manifest and required atlas files to the startup manifest.
  A bad HTTP response, undecodable image, or incompatible manifest must produce
  an actionable asset-loading error rather than indexing arbitrary UVs.
- A small `render/shadows.rs` owns mask resources and batching. A matching view
  helper resolves authored frame keys from the already selected sprite/model.
  Avoid a new scene graph, lighting engine or generalized material framework.
- Per-instance data: world ground anchor, authored scale, atlas rect, mask anchor,
  coverage/strength and receiver policy. Building transforms must use the same
  registration as `buildings::on_plot`; Trellis transforms use the rendered model's
  footprint scale. Never estimate the base from the image bounding box alone.
- Interpret the generated image as a **screen projection of a ground plane**.
  Convert its pixel offsets through the inverse of the fixed camera's ground
  basis, then place in world x/z at the recorded anchor. Do not flatten/project
  the already projected mask a second time.
- Drape a subdivided ground quad over the same terrain/flattened-plot height
  sampling used for the visible ground. A single horizontal quad floats on slopes
  or cuts through terrain. Apply the same distant-world bend, and fade small
  shadows at globe zoom. Use a small bounded depth bias, without modifying picking
  or the terrain height itself.
- Draw after opaque land/plots and before objects. Respect real depth so a mask
  cannot appear through a foreground cliff or building. Clip to eligible receivers;
  exclude ocean, sky, and out-of-map samples. Dock land/water boundaries require
  their own acceptance scene.
- Use receiver visibility and caster eligibility. Unseen casters contribute
  nothing; unseen ground receives nothing. Remembered static buildings may retain
  a muted shadow only while their remembered picture is drawn. Moving/hidden
  NPCs must not leak position across fog boundaries.
- Combine overlapping mask coverage in a dedicated shadow coverage target using
  maximum coverage, then composite once onto lit ground. Repeated ordinary alpha
  blending makes a dense woodline arbitrarily dark; depth-writing masks discard
  partial soft overlap in draw-order-dependent ways. Retain ground depth for the
  receiver check, and verify WebGL2 support for the chosen coverage format/blend.
- Keep scene alpha semantics intact: the existing post-process uses alpha for
  grading weight. The shadow pass must not turn terrain into an ungraded sprite
  or add ink outlines around a translucent shadow.
- Remove the previous contact blob/caster only for an object with an accepted
  replacement. A short-lived draft can retain the existing fallback for uncovered
  keys, with missing coverage visible in tooling. Release requires an explicit
  decision for every caster; silent fallback is not completion.

## Draft milestones and merge gate

1. **Contract and calibration:** agree on direction/length/softness; capture and
   generate the six representative subjects; review composites against the current
   Trellis preview and baseline sprite scene. Keep both model-retention choices
   possible until the visual decision is made.
2. **Vertical slice:** wire one building, villager idle/walk in both facings, and a
   tree through capture → generation → manifest → atlas → ground renderer. Demonstrate
   slopes, pause, fog, overlaps and clean removal of the old caster for these keys.
3. **Coverage:** expand one approved family at a time using the inventory. Track
   construction, depletion, tool/cargo poses, wildlife attacks and ship policy.
   Finish every required mapping or record an explicitly accepted no-shadow case.
4. **Quality and performance:** run the tests and visual matrix below; resolve
   defects and revise assets through the model; document device limits and costs.
5. **Review and release:** open/update the draft PR with current evidence and
   uncovered items. Obtain visual acceptance before removing Draft/merging. Publish
   an isolated preview app and save volume once the vertical slice is verified;
   do not replace the regular production app as part of design review.

## Verification

Focused automated tests should prove behavior, not mirror every struct field:

- Every required runtime frame resolves to a valid mask or reviewed exclusion;
  source hash changes invalidate the mapping; malformed/out-of-atlas entries fail.
- A mirrored NPC still casts along the shared light direction. Sprite and shadow
  switch together on walking, gathering/carrying, attack phase, pause and reconnect.
- Known anchor pixels project to the object's world contact point on flat ground,
  raised plots and slopes. Camera pan/zoom preserves relative registration.
- Ghosts, hidden snakes and culled/unseen objects produce no caster. Removing or
  depleting an object cannot leave a stale shadow instance.
- Overlap is order independent and bounded; the renderer preserves scene alpha.
  Shader validation covers native and WebGL2, including atlas mips and the chosen
  coverage blend operation.

Run the repository's format, workspace tests and native/WASM Clippy gates, rebuild
WebAssembly, run relevant asset validation, and perform the thermonuclear review.

Visual matrix: desktop 1280×800 and emulated phone 390×844 at DPR 2; normal and
maximum zoom; meadow, sand, cobblestone, hills, shoreline and dense woodline;
all building stages; idle/walk/tool/cargo poses; wildlife contact attacks; adjacent
shadows; fog edge; selected and occluded units; pan/zoom and globe view. Capture
before/after with the same seed, camera, tick and object placement. Record what was
actually executed; emulation is not physical-phone acceptance.

Measure startup/download bytes, decoded atlas memory including mips, shadow target
memory, draw calls, CPU instance preparation and GPU/frame time on the same scenes
before and after. Set the device budget from baseline measurements before broad
rollout. No invented performance numbers or untested "mobile ready" claim.

## Risks and decisions to resolve in review

- Image editing is not a physical renderer. Exact camera/light registration and
  temporal coherence require measured guides and human review; prompts alone do
  not make this deterministic.
- The existing paintings contain local shading. Generating a ground shadow cannot
  repair conflicting baked light on the object; do not silently repaint approved
  subjects as part of this work.
- Fixed-light masks support this game's fixed camera/sun. Rotating cameras or a
  day/night light cycle would require additional authored views or another shadow
  system and are outside this draft.
- Retain the four Trellis building meshes versus use the approved sprites: pending
  Jakob's preference. This does not block designing the shared mask contract.
- Ships/water, remembered shadows and animated-pose reuse need explicit visual
  examples before their policies are finalized.

## Sources

- [Rendering/input](../knowledge/rendering-and-input.md)
- [Asset pipeline and style gate](../knowledge/asset-pipeline.md)
- [Verification](../knowledge/verification-and-handoffs.md)
- [Camera](../../crates/client/src/camera.rs),
  [world view](../../crates/client/src/view.rs),
  [renderer](../../crates/client/src/render.rs),
  [world shader](../../crates/client/src/shaders/common.wgsl)
- Trellis experiment: branch `claude/quirky-bell-ah3f5z`, revision `447dd32`;
  `assets/models/README.md`, `crates/client/src/render/models.rs`,
  `docs/verification/shadow-spike/README.md` on that branch.
