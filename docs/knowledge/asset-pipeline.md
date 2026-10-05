# Asset generation and integration

Read before: Before generating, refining, repacking, replacing, or diagnosing game art.

Status: maintained guide. Source-reviewed 2026-10-05 against `b054655`; historical
PR results below are evidence, not newly run verification. Repository code paths
in backticks are relative to the root.

## Working with FAL and image refinement

Read [the asset policy](../../AGENTS.md#asset-workflow) and the relevant
[sprite](../../assets/sprites/README.md), [UI](../../assets/ui/README.md),
[terrain](../../assets/terrain/README.md) or [loading](../../assets/loading/README.md)
provenance. Start from approved art and retained originals.

1. Identify the asset family, manifest, packer, render size and target states.
   Check whether existing high-resolution sources solve the problem first.
2. For FAL generation, inspect the appropriate script under `assets/*/tools/`
   and its reference files before running it. Scripts can make multiple paid
   calls and assume a particular working directory; do not launch the whole
   generator to repair one frame. The existing
   [falcall.py](../../assets/ui/tools/falcall.py) uses `FAL_KEY`, submits to the
   queue, polls and records request/cost provenance. Check current credentials
   and network readiness without printing secrets; old environment notes are
   not current access checks.
3. Use the documented nano-banana generation/edit and BiRefNet cutout path;
   refine diffusion defects with the available image tool under its current
   instructions. Preserve identity, palette, geometry, anchor and transparent
   output. Retain the actual model, prompts, source references and request IDs.
4. Repack with the existing script for the affected family. For example,
   `python3 scripts/pack_hd_sprites.py` repacks retained HD sources offline and
   needs Pillow, NumPy and SciPy; it rewrites multiple atlases, so inspect its
   scope first. `scripts/normalize_icons.py --check` only checks;
   `--write` changes the icon set.
5. Run relevant checks, integrate manifest/renderer changes, rebuild the client
   when needed, and show actual previews at gameplay and maximum zoom.

```bash
python3 scripts/check_sprite_resolution.py
python3 scripts/check_field_preparation_assets.py
python3 scripts/check_transport_assets.py
python3 scripts/normalize_icons.py --check
```

Run only checks relevant to the asset family plus required integration gates.
Midjourney is an alternative with separate tooling/account requirements in
[its setup guide](../MIDJOURNEY.md); that document's environment observations are
historical. Do not assume a local login or provider access exists in a new session.

## Learned constraints and evidence

**Evidence:** [#9](https://github.com/koogle/age-of-agents/pull/9),
[#12](https://github.com/koogle/age-of-agents/pull/12) and
[#15](https://github.com/koogle/age-of-agents/pull/15) repeatedly corrected stride,
variant and depletion readability. [#23](https://github.com/koogle/age-of-agents/pull/23)
improved idle alone; [#62](https://github.com/koogle/age-of-agents/pull/62) later
recovered 183 undersized action/resource frames. Building scale needed successive
corrections in [#36](https://github.com/koogle/age-of-agents/pull/36) and
[#37](https://github.com/koogle/age-of-agents/pull/37).

**Lesson:** Agree on identity, scale relative to villagers, ground anchor,
construction/depletion stages and supported facings before packing. Fixing one
pose is not a claim that its variants or the full animation set are fixed. Prefer
recovering retained high-resolution sources over regenerating or enlarging a
small atlas. Manifests own atlas dimensions; an art-only update can still require
client integration. Preserve originals, rejected refinements and exact provenance.

**Check:** Use [the asset workflow](../../AGENTS.md#asset-workflow), strict frame
checks, contact sheets, real gameplay/max zoom and target-background alpha checks.
Compare siblings and unaffected pixels. Show images during progress as required
by [#66](https://github.com/koogle/age-of-agents/pull/66); a prose assurance of
quality gives the user no opportunity to catch the next mismatch.

## Keep this guide current

Update this file when developer steering, implementation changes or investigation
changes the procedure, contract, failure modes or verification limits. Record the
source and distinguish intended changes from implemented behavior; link any new
focused topic from the [knowledge index](INDEX.md).
