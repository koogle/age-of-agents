# Temple and artifact presentation

Evidence for the [temple and artifact guide](../../knowledge/temple-and-artifact.md).

```bash
cargo build --release -p aoa-game --example archipelago_preview
for stage in found carried home won; do
  target/release/examples/archipelago_preview 7 1 $stage > /tmp/temple_$stage.json
done
./scripts/build_web.sh
# Prepare scene files: drop "ships", set "simulation_speed" to 1, and for the temple
# scenes move the temple first in "buildings" so the initial camera frames it.
python3 docs/verification/temple-artifact/capture_sequence.py /tmp/claim /tmp/seq_found.json /tmp/seq_carried.json
python3 docs/verification/temple-artifact/capture_sequence.py /tmp/victory /tmp/seq_home.json /tmp/seq_won.json
```

Snapshots come from the real simulation: the whole run is charted by steering,
then the review fixture places villager-1 beside the temple, issues
`ClaimArtifact`, and for `home`/`won` places the bearer four or three cells from
the home town center. `capture_sequence.py` streams the first snapshot until the
page loads, then freezes page time and steps it while sending each later snapshot
once.

**Replay limit learned (2026-10-07):** claim and victory are status labels created
when consecutive snapshots differ. Under software rendering, streaming snapshots in
real time lets playback fall more than eight ticks behind; the client then
resynchronizes, which clears feedback without observing the transition, and no
label appears. Paused snapshots also show none, because labels age on simulation
time. Use 1× snapshots and a paused, stepped page clock, as
`docs/verification/replay_status.py` does.

| File | Shows |
| --- | --- |
| `temple-found-*.png` | The run-down sanctuary with the chest inside. |
| `claimed-*.png` | The emptied sanctuary and "Claimed the Artifact of the Gods" above the bearer. |
| `victory-*.png` | "Victory · the Artifact of the Gods is home" above the bearer near the home town center, with the artifact icon over its head. |
| `temple-before-after.png` | Desktop crops before and after the claim. |

Desktop 1280×800 and emulated phone 390×844 at DPR 2; no page errors. Not checked:
a live run sailing to the temple, physical devices.
