---
name: minimmit-invariant-review
description: Use when changing or reviewing constructors, state transitions, protocol-facing types, validation, trusted assumptions, or code that relies on invariants in Minimmit.
metadata:
  short-description: Invariant review for Minimmit state and types
---

# Minimmit Invariant Review

## Use This Skill When

Use this skill when a change creates, validates, stores, transforms, or assumes
state that must stay true for protocol or API correctness.

For protocol behavior, also use `minimmit-protocol-tdd`. This skill makes
invariants explicit; it does not replace paper claim evidence.

## Invariant Pass

Before finishing, answer these in the code, tests, or PR text as appropriate:

1. What invariant changed or became newly relevant?
2. Where is the invariant established?
3. Which functions or types rely on it?
4. Which event or transition resets it?
5. Which invalid or out-of-scope events must not establish it?
6. Which executable tests cover the writer, reader, reset, and invalid-event
   paths?
7. Can invalid states be made unrepresentable with the local type shape?
8. If invalid input is possible, is it rejected at the boundary with a precise
   error?
9. Do tests cover attempts to violate the invariant?

## Representation Rules

- Prefer types that encode one valid state over booleans or loosely coupled
  fields that can describe impossible states.
- Keep construction paths canonical; avoid side doors that skip validation.
- Do not expose mutable internals when mutation could break an invariant.
- Keep assumptions local and explicit when the type system cannot encode them.
- Avoid deriving or implementing traits that make invalid comparison,
  ordering, cloning, or display semantics look meaningful.

## Protocol-Facing State

- Name protocol assumptions in paper vocabulary when possible.
- Keep state transitions explicit and auditable.
- Do not hide consensus-relevant invariants in runtime, storage, networking,
  timer, or helper machinery.
- Do not duplicate full protocol rule lists in invariant comments. Link to the
  canonical evidence or claim when a local reminder is enough.

## Test Expectations

- Add negative tests for invalid construction and validation paths.
- Test edge cases around empty inputs, duplicates, unknown members, stale or
  mixed views, overflow, and deterministic ordering when those risks apply.
- Assert the invariant through public behavior rather than private field shape
  unless the private shape is the invariant under review.
