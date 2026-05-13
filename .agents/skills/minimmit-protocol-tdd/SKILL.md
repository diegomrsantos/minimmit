---
name: minimmit-protocol-tdd
description: Use for Minimmit core protocol implementation, protocol tests, evidence manifests, replay or restart tests, model comparison, or reviews that must ensure paper-driven TDD and protocol claim coverage.
metadata:
  short-description: Paper-driven TDD for Minimmit protocol core
---

# Minimmit Protocol TDD

## Use This Skill When

Use this skill for protocol-facing changes or reviews that affect behavior,
tests, evidence manifests, replay or restart fixtures, or model comparison.

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

If a change spans unrelated concerns, split it before implementation.

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

## Obligation Coverage

The skill is not the canonical protocol obligation ledger. Keep exact
obligation ids, statuses, source anchors, and evidence links in the evidence
manifest or the protocol claims document once those exist.

When a change touches protocol behavior, make the relevant claim explicit and
tie it to executable evidence. Unsupported claims must stay visible as deferred
or warning entries. Do not imply paper completeness from partial tests.

Before accepting protocol behavior, check for coverage of durable protocol
risks: threshold arithmetic, distinct valid sender counting, one vote per view,
view monotonicity, proposal validity, notarization, nullification,
finalization, deterministic parent selection, replay determinism, and restart
determinism.

## Evidence Manifest Rules

Use evidence manifests as a confidence ledger, not as a substitute for tests.
They should carry exact obligation ids, source anchors, executable evidence,
assumptions, and deferred gaps. Never mark an obligation passing without
executable evidence.

If a manifest or audit tool is not present yet, keep the claim and evidence gap
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

## Before Finishing

Before handing off protocol work, verify:

- the paper claim or evidence gap is named
- executable tests cover the behavior that changed
- deterministic replay remains plausible from the API shape
- the evidence manifest or PR text reflects the current support level
