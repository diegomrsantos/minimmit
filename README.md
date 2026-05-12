# Minimmit

Minimmit is an experimental, reviewable implementation of the Minimmit
protocol.

Consensus protocols are easy to make difficult to understand. A production
implementation has to deal with networking, asynchronous execution, storage,
timeouts, metrics, configuration, serialization, and operational failure modes.
Those concerns are necessary for a real node, but they can also hide the actual
protocol: which event was observed, which state changed, which threshold was
met, and which outputs became valid as a result.

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

Minimmit assumes `n >= 5f + 1`: the validator set must contain at least five
times the tolerated Byzantine fault count, plus one. This leaves enough
validators outside the faulty set for the protocol thresholds to overlap in the
ways required for safety.

This implementation is experimental and is not for production use.
