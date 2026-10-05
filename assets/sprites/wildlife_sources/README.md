# Wolf and bear sprites

Integrated atlas: `../wildlife.png`, with four original 627×627 frames in
`../wildlife.json`. Frame order: wolf idle/walk, bear idle/walk. Mirrored to face
left; reverse-facing and dedicated attack poses are not authored in this slice.

`wolf-bear-draft.png` was generated with FAL `fal-ai/nano-banana/edit` using
`assets/reference/diorama_primary.webp`; exact prompt/response are retained in
`wolf-bear-provenance.json`, with request ID/cost in `assets/ui/tools/ledger.jsonl`.
The earlier wolf/boar draft is retained as an unused source: the user requested a
bear before integration, replacing the proposed boar.

`refined.png` is the unmodified OpenAI image_gen refinement of that draft,
generated 2026-10-05: four square cells, consistent wolf/bear identities,
standing and walking poses, corrected anatomy, no text or numbers, transparent
background, painted cel shading and fine ink outlines. Original tool output:
`/workspace/generated_images/exec-b51c3a46-89ac-49c4-a244-206e2e8d496a.png`.
The atlas copies this output without resizing or claiming recovered detail.
The source is 1254×1254 despite a prompt requesting at least 1536×1536; each
627px frame still exceeds the project's 512px minimum. Rendering uses measured per-frame
paw baselines (611/589/541/545 pixels) to keep movement frames grounded. Run `python3 scripts/check_sprite_resolution.py` to audit bounds,
RGBA format, transparency and original cell sizes.
