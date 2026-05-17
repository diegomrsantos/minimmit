---
name: minimmit-protocol-scenario-testing
description: Use for event-driven protocol scenario tests, semantic effects, deterministic multi-replica stories, event-step tests, and selected post-state assertions in Minimmit.
metadata:
  short-description: Event-driven scenario testing for Minimmit
---

# Minimmit Protocol Scenario Testing

## Use This Skill When

Use this skill for protocol tests that go beyond one constructor or helper:
event-step tests, scenario runners, semantic effects, deterministic
multi-replica stories, or selected post-state assertions.

Also use `minimmit-protocol-tdd` for paper claims and evidence. Read
`docs/testing.md` when you need the source-backed doctrine or guidance for
later replay, property, model-conformance, or search layers.

## Preferred Shape

Use this flow:

```text
paper claim -> explicit event -> core step -> semantic effect -> evidence
```

- Drive the same deterministic core seam production will use.
- Prefer explicit protocol events over synthetic mega-events.
- Prefer semantic effects and selected post-state assertions over raw runtime
  buckets for most protocol tests.
- Keep raw `Ready` assertions for focused boundary tests.
- Keep scenarios short enough for reviewers to read.

## Scenario Pass

Before accepting a scenario test, check:

1. The paper claim or evidence gap is named.
2. The event sequence is explicit and deterministic.
3. The expected semantic effect or state fact is clear.
4. The failure would be actionable from the test output.
5. The scenario does not depend on hidden time, randomness, networking,
   storage, async scheduling, or private runtime queues.

## Assertions

- Assert semantic effects and selected post-state facts when they explain the
  protocol story directly.
- Use exact `Ready` output assertions only for focused shell-boundary behavior.
- Avoid assertions over private buffers, queue depths, scheduler internals, or
  implementation-specific interpretation.

## Later Layers

Replay fixtures, `always`/`sometimes` property packs, Quint or model
conformance, bounded event search, and broader simulation need separate skills
when the implementation reaches those layers. Until then, keep them documented
in `docs/testing.md` rather than active in this skill.
