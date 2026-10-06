# Single-row building card — 2026-10-06

Jakob liked the brick arrow but said the card still looked like two rows of text.
Working assumption stated in chat: show only the building name in one row beside
the portrait and brick arrow; move its normal description/current work status to
hover or tap on the name area. The optional clarification remained unanswered
before proceeding with this reversible layout change.

Completed building cards have a fixed 52px height. Names stay on one line and
fit the space before the 44px upgrade target. The completed-material suffix is
available through the upgrade explanation instead of lengthening the name.
Active work keeps its visual progress bar, with status in the name-area hint.
Foundations and other selections retain their existing layouts.

The name area dispatches Explain; the brick arrow dispatches the existing typed
upgrade action. Hit regions are separate. Name/action hovering retains building
identity and geometry. No simulation, payment, save or asset changes.

99 client tests, strict native/WASM lint, formatting and rebuilt WebGL pass.
The all-building matrix covers 17 kinds, four upgrade states and desktop/phone/
landscape sizes, checking compact height, name-area Explain and upgrade dispatch.
Existing overlap tests remain passing. Final desktop mouse and DPR2-phone touch upgrade/research/payment/reload flows
pass without page errors. Name-area inspection does not charge resources.
Desktop captures show the role on name hover and the upgrade payoff on arrow
hover; phone captures show the compact row. Phone explanation text can expire
before slow software-WebGL screenshot capture; Explain dispatch is verified in
the shared-input layout matrix.
[Results and tested bundle hash](result.json) · [Single-row preview](preview.jpg).

Thermonuclear review: removes permanent detail text and height calculations for
completed buildings, reuses the existing explanation/input path, adds no new
mode or abstraction, and leaves authoritative queues/costs in the domain.
