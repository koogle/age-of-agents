# Greek transport ship

`draft.png`: FAL `fal-ai/nano-banana/edit`, approved style reference
`assets/reference/diorama_primary.webp`. Exact prompt in `request.json`.

`refined.png`: OpenAI image_gen refinement of the draft, output
`exec-2805c8bd-93a8-4b8a-b949-f74af627dca3`; requested cleaner connected rigging,
coherent planks/crates, preserved sail and palette, and paired opposite views.
The second panel did not give a distinct rear view, so only the first panel was
retained as `front.png` (768×1024 source). No low-resolution enlargement.

`rear.png`: FAL `fal-ai/nano-banana/edit` using `front.png`, opposite-side view
with the mast in front of the sail; prompt in `rear-request.json`.

Both sources were cut out with `fal-ai/birefnet/v2`. Request IDs and estimated
FAL costs ($0.10 total) are retained in `ledger.jsonl`. Original renders and
cutouts are retained. `scripts/pack_transport_sprites.py` crops transparent
margins and downsamples into two registered 512×512 RGBA cells (1024×512 atlas).
Views are mirrored for the other two heading quadrants; stationary ships retain
the last heading. This slice has no distinct rowing/loading animation.
