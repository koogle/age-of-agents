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

## Wharf-matched refinement (2026-10-04)

The initial wharf-matched atlas replaced the original large transport with the small coastal
boat shown beside the approved wharf. `wharf-refinement.png` is the retained
1774×887 RGBA OpenAI image_gen output `exec-7807202d-85af-4268-883e-1ef45fe188ee`.
References: `building_sources/clean_roofs/dock_complete.png` and the preceding
`transport.png` two-view layout. The edit requested a low open clinker hull,
pale timber with sage-grey bands, simple benches, one plain ivory sail and
slanted yard, simple rigging, matching front/reverse views, and native transparency;
no tall curled ends, cabin, upper decks, striped sail or cargo pile.

The original output is preserved unchanged. Its two 887×887 panels were used as
`front-cutout.png` and `rear-cutout.png`; the existing packer downsamples these
into registered 512px cells. Earlier FAL renders and provenance above are retained
as history. The game renders the boat at 1.6 world units (formerly 2.8), with a
0.4-unit selection ring, matching the dock boat at gameplay zoom.


## Consistent interior layout (2026-10-04)

The current cutouts come from `layout-refinement.png` (1774×887 transparent
RGBA, OpenAI image_gen output `exec-b44fefab-3f7b-4e16-983f-12c3fd096f4c`).
`layout-draft.png` retains the preceding layout correction
(`exec-63e67e0c-689f-4088-b395-b707206bdc77`). Exact prompts and references are in
`layout-request.json` and `layout-refinement-request.json`.

Both views depict **three benches**, **one mast mounted on the middle bench**,
and **one sack on the centerline in the stern bay behind the aft bench**.
The sack appears at the far end in the bow view and near end in the stern view;
it stays in the same boat-relative compartment. The pointed bow and flat stern
identify the views. Future edits must retain this layout, including mirrored
facings. These semantic details are visually checked; the automated checks
verify resolution, transparency and registration, not object counts.

The final sheet is split into two unchanged 887×887 source panels and packed
with the existing script. Earlier sources are retained as provenance.
