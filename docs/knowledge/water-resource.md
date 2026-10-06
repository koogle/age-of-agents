# Fresh water resource integration

Read before: changing water gathering, field water costs, or irrigation.
Status: implemented and locally verified; not deployed. Verification completed
2026-10-06 UTC.
Source review: 2026-10-05, based on `1bed03a`.

Jakob requested water after the river-placement upgrade and approved renewable
riverbank collection with a one-time 10-water field preparation cost on 2026-10-05.

## Gameplay and authoritative implementation

Water is a basic resource in `ResourceKind`, `ResourceKind::ALL`, `Stockpile`,
and the domain catalog. Stock validation, island spending, ship transfers and
50-resource holds include water through the existing typed inventory paths.
Town centers and docks accept deposits, as do stopped reachable shore ships with
room. Farms/granaries retain their food/fiber storage specialism.

`crates/game/src/game/worldgen/water.rs` adds up to eight spaced collection
points on dry cells sharing an edge with a river. It excludes fords and adjacent
cells using the generator's ford mask (fords are Beach terrain, not a separate
biome). It tries nearer reachable banks first. A candidate cannot remove any
other reachable land or the last approach to an existing resource. The normal
seed acceptance check requires water on every generated island.

Sources use the existing one-cell resource occupancy and gather command. Their
finite 120-unit amount/capacity represents a permanently replenished source;
`gathering.rs` does not decrement it when water is drawn. Inventory/cargo remain
finite, with the normal 20-unit villager carry limit and delivery/resumption.
No idle workers start collecting automatically, and Stop remains authoritative.

`fields.rs::FIELD_COST` is shared by planting, replenishment, and HUD costs:
10 wood + 5 stone + 10 water. It is reserved once when preparation starts.
Helpers, Stop/resume and reload do not pay again. The harvest yields 120 food, or 180 with the merged nearby completed granary
bonus, and does not drain water while growing or being harvested. Replenishment remains
an explicit paid order. Other recipes and building availability are unchanged.

## Client and assets

Riverbank jug markers use a new frame in the existing resource atlas; old frames
are pixel-identical. `view.rs` maps Water to that frame and uses the existing
foraging work pose at the collection cell. HUD icons, resource-name inspection,
field costs and ship cargo use the normal shared mouse/touch paths.

The retained generation/refinement and packing provenance is in
[`assets/ui/sources/water/provenance.json`](../../assets/ui/sources/water/provenance.json).
Repack with `python3 scripts/pack_water.py`. Crop to alpha > 40 before fitting
the world frame: faint refinement fringe otherwise shifts the visible ground
contact above the shared anchor. Approved family references accompany
the generated icon in [the comparison](../verification/water/icon-comparison.png).
Consult [assets](asset-pipeline.md) before revising it.

## Persistence and verification

Save version 16 adds required `water` balances, resetting incompatible hosted
worlds under the development policy. Matching-version saves persist and corrupt
current saves remain errors. See [server and saves](server-and-saves.md).
The shared browser fixture and production inventory verifier include water.

Focused regressions cover reachable riverbank sources across first/later islands,
repeated delivery and Stop at 1x/2x, finite cargo and source renewal, save/reload,
atomic insufficient-water rejection, single payment with helpers/resumption,
full harvest without more water, ship capacity and island-local spending.
`crates/game/examples/water_fixture.rs` prints a controlled current-format save
for isolated browser acceptance; `--serve` accepts typed commands and advances
validated fixed ticks for the WebGL driver. It is not a production seed or migration.

Verification results and release status belong in
[the handoff](../../OPEN_WORK.md) and [water evidence](../verification/water/README.md).
Update this guide when source renewal, deposit compatibility, field costs,
additional water consumers, or saved representations change.
