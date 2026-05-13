---
name: minimmit-protocol-tdd
description: Use for Minimmit core protocol implementation, protocol tests, evidence manifests, replay or restart tests, model comparison, or reviews that must ensure paper-driven TDD and protocol claim coverage.
metadata:
  short-description: Paper-driven TDD for Minimmit protocol core
---

# Minimmit Protocol TDD

## Use This Skill When

Use this skill for any change that touches Minimmit protocol behavior,
`crates/core`, protocol tests, evidence manifests, replay or restart fixtures,
model comparison, or paper-compliance review.

## Semantic Authority

Use this order when behavior is disputed:

1. The Minimmit paper.
2. Commonware Quint/spec behavior when it matches the paper.
3. Commonware Rust implementation as comparison material.
4. `proto-core-lab` only as deterministic architecture and test-harness
   inspiration.

Do not treat `proto-core-lab` as a semantic oracle. If its behavior disagrees
with the paper, the paper wins.

## Required Development Loop

For every protocol behavior:

1. Identify the paper claim or protocol obligation.
2. Write the failing test first, or explicitly record the missing evidence gap.
3. Implement the smallest deterministic core behavior that satisfies the test.
4. Update the evidence manifest when one exists.
5. Mark an obligation satisfied only when executable evidence backs it.

Keep each PR to one protocol obligation or one tightly coupled group of
obligations. Do not mix protocol behavior with docs, refactors, CI, formatting,
or test infrastructure unless the dependency is real.

## Core Shape

The protocol core should remain a deterministic state machine:

```text
Event -> Core -> Ready
```

Keep protocol concepts explicit in the core API. Use literal names for protocol
types such as views, blocks, proposals, votes, certificates, replicas, and
thresholds.

Do not put protocol logic inside async runtimes, networking, storage engines,
wall-clock timers, real cryptography, or production node orchestration. Those
concerns belong outside `crates/core`.

Initial signature modeling may use signer identities, but tests must enforce
valid membership and distinct sender counting.

## Required Obligation Coverage

Track these obligations as implementation work proceeds:

- `thresholds`: `n >= 5f + 1`, `M = 2f + 1`, `L = n - f`
- `distinct_valid_senders`
- `one_vote_per_view`
- `view_monotonicity`
- `proposal_validity`
- `multiple_m_notarizations`
- `parent_selection`
- `timeout_nullification`
- `condition_b_nullification`
- `vote_on_m_notarization`
- `finalization`
- `no_conflicting_finalization`
- `no_liveness_required_l_forwarding`
- `replay_determinism`
- `restart_determinism`

Unsupported obligations must stay explicit as deferred or warning entries. Do
not imply paper completeness from partial tests.

## Evidence Manifest Rules

Use evidence manifests as a confidence ledger, not as a substitute for tests.

Each protocol obligation should connect to:

- a stable obligation id
- the relevant paper claim or algorithm reference
- executable tests or fixtures
- explicit assumptions
- deferred gaps when evidence is not executable yet

Never mark a protocol obligation as passing without executable evidence. If a
manifest or audit tool is not present yet, keep the claim and evidence gap
visible in the PR text or nearby protocol documentation.

## Test Expectations

Prefer these test layers:

- unit tests for pure protocol rules
- deterministic transition tests for `Event -> Core -> Ready`
- cluster scenario tests for multi-replica behavior
- replay tests from serialized event logs
- restart tests from snapshots
- model/spec comparison tests when a stable model fixture exists

Known discrepancy risks require regression tests before or with implementation:

- multiple M-notarized blocks in the same view
- deterministic parent selection across multiple M-notarizations
- duplicate signers not inflating quorum counts
- non-members not contributing to certificates
- M-notarization receipt triggering the required vote path
- L-notarization finalizing without being treated as required liveness
  forwarding
- timeout and condition-b nullification following the paper conditions

## Review Checklist

When reviewing protocol work, ask:

- Which paper claim is being implemented?
- Was the failing test or evidence gap created before implementation?
- Are thresholds and signer sets explicit enough to audit?
- Are state transitions deterministic and replayable?
- Did any production shell concern leak into `crates/core`?
- Does the evidence manifest match the implementation and tests?
