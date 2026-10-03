# Open Work

Updated 2026-10-03. Checkout `work`, based on remote `master` at `c5e65fb` (merged Shift-drag box selection). Current request: fix villagers choosing unsuitable drop-off routes and occasionally abandoning the gathering loop.

## Current change

Automatic drop-offs rank completed compatible buildings by currently available route length, considering villager traffic, with static fallback when all sites are busy. Temporary approach reservations no longer make a gatherer abandon its order or skip the next nearby node. Cargo, gather phases, save formats, explicit Deposit semantics and idle behavior remain intact. README and ROADMAP synchronized. Review: docs/GATHERING_ROUTING_REVIEW.md.

## Verification

All 103 workspace tests pass, including five gathering regressions and randomized material/collision invariants. Formatting, strict WASM Clippy, server build, asset checks and diff whitespace checks pass; strict native Clippy passes too. Headless Chromium loaded the actual WebGL2 demo and observed three full food deliveries (20/40/60) at the closer granary, repeated resumption and final idle after depletion, with no page errors. Evidence and an isolated SQLite file are under `/workspace/scratch/gathering-check/`; existing saves remain untouched.

## Next action

Jakob explicitly requested a PR, merge and push. Create the PR, merge it into `master`, synchronize this checkout, and verify the automatic Modal deployment. Direct Modal CLI credentials are not configured in this environment; GitHub authentication is available.

## Preserved work

Historical root checkout `/Users/jakob/Projects/age-of-agents` retains its older prototype/artwork; do not bulk merge or deploy it. Older native work is preserved by recovery commit `da820bb` on `codex/native-sprite-rendering`. Prior selection change is merged as PR #43. Additive touch and accessible DOM controls remain prior open gaps.
