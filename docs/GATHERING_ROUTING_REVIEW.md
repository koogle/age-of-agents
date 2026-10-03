# Gathering routing review

The automatic drop-off selector used static path costs even when villagers occupied the only approach. It could repeatedly choose a blocked granary over an accessible town center. Gathering also treated another villager's reservation of every approach cell as a permanently unreachable resource, abandoning the order or skipping the next node.

Drop-off selection now ranks complete compatible buildings by the shortest route clear of current villager claims and reservations at the destination, with building ID breaking ties. If every compatible site is busy, the static route remains a fallback and cargo is retained. Geometric reachability ignores temporary reservations; actual movement still respects them and waits when they exclude every interaction cell. This keeps the existing gather phases, cargo, save format and explicit Deposit command semantics.

Thermonuclear review: changes are confined to the shared Rust simulation. Two existing path trees distinguish current traffic from permanent geometry without stored route state or a new engine abstraction. Commands remain typed and atomic; collision claims, reservations, costs and resource crediting are unchanged. Idle villagers receive no new tasks. The same rule serves gathering and pre-construction unloading. Existing mouse/touch paths remain shared. No dependencies, generated assets or scope beyond gathering reliability were added. Focused regressions cover compatible completed sites, obstructed drop-offs, repeated granary deliveries, all-sites-busy cargo retention, and reserved approaches during gathering and depleted-node continuation.

Verification is recorded in OPEN_WORK.md after completing the checks.
