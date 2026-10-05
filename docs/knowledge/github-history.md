# Inspecting GitHub PRs and discussions

Read before: researching prior work, reconciling PR state, integrating a change or
recovering developer feedback. Procedure learned during the 2026-10-05 PR #1–#90
review; access and remote state must be checked again in a later session.

## Read-only inspection

The repository is `koogle/age-of-agents`, with `master` as the integration branch.
Use the connected GitHub tools or an authenticated `gh` CLI. These examples read
metadata; they do not open, merge, close or comment on PRs:

```bash
gh pr list --repo koogle/age-of-agents --state all --limit 100 --json number,title,state,mergedAt,url
gh pr view 90 --repo koogle/age-of-agents --json number,title,state,mergedAt,baseRefName,headRefName,body,url
gh api --paginate 'repos/koogle/age-of-agents/issues/49/comments?per_page=100'
gh api --paginate 'repos/koogle/age-of-agents/pulls/49/comments?per_page=100'
gh api --paginate 'repos/koogle/age-of-agents/pulls/49/reviews?per_page=100'
gh run list --repo koogle/age-of-agents --workflow deploy.yml --limit 10
```

Numbers above are concrete examples; select the actual task's PR/run. A list at
its limit is not proof of full coverage: increase the limit or paginate and record
the cutoff. Use `gh run view RUN_ID --repo koogle/age-of-agents` with an observed
run ID to inspect the result before declaring a release successful.

## Discussion sources are not interchangeable

Read the PR body, issue comments, review submissions and inline review threads.
During the review, REST-style inline comment listings returned no comments for
#49 while the connector's `github_list_pull_request_review_threads` returned three
direct user comments about expansion, handoff cleanup and asset refinement.
When one source is empty, check review threads through the connector or GitHub
GraphQL, including pagination if present. Preserve comment permalinks and label
unavailable sources; do not infer that the user gave no feedback.

PR bodies can summarize requests but are not full transcripts. Linked Claude
sessions and other ChatGPT conversations were unavailable in this review. Separate
direct user words, author summaries, source observations and inferred recommendations.
The [audit source record](../REWORK_LESSONS.md) preserves the coverage and attribution.

## Turning findings into usable knowledge

1. Confirm live PR status and compare the checkout's revision. A body can still
   say “not merged” after metadata reports a merge; a merge does not prove deployment.
2. Trace the claimed behavior to current source/tests. An open PR may describe a
   future modifier scheme or an obsolete module layout.
3. Update the relevant system guide with how to interact, what failed, the current
   contract and the source link. If the subject has no guide, create one and add
   its reading trigger to [INDEX.md](INDEX.md).
4. Reconcile unresolved work without silently treating an incorporated behavior
   as proof that its old PR was merged or closed. Remote writes require task
   authorization; documenting a finding does not require posting a GitHub comment.

Update this guide when an API/source proves incomplete, a reliable query changes,
or investigation reveals a reusable way to recover context. Do not paste raw PR
payloads or inaccessible transcript contents into the knowledge folder.

## CLI compatibility when updating an authorized PR

In this environment on 2026-10-05, `gh pr edit --body-file` failed because its GraphQL query references deprecated classic-project cards. Updating PR #92 succeeded through `gh api --method PATCH repos/koogle/age-of-agents/pulls/92 --input /tmp/pr-update.json`, where the file is JSON containing the exact `body` string. Use a JSON serializer to preserve Markdown newlines; inspect the returned PR state. This fallback requires the same explicit task authorization as any remote PR write.

## CLI credential failure during wildlife merge (2026-10-05)

`gh pr view` returned HTTP 401 in the merge session, while Git fetch and the
connected GitHub app remained available. Use the app for PR metadata and an
expected-head-SHA merge; do not replace or expose credentials. Public repository
Actions run metadata can also be read through GitHub REST without authentication.
