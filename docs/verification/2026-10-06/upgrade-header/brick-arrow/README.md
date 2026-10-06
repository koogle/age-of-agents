# Brick arrow and hover-only upgrade copy — 2026-10-06

Jakob requested removing brick-upgrade text from building descriptions and showing
it on button hover. His follow-up replaces the proposed arrow-plus-brick badge
with an arrow made of brick. The header now uses that single icon, retaining its
44px target, hover lift and disabled greyscale. Upgrade costs/benefits remain in
the existing hover explanation; normal building descriptions retain their role.

[Source and provenance](../../../../../assets/ui/sources/upgrade/README.md).
[Approved brick resource and final arrow](art-comparison.png).

Style review: muted terracotta and warm brown ink match the resource kit; sparse
staggered seams preserve brick readability. The upright silhouette communicates
upgrade without a second badge. Transparent margins and optical normalization
pass. No new coin or restyling of resource icons is introduced.

Thermonuclear review: removed description augmentation; reuse typed upgrade
commands and existing hit geometry. One icon is registered in both manifest and
client atlas list. A regression assertion covers missing atlas registration,
which the first browser preview exposed before completion. No game rules,
payment, save format or resource limits change.

The 307-test workspace suite passed; after final icon registration/rendering,
all 99 client tests pass again. Strict native/WASM lint, rebuilt WebGL, formatting
and icon normalization pass. Desktop mouse and DPR2-phone touch upgrade/research/payment/reload flows pass
with no page errors. Desktop hover shows cost, duration and research unlock;
phone captures show the clean description and brick arrow.
[Results and bundle/icon hashes](result.json) · [Final preview](preview.jpg).
