# Wolves and bears review (2026-10-05)

The user requested dangerous animals instead of timed events, then suggested a
bear. PR #98 replaces the withdrawn drought with territorial wolves and bears.
Timing-based world events are absent from the final implementation.

## Behavior and maintainability review

One focused domain module owns animal generation, typed explicit hunting,
territorial pursuit, contact damage and death cleanup. Animals use the existing
occupancy and PathTree rules, including reserved step targets and no corner
cutting. Friendly units acquire no autonomous retaliation or task selection.

The initial balance gives a wolf 40 HP / 8 damage and a bear 100 HP / 16 damage.
Bears are slower and have a shorter pursuit range. Players can retreat or order
several units to attack; a guard can defeat a bear. These are provisional values.

Health, cooldowns, positions and orders are serialized; phases are not an event
system. Group rejection is atomic. Dead units release claims/reservations and
housing once; their carried load is lost, while stockpiles and paid building work
remain. Passengers retain health and are not attacked at sea. Dead animals clear
all hunting orders. Live animals are sent only under current fog visibility.

Store version 13 resets incompatible saves under the accepted no-migrations
policy, including version 11 and the withdrawn draft's version 12. Corrupt current
states return errors, including missing terrain, instead of panicking/resetting.

No new dependencies, general combat engine, factions, raids, weather, ranged
attacks, healing, building damage, respawns or loot. The new client interaction
module keeps the main client below 1,000 lines. The existing DOM accessibility
gap remains. Reverse-facing and dedicated animal attack sprites remain art work;
this slice has authored idle/walk frames, mirrored as needed, with measured feet
anchors. Friendly attacks reuse existing work/military action sprites.

## Evidence

237 Rust tests passed: 13 server, 76 client, 148 domain; one existing manual
benchmark remains ignored. Ten focused wildlife regressions cover deterministic
safe spawning, visibility/atomic rejection, attacks and cleanup, cargo loss,
pursuit/return, pause/reload, discovery, blocked diagonal contact and corruption.
Native/WASM strict lint and formatting pass. The 286-frame asset audit includes
four original 627px wildlife frames; all four have transparent corners. Field,
transport, icon and Python deployment-verifier checks pass.

The [browser driver](../verify_wildlife.py) uses a real isolated SQLite server,
seed 123, Chromium with software WebGL, desktop mouse at 1280×800 and emulated
phone pinch/touch at 390×844 DPR2. Its controlled fixture places two guards and a
wolf/bear near the camera; this does not represent natural spawn placement.
It asserts explicit UI attack orders, damage to both sides and bear defeat, and
records screenshots plus `results.json`. A follow-up desktop run (`results-desktop.json`) verifies the final damage-label de-duplication; its focused regression also passes. A phone close-up precedes zooming out to
bring the guards into its narrower viewport. Physical phones and native desktop
appearance are not covered by emulation.

Asset prompts, original draft, refinement, request/cost provenance and limitations
are retained in [wildlife sources](../../../assets/sprites/wildlife_sources/README.md).

Direct release remains blocked: Modal 1.5.3 status again returned "Could not
connect to the Modal server" with configured credential bindings ready. No
production state was changed.
