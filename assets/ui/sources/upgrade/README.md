# Brick upgrade arrow — 2026-10-06

Final user direction: make the up arrow itself out of brick, replacing the earlier
sage arrow with overlapping brick badge. The earlier `arrow.png` is retained as
superseded source; only `brick-arrow.png` supplies the shipped icon.

OpenAI image_gen edit references: `arrow.png` for the upward silhouette and the
approved `assets/ui/icons/resource_bricks.png` for muted terracotta palette, soft
two-tone painted shading and restrained brown ink. Prompt specified a broad
triangular head, short wide stem, about 8–12 large staggered brick shapes, no
frame, badge, lettering or surrounding medallion, transparent background and
readability at 36px. Final output ID: `exec-4f95fc60-0ad7-4b4e-9404-f8b4113d0275`.
Initial arrow output: `exec-bd4cb9bd-e398-4b1e-aa7a-2145d35077d4`, referenced against
`command_back.png`; superseded by user steering before release.

Runtime `command_upgrade.png` is downsampled to 128px and optically normalized
with `scripts/normalize_icons.py`. HUD uses its painted bounds at 36px inside
one 44px target; existing disabled greyscale and hover lift remain.
