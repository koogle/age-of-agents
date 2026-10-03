# Open Work

Updated 2026-10-03. Branch `codex/shift-box-selection`, based on current remote default branch `master` at `b78e279`. Jakob requested a PR for Shift-drag box selection and explicitly authorized merging it into the default branch.

## Current change

Shift-left-drag from empty ground draws a gold rectangle and adds enclosed visible unit ground anchors to the current selection. Empty boxes preserve selection; IDs are deterministic and duplicate-free. Shift-click toggles individual units; ordinary drag pans; touch tap/pan, HUD and placement paths remain intact. Shared native/WebGL2 gesture and selection code extracted into small modules; all client files remain below 1,000 lines. README and ROADMAP synchronized; rebuilt web/pkg.

## Verification

98 workspace tests pass, including two focused selection regressions. Formatting, strict native/WASM Clippy and release builds pass. Browser DPR 1/2 verifies box/reversed additive selection, Shift-click, Escape and pan without page errors. Review: docs/BOX_SELECTION_REVIEW.md. Evidence and previous handoff: artifacts/box-selection/.

## Next action

Create the PR, merge into `master` (the repository has no `main` branch), and verify automatic Modal deployment. Refresh/relaunch the local native bundle with the merged source. Additive touch and accessible DOM controls remain prior open gaps.

## Preserved work

Historical root checkout `/Users/jakob/Projects/age-of-agents` retains its older uncommitted prototype/artwork; do not bulk merge or deploy it. Older native work is preserved by recovery commit `da820bb` on `codex/native-sprite-rendering`. Existing server and SQLite saves remain untouched. Previous native app was launched from `b78e279` before this change. Local-change audit: docs/LOCAL_CHANGE_AUDIT.md (historical).

Deployment prerequisite: the previous master workflow failed after deployment because its verification still expected 60×40/2,400 cells. Updated the existing check to the current 120×80/9,600 cells; no runtime or save changes.
