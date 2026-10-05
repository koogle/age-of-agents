# Compact mobile HUD review — 2026-10-05

The phone layout preserves the game's painted medallions and parchment. Actions align bottom-left beside an 80-logical-pixel globe with pause, 1× and 2× grouped in one row above it, instead of stacking navigation above the action area. Compact layout applies below 600 logical pixels wide or 500 high. Building menus and queues expand upward in the left column; choosing a building restores the compact placement controls. Queue backgrounds fit their contents in landscape.

## Implementation review

Rendering and hit testing use the same geometry in `crates/client/src/hud/layout.rs`. Speed controls retain their 30px artwork with separate 44px touch targets. Action labels share their button's hit region. Selection titles and details wrap within the available left column. Desktop keeps its existing layout.

The thermonuclear review found no need for another layout abstraction: the existing function owns these positions and responsive sizes. No dependencies, art assets, interaction modes, commands, simulation rules or persistence changes were added. Idle units remain idle. The layout module remains under 500 lines and the tests are separate. The branch integrates master `61fc87c`, including transport, islands, field harvesting and edge panning.

## Verification

- Automated layout checks exercise 320, 360, 390, 430 and 599px portrait, 844×390 landscape and 1440×900 desktop at 1× and 2× scale. They cover unit/town selection, all building groups, placement, ship commands, hit dispatch, bounds and target separation. Full queues have separate overlap coverage.
- Chromium DPR-2 touch verification against an isolated SQLite world: 0×/1×/2× update authoritative state; six production/research orders fill the queue; cancellation removes a waiting task and refunds its cost. Build categories, Town submenu and House placement were exercised, with the menu collapsing during placement. No page errors.
- Phone, narrow-phone, landscape and desktop screenshots are in `docs/verification/2026-10-05/`. Under software WebGL, the continuous full-snapshot stream lagged behind a rapid sequence of orders. Reloading the paused isolated world supplied its current queue for cancellation and visual verification; rendering waits alone were insufficient. This existing transport/rendering limitation was not changed by the layout work.
- Browser build, all 185 workspace tests (13 server, 53 client, 119 game), formatting and strict native/WASM Clippy pass. Both layout tests pass again after the landscape queue adjustment. Final combined browser-local DPR-2 phone/menu/landscape replay has no page errors.

## Boundaries

Direct Modal deployment fails with “Token missing”; this review does not certify a production rollout. Native platform appearance and physical-device safe-area behavior have not been checked. Existing canvas accessibility work remains a separate backlog item; this change does not claim to implement accessible DOM buttons. Desktop globe/speed rectangular hit bounds already overlap at an empty corner; the new compact targets do not.

Earlier visual research and rejected/superseded proposals remain in `docs/grid-proposal/`; its mobile study is not the implemented layout.

Follow-up: pause now shares the horizontal 1×/2× row instead of sitting beside the globe. The compact band height and 44px touch targets are unchanged. Current screenshot: `verification/2026-10-05/compact-time-row.png`; earlier screenshots show the previous pause position.

Merge integration: retained master `9393efe` field-route protection and action feedback, resolving documentation conflicts and regenerating the combined browser bundle. All 192 tests (13 server, 57 client, 122 game), formatting and both strict lint targets pass. Fresh DPR2 browser touch at the relocated pause target pauses the game with no page errors. Reviewed integration: no changes to the incoming domain or feedback behavior.

## Tighter time row — 2026-10-05

User-requested follow-up reduces mobile speed centers from 44px to 36px, keeping the 30px art and the rightmost coin fixed. Visible gaps shrink from 14px to 6px. Hit regions become separate 36×44px rectangles, and the reserved navigation width follows their new left edge. This supersedes the earlier 44px-wide speed targets; desktop geometry remains unchanged.

Code-quality review: three geometry adjustments in the existing shared draw/hit path, plus the matching minimum-width assertion in the existing viewport matrix. No new abstractions, dependencies, simulation changes, commands or save changes. Verification results are recorded in the current handoff.

Text transparency follow-up: glyph quads now sample only level-zero alpha, avoiding neighboring packed atlas content bleeding through mipmaps into small labels. Sprite filtering and coin artwork stay intact. Code-quality review keeps this in the existing glyph emitter and HUD shader; no asset rewrite, dependency or gameplay change. Before/after speed states are in `verification/2026-10-05/time-label-{before,after}-{0,1,2}.png`.
