# Resource names on demand

2026-10-05, `fix/resource-icon-tooltips`. The top resource bar displays icons and
quantities. Hovering or tapping an icon/count reveals its name in the existing
explanation pill; the removed label space reduces each resource row from 82 to
66 logical pixels. Existing action/error messages retain priority.

Verified in Chromium with the 13-resource presentation snapshot used by
[`replay_cargo.py`](../replay_cargo.py), with the ship selected:

- Desktop, 1280×800/DPR1: move to the wood icon at (464, 33), then away to
  (900, 440). The name appears only while hovered. Captures:
  [idle](desktop-icons.png), [hover](desktop-hover.png).
- Emulated phone, 390×844/DPR2: tap the wood icon at (150, 33), then advance
  the controlled clock 3.2 seconds. The name appears and expires. Captures:
  [tap](phone-tap.png), [dismissed](phone-dismissed.png).
- Neither inspection path emitted a game command; neither produced a page error.

The HUD layout matrix continues checking all region bounds, overlap and hit
results. Resource-name regions belong to the top bar, so the bottom-action-band
assertion excludes those explanations. This is a presentation/input check;
physical-phone behavior and accessible DOM controls remain separate gaps.

Review: resource inspection reuses the shared `Action::Explain` handler and
existing hover state. It adds no game command, dependency, input mode or saved
state; active action/error explanations take priority. Both modified client
files remain below 1,000 lines. The requested removal of permanent names is
explicit user steering.
