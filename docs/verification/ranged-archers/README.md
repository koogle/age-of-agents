# Ranged archers — 2026-10-06

Separate follow-up to stronger wildlife PR #135, requested by Jakob.

Archers approach a clear firing position within four cells and hold there,
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

- 302 workspace tests pass (one existing manual benchmark ignored), including
  seven new ranged-domain regressions and the client bow-pose regression.
- Strict native/WASM Clippy, formatting, rebuilt server and WASM pass.
- Domain coverage: range boundaries, approach/stop, obstacles/corners, moving
  targets, windup reset, river crossings, mixed-group rejection, pause, replay,
  Stop, hidden targets and one-to-five-archer wolf fights.
- Open-ground ranged fixture: one/two archers lose; three/four/five win with
  two/three/four survivors. Existing adjacent-contact outcomes remain covered.
  Water barriers and terrain can change outcomes; no claim applies to every layout.

Browser reproduction after rebuilding: `python3 docs/verification/verify_wildlife.py
--output /tmp/ranged-archers-browser`. The isolated SQLite world uses actual mouse
and emulated touch to attack, followed by an explicit squad command over WebSocket
(additive touch selection remains unimplemented). It requires a stationary archer
with an active shot timer beyond melee distance, captures its paused bow pose,
and checks mutual damage, defeat, order cleanup and zero page errors.
Desktop (1280×800 DPR1) and emulated phone (390×844 DPR2) both pass, with
no page errors. [Observed state](results.json), [desktop firing](desktop-ranged.png),
and [phone firing](phone-ranged.png) retain the result. Capture pixel dimensions
are verified. Physical phones and native-window appearance remain unverified.

WASM SHA256: `05359f7b4cf333d38afb8d3d769bf3baeb915afbac9ca1b1a6a64de4d910e704`.

## Code-quality review

Combat rules stay in the shared Rust domain. One bounded firing-position goal
reuses the existing weighted movement machinery. The short line trace is local
to ranged attacks; rendering follows authoritative timers and cannot deal damage.
There is no new combat engine, idle retaliation, save model or command envelope.
The shared view remains below 1,000 lines. Existing authored bow art is reused;
this changes its firing distance and activation, not its style.
