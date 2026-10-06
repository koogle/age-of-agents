# Calibration review

Original villager endpoint confirmed: `fal-ai/nano-banana/edit`. Base identity ledger
contains master candidates 0/1/2; the retained master corresponds to named master1
in the generation script, but the original identity-master exact prompt is not retained.
Animation prompt wording is retained in `assets/sprites/tools/strips.py` and adapted here.

- First water attempt rejected: model copied/enlarged the low-resolution runtime subject,
  producing blurred edges and a black background. Retry removes that subject image and
  explicitly requests a new high-resolution illustration from approved style references.
- Wolf: clear species anatomy and NPC-like broad color areas; fur still has more marks than
  the villager. Viable direction candidate, not an approved animation replacement.
- Stone: cooler grey highlights achieved without black joints; pattern needs mirrored-repeat
  review and gameplay scale before runtime acceptance.
- Dock first request rejected by API validation: 2:1 is unsupported. Corrected to 16:9.
  Request `01a1102b-18f2-7502-be2e-fdb5e85453cd`; no output, billable cost unknown.

- Dock pair rejected for orientation: both views show front/side facades, not a true rear. Color matching is improved, but separate front/rear requests now retain each original geometry directly.

- Separate dock requests preserve front/rear geometry. Front has warm brown timber; first separate rear still preserves grey-green walls and FAILS material continuity. A final targeted rear attempt attaches the new front as material-only reference plus approved villager/diorama and original rear geometry.
- Resource batch has clearer material hue separation. Agriculture seedlings are too small at 24px; rations apple highlight and cloth saturation need restraint before any integration. These stay unapproved candidates.
