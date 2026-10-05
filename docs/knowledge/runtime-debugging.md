# Runtime diagnosis

Read before: Before fixing stuck units, frozen worlds, delayed input, or mismatched visible state.

Status: maintained guide. Source-reviewed 2026-10-05 against `b054655`; historical
PR results below are evidence, not newly run verification. Repository code paths
in backticks are relative to the root.

## Working with the running game

First identify the world source and use the [server/saves guide](server-and-saves.md)
for protocol and isolated database setup. Capture the reported sequence of actions,
seed if known, current task/cargo, and the revision actually served.

On a local hosted test server, inspect without changing the world:

```bash
curl -fsS http://localhost:8000/state
```

This returns a filtered `WorldSnapshot`, not the full persisted `GameWorld`; do
not use it as a raw SQLite fixture. Compare samples of `tick` and
`simulation_speed`, then inspect command results and the WebSocket snapshot stream.
See [verification](verification-and-handoffs.md) for a controlled presentation replay
when the problem is visual. Store a minimal reproducer and the conditions that
make it fail, not a full command transcript.

## Learned constraints and evidence

**Evidence:** [#32](https://github.com/koogle/age-of-agents/pull/32) found a paused
world; [#35](https://github.com/koogle/age-of-agents/pull/35) found disconnected
clients and stale caches after deployments;
[#40](https://github.com/koogle/age-of-agents/pull/40) found healthy snapshots but
expensive terrain rebuilds. [#44](https://github.com/koogle/age-of-agents/pull/44)
and [#80](https://github.com/koogle/age-of-agents/pull/80) were actual routing faults.

**Diagnostic order:**

1. Establish mode (hosted/native/browser-local), revision/bundle and reported
   seed/layout. Inspect authoritative tick, speed, order, cargo and queue state;
   do not assume the production world matches a local fixture.
2. Check WebSocket connection, advancing snapshot sequence, command rejection
   or acknowledgement, and current assets. Preserve reconnect sequence reset
   and the server's no-cache policy.
3. If state advances but the picture lags, measure rendering/upload time and
   presentation interpolation. Reproduce with controlled snapshots when useful.
4. If the authoritative task stalls, distinguish temporary reservations from
   missing drop sites, disconnected terrain, blocked spawns or exhausted nodes.
5. Reproduce in an isolated save. Without the reported save, describe a reproduced
   cause rather than claiming it is certainly the user's exact incident.

Never reset or resume a shared production world merely to investigate. Native
and browser-local `Source::Local` are in-memory simulations; hosted SQLite saves
are a separate path. Sources: `src/main.rs`, `crates/client/src/source.rs`,
[field audit limits](../FIELD_GATHERING_REVIEW.md#limits).

## Keep this guide current

Update this file when developer steering, implementation changes or investigation
changes the procedure, contract, failure modes or verification limits. Record the
source and distinguish intended changes from implemented behavior; link any new
focused topic from the [knowledge index](INDEX.md).
