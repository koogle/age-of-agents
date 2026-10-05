# Dedicated menu icon audit

Audited 2026-10-05 against master `75e7540`. Scope: the Rust HUD resource bar,
selection portraits, construction categories, command medallions and queued jobs.
Each fix goes through a PR and the repository verification/release process.

| Meaning | Before | Required correction | Status |
| --- | --- | --- | --- |
| Stop unit/ship | Cancel X; unused authored hand exists | Load and use `command_stop` | [PR #104](https://github.com/koogle/age-of-agents/pull/104), verified |
| Coal | Iron ore | Dedicated coal icon | Pending |
| Steel | Iron ore; villager on production/queue | Dedicated steel across surfaces | Pending |
| Bricks | Clay; villager on production/queue | Dedicated brick stack | Pending |
| Cloth | Fiber; villager on production/queue | Dedicated folded cloth | Pending |
| Rations | Food; villager on production/queue | Dedicated packed provisions | Pending |
| Guard, Archer, Healer, Siege cart | Villager training and selection art | Distinct authored unit portraits on selection, production and queue | Pending |
| Transport queue | Villager training icon | Use existing transport portrait consistently | See integration PRs |
| Land passengers | Group portrait | Dedicated disembark icon | Pending |
| Sail to island | Transport portrait | Dedicated sailing command | Pending |
| Explore beyond coast | Transport portrait | Dedicated exploration command | Pending |
| Ship cargo at dock | Raw wood | Dedicated cargo icon | Pending |
| Town, Gathering, Production, Military categories | First building in each group | Dedicated category symbols | Pending |
| All types (back) | Cancel X | Dedicated back icon | Pending |

## Already covered

All 17 buildable buildings and Field have distinct finished generated sprite
portraits. All five technologies have dedicated icons. Wood, timber, food, stone,
gold, iron, clay and fiber have dedicated resource art. Villager training, villager
and group portraits, Build, Cancel/Close, transport portrait, research completion
and cargo Load/Unload already have dedicated artwork. Shared Cancel/Close and Stop
across entity types represent the same action and do not need duplicate drawings.

Grid/reset text controls, labelled simulation speeds and cargo pagination are
intentional non-medallion controls; this audit preserves their layout and behavior.
No new gameplay feature, cost, unlock, save model or interaction is introduced.

## Verification

For every integration: inspect the asset on light/dark backgrounds at HUD size,
check runtime loading and all mapped surfaces, run relevant asset checks plus
format/tests/native and WASM lint, rebuild bindings, exercise desktop and DPR-2
phone WebGL, and perform the thermonuclear review. Record PR/release evidence as
work completes; an open PR or local screenshot does not establish deployment.

Stop evidence: [desktop](verification/menu-icons/stop/desktop.jpg), [phone](verification/menu-icons/stop/phone.jpg), [checks](verification/menu-icons/stop/checks.txt).

## Integration PRs

Each row links its verified change; consult GitHub for live merge/release state.

| Icon | PR | Evidence |
| --- | --- | --- |
| Stop | [#104](https://github.com/koogle/age-of-agents/pull/104) | [Checks](verification/menu-icons/stop/checks.txt) |
| Transport queue | PR_PENDING_transport_queue | [Checks](verification/menu-icons/transport_queue/checks.txt) |
