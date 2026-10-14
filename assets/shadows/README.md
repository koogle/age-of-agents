# Generated ground shadows (draft)

Status: preview for Jakob's review (2026-10-14), not yet style-approved. Design
background: [generated shadow proposal](https://github.com/koogle/age-of-agents/pull/166).

## What is here

- `tools/generate.py` builds one 1024×1024 canvas per building stage or character
  facing: the sprite frame scaled to 512 px at (128, 320) on flat ground, plus a
  guide pole whose drawn shadow fixes the sun (right and slightly away from the
  viewer, length ≈ height). `fal-ai/nano-banana-pro/edit` returns a black-and-white
  mask of that framing, guided by an approved example pair per family
  (`tools/example_*`). Request IDs are written to `requests.jsonl` before polling,
  so reruns resume instead of paying twice; `reject` frees a key for regeneration.
- `sources/` holds the raw model masks (greyscale), one per key.
- `tools/clean.py` drops the model's noise floor, the object's own pixels (the
  sprite covers them) and the guide pole, then softens edges.
- `tools/pack.py` packs four masks per 256 px cell (one per channel) into
  `assets/sprites/shadows.png` and maps every drawn frame to a mask in
  `shadows.json`. Animation frames share their idle facing's mask; mirrored
  facings have their own (the sun does not mirror). Foundations and crop fields
  cast none.

```bash
python3 assets/shadows/tools/generate.py plan
python3 assets/shadows/tools/generate.py run
python3 assets/shadows/tools/pack.py
```

## Rendering

`crates/client/src/shadows.rs` looks up each drawn sprite's mask;
`shaders/shadow.wgsl` slides the mask quad's corners along the view ray onto the
ground at the sprite's anchor and darkens the ground (after terrain, before sea
and sprites). Overlapping shadows currently stack.

## Known limits

- The model follows the guide pole only roughly; direction and length vary per object.
- Some generations fail (ghost copy of the object, filled background, a borrowed
  example shape); those keys are rejected in `requests.jsonl` and left without a
  shadow until regenerated. `pack.py` lists them as missing.
- No style review yet; this is not a merge-ready art family.
