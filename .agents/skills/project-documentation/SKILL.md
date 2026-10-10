---
name: project-documentation
description: "Consult and continuously maintain a folder of focused project knowledge: system guides, interaction procedures, research, troubleshooting, reusable learnings, and developer steering. Use automatically before repository implementation, debugging, planning, review, or system/tool interaction, and update during work and at handoff. Also use for documentation and history audits; not for unrelated general questions."
---

# Project documentation

Maintain a useful, navigable body of knowledge that another coding agent can
consult without the conversation. Capture how systems work, how to interact with
them, what investigation taught, and what developer corrections mean for future
work. Decisions are one kind of knowledge; they are not a substitute for guides,
procedures, research and learnings.

The working loop is **consult → work → learn/update → continue**. Updating knowledge
is part of doing the task, not a separate end-of-session documentation request.

## 1. Consult before changes or system interaction

1. Find the repository root, read applicable `AGENTS.md`, and inspect the current
   branch/revision and worktree changes. Preserve unrelated work.
2. In Age of Agents, read required session context (`README.md`, `decisions.md`,
   `OPEN_WORK.md`) and `docs/knowledge/INDEX.md`. The index routes you to focused
   guides; **open the relevant guides before editing code, choosing an approach,
   or interacting with the corresponding system**. Index-only reading is not enough.
3. Follow dependencies only where they affect this task. When work moves from
   gameplay to assets, hosting, persistence or another system, consult that system's
   guide before proceeding. Use `docs/INDEX.md` for wider specifications/evidence.
4. Verify relevant claims against current source, tools and environment. A dated
   guide or old PR is not authority over current user steering. Identify the
   requested outcome, constraints, known failure modes and appropriate checks.
5. If coverage is missing, investigate with available source/tools, then create a
   focused guide once there is useful knowledge. Do not invent instructions or
   wait for a final decision before documenting a reusable research finding.

For other repositories, discover their knowledge folder and documentation owners;
reuse those conventions. If there is no maintained knowledge folder, establish
one with an index and individual topic files. Do not copy Age of Agents paths or
create a competing documentation tree beside an existing maintained one.

## 2. Capture knowledge while it is fresh

Update the owning guide as part of normal work at these checkpoints:

- **Developer steering or correction:** before the next affected implementation
  step, record the durable constraint or clarified behavior and reconcile the old
  guidance. Note its source and reason when known; mark intended behavior as
  pending implementation until code supports it. Do not turn a one-task exception
  or ambiguous remark into a universal rule. Ask only if the ambiguity blocks work.
- **New system/tool/API learning:** record the successful interaction, prerequisites,
  useful command/schema, response interpretation and failure recovery. An auth,
  network or version finding needs environment/date scope; never store secrets.
- **Investigation or failed approach:** save the reproducible finding, why it failed,
  what works instead, and remaining uncertainty when useful to future work. Label
  hypotheses and proposals; research can be valuable without an accepted decision.
- **Meaningful milestone or change of subsystem:** reconcile changed behavior,
  useful test/fixture knowledge and affected dependencies before moving on.
- **Before handoff, commit, push, deploy or ending:** review the diff and accumulated
  steering for any useful knowledge not yet captured; update the guides and handoff.

This is a regular, event-driven habit during active work. It does not require or
create a background daemon, scheduled job, automatic external publication, or
permission prompt for routine repository documentation edits. If a checkpoint
produces no durable learning, do not manufacture a note.

Examples of useful capture in this repository:

- A developer clarifies how ships should travel: reconcile the archipelago guide
  and its linked contract, marking the requested behavior pending until implemented.
- One GitHub comment API returns nothing but review threads contain feedback:
  update `github-history.md` with the retrieval procedure and limitation. This
  is operational learning even though it introduces no product decision.
- A verifier assumes an obsolete map shape: document the mismatch and diagnosis
  in the release guide and put the repair in open work. Do not leave the finding
  only in a status message or label the verifier fixed before changing it.

## 3. Put each topic in its own home

