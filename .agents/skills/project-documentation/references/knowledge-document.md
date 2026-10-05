# Focused knowledge document scaffold

Create one file per system, reusable procedure, research question or coherent
learning. Start with the headings below and omit any that add no useful content.
Replace placeholders with evidence; the scaffold is not a required long form.

```markdown
# Specific system or procedure

Read before: concrete tasks or symptoms that make this useful.
Status: current guide / proposed / research / superseded.
Last source review: date and revision or API version; what was actually checked.

## What this system does

Ownership, entry points and dependencies. Link code and canonical specifications.

## How to work with it

Ordered interaction steps, commands/API shapes, prerequisites, expected responses,
and relevant side effects. Label placeholders and environment-specific assumptions.

## Learnings and failure modes

What was learned, why it matters, what to do next time, and supporting evidence.
Record rejected approaches only where their rationale prevents repeated work.

## Verification and open questions

How to check the result; distinguish recipes from checks actually run.
State known limitations, uncertainty and intended behavior not yet implemented.

## Keep this document current

Specific steering, interface changes or new findings that should trigger an edit.
```

Link the document from the knowledge index with a reading trigger. Use local
relative links for repository sources and stable URLs for external evidence.
Keep transient task/PR status in the handoff and asset-specific provenance next
to the asset. When splitting a topic, move its content and repair inbound links
instead of leaving competing copies. Research is useful before it yields a
decision; label confidence and unanswered questions rather than omitting it.
