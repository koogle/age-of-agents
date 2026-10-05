# Decisions recovered from PR history

Reviewed 2026-10-05 against checkout `b054655`. Historical audit record. Maintained procedures and learnings now live in
[the knowledge folder](knowledge/INDEX.md); [decisions.md](../decisions.md) owns current
choices and [AGENTS.md](../AGENTS.md) owns instructions.

## Evidence and limits

Surveyed GitHub PR records #1–#90, local commit history, relevant checked-in
reviews, current source/tests, and the available conversation in which the user
requested documentation-based context and this retrospective. At review time,
83 PRs were merged, #6/#7/#51 were closed unmerged, and #72/#86/#87/#88 were open.
Fetched issue comments, review submissions, inline comments, and review threads;
the review-thread endpoint exposed three comments on #49 that the inline-comment
listing did not return. Do not equate an empty comment listing with no discussion.

Other ChatGPT conversations were not accessible. Several PRs link Claude sessions,
but their full transcripts were not retrieved. A PR author's account of a user
request is secondary evidence; it is not a verbatim conversation. Test and browser
results in the linked guides are historical reports, not checks rerun by this review.

The repeated fixes are observable. Claims that an earlier document would have
prevented them are recommendations inferred from that history, not measured time
savings. Some changes were intentional product evolution, especially transport;
the goal is to avoid resurrecting discarded designs, not prohibit iteration.

## Direct user decisions recovered

| Source | Decision | Canonical home |
| --- | --- | --- |
| [#49: expanding world](https://github.com/koogle/age-of-agents/pull/49#discussion_r4174501951) | “lets not keep the world small” and make it “ever expanding”; prototype size must not become a product limit | [Product direction](../README.md#proposed-gameplay-loop), [continuous map](CONTINUOUS_MAP.md) |
| [#49: handoff cleanup](https://github.com/koogle/age-of-agents/pull/49#discussion_r4174502952) | Remove completed work from the open-work file | [Handoff](../OPEN_WORK.md), [documentation workflow](../AGENTS.md#documentation-as-project-context) |
| [#49: asset process](https://github.com/koogle/age-of-agents/pull/49#discussion_r4174507057) | Document FAL and ChatGPT refinement, maintain consistent style, and clean up initial generations | [Asset workflow](../AGENTS.md#asset-workflow), asset provenance records |
| Current conversation, 2026-10-05 | Turn the documentation article into agent guidance and a self-selectable skill, then recover decisions from past work | [Project documentation skill](../.agents/skills/project-documentation/SKILL.md) |

## Topic guides

The individual learnings have moved into maintained system guides. Consult these
before changing the corresponding system; this audit is only the source record.

| Topic | Maintained document |
| --- | --- |
| Archipelago and transport | [Archipelago and transport](knowledge/archipelago-and-transport.md) |
| Villager tasks and cargo | [Villager tasks and cargo](knowledge/tasks-and-cargo.md) |
| Placement and route preservation | [Placement and route preservation](knowledge/placement-and-routes.md) |
| Rendering, camera and input | [Rendering, camera and input](knowledge/rendering-and-input.md) |
| Runtime diagnosis | [Runtime diagnosis](knowledge/runtime-debugging.md) |
| Asset generation and integration | [Asset generation and integration](knowledge/asset-pipeline.md) |
| HUD, feedback and accessibility | [HUD, feedback and accessibility](knowledge/hud-and-accessibility.md) |
| Build, integration and Modal release | [Build, integration and Modal release](knowledge/build-integration-and-release.md) |
| Verification and durable evidence | [Verification and durable evidence](knowledge/verification-and-handoffs.md) |
