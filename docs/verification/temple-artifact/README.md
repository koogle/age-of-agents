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

## Full run

`crates/game/tests/full_run.rs` plays seed 7 from a fresh world to victory using
only typed player commands and ticks, without editing state: gather stone and
wood, build a lumber mill, make 20 timber, scout the open-sea coast, build a dock,
launch a ship, board, steer into the fog toward the temple's site, land, claim the
artifact, board again, take the one-tap return home, land, and walk the party
beside the town center. It is won after 456 simulated seconds.

```bash
AOA_RUN_SNAPSHOTS=/tmp/run cargo test --release -p aoa-game --locked --test full_run -- --nocapture
python3 docs/verification/temple-artifact/capture_sequence.py /tmp/claim /tmp/run/0299-landed.json /tmp/run/0304-artifact_claimed.json
python3 docs/verification/temple-artifact/capture_sequence.py /tmp/win /tmp/run/0446-landed_home.json /tmp/run/0456-victory.json
```

| File | Shows |
| --- | --- |
| `full-run-claim-desktop.png` | Real run: the claim line appears at the sanctuary (top right). The party is still drawn at the ship because the replay jumps five seconds of walking in one step. |
| `full-run-victory-desktop.png` | Real run: the victory line above the bearer beside the home town center. |

Findings: no wildlife met the party, and the ship's landing spot was a five-second
walk from the temple. A single Move order for the bearer waited forever behind
the idle second villager in a one-cell corridor beside the dock, a documented
movement limit ([roadmap](../../ROADMAP.md#recovered-acceptance-gaps--reviewed-2026-10-05));
the playthrough walks both villagers home with one group move instead.
