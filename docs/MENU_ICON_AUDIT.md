# Dedicated menu icon audit

Audited 2026-10-05 against master `75e7540`. Scope: the Rust HUD resource bar,
selection portraits, construction categories, command medallions and queued jobs.
Each fix goes through a PR and the repository verification/release process.

| Meaning | Before | Required correction | Status |
| --- | --- | --- | --- |
| Stop unit/ship | Cancel X; unused authored hand exists | Load and use `command_stop` | [PR #104](https://github.com/koogle/age-of-agents/pull/104), verified |
| Coal | Iron ore | Dedicated coal icon | [PR #107](https://github.com/koogle/age-of-agents/pull/107), verified |
| Steel | Iron ore; villager on production/queue | Dedicated steel across surfaces | [PR #107](https://github.com/koogle/age-of-agents/pull/107), verified |
| Bricks | Clay; villager on production/queue | Dedicated brick stack | [PR #107](https://github.com/koogle/age-of-agents/pull/107), verified |
| Cloth | Fiber; villager on production/queue | Dedicated folded cloth | [PR #107](https://github.com/koogle/age-of-agents/pull/107), verified |
| Rations | Food; villager on production/queue | Dedicated packed provisions | [PR #107](https://github.com/koogle/age-of-agents/pull/107), verified |
| Guard, Archer, Healer, Siege cart | Villager training and selection art | Distinct authored unit portraits on selection, production and queue | [PR #107](https://github.com/koogle/age-of-agents/pull/107), verified |
| Transport queue | Villager training icon | Use existing transport portrait consistently | [PR #107](https://github.com/koogle/age-of-agents/pull/107), verified |
| Land passengers | Group portrait | Dedicated disembark icon | [PR #107](https://github.com/koogle/age-of-agents/pull/107), verified |
| Sail to island | Transport portrait | Dedicated sailing command | [PR #107](https://github.com/koogle/age-of-agents/pull/107), verified |
| Explore beyond coast | Transport portrait | Dedicated exploration command | [PR #107](https://github.com/koogle/age-of-agents/pull/107), verified |
| Ship cargo at dock | Raw wood | Dedicated cargo icon | [PR #107](https://github.com/koogle/age-of-agents/pull/107), verified |
| Town, Gathering, Production, Military categories | First building in each group | Dedicated category symbols | [PR #107](https://github.com/koogle/age-of-agents/pull/107), verified |
| All types (back) | Cancel X | Dedicated back icon | [PR #107](https://github.com/koogle/age-of-agents/pull/107), verified |

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

## Individual art PRs

Art lands separately from runtime wiring in [#107](https://github.com/koogle/age-of-agents/pull/107). Each entry has a visual style review and retained provenance. The initial draft set was rejected under [#110](https://github.com/koogle/age-of-agents/pull/110).

| Icon | Review/source | PR |
| --- | --- | --- |
| `resource_coal` | [Style review](../assets/ui/sources/menu_icons/resource_coal/STYLE_REVIEW.md) | [#112](https://github.com/koogle/age-of-agents/pull/112) |
| `resource_steel` | [Style review](../assets/ui/sources/menu_icons/resource_steel/STYLE_REVIEW.md) | [#113](https://github.com/koogle/age-of-agents/pull/113) |
| `resource_bricks` | [Style review](../assets/ui/sources/menu_icons/resource_bricks/STYLE_REVIEW.md) | [#114](https://github.com/koogle/age-of-agents/pull/114) |
| `resource_cloth` | [Style review](../assets/ui/sources/menu_icons/resource_cloth/STYLE_REVIEW.md) | [#115](https://github.com/koogle/age-of-agents/pull/115) |
| `resource_rations` | [Style review](../assets/ui/sources/menu_icons/resource_rations/STYLE_REVIEW.md) | [#116](https://github.com/koogle/age-of-agents/pull/116) |
| `unit_guard` | [Style review](../assets/ui/sources/menu_icons/unit_guard/STYLE_REVIEW.md) | [#117](https://github.com/koogle/age-of-agents/pull/117) |
| `unit_archer` | [Style review](../assets/ui/sources/menu_icons/unit_archer/STYLE_REVIEW.md) | [#118](https://github.com/koogle/age-of-agents/pull/118) |
| `unit_healer` | [Style review](../assets/ui/sources/menu_icons/unit_healer/STYLE_REVIEW.md) | [#119](https://github.com/koogle/age-of-agents/pull/119) |
| `unit_siege_cart` | [Style review](../assets/ui/sources/menu_icons/unit_siege_cart/STYLE_REVIEW.md) | [#120](https://github.com/koogle/age-of-agents/pull/120) |
| `command_disembark` | [Style review](../assets/ui/sources/menu_icons/command_disembark/STYLE_REVIEW.md) | [#121](https://github.com/koogle/age-of-agents/pull/121) |
| `command_sail` | [Style review](../assets/ui/sources/menu_icons/command_sail/STYLE_REVIEW.md) | [#122](https://github.com/koogle/age-of-agents/pull/122) |
| `command_explore` | [Style review](../assets/ui/sources/menu_icons/command_explore/STYLE_REVIEW.md) | [#123](https://github.com/koogle/age-of-agents/pull/123) |
| `command_cargo` | [Style review](../assets/ui/sources/menu_icons/command_cargo/STYLE_REVIEW.md) | [#124](https://github.com/koogle/age-of-agents/pull/124) |
| `command_back` | [Style review](../assets/ui/sources/menu_icons/command_back/STYLE_REVIEW.md) | [#125](https://github.com/koogle/age-of-agents/pull/125) |
| `category_town` | [Style review](../assets/ui/sources/menu_icons/category_town/STYLE_REVIEW.md) | [#126](https://github.com/koogle/age-of-agents/pull/126) |
| `category_gathering` | [Style review](../assets/ui/sources/menu_icons/category_gathering/STYLE_REVIEW.md) | [#127](https://github.com/koogle/age-of-agents/pull/127) |
| `category_production` | [Style review](../assets/ui/sources/menu_icons/category_production/STYLE_REVIEW.md) | [#128](https://github.com/koogle/age-of-agents/pull/128) |
| `category_military` | [Style review](../assets/ui/sources/menu_icons/category_military/STYLE_REVIEW.md) | [#129](https://github.com/koogle/age-of-agents/pull/129) |

## Integration review

PR #107 applies the accepted artwork across resources/costs/cargo, production and
queues, specialist portraits, ship commands, categories and Back. Category and
hover/queue information-panel thumbnails follow the same art. Cancel/Close
retains its X; Stop retains its authored hand. Actions, costs, unlocks, input
geometry and persistence are unchanged.

Current combined validation: 272 Rust tests (one existing manual benchmark
ignored), native/WASM strict lint, rebuilt web bundle, 298 world frames,
normalization, exact reproduction of all 18 new icons from retained sources,
and six release-verifier tests pass. Desktop and correctly sized DPR-2 phone scenes pass; evidence is in
[the integration record](verification/menu-icons/integration/checks.json).
