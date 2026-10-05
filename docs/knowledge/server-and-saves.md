# Server protocol, snapshots and saves

Read before: changing server transport, persistence, migration, test fixtures, or
interacting with a hosted game. Source-reviewed 2026-10-05 against `b054655` in
[main.rs](../../src/main.rs), [store.rs](../../src/store.rs),
[migration.rs](../../src/store/migration.rs) and
[client source.rs](../../crates/client/src/source.rs).

## Modes and entry points

| Mode | State owner | Persistence |
| --- | --- | --- |
| Hosted browser at `/` or `/play` | Axum server; client sends commands over `/ws` | SQLite at `AGE_OF_AGENTS_DB`, default `age_of_agents.db` |
| Native client | Shared `GameWorld` in `Source::Local` | In-memory |
| Browser with `?local` | Same domain in browser WASM | In-memory |

`GET /state` returns a filtered `WorldSnapshot` for debugging. It is not a complete
save: hidden information, internal counters and serialization shape differ.
`POST /reset?seed=N` replaces the hosted world after saving the generated state;
it is a destructive operation, not a way to add an island. `AGE_OF_AGENTS_SEED`
only selects the initial seed when no saved world exists.

## Start an isolated hosted fixture

From the root, after building the web client, use a new task-specific directory:

```bash
aoa_fixture_dir=$(mktemp -d /tmp/aoa-fixture.XXXXXX)
AGE_OF_AGENTS_DB="$aoa_fixture_dir/world.db" AGE_OF_AGENTS_SEED=123 cargo run --release --locked
```

The server binds port 8000; check whether an existing process already owns it.
In another terminal, inspect `http://localhost:8000/state` and open the browser
there. A normal browser connection is hosted mode; `?local` bypasses this SQLite
world. Keep a useful fixture/procedure in the repository instead of relying on
that temporary directory across sessions. Never point a test at the live database.

## WebSocket interaction

The wire envelope is defined by `ClientMessage` in `src/main.rs`; typed commands
are defined by `Command` in `crates/game/src/game.rs`. For example, on an isolated
test world, this pauses simulation:

```json
{"type":"command","request_id":"fixture-pause-1","command":{"type":"set_simulation_speed","multiplier":0.0}}
```

Expect `command_result` with the matching `request_id`, `ok`, optional `error`
and `applied_sequence`; snapshots have `type: snapshot`, `sequence` and `world`.
Follow the acknowledgement and resulting snapshot instead of assuming a send
succeeded. The server serializes commands/ticks under the world lock. Accepted
commands save the candidate before publishing it as authoritative; save failure
must not commit the in-memory command. Request IDs correlate replies; do not
assume they provide deduplication for replayed commands.

The server ticks at 100 ms and publishes snapshots. The client reconnects after
deploys and resets its sequence tracking because a restarted server starts a new
sequence. Preserve the no-cache response policy and test reconnect plus rejection
when changing this boundary. Do not replay uncertain state-changing commands
blindly after reconnecting.

## Snapshot and migration pitfalls

- Terrain is encoded through [terrain_codec.rs](../../crates/game/src/game/terrain_codec.rs).
  `cells` and `heights` can be run-compressed; use the current decoder and runtime
  dimensions. String length is not decoded cell count.
- Hosted saves contain authoritative `GameWorld` JSON in SQLite. Legacy island
  and ship resources are pooled once, then islands are translated into continuous
  coordinates and validated. Saving writes the current representation.
- Corrupt stock, duplicate ownership and invalid occupancy are errors. The old
  schema-reset code in `Store::initialize` is historical compatibility behavior,
  not permission to reset a failing fixture or a user's world.
- Serialization changes affect hosted loads, snapshots, native/browser-local
  simulation, fixtures and production verifiers. Consult
  [archipelago](archipelago-and-transport.md) and
  [the known deployment verifier mismatch](build-integration-and-release.md#known-verifier-mismatch).

## Verification and upkeep

Use focused server/store tests and save round trips, including malformed values,
repeated migration, failed persistence, and reload during movement/queues. The
existing server test suite is `cargo test -p age-of-agents --locked`; command and
snapshot changes also need domain/client checks and a real local WebSocket flow.
These are recipes, not results of this documentation edit.

Update this guide when an endpoint, envelope, mode, serialization rule or debugging
procedure changes. Capture the new interaction and failure conditions, not just
the final architectural decision. Historical sources:
[#35](https://github.com/koogle/age-of-agents/pull/35),
[#83](https://github.com/koogle/age-of-agents/pull/83),
[#90](https://github.com/koogle/age-of-agents/pull/90).
