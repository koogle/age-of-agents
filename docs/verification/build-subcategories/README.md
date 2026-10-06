# Build subcategory card removal

2026-10-06: user requested removal of the top title/description card inside build subcategories. The shared HUD draws the building buttons and sends hovered costs to the existing explanation toast without drawing the selection card. No simulation, command or save changes.

Verification uses the controlled `menu-icon-fixture.json` snapshot at active simulation speed and the serving/capture procedure in `../menu_icons.py`: select the villager, open Build, choose Town, then move the pointer away. Chromium desktop (1280×800, DPR1) and emulated touch phone (390×844, DPR2) captures show the building buttons and All types navigation without the card. The captured WebGL quads also omit the former card rectangle. This verifies presentation, not authoritative gameplay or physical phones.

Capture inputs: desktop select (640, 380), Build (609, 750), Town (485, 724); phone select (195, 402), Build (42, 804), Town (42, 700). Move the mouse away before capture. Software rendering is throttled to approximately 10 FPS.

- [Desktop](desktop.png)
- [Phone](phone.png)

Formatting, 289 workspace tests (one existing ignored benchmark), strict native/WASM lint and release WASM build passed. Code-quality review: one direct layout branch, shared mouse/touch behavior, no new dependency or persistent state. Integrated master `f4d5b01`, preserving roads, dedicated menu art, wildlife and granary changes. User authorized PR creation and merge on 2026-10-06; production delivery follows the master release workflow.
