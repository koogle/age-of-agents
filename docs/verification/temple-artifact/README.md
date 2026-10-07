# Temple and artifact presentation

Evidence for the [temple and artifact guide](../../knowledge/temple-and-artifact.md).

```bash
cargo build --release -p aoa-game --example archipelago_preview
for stage in found carried won; do
  target/release/examples/archipelago_preview 7 1 $stage > /tmp/temple_$stage.json
done
./scripts/build_web.sh
python3 docs/verification/archipelago-plan/capture.py /tmp/shots /tmp/scene_found.json /tmp/scene_carried.json /tmp/scene_won.json
```

Before capture, the scene files drop `ships` and, for `found` and `carried`, move
the temple first in `buildings` so the initial camera frames it (the client frames
the first ship, else the first building). Snapshots come from the real simulation:
the whole run is charted by steering, then the fixture places villager-1 beside the
temple, issues `ClaimArtifact`, and for `won` places the bearer near home.

| File | Shows |
| --- | --- |
| `scene_found-*.png` | The sanctuary with the chest inside; the villager stands behind it as an occlusion silhouette. |
| `scene_carried-*.png` | The emptied sanctuary; the artifact icon floats above the bearer. |
| `scene_won-*.png` | The bearer beside the home town center and the victory pill. |
| `temple-before-after.png` | Desktop crops of the temple before and after the claim. |

Desktop 1280×800 and emulated phone 390×844 at DPR 2; no page errors. Not checked:
a live run sailing to the temple, physical devices.
