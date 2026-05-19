---
name: minimmit-acceptance-evals
description: Use for ambiguous, high-risk, cross-cutting, or expensive Minimmit work to define goal, non-goals, invariants, acceptance checks, evidence, and stop conditions before implementation.
metadata:
  short-description: Acceptance criteria for risky Minimmit work
---

# Minimmit Acceptance Evals

## Use This Skill When

Use this skill before implementing work that is ambiguous, high-risk,
cross-cutting, expensive to unwind, or likely to affect protocol evidence,
public APIs, repository workflow, or multiple concerns.

For protocol behavior, also use `minimmit-protocol-tdd`. For tests, also use
`minimmit-test-quality`.

## Acceptance Packet

Before implementation, produce a compact packet:

1. `Goal`
2. `Non-goals`
3. `Invariants`
4. `Acceptance checks`
5. `Evidence to collect`
6. `Stop conditions`

## Workflow

1. Restate the desired behavior in operational terms.
2. Convert vague success language into observable checks.
3. List invariants that must remain true.
4. Identify the narrowest useful command set for verification.
5. Mark claims that require protocol evidence, manual review, or later
   runtime validation.
6. Stop before implementation if the goal, scope, or evidence threshold is
   still unclear.

## Minimmit Bias

- Keep protocol acceptance tied to paper claims, assurance entries, or explicit
  evidence gaps.
- Separate core behavior from future shell concerns such as async execution,
  networking, storage, wall clocks, and production node behavior.
- Prefer checks that prove Minimmit-owned behavior through public APIs,
  deterministic transitions, or assurance-ledger evidence.
- For instruction, workflow, or documentation changes, verify the changed
  artifact directly instead of running unrelated Rust test suites.
