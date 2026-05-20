# minimmit-core

`minimmit-core` is for protocol logic only.

The crate is expected to grow as a deterministic state machine. Protocol
behavior should be inspectable from state transitions and tests, not from
runtime side effects.

This crate should not contain:

- async runtime integration
- networking
- storage engine code
- wall clock behavior
- production node orchestration

The intended protocol shape is:

```text
Event -> Processor -> Ready
```

`Event` represents deterministic protocol input, `Processor` owns local
protocol state for one validator identity and applies the transition, and
`Ready` describes deterministic outputs for an outer shell to perform.

Shell-owned completion enters the processor through the separate lifecycle
boundary:

```text
Lifecycle -> Processor -> Ready
```

`Lifecycle` is intentionally empty today. The current leader proposal
transition returns storage and network work together in `Ready`; when both
outputs refer to the same proposal, the shell persists first and broadcasts
only after the proposal is durable.
