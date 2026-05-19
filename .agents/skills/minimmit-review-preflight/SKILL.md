---
name: minimmit-review-preflight
description: Use before reviewing Minimmit pull requests, diffs, or handoffs to check scope relevance, sensitive paths, test-vs-production boundaries, protocol evidence, and verification claims.
metadata:
  short-description: Scope and evidence preflight for Minimmit reviews
---

# Minimmit Review Preflight

## Use This Skill When

Use this skill before a review, PR handoff, or final implementation pass where
scope discipline, sensitive files, protocol evidence, or verification claims
matter.

For Rust review, also use `minimmit-rust-quality`. For protocol behavior, also
use `minimmit-protocol-tdd`. For evidence manifests, use the evidence reviewer
agent when a subagent review is appropriate.

## Preflight Checks

1. Identify the stated goal and changed files.
2. Map every changed file to the goal. Treat unrelated edits as blocking unless
   the PR text justifies them.
3. Check sensitive paths:
   - `.agents/**`
   - `.codex/**`
   - `.github/**`
   - `AGENTS.md`
   - CI, workflow, release, dependency, and assurance files
4. If a sensitive path changed, require an explicit reason it belongs in this
   change.
5. For test-only work, flag production-code edits unless runtime or protocol
   use is justified.
6. For production-code additions motivated by tests, require:
   - the runtime or protocol call site
   - why a test-side helper is insufficient
   - the maintenance risk of keeping the code in production

## Protocol And Evidence Checks

- If protocol behavior changed, confirm the relevant paper claim or evidence
  gap is named.
- If `crates/core/assurance.yaml` changed, check that satisfied obligations
  point to executable evidence.
- If tests were renamed or moved, check for stale evidence references.
- Do not present a defect unless it is verified by source, tests, or direct
  code reasoning. Otherwise frame it as a question or residual risk.

## Verification Reporting

- Report only checks that materially support the final claim.
- For behavior changes, include the narrow tests or checks run, or explain why
  they were not run.
- For docs, skills, agents, or workflow changes, verify the changed artifacts
  directly instead of adding unrelated format, lint, or test boilerplate.
- Label unverified behavior as unverified instead of implying certainty.
