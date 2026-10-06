# Brick upgrades unlock specialist research — 2026-10-06

User direction: brick upgrades can unlock noncritical research, with Farm as the
example. Working scope stated in chat: move the four optional specialist bonuses
off the Town Center and require the corresponding building's completed upgrade.

| Research | Before | After |
| --- | --- | --- |
| Agriculture (+20% food gathering) | Town Center or base Farm | Upgraded Farm |
| Forestry (+20% wood gathering) | Town Center or base Lumber Mill | Upgraded Lumber Mill |
| Mining (+20% iron/gold gathering) | Town Center or base Mining Camp | Upgraded Mining Camp; still requires Masonry |
| Textiles (+20% fiber gathering) | Town Center or base Weaver | Upgraded Weaver; still requires Agriculture |
| Masonry (+20% stone/clay gathering) | Town Center or base Kiln | Unchanged; no brick upgrade required |

Basic field preparation, local farm gathering, granary yield, all product recipes,
housing and first departure do not need upgrades. The upgrade grants access to
research; it does not automatically apply the bonus. Research still costs 40 food
and 20 wood, takes eight seconds, and completes globally once. The alternatives
presented were all four specialists or starting with Farm alone; no new effects
were added. Save version 16 resets old persisted catalog/jobs under the existing
no-backward-compatibility policy. No production reset/deployment was performed.

`BuildingKind::technologies()` owns the catalog for creation, snapshots, commands
and validation. `TechnologyKind::requires_masonry()` supplies the shared domain/UI
rule. Pending or cancelled upgrades do not unlock research. A locked command
rejects before payment; corrupt locked research jobs fail validation. Disabled
research remains visible with its upgrade requirement; completed research remains
visible with its effect.

## Verification

The combined source integrates master `f985a92`: productive building availability,
reviewed menu/wildlife art and granary yield are preserved. The starter catalog has
ten useful buildings; Workshop's stone construction still needs discovered metal
inputs for productive use. The earlier PR #131 conflicts are resolved.

- 290 Rust tests pass: 13 server, 96 client, 181 domain; one manual benchmark ignored.
- Seven progression and eight store tests rechecked after lint cleanup and adding
  explicit reset coverage for old version-15 catalogs.
- Formatting, strict native/WASM lint and web/server builds pass.
- All 387 sprite frames meet the HD minimum; icon normalization and six Python
  release-verifier tests pass.
- Domain tests cover all four gates, premature commands, payment atomicity,
  upgrade completion/reload, cancellation, corrupt saved research jobs, technology
  prerequisites and global duplicate research. HUD tests cover locked, pending,
  available and completed Agriculture controls.

Reproduce the isolated browser flow (never points at a production database):

```sh
cargo run -p aoa-game --locked --example research_fixture > /tmp/research-fixture.json
python3 docs/verification/check_brick_research.py \
  --fixture /tmp/research-fixture.json --output /tmp/brick-research-browser
```

The driver owns loopback port 8000 and a temporary SQLite database for each mode;
it terminates the server afterwards. It uses actual mouse/touch controls and
server state for locked click, active upgrade, completed upgrade, research
payment/completion and reload. Chromium desktop is 1440×900 DPR1; emulated phone
is 390×844 DPR2. Physical devices and native appearance are not claimed.

Thermonuclear review: one shared catalog and a small typed prerequisite; no new
research engine, bonus stacking, production restriction or client authority.
Existing queue payment/refunds and resource-discovery gates remain in force.

Both desktop mouse and phone touch flows passed with no page errors. Locked
clicks sent no research command; completion enabled paid Agriculture, and reload
preserved its completion. [Results and bundle hash](result.json),
[locked/unlocked phone preview](preview.jpg).
