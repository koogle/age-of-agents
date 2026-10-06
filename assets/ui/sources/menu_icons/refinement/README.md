# Menu style refinement sources

`draft-sheet.png` contains the original FAL drafts. `refined-sheet.png` is the
accepted reference-anchored style pass; `provenance.json` records its exact prompt,
approved reference revision and the two brick calibration prompts. Approved wood
and masonry reference pixels are retained here. `cutout-ledger.jsonl` records
request IDs for the packed icons; the disembark directory records its Light-model
retry after the Heavy-model mask erased the platform.

`rejected-bold-sheet.png` and `rejected-bold-provenance.json` preserve the first,
off-style refinement. Historical `../refinement/refined-sheet.png` references
inside each `rejected/provenance.json` refer to that rejected bold sheet, not the
accepted sheet now at that name. Historical `draft-sheet.png` refers to the FAL
draft sheet here. The first brick calibration is retained as
`rejected-brick-texture.png`; its mottling was rejected. The second is
`brick-calibration.png`. Rejected files are never approved style references.

Final icon files normalize directly from each retained `cutout.png` through
`scripts/normalize_icons.py`. Unit icons normalize from retained `source.png`
frames extracted from the approved units atlas; their provenance names the frame.
