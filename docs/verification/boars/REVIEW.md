# Boar integration — 2026-10-06

Adds one boar to newly generated first islands and 2–3 to later islands,
retaining the existing wolf/bear roster, territorial combat, safe-start distances,
fog and explicit hunting controls. Existing saves retain their animals.

## Style review — superseded by user rejection

The user said the result is not there yet after the gameplay preview. The
assessments below describe the previous agent review, not current acceptance.
The selected NPC-referenced replacement is documented at the end of this review.

The user's concept is an identity reference. The approved NPC-matched
`npc-style-refined.png` is the direct style reference in both image passes.
The first pass is retained as rejected: too much small fur hatching.
The second pass removes it, using broad muted brown cel shapes instead.

- Ink: pass; fine warm brown contours match the existing animals.
- Color: pass; muted warm brown with ivory tusks, legible on green terrain.
- Shading: pass; broad two-tone cel masses replace dense texture.
- Shape: pass; stocky boar silhouette, snout and tusks distinguish it from bear.
- Camera/scale: pass in source comparison; same elevated right-facing view,
  four consistent poses, authored rear-hoof registration and no source enlargement.
- Alpha: pass in light/green/blue comparison; clean silhouette, no white matte.

See `species-comparison.png` (removed 2026-10-10: iteration evidence; final state kept); the final runtime checks are recorded below.
The packer preserves the eight existing wolf/bear cells byte-for-byte.
Desktop maximum zoom confirms the boar sits on its ground ring and matches the
guard family’s thin brown contours and cel shading (`hunting/desktop-boar-maxzoom.png` (removed 2026-10-10: iteration evidence; final state kept)).

## Code review

The change extends the typed species enum and appends deterministic spawn slots.
It reuses existing authoritative pursuit, contact damage, attacks, death cleanup,
occupancy, fog and save validation. No new commands, dependencies, idle friendly
behavior, framework or persistence migration. The client adds one species-to-row
mapping and keeps shared mouse/touch picking and combat-phase animation.
All modified client files remain below 1,000 lines. No feature removed.

## Verification

294 workspace tests pass (one manual benchmark ignored), including species
combat/serialization, deterministic generation across five seeds and three
discoveries, safe starter placement and renderer manifest/phase matching.
The 307-frame sprite audit, existing-atlas pixel comparison, formatting and
whitespace checks pass. Strict native/WASM lint and rebuilt browser/server pass. Real-server mouse and emulated DPR-2 phone touch boar hunting pass: explicit
attack orders, damage both ways, defeat and cleanup, no browser errors. Normal
and maximum-zoom captures are retained in `hunting/`. Both hunting viewport
results report injury, defeat and no browser errors. All screenshot dimensions
match desktop 1280×800 or phone 780×1688 (390×844 at DPR 2). Not deployed.

The additional pose replay passed desktop windup/strike/recovery, pause and
mirrored UV assertions and captured all desktop maximum-zoom poses. Phone
windup/strike/pause/mirroring assertions completed (the mirrored screenshot is
after those assertions); the first phone close-up was captured. The remaining
redundant phone species close-ups were interrupted because SwiftShader was
extremely slow; the replay did not reach its final results-file write. Do not
claim a fully completed pose replay. The main real-server verifier independently
completed phone maximum-zoom boar captures and touch hunting. The initial software-rendered pose replay timed out
waiting for an updated UV buffer; the verifier now allows 120 seconds and emits
actual UV values plus a screenshot on failure. Physical-device and native-window
appearance are outside browser emulation coverage.

## User-selected replacement — 2026-10-06

Jakob chose the first follow-up study (`boar-npc-study.png`), overriding the
agent’s earlier reservations. The new four-pose family preserves its lighter
taupe coat, broad shading, face, hoof shapes and fine brown lines. A padding
repair keeps every snout inside its cell; the rejected edge draft is retained.
`approved-comparison.png` (removed 2026-10-10: iteration evidence; final state kept) compares the final packed idle pose with unchanged
wolf/bear sprites at source and small sizes on three backgrounds.

Style review against the selected reference: fine brown ink, muted taupe color,
quiet cel shading and consistent identity pass. All four poses keep the same
camera and scale. Transparent margins are clean and planted rear hooves register
to idle. The original 627px cells are packed without enlargement. Existing
wolf/bear pixels and all frame rectangles remain identical.

Fresh checks: 307 sprite-frame audit, focused client wildlife frame/phase test,
formatting and strict native/WASM lint pass. This refinement changes only art and
packing; the 294-test gameplay suite and real mouse/touch hunting evidence above
remain from the earlier implementation. No new simulation or WASM change.

Fresh desktop/phone normal and maximum-zoom art evidence is in `approved/`;
`capture_approved.py` reproduces it using a paused fixture and synthetic zoom.
It is an art preview, not a new touch-combat or physical-device test.
Final capture status is recorded in `approved/results.json`. Not deployed.

## Authorized merge integration

User approved merging on 2026-10-06. Integrated master `806512c`, preserving
300-HP/35-damage wolves, 600-HP/50-damage bears, stationary status feedback,
and updated stone-road/disembark art. Boars retain 60 HP/10 damage as the
lighter threat. No balance regression, save reset or existing feature removal.

Combined 295 workspace tests pass (one manual benchmark ignored), as do native
and WASM strict lint, formatting, rebuilt browser/server, all asset checks,
generated-JS syntax and six deployment-verifier tests. Thermonuclear review:
typed species extension only, deterministic bounded generation, shared existing
combat paths, no dependencies/frameworks or save-model changes; master behavior
is preserved. Combined mouse/touch browser acceptance is recorded separately
in `merged-hunting/results.json` after completion.