| Information | Destination in this repository |
| --- | --- |
| How a system works and how to change/use it | Individual file in `docs/knowledge/` |
| Tool/API procedures, troubleshooting, reusable research or lessons | Relevant knowledge file; a new focused file if independently useful |
| Where and when to read those files | `docs/knowledge/INDEX.md`, links and reading triggers only |
| Agent workflow and stable constraints | `AGENTS.md`, with links to details |
| Product summary and basic run commands | `README.md` |
| Accepted choice and concise rationale | `decisions.md`, linking the detailed guide |
| Proposed product work and acceptance criteria | `ROADMAP.md` |
| Current task, blockers, PR state and next steps | `OPEN_WORK.md`, compact current state |
| Detailed existing specs, benchmarks and verification artifacts | Their existing owner under `docs/`, linked from the guide |
| Asset-specific sources, prompts, edits and requests | Existing asset README/provenance records |

A knowledge document should answer: **when to read it, what the system does, how
to interact or make a change, what was learned, how to verify, and what remains
uncertain**. Use [the document scaffold](references/knowledge-document.md) if helpful.
Scale detail to the topic. Include source paths/links and review date/revision
where freshness matters; distinguish source inspection from runtime verification.

Use one file per system, coherent lesson or independently reusable procedure.
Split when sections have different reading triggers, not arbitrarily by paragraph
or session. Give files descriptive names, link related topics and list every new
guide in the index. Keep indexes short; do not accumulate the guide bodies in
`AGENTS.md`, `decisions.md`, the index or a giant retrospective.

Keep the README concise, following the developer’s 2026-10-05 cleanup request: product direction, a short feature summary and basic setup. Put detailed behavior, UI changes and verification notes in their existing guides, not in the README.

Update the existing owner in place; avoid duplicate competing instructions. Keep
historical evidence and rejected approaches only where they explain a constraint.
Label superseded material and link its replacement. Do not copy transcripts,
routine logs, credentials or unrelated personal information into this folder.

## 4. Reconcile and verify the knowledge

1. Review affected guides for contradictions with developer steering, code,
   procedures and each other. Resolve supported facts; label unresolved issues.
2. Repair index routes, dependencies and inbound links when moving or splitting
   documents. Historical reviews remain evidence, not a second live instruction set.
3. Preserve each unresolved handoff item until completed, superseded with a reason,
   or intentionally deferred. Distinguish implemented, tested, merged, deployed
   and independently verified. Old test counts and temporary paths are not fresh evidence.
4. Validate links/anchors, source paths, command/API shapes and skill references;
   run `git diff --check` and the applicable quality review. Documentation-only
   work requires documentation checks; don't deploy or run mutating examples to
   validate prose. Runtime changes still require the project's normal checks.
5. Briefly report which knowledge was added/corrected and important limitations.
   This completion report is not the only place reusable findings should survive.

## Recovering knowledge from prior work

Enumerate relevant PRs with current metadata; inspect bodies, comments, review
threads and commits. An empty comment endpoint is not proof of no review feedback.
Use accessible conversations and state which transcripts are unavailable. Separate
direct developer instructions, author summaries, source observations and inferred
recommendations; product evolution is not automatically a mistake.

For each useful finding, update the relevant system/procedure document with what
future work needs, including evidence and verification limits. Create new topic
files where needed. Do not collect all learnings in one audit file. Open PRs do not
establish shipped behavior and this workflow does not authorize remote comments
or changes to PR state.

## Invocation and discovery

The repository location is `.agents/skills/project-documentation/SKILL.md`.
Select automatically from this description or invoke with `$project-documentation`.
`AGENTS.md` also directs agents to read it, providing a direct-read fallback when
a host has not refreshed its skill catalog. Some hosts need a new session for
catalog discovery; this repository skill is not globally installed.

## Source

Adapted from Kevin Liao's [Agents Don't Need Memory. They Need Documentation.](https://liao.gg/blog/agents-dont-need-memory)
(October 3, 2026), repository evidence, and developer steering in this conversation
on October 5, 2026: maintain a folder of individual system/learning documents,
consult them before changes and update them regularly, especially after corrections.
The article links [Operator Memory](https://github.com/aerovato/operator-memory);
this workflow does not require installing that plugin.
