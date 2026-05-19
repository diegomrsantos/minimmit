---
name: minimmit-bug-repro
description: Use when debugging Minimmit regressions, flaky behavior, suspected correctness bugs, or protocol mismatches that need a minimal evidence-first reproduction before a fix.
metadata:
  short-description: Evidence-first Minimmit bug reproduction
---

# Minimmit Bug Repro

## Use This Skill When

Use this skill for regressions, flaky behavior, suspected correctness bugs,
unexpected protocol outputs, or mismatches between tests, evidence, and core
behavior.

For protocol behavior, also use `minimmit-protocol-tdd`. For test design, also
use `minimmit-test-quality`.

## Workflow

1. Define the symptom in one sentence.
2. Identify the narrowest executable reproduction:
   - an existing failing test
   - a new focused test
   - one command plus one concrete failure signature
3. Capture expected and actual behavior.
4. Locate the likely failure boundary before changing code.
5. Prefer a failing test before the fix when feasible.
6. Keep reproduction logic out of production code unless runtime use is
   explicitly justified.

## Output

- `Symptom`
- `Repro command or test`
- `Expected`
- `Actual`
- `Likely failure boundary`
- `Evidence collected`
- `Residual uncertainty`

## Minimmit Bias

- For protocol bugs, name the paper claim, assurance entry, or explicit
  evidence gap involved.
- Record validators, views, blocks, proposals, votes, certificates, and
  thresholds when they are relevant to the symptom.
- Treat duplicate senders, non-members, mixed views, stale inputs, and
  threshold boundaries as first-class repro dimensions.
- Do not infer correctness from private state shape when a public protocol
  output can demonstrate the behavior.
