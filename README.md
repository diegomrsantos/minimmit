# Minimmit

Minimmit is an experimental, reviewable implementation of the Minimmit
protocol.

## What Is Minimmit?

Minimmit is a Byzantine fault tolerant consensus protocol for a validator set of
size `n` that may contain up to `f` Byzantine validators. Validators exchange
votes across views, and the protocol uses explicit thresholds to decide when a
proposal can be notarized, when a view can be nullified, and when a value can be
finalized.

Minimmit assumes `n >= 5f + 1`: the validator set must contain at least five
times the tolerated Byzantine fault count, plus one. This leaves enough
validators outside the faulty set for the protocol thresholds to overlap in the
ways required for safety.

The implementation should make those rules visible. A reader should be able to
see which messages were counted, which senders were distinct, which view was
being considered, and which threshold caused a state transition.

## Motivation

Consensus protocols are easy to make difficult to understand. A production
implementation has to deal with networking, asynchronous execution, storage,
timeouts, metrics, configuration, serialization, and operational failure modes.
Those concerns are necessary for a real node, but they can also hide the actual
protocol: which event was observed, which state changed, which threshold was
met, and which outputs became valid as a result.

When those concerns are mixed together, review becomes harder than it needs to
be. A protocol bug can look like a scheduling issue. A storage optimization can
obscure a safety invariant. A timer path can quietly become part of consensus
behavior. Tests can end up exercising a node process rather than the protocol
rule they were meant to check.

## Deterministic Core

This repository starts from the opposite direction. The core protocol should be
a deterministic state machine whose behavior can be read, replayed, tested, and
reviewed directly. Runtime machinery should sit outside that core. A reviewer
should be able to reason about safety and liveness-relevant transitions without
first understanding an async task graph, a network reactor, a storage engine, or
a timer system.

That separation has practical advantages:

- tests can drive the protocol with explicit input events
- failures can be reproduced by replaying the same event sequence
- threshold logic can be audited at the point where decisions are made
- fuzzing and simulation can target the protocol without a production node
- outer runtime code can change without changing protocol behavior

The intended direction is a small core shaped around explicit inputs and
outputs:

```text
Event -> Core -> Ready
```

`Event` is what the protocol observes, `Core` is the deterministic state machine
that applies the protocol rules, and `Ready` is what an outer runtime should do
next.

Assurance guidance is documented under `docs/assurance/`; the core crate ledger
lives at `crates/core/assurance.yaml`. Dependency policy is documented in
`docs/dependencies.md`.

## Status

This implementation is experimental and is not for production use.
