# Planned archipelago and temple marker

Evidence for the 2026-10-06 planned-run change; see the
[archipelago guide](../../knowledge/archipelago-and-transport.md#planned-run-archipelago-2026-10-06).

## Reproduction

```bash
cargo build --release -p aoa-game --example archipelago_preview
target/release/examples/archipelago_preview 7 1 > /tmp/snap_7_1.json   # fresh run
target/release/examples/archipelago_preview 7 4 > /tmp/snap_7_4.json   # after steering to three more islands
./scripts/build_web.sh
python3 docs/verification/archipelago-plan/capture.py /tmp/shots /tmp/snap_7_1.json /tmp/snap_7_4.json
```

`capture.py` serves the checked-in `web/` and `assets/` on loopback :8017, sends
the exported snapshot over a local WebSocket, and captures desktop 1280×800 and
emulated phone 390×844 at DPR 2. It requires Python `playwright` and `aiohttp`.
The snapshots come from the real simulation (the ship steered into the fog
tick by tick); the browser does not run the simulation. No page errors were reported.

## Results

| File | Shows |
| --- | --- |
| `snap_7_1-desktop.png` (removed 2026-10-10: iteration evidence; final state kept), `snap_7_1-phone.png` (removed 2026-10-10: iteration evidence; final state kept) | Fresh run, seed 7: the globe reaches the temple island; everything but the start island is fog; only the temple marker is shown. |
| `snap_7_4-desktop.png` (removed 2026-10-10: iteration evidence; final state kept), `snap_7_4-phone.png` (removed 2026-10-10: iteration evidence; final state kept) | Same seed after steering to three more islands (archipelago PR captures): the sailed route and coasts seen from the ship are revealed; uncharted islands stay hidden. |
| `steer-desktop-selected.png`, `steer-phone-selected.png` | Ship steering PR: the ship steered into the fog sailed on to island 4's coast; its panel offers Return home and Sail to islands 2 and 3, with no Explore. |
| `globe-crops.png` | Both globes enlarged 3× from the desktop captures. |

Not checked: physical devices, a live server session, or a full run reaching the
temple island (the temple itself is not implemented yet).
