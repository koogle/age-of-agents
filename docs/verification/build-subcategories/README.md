# Build subcategory card removal

2026-10-06: user requested removal of the top title/description card inside build subcategories. The shared HUD draws the building buttons and sends hovered costs to the existing explanation toast without drawing the selection card. No simulation, command or save changes.

Verification uses the controlled `menu-icon-fixture.json` snapshot and the serving/capture procedure in `../menu_icons.py`: select the villager, open Build, choose Town, then move the pointer away. Chromium desktop (1280×800, DPR1) and emulated touch phone (390×844, DPR2) captures show the building buttons and All types navigation without the card. The captured WebGL quads also omit the former card rectangle. This verifies presentation, not authoritative gameplay or physical phones.

- [Desktop](desktop.png)
- [Phone](phone.png)

Formatting, 267 workspace tests (one existing ignored benchmark), strict native/WASM lint and release WASM build passed. Code-quality review: one direct layout branch, shared mouse/touch behavior, no new dependency or persistent state. Not deployed.
