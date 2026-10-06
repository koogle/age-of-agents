# Build menu details and overhead feedback

The top-middle build hint is removed. Hovered build details and persistent placement costs/instructions use the bottom selection information area. NPC placement failures, unavailable commands and authoritative rejections use the existing fading overhead labels. Explicit movement orders also announce their status. Resource/system hints remain available in the selection detail area or above bottom navigation when unselected.

Command rejections retain the original unit IDs in local mode and correlate WebSocket replies with their existing request IDs in hosted mode. A later selection change cannot redirect a complaint. The simulation, command wire format and saved world are unchanged.

Run from the repository root after rebuilding with `scripts/build_web.sh`:

```bash
python3 docs/verification/check_build_feedback.py --output docs/verification/build-feedback
```

The loopback WebSocket fixture checks desktop mouse (1280×800 DPR1) and emulated phone touch (390×844 DPR2), including menu/placement details, absence of a top banner, local blocked-site feedback, fade expiry and a delayed rejection after selecting a different unit. Chromium animation is throttled to two frames per second for software rendering. The driver advances only the game’s presentation clock, in steps below its 250ms frame clamp, waiting for each HUD upload; screenshots therefore hold a reproducible fade age. This verifies presentation and command routing, not a physical phone or production deployment.

## Code-quality review

The existing menu information panel and `Feedback` renderer are reused. A small failure payload and pending request-to-unit map preserve command ownership; no new renderer, dependency, simulation rule or persistence state is introduced. Mouse and touch share the same handlers. Feedback replaces the prior label for each unit, keeps its spawn anchor and retains the existing 64-label bound. All frontend source files remain below 1,000 lines.

The capture harness replaces both `requestAnimationFrame` and `cancelAnimationFrame` with matching timeout scheduling/cancellation. winit cancels pending requests during input: returning timeout IDs while retaining native cancellation accumulates duplicate callbacks and saturates software rendering. Broad Playwright clock replay also makes every intermediate WebGL frame expensive, so only `performance.now` is controlled here. Browser cleanup runs in `finally`.

## Verification — 2026-10-06

Integrated master `9919648`, preserving road resumption, building-selection gathering and stationary status anchors. Formatting, all 302 workspace tests (one existing manual benchmark ignored), strict native and WASM Clippy, and the release web rebuild pass. Desktop mouse and emulated DPR2-phone touch acceptance pass with no page errors; [results and rebuilt WASM hash](results-both.json).

Visual review confirms readable, wrapped menu details; no top-middle build banner; retained placement controls; and overhead complaints visible above the placement ghost. The delayed rejection captures show Villager 2 selected while Villager 1 receives the complaint. Both viewports verify that labels expire. No production deployment or physical-phone/native-window appearance verification was performed.

- [Desktop road details](desktop-road-hover.png) and [placement instructions](desktop-road-placement.png).
- [Desktop blocked placement](desktop-blocked-building.png) and [delayed rejection](desktop-server-rejection.png).
- [Phone road placement](phone-road-placement.png), [blocked placement](phone-blocked-building.png) and [delayed rejection](phone-server-rejection.png).

User authorized merge on 2026-10-06. Integrating master `9919648`; complaints preserve its stationary spawn anchors while rising/fading. Combined Rust/lint/asset checks and six release-verifier tests pass. Refreshed desktop/phone browser acceptance passes with zero page errors. Merge and production deployment are tracked by the pull request and master release workflow.

Final integration also includes master `7725420` (approved dirt-road artwork). That update changes only assets/documentation, so the verified source and WASM are unchanged. [PR #151](https://github.com/koogle/age-of-agents/pull/151) is authorized for merge; deployment is verified separately.
