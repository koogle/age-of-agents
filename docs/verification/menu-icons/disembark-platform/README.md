# Disembark platform correction — 2026-10-06

[Art review and before/after](../../../../assets/ui/sources/menu_icons/disembark_platform/README.md).

Base `b786f46`; 293 Rust tests (one existing benchmark ignored), formatting, strict native/WASM lint, rebuilt identical client, normalization/alpha checks, 303 world frames and six release-verifier tests pass. [Checks and hashes](checks.json).

The existing `docs/verification/menu_icon_batch.py` worker used fresh contexts for `command_disembark` in desktop and phone modes. Both captures pass without page errors and send the typed Disembark command. Desktop is 1280×800 DPR1; phone is 390×844 DPR2 with verified 780×1688 image pixels. This is presentation and pointer/touch wire verification using a controlled snapshot, not physical-phone, native-window or persisted-world validation.

Thermonuclear review: one runtime PNG, existing icon key and normalization, no runtime code/dependencies, feature, cost or save changes. The art review explicitly checks deck construction rather than counting fewer lines as success.
