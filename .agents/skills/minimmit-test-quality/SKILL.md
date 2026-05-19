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
- Before adding or relocating a test, identify the Minimmit-authored behavior
  it protects. If the test only preserves mechanical coverage, do not add it;
  remove it when cleanup scope permits.
- Avoid tests that merely mirror private implementation structure.

## Test Implementation Shape

- Prefer one behavior per test. Split grouped checks when failures would point
  to different rules, errors, or protocol facts.
- Make the setup, action, and assertion visible in the test body.
- Keep important inputs literal at the call site: validators, views, blocks,
  votes, nullifications, and expected errors should be easy to see.
- Prefer explicit expected values over recomputing them with test logic.
- Avoid loops, conditionals, clever builders, or broad table fixtures unless
  they make the tested behavior clearer.
- Prefer DAMP test code over DRY test code when duplication makes the behavior
  easier to audit.
- Do not add test framework, assertion, fixture, property-test, or table-test
  dependencies unless the task explicitly justifies the review cost.

## Helper Rules

- Use helpers for meaningful valid domain construction, such as a threshold
  notarization or nullification fixture.
- Do not add helpers that merely rename scalar public constructors. Prefer
  direct constructors like `BlockId::new`, `ViewNumber::new`,
  `ValidatorId::new`, and `TransactionId::new` at the call site.
- Do not let helpers hide the protocol fact or edge case under test.
- Name helpers by domain meaning, not mechanics, such as `m_notarization` or
  `view_5_nullification`.
- Keep helper parameters behavior-specific and visible. Avoid generic builders
  that require readers to inspect defaults before trusting the test.
- Keep helpers local to the test file until more than one file needs the same
  behavior shape.

## Existing Test Refactor Pass

When touching existing tests, make the local implementation easier to read:

- Rename unclear tests to state the behavior they prove.
- Split broad tests that combine unrelated success and failure paths.
- Extract only the fixture construction that distracts from the assertion.
- Keep assertion output actionable without requiring a debugger or large trace.
- Remove tests that only preserve mechanical coverage when the task explicitly
  permits removing or replacing coverage.
- Preserve current coverage unless the task explicitly removes or replaces it.

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
