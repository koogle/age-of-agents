# Planned archipelago and temple marker

Evidence for the 2026-10-06 planned-run change; see the
[archipelago guide](../../knowledge/archipelago-and-transport.md#planned-run-archipelago-2026-10-06).

## Reproduction

```bash
cargo build --release -p aoa-game --example archipelago_preview
target/release/examples/archipelago_preview 7 1 > /tmp/snap_7_1.json   # fresh run
target/release/examples/archipelago_preview 7 4 > /tmp/snap_7_4.json   # after three Explore voyages
./scripts/build_web.sh
python3 docs/verification/archipelago-plan/capture.py /tmp/shots /tmp/snap_7_1.json /tmp/snap_7_4.json
```

`capture.py` serves the checked-in `web/` and `assets/` on loopback :8017, sends
the exported snapshot over a local WebSocket, and captures desktop 1280×800 and
emulated phone 390×844 at DPR 2. It requires Python `playwright` and `aiohttp`.
The snapshots come from the real simulation (Explore voyages sailed tick by
tick); the browser does not run the simulation. No page errors were reported.

## Results

| File | Shows |
| --- | --- |
| `snap_7_1-desktop.png`, `snap_7_1-phone.png` | Fresh run, seed 7: six planned islands, five uncharted discs, the temple marker, "Island 1 of 6". |
| `snap_7_4-desktop.png`, `snap_7_4-phone.png` | Same seed after three Explore voyages: four charted islands, the revealed sailing route, one uncharted disc and the temple. |
| `globe-crops.png` | Both globes enlarged 3× from the desktop captures. |

Not checked: physical devices, a live server session, or a full run reaching the
temple island (the temple itself is not implemented yet).
