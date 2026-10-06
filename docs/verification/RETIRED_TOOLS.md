# Retired one-off tools — 2026-10-06

Jakob requested a substantial reduction in Python and repetitive verification.
These historical scripts are removed from the maintained tree. Original renders,
cutouts, prompts recorded in provenance, ledgers and recorded results remain.
Their exact historical code (including any prompts embedded only in scripts) is
available at revision `1984e9b865fde8f5ac03e86801f12636ad27232a`:

```bash
git show 1984e9b865fde8f5ac03e86801f12636ad27232a:PATH
```

This retires their automatic reproduction commands; it does not claim the retained
drivers reproduce every historical screenshot. Old evidence is historical evidence.

## Maintained checks

- `check_roads.py`: real-server desktop/touch placement, work and payment.
- `verify_wildlife.py`: real-server combat and health/death behavior.
- `replay_cargo.py`: cargo UI, paging and dispatch.
- `menu_icon_batch.py`: current menu fixtures and desktop/touch dispatch.
- `replay_presentation.py`, `replay_reconnect.py` and `replay_animal_attacks.py`: animation, current wildlife poses and network playback.
- New upstream health/status, build-feedback and building-selection drivers remain active.
- `scripts/check_*.py`, `normalize_icons.py`, release verifier: asset and release gates.
- `scripts/pack_*.py` and their imported/called sprite tools: offline asset rebuilds.

FAL queue access remains in `assets/ui/tools/falcall.py`. Future art uses approved
references and current provenance under the asset guide; the old batch generators
assumed transient directories and are not a maintained regeneration interface.

Real world-generation, coast, wildlife pose and progression behavior remains
covered by Rust tests. Synthetic fixture self-tests are not gameplay guarantees.

## Historical art generation and contact sheets

- `assets/loading/tools/bld_gen.py`
- `assets/loading/tools/bld_pack.py`
- `assets/loading/tools/load_contact.py`
- `assets/loading/tools/title_final.py`
- `assets/loading/tools/title_fix.py`
- `assets/loading/tools/title_fix_x2.py`
- `assets/sprites/tools/cat_gen.py`
- `assets/sprites/tools/hd_contact.py`
- `assets/sprites/tools/hd_idle.py`
- `assets/sprites/tools/r3_contact.py`
- `assets/sprites/tools/res_contact.py`
- `assets/sprites/tools/res_strips.py`
- `assets/sprites/tools/scen_gen.py`
- `assets/sprites/tools/spr_contact.py`
- `assets/sprites/tools/strips.py`
- `assets/sprites/tools/tc_contact.py`
- `assets/sprites/tools/tc_gen.py`
- `assets/sprites/tools/unit_gen.py`
- `assets/sprites/tools/var_gen.py`
- `assets/terrain/tools/terrain.py`
- `assets/ui/tools/contact_sheet.py`
- `assets/ui/tools/icons.py`
- `scripts/process_asset_sheets.py`

## Historical visual checks superseded by current acceptance and retained evidence

- `docs/verification/2026-10-05-animal-attacks/comparison.py`
- `docs/verification/2026-10-05-animal-style/comparison.py`
- `docs/verification/building-progression/browser.py`
- `docs/verification/check_dock_facings.py`
- `docs/verification/island-generation/browser.py`
- `docs/verification/island-generation/preview.py`
- `docs/verification/menu_icons.py`
- `docs/verification/water/check_art.py`
- `docs/verification/water/check_browser.py`
