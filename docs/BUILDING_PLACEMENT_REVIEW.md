# Building placement route preservation

A builder already beside a planned foundation satisfied `can_reach_beside` even
when the footprint sealed its only exit. Foundations and completed buildings
claim the same cells, so completion could leave the villager with no way out.

The authoritative Build command now compares static reachability before and
after the proposed footprint, before changing any state. Every unit retains
access to its formerly reachable ground except the cells the building occupies.
This also prevents cutting off another villager or closing a narrow passage.
Existing disconnected islands are compared independently. A moving unit is
checked from the step destination it will finish occupying. Temporary traffic
and reservations are ignored for connectivity; existing exclusive-claim and
reservation checks still decide whether the footprint itself is free.

Thermonuclear review: one domain helper reuses the existing corner-safe path
search. Units in the same checked component reuse its result. No client rules,
autonomous actions, dependencies, save migrations, or completion-time recovery.
Rejected commands retain costs, IDs, orders and reservations without rollback.
The existing builder-approach check still applies. Edge-sharing buildings remain
legal when they preserve routes. No changed implementation file crosses 1,000
lines. Existing traps in saved worlds are not repaired by this prevention fix.

Focused tests cover builder and bystander traps, blocked diagonal exits,
in-flight orders, safe construction followed by movement, already disconnected
land, and the existing touching-house regression. Full verification results are
recorded in OPEN_WORK.md and the PR.
