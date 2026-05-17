# Milestone Guidance

This document describes what a useful Minimmit milestone should contain.
Milestones are checkpoints for a coherent slice of work. Issues remain the
implementation handoff; see `docs/issues.md` for issue-writing guidance.

## Default Shape

A good milestone describes:

- the purpose of the checkpoint
- the concrete deliverable expected when it closes
- the included issue set or claim set
- the done criteria
- explicit out-of-scope work
- ordering or dependency notes when order matters

Keep milestone descriptions short enough to scan from GitHub. Link to issues,
claims, or docs instead of copying their full text.

## Protocol Milestones

Protocol milestones should make their assurance posture explicit. Include:

- relevant `crates/core/assurance.yaml` claim ids or evidence areas
- the executable evidence expected before closure
- whether known gaps may remain deferred
- the runtime concerns that remain out of scope

GitHub milestone progress is based on closed issues and pull requests. That is
useful for tracking work, but it is not the same as protocol readiness. A
protocol milestone closes only when the scoped claims are implemented,
evidenced, or explicitly deferred according to the milestone criteria.

## Closure Checklist

Before closing a milestone, verify:

- all required issues are closed or explicitly deferred
- relevant assurance entries are current
- linked evidence exists for claims marked evidenced
- required checks passed in the relevant pull requests
- remaining gaps are visible in issues, docs, or the assurance ledger

## References

- GitHub milestones:
  <https://docs.github.com/en/issues/using-labels-and-milestones-to-track-work/about-milestones>
- GitLab milestones:
  <https://docs.gitlab.com/user/project/milestones/>
- GitLab milestone guidelines:
  <https://handbook.gitlab.com/handbook/marketing/project-management-guidelines/milestones/>
- Atlassian project deliverables:
  <https://www.atlassian.com/work-management/project-management/project-deliverables>
- Atlassian definition of done:
  <https://www.atlassian.com/agile/project-management/definition-of-done>
