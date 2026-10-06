# Four simpler menu icons — 2026-10-06

User requested slightly simpler Cargo, Gathering, Town and Disembark refinements,
then explicitly requested showing and merging them. Only four existing runtime
PNG paths change. No game commands, costs, unlocks, menu geometry or save data
change. Sources and per-icon findings live in the
[art review](../../../../assets/ui/sources/menu_icons/simplification/STYLE_REVIEW.md).

## Validation

Base: master `79d5899` (preserves roads, refined wildlife and removed build-subcategory title cards). 289 Rust tests pass
(13 server, 95 client, 181 domain; one existing manual benchmark ignored).
Formatting and strict native/all-target/all-feature and WASM Clippy pass. The
client was rebuilt from this combined source; its WASM hash is recorded separately
from the upstream artifact (the Rust source itself is unchanged by this PR). Asset checks cover all 302 world
frames, field/transport checks, icon normalization, four RGBA PNG signatures,
transparent corners and six release-verifier tests. [Checks/hashes](checks.json).

The [before/after comparison](../../../../assets/ui/sources/menu_icons/simplification/comparison.png)
shows original references plus each pair at 128/24/32px on parchment, dark green
and blue. All four retain the fine brown ink/quiet watercolor family, coherent
perspective and distinct silhouettes; unnecessary props and marks are removed.

## Browser evidence

Fresh combined-candidate preview passes all six captures: desktop 1280×800 DPR1
and phone 390×844 DPR2 (780×1688 captured pixels), with no page errors.
Disembark dispatch is asserted in both modes. The existing
`docs/verification/menu_icon_batch.py` loads current code/assets and supplies
controlled snapshots on loopback :8012. Scenarios: `category_town` shows both Town
and Gathering; `command_cargo` shows the dock Cargo action; `command_disembark`
shows the landing action and asserts typed wire dispatch. Run all desktop scenes
then all phone scenes, requesting `"fresh_context": true` for each fixture. Each request contains `name`,
`mode` and `output` and is placed in the worker's `--queue` directory.

The browser check covers presentation and pointer/touch dispatch with all four
candidate PNGs together, not persistent-world authority or physical phones.
Individual PRs replace one independent PNG at a time. This shared evidence is
explicitly a combined preview, not a screenshot of each intermediate master.

## Thermonuclear review

Four direct asset substitutions use unchanged keys and renderer code. Existing
normalization handles optical weight and safe bounds; the small reproducible
packer crops retained source cells and calls that implementation. No runtime
code/dependency, domain behavior, feature loss or new abstraction. Original art,
refinement prompts, rejected pass and previous runtime PNGs are retained.

Each PR must complete the visual/alpha and browser checks before merge. Production
release is the merge-triggered workflow, verified independently of PR merge.
