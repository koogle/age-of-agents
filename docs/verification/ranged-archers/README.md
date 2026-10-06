# Ranged archers — 2026-10-06

Separate follow-up to stronger wildlife PR #135, requested by Jakob.

Archers approach a clear firing position within six cells and hold there,
shooting for 18 damage per one-second windup. Existing travel/road weighting,
cell claims and traffic handling remain authoritative. Shots can hit moving
animals if both step endpoints are clear and in range. Moving or losing a clear
shot resets the windup; hidden/dead targets cancel orders. Archers do not
retreat automatically. Melee units still need contact.

Buildings, live non-water resources and mountains block shots; water and units
do not. Supercover tracing checks both sides of exact diagonal corners. A mixed
attack order rejects atomically if any member cannot reach its own attack range.
Existing `AttackAnimal` commands and serialized timers suffice; no save reset,
projectile entities, ammunition system, dependencies or new art are introduced.
The existing bow action aims at the animal while the server timer is active.

## Verification

- 304 workspace tests pass (one existing manual benchmark ignored), including
  nine ranged-domain regressions and the client bow-pose regression.
- Strict native/WASM Clippy, formatting, rebuilt server and WASM pass.
- Domain coverage: range boundaries, approach/stop, obstacles/corners, moving
  targets, windup reset, river crossings, mixed-group rejection, pause, replay,
  Stop, hidden targets, actual first-hit distance, bear response beyond proximity
  aggro and one-to-five-archer wolf fights.
- Open-ground ranged fixture: one/two archers lose; three/four/five win. Existing adjacent-contact outcomes remain covered.
  Water barriers and terrain can change outcomes; no claim applies to every layout.

Browser reproduction after rebuilding: `python3 docs/verification/verify_wildlife.py
--output /tmp/ranged-archers-browser`. The isolated SQLite world uses actual mouse
and emulated touch to attack, followed by an explicit squad command over WebSocket
(additive touch selection remains unimplemented). It requires a stationary archer
with an active shot timer beyond melee distance, then requires actual damage
before the nearest archer is within three cells. Desktop and phone both measured
the first hit at 3.5 cells (wolf HP 282), with four survivors. Captures show
[first damage on desktop](desktop-combat.png) and [phone](phone-combat.png).
Mutual damage, defeat, order cleanup and zero page errors also pass.
Desktop (1280×800 DPR1) and emulated phone (390×844 DPR2) both pass, with
no page errors. [Observed state](results.json), [desktop firing](desktop-ranged.png),
and [phone firing](phone-ranged.png) retain the result. Capture pixel dimensions
are verified. Physical phones and native-window appearance remain unverified.

WASM SHA256: `f26b95c13b97b4bf54e69b10df8988159af07e0eb0b3f28b9217ce479e6b67e5`.

## Code-quality review

Combat rules stay in the shared Rust domain. One bounded firing-position goal
reuses the existing weighted movement machinery. The short line trace is local
to ranged attacks; rendering follows authoritative timers and cannot deal damage.
There is no new combat engine, idle retaliation, save model or command envelope.
The shared view remains below 1,000 lines. Existing authored bow art is reused;
this changes its firing distance and activation, not its style.

## Recorded examples

[Looping GIF](ranged-examples.gif) and [MP4](ranged-examples.mp4) show a lone
archer losing, followed by four archers defeating a full-health wolf; the current recording summary gives survivors. Actual local browser capture at normal simulation speed; the title
labels identify examples. These are controlled encounters, not production footage.
[Recording summary](recording-summary.json) records outcomes and capture settings.

Reproduce clips with `python3 docs/verification/record_ranged_archers.py --output
/tmp/ranged-clips` after building the server/client. Install Playwright's recording
encoder with `python3 -m playwright install ffmpeg`; system ffmpeg handles export.
The recorder pauses only after the outcome and allows the rendered snapshot to
settle before closing. Concatenate the two MP4s with ffmpeg's concat filter; GIF
export uses 10fps, width 800, a 128-color palette and Bayer dithering. No gameplay
code changed for the recording.

The original four-cell recording was rejected because the first windup did not
prove earlier damage. The six-cell correction keeps wolves closing the gap and
preserves explicit retreat. Wildlife responds to ordered attackers within its
territory, even outside ordinary aggro range; no new persisted target is needed.
