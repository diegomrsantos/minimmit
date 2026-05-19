---
name: minimmit-resilience-review
description: Use to review Minimmit protocol and core designs against adversarially efficient input, bounded-state risks, cheap rejection, duplicate or non-member sender handling, threshold edge cases, and replay debt.
metadata:
  short-description: Resilience review for Minimmit protocol inputs
---

# Minimmit Resilience Review

## Use This Skill When

Use this skill to review Minimmit behavior or designs for adversarially
efficient inputs: small or valid-looking inputs that cause disproportionate
CPU, memory, state, evidence, replay, or liveness debt.

For protocol behavior, also use `minimmit-protocol-tdd`. For invariants, also
use `minimmit-invariant-review`.

## Mission

Act as a defensive reviewer. Optimize for:

- bounded state
- cheap rejection
- deterministic cleanup
- explicit invalid-input outcomes
- preserved useful progress under mixed valid and invalid input
- reproducible local tests

Frame findings as resilience engineering, not public exploit guidance.

## Core Question

Ask this for every input path:

```text
What does the sender pay, and what future work or state does the core accept?
```

## Review Surface

Prioritize Minimmit-owned behavior:

- committee membership and distinct sender counting
- threshold arithmetic and overflow boundaries
- duplicate, non-member, stale, future-view, or mixed-view messages
- proposal validation and parent selection
- notarization, nullification, finalization, and view progression
- replay or restart assumptions once those artifacts exist
- future shell queues, storage, timers, or networking only when a change crosses
  the core boundary

## Checklist

- Does the code reject cheap structural invalidity before expensive work?
- Can duplicates or non-members inflate quorum counts or retained state?
- Can many views, blocks, proposals, or certificates grow state faster than
  useful progress?
- Are cleanup and replacement rules deterministic and monotonic?
- Can stale or low-value input suppress useful state?
- Are threshold boundaries, overflow, and empty inputs tested?
- Does the public API make accepted debt explicit enough to replay and review?

## Defensive Patterns

- Validate membership, view, and shape before retaining or aggregating input.
- Deduplicate before threshold or certificate work.
- Bound retained state by protocol key, view, block, sender, and age when those
  concepts exist.
- Prefer explicit accept, reject, defer, and expire outcomes over silent drops.
- Keep protocol-relevant cleanup in deterministic core transitions.
- Add local tests that compare normal input, pathological input, and mixed
  normal-plus-pathological input when useful.

## Finding Language

- Say `risk multiplier` unless correctness evidence proves a bug.
- Say `accepted debt` for input that creates future work or retained state.
- Say `adversarially efficient` for high damage-to-input-cost cases.
- If evidence is from a synthetic or partial harness, say so and name the next
  real-code validation step.
