# Replenishable fields review

Fields extend the existing resource node with optional preparation progress; old resources deserialize unchanged. They are not another building kind or production queue. PlantField and Cultivate are typed, authoritative commands using the existing transactional command wrapper. Preparation reserves 10 wood and 5 stone once; only explicit villager work creates the next 120 food. Stop, reassignment, save/load and additional helpers preserve that reservation. Completion releases all participating workers to idle.

The 3×3 footprint stays occupied while empty or under preparation. Placement checks bounds before enumerating cells, free land and reachability; failed orders preserve both the stockpile and previous task. Validation rejects inconsistent preparation/food state and out-of-map resource footprints. The existing gathering, carrying, deposits and movement rules handle ripe fields. No automatic replenishment, new dependencies, transport or combat behavior was added.

Client changes reuse building placement and mouse/touch gestures, the grouped build menu, and the existing 512px farm atlas stages. Field hits use their full footprint and center. Carrying takes precedence over the work pose. All client files remain below 1,000 lines. Field art intentionally shares the farm artwork pending distinct artwork; no upscaling or DPI metadata changes are presented as new detail.

Focused tests cover completed-farm/material/site gates, atomic failure, extreme coordinates and unreachable sites, shared/interrupted/persisted progress, full food harvest/deposit, paid replenishment, retained occupancy, carrying before work, old save compatibility and corrupt field state. Verification results and publication status are recorded in OPEN_WORK.md.
