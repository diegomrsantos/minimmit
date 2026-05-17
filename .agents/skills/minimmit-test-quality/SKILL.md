---
name: minimmit-test-quality
description: Use when adding, modifying, or reviewing tests, test helpers, regression coverage, boundary cases, invalid-input checks, deterministic ordering assertions, or behavior-focused test design in Minimmit.
metadata:
  short-description: Behavior-focused tests for Minimmit
---

# Minimmit Test Quality

## Use This Skill When

Use this skill for tests, test helpers, regression coverage, and reviews of
test readability or adequacy.

For protocol behavior, also use `minimmit-protocol-tdd`. This skill improves
test quality; it does not decide whether a protocol claim is evidenced.

## Test Design

- Name tests by the behavior they prove.
- Prefer the smallest deterministic test that gives useful confidence.
- Prefer assertions against public behavior and observable outputs.
- Make failures actionable from the test name and assertion output.
- Keep setup small enough that the assertion remains easy to audit.
- Use helpers when they remove noise, but do not let helpers hide the behavior
  being tested.
- Prefer readable, behavior-specific tests over over-abstracted test code.
- Prefer table-style tests only when the cases share one clear behavior.
- Avoid hidden time, randomness, network, shared state, or execution-order
  dependencies unless they are explicitly controlled by the test.
- Avoid tests that merely mirror private implementation structure.

## Coverage Pass

Before finishing test changes, check relevant cases:

- valid baseline behavior
- invalid inputs and precise errors
- empty inputs
- duplicate inputs
- unknown or out-of-scope identities
- threshold boundaries and overflow
- stale, future, or mixed views
- deterministic ordering where order is observable
- regression coverage for a fixed bug

## Protocol Tests

- Tie protocol behavior tests to the paper claim or explicit evidence gap via
  `minimmit-protocol-tdd`.
- Keep test scenarios literal: votes, views, proposals, notarizations,
  nullifications, finalization, replay, and restart behavior should be visible
  in the setup.
- Do not mark a larger protocol obligation covered because a helper or
  primitive has tests.

## Before Finishing

- Run the narrowest useful test command first, then broader tests when the
  changed behavior can affect shared code.
- If a test cannot be added yet, record the concrete gap and why it is
  deferred.
