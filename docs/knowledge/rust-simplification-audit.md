# Source subtraction and shared Rust rules

Read before refactoring shared command rules, test fixtures or Python tooling.
Reviewed 2026-10-06 against master `a8188f1`; implementation is on
`codex/reduce-tests-tooling`. Release/verification status lives in
[OPEN_WORK](../../OPEN_WORK.md) and [the verification record](../verification/rust-refactor/README.md).

## Scope and developer steering

Jakob requested less Python and simpler Rust, then rejected a 101-line reduction
as immaterial. He explicitly authorized cutting repetitive tests/fixtures and
substantially reducing Python. Prefer retiring obsolete paths and removing
competing rule owners; moving code, shrinking formatting or deleting useful
regressions does not establish a useful reduction.

## Implemented boundaries

| Concern | Owner after the refactor |
| --- | --- |
| Production/research eligibility | Pure `Building` checks shared by commands and HUD; command validation remains authoritative. |
| Population/housing | Shared domain calculations include passengers and active/waiting trainees. |
| Building capabilities | `BuildingKind`; snapshots derive discovery-filtered offers instead of persisting mutable vectors. |
| Costs/refunds | `RESEARCH_COST`, `BuildingJob::cost` and `Stockpile::missing_resource`; island spending remains authoritative. |
| Resource fields/names/accessors | One fixed local macro preserves enum order and named stockpile serialization. |
| Rejection text | Domain `CommandError` formatting; the existing string envelope and upstream unit-correlated feedback remain. |
| Worker validation | `ordered_villager`; the existing cargo helper is named `unload_before_work`. |
| Movement presentation | Shared cell/step interpolation; routing and speeds remain separate. |
| Hosted atomic commands | One owned candidate via `with_command`, saved before commit/publication. |
| GPU bind groups | One local helper for contiguous bindings, preserving binding order/views/layout/filtering. |

No ECS, engine framework, new dependency or gameplay requirement is introduced.
The proposed expanded footprint helpers and typed rejection wire payload were
not retained: they added source without enough simplification. Existing geometry,
progression, queue/error precedence, fogged placement, paid-job refunds and group
behavior remain explicit.

Removing persisted building capability vectors bumps store version to 17.
Earlier hosted development saves reset under the current policy; matching saves
persist and corruption remains an error. Snapshot capability fields keep their
wire shape. See [server/saves](server-and-saves.md) and
[economy/queues](economy-and-queues.md).

## Tests and tooling

The mechanics fixture now uses flat terrain and fixed resource patches instead
of a second Voronoi/resource generator. IDs, order and cells are preserved.
Three self-tests of that synthetic generator are removed; actual world-generation
and gameplay regressions remain. A real SQLite write-failure regression protects
save-before-commit and publication behavior.

32 historical Python generation/contact/capture scripts are retired. Active
packers, release checks and current acceptance drivers remain, including the
newly updated boar-pose replay. Original art, provenance and recorded evidence
are retained. [The retirement record](../verification/RETIRED_TOOLS.md) lists every
path and immutable Git recovery; retired commands are not current procedures.

The cleanup integrates newer boars, connected-road resume, menu/error feedback,
stationary status and health bars. Source savings, test results, browser hashes
and release limitations are measured in the verification record. Clone count is
reduced from two to one; no latency or memory-percentage speedup is claimed.
