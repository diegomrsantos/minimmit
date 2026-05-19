# Bounded Degradation

This document describes how Minimmit should handle overload in sync, storage,
simulation, and shell-facing work. It does not add protocol behavior to
`minimmit-core`.

The core rule is:

```text
overload must become explicit policy, not hidden queue debt
```

When incoming work exceeds useful capacity, the system still has to do
something: admit it, delay it, reject it, drop older work, shed lower-priority
work, or spend more resources tracking it. Those outcomes should be bounded,
observable, and replayable at the deterministic boundary that owns them.

## Boundary

`minimmit-core` should stay focused on protocol state transitions. Runtime
queues, fetch scheduling, peer selection, storage pressure, and production
resource management belong outside the core.

Bounded degradation belongs in later deterministic crates and adapters:

- `minimmit-sync` owns fetch admission, retry, peer targeting, delivery, and
  bounded amplification policy.
- `minimmit-store` owns artifact lookup, retention, prune refusal, and visible
  storage pressure in deterministic stores.
- `minimmit-sim` owns deterministic overload scenarios and replayable
  observations across core, sync, and store.
- runtime shells own production queue execution and translate runtime outcomes
  into explicit deterministic inputs or observations when they matter.

Do not use vague backpressure claims as a substitute for concrete accept,
delay, reject, drop, or expiry semantics.

## Vocabulary

Use this vocabulary when designing sync, store, simulation, and shell milestone
issues.

- `Admission`: decision at ingress time: accepted, delayed, or rejected.
- `WorkClass`: traffic class such as current-view critical work, catch-up
  fetches, historical repair, or background cleanup.
- `QueueLimits`: explicit item, byte, and age bounds for a modeled backlog.
- `Expiry`: removal of admitted work that is now too old to be useful.
- `Shedding`: deliberate removal of lower-value work to preserve capacity for
  more important work.
- `Supersession`: removal of work made obsolete by newer information.
- `StateBudget`: bound on tracked entities, not only bytes. Examples include
  peers, missing artifacts, dependency edges, pending requests, fork
  candidates, or invalid-response records.
- `OverloadObservation`: deterministic record of an overload-relevant outcome.

The names do not require one type per term. The point is to keep the policy
visible and testable.

## Observable Outcomes

Deterministic overload observations should be facts a scenario or comparison
report can replay and assert:

- accepted, delayed, rejected, dropped, expired, or superseded work
- reason codes for delay, rejection, shedding, or expiry
- queue depth by work class
- age of the oldest queued item
- queued bytes or requested bytes
- tracked-state cardinality against a named budget
- duplicate, invalid, or stale peer responses
- retry count, amplification count, and peer targeting decisions
- recovery tail after partitions, invalid floods, or missing-artifact storms

Operational metrics may later report similar facts, but runtime telemetry is
not the source of protocol or deterministic sync evidence. First make the
deterministic observation useful; runtime metrics can follow in shell crates.

## Priority And Isolation

Separate work classes are preferable to one mixed backlog. Current-view work,
evidence needed for safe advancement, catch-up fetches, historical repair, and
background cleanup should not silently compete as indistinguishable items.

Priority does not mean starvation is acceptable. A bounded degradation policy
should say which work may be delayed or shed first, which work must eventually
make progress under the scenario assumptions, and which overload outcomes are
expected evidence rather than test noise.

Useful policies should preserve isolation where possible: an invalid-response
flood, a hot dependency, or stale historical repair should not fully poison
current-view progress unless the scenario is explicitly demonstrating that
failure mode.

## Testing Implications

Overload tests should drive deterministic public boundaries and assert
Minimmit-owned behavior:

- bounded queues refuse or expire work instead of growing silently
- priority rules protect critical or current-view work from best-effort backlog
- invalid or duplicate responses are observed and bounded
- stale work expires with a visible reason
- tracked-state pressure is visible before it becomes unbounded memory growth
- replaying the same scenario produces the same overload observations

Avoid assertions over private runtime queues, executor internals, or production
telemetry state. Queue depth and queue age are useful assertions only when they
are explicit deterministic observations exposed by the crate under test.

## Non-Goals

This doctrine does not require:

- a generic overload framework in the repository
- overload machinery inside `minimmit-core`
- a dependency on overload research crates
- a production networking runtime
- benchmark or throughput claims before deterministic policy is legible
- one universal scheduler for all future crates

The first useful step is documentation and milestone guidance. Concrete APIs
should wait for the sync, store, and simulation boundaries that need them.
