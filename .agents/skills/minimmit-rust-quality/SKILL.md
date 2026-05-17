---
name: minimmit-rust-quality
description: Use when creating, modifying, or reviewing Rust implementation code in Minimmit for idiomatic Rust, narrow APIs, deterministic core behavior, checked arithmetic, explicit errors, construction-time invariants, panic avoidance, and maintainability.
metadata:
  short-description: Idiomatic and reviewable Rust for Minimmit
---

# Minimmit Rust Quality

## Use This Skill When

Use this skill for Rust implementation changes or reviews, especially in
`minimmit-core` or shared code that shapes protocol behavior.

For protocol behavior, also use `minimmit-protocol-tdd`. For docs and comments,
also use `minimmit-doc-quality`.

## Core Boundaries

- Keep protocol logic deterministic and replayable.
- Do not add hidden IO, wall-clock time, randomness, globals, async runtime
  behavior, networking, storage, metrics, or production shell concerns to
  `minimmit-core`.
- Use deterministic collections when iteration order can affect observable
  behavior, test output, evidence, replay, or review.

## Rust Design

- Prefer simple domain types over raw integers or strings at protocol
  boundaries.
- Enforce invariants at construction or validation boundaries.
- Use fallible constructors when callers can provide invalid input.
- Prefer explicit error enums over stringly errors for public or shared APIs.
- Use checked arithmetic for protocol counts, thresholds, lengths, and indexes
  where overflow could change behavior.
- Add `#[must_use]` when ignoring a returned value is likely a bug.
- Avoid intentional panics in implementation paths unless the contract is
  documented and tested.
- Prefer clear ownership and borrowing over clever lifetimes, macros, generic
  abstractions, or type tricks.
- Add abstractions only when they remove real complexity or match an existing
  local pattern.

## Before Finishing

Make a focused implementation-quality pass:

1. Check that changed APIs are narrow and named in protocol vocabulary.
2. Check that invalid states are rejected before later code can rely on them.
3. Check that arithmetic, ordering, and duplicate handling are explicit.
4. Check that errors are useful to callers and tests can match them.
5. Check that the change does not mix unrelated refactors, formatting, docs,
   dependencies, test infrastructure, or protocol behavior.
