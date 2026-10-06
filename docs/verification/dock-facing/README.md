# Shore-facing dock verification — 2026-10-05

Historical evidence: feature-specific generation/capture scripts were retired
on 2026-10-06. Old commands below record how these results were obtained;
consult [the retirement record](../RETIRED_TOOLS.md) for current checks and exact historical code.

The dock pier now faces the edge of its 4×4 plot with the most adjacent water.
Ties prefer south, east, north, then west; diagonal water and rivers do not count.
The same domain helper serves construction eligibility, placement preview and
placed sprites. Static terrain supplies the orientation without persisted fields.

## Checks performed

- Full workspace: 235 tests passed, one manual benchmark ignored, including
  snapshot coverage for unknown terrain, map edges and river exclusion; all three coast tests and all five building geometry tests pass.
- `cargo fmt --all --check`, strict native and WASM clippy, and release WebGL build
  pass. The client entry point remains below 1,000 lines.
- `python3 scripts/check_sprite_resolution.py`: 294 frames pass, including twelve
  new directional building frames. Original four atlas rows are pixel-identical.
- `python3 docs/verification/check_dock_facings.py --output /tmp/dock-facing`:
  32 captures, no JavaScript errors; mouse and touch picking assertions pass for
  all four directions. Desktop is 1200×900, phone is 390×844 at DPR 2, Chromium
  with software WebGL. Both use maximum zoom, so phone edges can crop the sprite.
  Selected completed captures are retained here; the driver regenerates all
  foundation, wall and roof captures as well.

These are controlled presentation fixtures using current snapshot shape. They
exercise the actual renderer and picking, but are not physical-device or
production acceptance, or a full player-driven build transaction. Existing domain
construction/placement tests cover coastal eligibility and atomic rejection.

## Visual evidence

| Facing | Desktop | Emulated phone |
| --- | --- | --- |
| South (original) | [image](desktop-south.png) | [image](phone-south.png) |
| East | [image](desktop-east.png) | [image](phone-east.png) |
| North | [image](desktop-north.png) | [image](phone-north.png) |
| West | [image](desktop-west.png) | [image](phone-west.png) |

## Quality review

Applied [THERMONUCLEAR_REVIEW](../../THERMONUCLEAR_REVIEW.md): kept authority in
the domain, removed the duplicated client water-edge scan, retained the footprint,
existing placement eligibility, costs, routes and save format. No new interaction
mode, dependencies, autonomous behavior or feature removals. All orientations
preserve art proportions and fit their registered plots. Art sources, the rejected
undersized construction draft, exact prompts and provenance remain under
`assets/sprites/building_sources/directions`.

## Release limit

Direct deployment was unavailable during local verification. Modal 1.6.1 required `modal[api-proxy-support]` to connect through
this environment's proxy. After installing the extra, the CLI connected, but its
only environment-selected workspace was `radiantai`, with no configured profiles;
the repository's production target is `koogle-frick`. Production credentials for
that target are unavailable here. No direct deployment or production reset was attempted. The user subsequently
authorized merging [PR #99](https://github.com/koogle/age-of-agents/pull/99);
production release uses the existing GitHub Actions workflow after merge.
