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

The first lifecycle input is persistence completion. The shell owns the storage
work and reports completion with the matching persistence identifier. Current
processor transitions do not emit persistence work yet, so persistence
completion is a deterministic no-op until a real protocol transition records
pending persistence state.
