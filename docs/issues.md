# Issue Guidance

This document describes what a useful Minimmit issue should contain. The goal
is to let a fresh reviewer, engineer, or agent understand the work without
rereading unrelated history or rediscovering the whole paper.

## Default Shape

Keep each issue to one concern. Split unrelated protocol behavior, tests,
docs, CI, refactors, and workflow changes into separate issues.

Size issues so they can normally be closed by one small, reviewable pull
request. If the work is too broad for one focused PR, narrow the issue before
implementation or split out only the next useful execution slice.

A good issue includes:

- a concise title in the form `verb + object`
- the goal and reason for the work
- the current behavior, gap, or problem
- the desired outcome
- acceptance criteria or required evidence
- explicit out-of-scope work
- related issues, pull requests, or decisions

For roadmap or milestone work, link the relevant roadmap section or milestone
instead of copying broad scope into the issue. Keep broad tracking in roadmaps
or milestones; the issue should still describe the specific PR-sized slice
being implemented now.

For bugs, include exact reproduction steps, expected behavior, and actual
behavior. For docs and process work, describe the reader or workflow that
should improve.

If later discussion changes the work, update the issue description so the
current decision is visible without reading the full comment thread.

## Protocol Issues

Protocol issues need enough source context to start work, but they should not
copy the paper into every issue.

Include:

- the `crates/core/assurance.yaml` claim id, or state that the issue records a
  new evidence gap
- the relevant paper section, label, or anchor
- the expected protocol behavior in concrete terms
- the failing test, scenario, or evidence artifact expected
- whether the assurance ledger status or evidence list should change
- out-of-scope runtime concerns such as async execution, networking, storage,
  wall-clock timers, and production node behavior

The assurance ledger is the working index for paper-backed claims. The paper is
still the semantic authority when behavior is disputed.

## Labels

Labels help route and filter work. They do not replace a clear issue body.

Use these area labels when they apply:

- `area:protocol`: protocol semantics, state-machine behavior, or paper-claim
  work
- `area:assurance`: assurance ledger, claim coverage, evidence manifests, or
  evidence status
- `area:tests`: Rust tests, scenario tests, test helpers, replay/model
  evidence, or regression coverage
- `area:workflow`: repository process, issue or pull request templates, CI
  workflow, release workflow, or contributor guidance

Use GitHub default labels such as `bug`, `documentation`, `enhancement`, and
`question` for the issue type when useful.

## References

- GitHub issue forms:
  <https://docs.github.com/en/communities/using-templates-to-encourage-useful-issues-and-pull-requests/syntax-for-issue-forms>
- GitHub labels:
  <https://docs.github.com/en/issues/using-labels-and-milestones-to-track-work/managing-labels>
- GitLab issue triage:
  <https://docs.gitlab.com/tutorials/issue_triage/>
- GitLab project issue guidelines:
  <https://handbook.gitlab.com/handbook/marketing/project-management-guidelines/issues/>
- Atlassian acceptance criteria:
  <https://www.atlassian.com/work-management/project-management/acceptance-criteria>
