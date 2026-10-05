# Continuous archipelago

Islands retain their 120×80 procedural layout. Discovery adds the next region in an expanding square spiral with 64 cells of open ocean between neighboring regions. The domain detects a ship within 12 cells of the next frontier and generates the adjacent island synchronously, before the vessel reaches that edge. The Explore shortcut sails toward that frontier; known-island shortcuts sail to a dock or clear coast. Manual sea commands use the same continuous movement. Camera position and entity identities survive discovery.

All discovered settlements run in the same simulation, with shared resources and research. No distant-settlement pause or autonomous unit tasks are introduced. Legacy local-map saves are validated before conversion, translated into their permanent regions, and validated again. Unit steps, destinations, resource references, passengers, fog, buildings and queues are retained.

## Memory and scaling

One raw island layout contains 9,600 terrain cells at 12 bytes each: **115,200 bytes**, before resources and entities. Keeping these layouts in memory is reasonable for the current game. The current authoritative representation is a dense rectangle, including ocean; it is not a chunk-streaming implementation. Its rectangle grows proportionally to island count because placement fills square rings.

Measured on this development host with `cargo test -p aoa-game --release archipelago_budget -- --ignored --nocapture`:

| Islands | Rectangle cells | Terrain allocation | Snapshot wire size | Idle tick | Snapshot + encoding |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1 | 9,600 | 0.12 MB | 4.0 KB | 0.063 ms | 0.16 ms |
| 4 | 68,096 | 0.82 MB | 4.1 KB | 0.067 ms | 0.82 ms |
| 16 | 344,064 | 4.13 MB | 4.4 KB | 0.063 ms | 3.94 ms |
| 64 | 1,531,904 | 18.38 MB | 5.7 KB | 0.064 ms | 12.37 ms |

These measurements retain only the starter population and its exploration; they are **not** a benchmark of 64 busy settlements. Terrain allocation excludes entities, temporary command clones, snapshots, client heights, textures and asset memory. Revealing more land increases encoded snapshots, and more workers increase simulation/pathfinding work.

Current safeguards against unnecessary work:

- Visibility examines sight-radius neighborhoods rather than every ocean cell for every observer.
- Single-destination sailing uses A* rather than flooding all reachable ocean on every step. Movement still uses dense search/occupancy arrays; allocations scale with map area.
- Snapshot terrain uses bounded run decoding and compresses repeated unseen/water cells. Older uncompressed snapshots remain readable. Full snapshots still arrive at 10 Hz.
- Ground geometry covers a camera-sized region, with lower tessellation at distant zoom. It is rebuilt when that region changes or terrain/plots change, rather than allocating detailed geometry for the entire archipelago.
- The minimap, map picking, camera bounds, fog textures and sea plane follow the growing map.

There is no configured island-count limit, but memory is finite. Coordinates remain `u16`, and full-map GPU textures eventually encounter the adapter's maximum texture dimension. Large, heavily populated archipelagos will need chunked texture/snapshot updates and more local occupancy/path searches before approaching those limits. This change deliberately avoids adding a streaming engine before measured gameplay requires one.
