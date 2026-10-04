# Field preparation follow-up review — 2026-10-03

Reviewed against `docs/THERMONUCLEAR_REVIEW.md`. No code or simulation blockers found; DPR-2 art verification remains incomplete.

Preparation completes by switching only villagers still assigned to that field to the existing typed Gather task. Helpers carrying goods use Returning first, preserving partial-load delivery. The authoritative simulation owns the handoff; the client only selects a new authored sprite sheet. No task chooser, automatic replenishment, new cost path, schema or persistence migration was introduced. Stop and unrelated orders remain untouched, and cell/step claims remain unchanged.

The renderer reuses the existing per-person HD atlas shape, mirroring, timing, walking and cargo priorities, and team silhouettes. Existing view tests moved to their own module to keep `view.rs` below 1,000 lines. No extra rendering dependency or input mode was added. The hoeing cycle has four distinct front-facing frames per appearance; back work directions mirror the front art, following existing work-animation conventions.

## Verification

- Combined latest-master tests: 161 passed (13 server, 46 client, 102 simulation), including automatic harvesting, depletion without replenishment, shared paid progress, stopped workers, and a helper finishing delivery after the field becomes ready. Existing partial-cargo rendering regressions also pass, with field work checked against its new sheet.
- Formatting, whitespace checks, native/all-feature lint and WebGL lint passed. WebGL and native release rebuilds passed.
- New field asset check: 12 distinct 512px frames, original 768×1024 pose source regions, transparent PNG corners, consistent sandal baseline, and no source enlargement. Initial resource-activity and depleted-tree checks passed. After synchronizing with latest master a27a564, the full resolution check passes all 280 frames, including the latest HD resource/unit sheets and these twelve new poses. The cycle also has a strict CI check.
- Generated art matches approved villager masters: identity, blue team scarf, cream clothes, brown belt/sandals, upper-left lighting and fine brown pen lines. Original pose pairs, rejected low-resolution drafts, exact prompts and built-in generation IDs are retained. FAL was unavailable, so the built-in image tool generated and refined this cycle.
- Live isolated browser world: all three appearances inspected, close zoom inspected, preparing worker selection displayed Preparing field, workers harvested and deposited without a new order, then went idle with all plots empty. A new field click charged exactly 10 wood and 5 stone and resumed preparation. Existing saves were untouched.
- Desktop 1280×900 and phone 390×844 layouts inspected. Evidence is in `field-preparation-verification/`. The in-app browser reports DPR 1; DPR-2 inspection was interrupted when the user requested no further local computer use, and no further UI automation was performed.

## Remaining limits

DPR-2 phone inspection is pending. The earlier screenshots predate integration of the latest HD field and unit art; no further computer use was performed after the user prohibited it. Combined checks pass and the feature is deployed to the intended `koogle-frick` production account. Production bootstrap/catalog/preparation assets match and the authoritative state endpoint passes verification; details are recorded in `OPEN_WORK.md`. The stale checkout initially failed to read the newer production schema; recovery uses compatible source without editing or resetting the saved world.
