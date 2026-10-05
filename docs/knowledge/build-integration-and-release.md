# Build, integration and Modal release

Read before: Before building clients, integrating branches, regenerating WASM, or releasing.

Status: maintained guide. Source-reviewed 2026-10-05 against `b054655`; historical
PR results below are evidence, not newly run verification. Repository code paths
in backticks are relative to the root.

## Local build and integration

Run commands from the repository root. Follow [README setup](../../README.md#run-locally)
for Rust and the wasm-bindgen CLI version matching the lockfile.

```bash
./scripts/run_native.sh
```

This rebuilds and launches a native in-memory game. For a browser build:

```bash
./scripts/build_web.sh
```

This rewrites `web/pkg/aoa_client.js` and `web/pkg/aoa_client_bg.wasm`. Run the
server with an [isolated database](server-and-saves.md) for gameplay checks.
After integrating source, rebuild the combined bundle and follow
[README validation](../../README.md#contributing); a branch's old binary cannot
validate a merged renderer or local simulation.

## Test iteration and compilation

Use `cargo test -p aoa-game --locked` during simulation-only work, and the full
workspace suite before shipping. `[profile.test.package.aoa-game]` keeps domain
code optimized with debug assertions; graphics/windowing dependencies use the
default unoptimized test profile. This reduces first-build work, at the cost of
slower client tests. A profile change invalidates affected build artifacts once.

Separate compilation (`cargo test --workspace --locked --no-run --timings`) from
execution (repeat `cargo test --workspace --locked` without editing sources).
Cargo writes the dependency timeline to `target/cargo-timings/cargo-timing.html`.
Do not compare runs while another build or benchmark competes for CPU.

The 2026-10-05 investigation at `b054655` measured 115.8s warm on a four-CPU cloud
quota: 0.25s build check, 0.48s server, 4.03s client, 110.12s domain. Instrumented
serial soundness took 103.36s; 45,911 route searches accounted for 90.19s (87%).
Explicit extra validation was about 1%, so removing invariant checks was rejected.
The same eight seeds and 9,600 ticks on four workers took 31.91s in isolation.
Soundness now uses at most four available workers and retains its aggregate
accepted-build/gather assertions. After integrating `2bddab9`, the final warm workspace run with these changes
passed 233 tests in 52.74s: 0.29s build check, 1.20s server, 15.72s client and
34.79s domain. The baseline had 215 tests, so this is not an identical-suite
comparison. The narrower profile increases client execution time while reducing
compilation work. These are dated measurements, not CI budgets.

## Interacting with Modal and GitHub Actions

[deploy.yml](../../.github/workflows/deploy.yml) runs quality checks then deploys
on pushes to `master` (also supports workflow dispatch). Use
[GitHub inspection](github-history.md) to establish the actual run and revision.
[scripts/modal_manage.py](../../scripts/modal_manage.py) provides:

| Command | Effect |
| --- | --- |
| `python3 scripts/modal_manage.py status` | Lists apps and containers using configured Modal credentials |
| `python3 scripts/modal_manage.py history` | Reads deployment history |
| `python3 scripts/modal_manage.py logs --tail 100` | Reads recent application logs |
| `python3 scripts/modal_manage.py verify` | Reads the fixed production URL and compares selected state/assets with the checkout |
| `MODAL_PROFILE=koogle-frick python3 scripts/modal_manage.py deploy` | Deploys this checkout to the documented account, then verifies; use for authorized releases |

Check that the selected profile/credentials are actually configured; do not invent
credentials or treat historical availability as current. `rollover` restarts live
containers, `stop` stops the app, and `deploy` changes production: they are not
read-only diagnostics. Documentation maintenance does not require them.

## Known verifier mismatch

The earlier source review at `b054655` found fixed 9,600-cell/120×80 assumptions
and no run decoding. Upstream commit `250f3ce`, integrated here via `0a863a4`,
repairs those assumptions: `unpack_terrain` bounds run decoding by advertised
runtime dimensions, and `scripts/test_modal_manage.py` covers the decoder in CI.
Do not reopen that completed fix or restore the old character-count check.

The verifier still requires a land unit and some unseen terrain; those are fixture
assumptions, not universal world invariants. If such a check fails, inspect the
actual world and distinguish verifier assumptions from deployment failures. Never
reset production to satisfy the verifier or skip it and claim success.

The script byte-compares only the listed bootstrap/asset files and checks WASM
signatures/strings; it does not establish byte equality of the full bundle or
real browser acceptance. Preserve this distinction in release reports.

## Learned constraints and evidence

**Evidence:** Repeated integration commits explicitly preserve concurrent work,
including [queue/HD integration](https://github.com/koogle/age-of-agents/commit/380c1b8),
[shared resources/research](https://github.com/koogle/age-of-agents/commit/ccb0c25)
and [minimap/archipelago](https://github.com/koogle/age-of-agents/commit/0ccf681).
[Native fixes](../NATIVE_FIXES_REVIEW.md) deliberately excluded obsolete camera/depth
alternatives rather than replacing newer whole files.

**Lesson:** Before integrating, inspect current base and overlapping changes,
including handoff, atlas manifests, HUD, domain enums and browser output. Resolve
source semantically, retain both independent behaviors, then regenerate JS/WASM
from the combined source. Choosing one branch's generated bundle or whole
`view.rs` can silently discard the other branch. Verify the affected combined
flow after integration; older branch evidence only certifies that older build.

Several later PRs report unavailable direct Modal credentials, including #71,
#73, #75 and #84. Check deployment readiness before a manual attempt; unchanged
missing credentials are a release-path limitation, not an application failure.
Use the existing authorized merge-triggered workflow when applicable, and report
its actual result instead of repeating an unauthenticated deploy or claiming success.

This does not authorize merging any PR. Open proposals and incorporated behavior
are tracked separately in [OPEN_WORK.md](../../OPEN_WORK.md).

## Keep this guide current

Update this file when developer steering, implementation changes or investigation
changes the procedure, contract, failure modes or verification limits. Record the
source and distinguish intended changes from implemented behavior; link any new
focused topic from the [knowledge index](INDEX.md).
