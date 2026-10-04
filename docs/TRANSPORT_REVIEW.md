# Local transport review

A completed dock can queue a 60-wood/20-timber transport, produced after 20
seconds when a water berth is free. Ships carry four existing units plus 200
goods, sail only on water, return to a clicked dock, and land at clear shores.
Stockpile goods transfers require a completed dock. No destination islands or
local settlement inventories are claimed in this slice.

## Structure and invariants

- `crates/game/src/game/ships.rs` owns authoritative water routing, manifests,
  boarding seats, landing and transfer validation. It reuses `PathTree` and the
  existing building task queue; no new engine, dependencies or autonomous jobs.
- Typed commands use the existing clone/validate/commit boundary. Payment occurs
  once at queue submission; blocked output waits and queued cancellation refunds.
- Land units and aboard units have exclusive ownership. Boarding clears the old
  action but preserves identity/cargo, and releases land occupancy. A landing
  computes all free cells before moving the manifest. Housing counts passengers.
- Water cells, steps and destinations are reserved against other ships. Stop
  finishes the current step. Reassigning departure cancels unfinished boarding.
  Boarding routes start from the reserved step endpoint to avoid oscillation.
- Old saves default to an empty ship list, and dock products refresh from the
  authoritative recipe. Invalid manifests fail validation; persistence paths
  and the existing database schema are unchanged.
- Client controls only emit commands. One tap path handles mouse and touch;
  generated ship art supplies both world sprites and the production portrait.
  Cargo buttons have persistent labels for touch use.

## Verification

Focused domain tests cover exact costs, waiting output/refunds, housing, capacity
reservations/cancellation, cargo conservation and invalid quantities, blocked and
full shore landings, invalid land navigation, departure interruption, conflicting
destinations, docking, old saves, corrupt manifests and deterministic mid-sail
reload. A HUD regression covers dock/stopped/capacity gating.

FAL sources, OpenAI refinement, prompts and request IDs are retained in
`assets/sprites/transport_sources/`. Two 512px authored frames pass the shared
270-frame resolution audit and transport transparency/registration checks.

Real desktop mouse controls and DPR-2 phone touch controls passed production, cargo loading, boarding, sailing, docking, landing and unloading against an isolated SQLite server, with no page errors. Desktop stopping and phone pinch zoom were also exercised. Full verification passed 171 tests, strict native/WASM lint, formatting, rebuilt release WASM and asset checks.

Browser evidence and the missing-Modal-token release blocker are recorded in `OPEN_WORK.md`.
`cargo run -p aoa-game --example transport_fixture` prints an isolated coastal
save for reproducible acceptance; it never writes to a live database.

## Limits

This is local transport around the existing island. Future first-completion
island generation, persistent destinations, local inventories and trade remain
B3/B4 work. Two static authored ship views are mirrored across headings; there
is no dedicated rowing/loading animation. The broader canvas accessibility
workstream remains open (A3); this change uses the existing controls.
