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

The intended future shape is:

```text
Event -> Core -> Ready
```

`Event` represents deterministic input, `Core` owns protocol state and applies
the transition, and `Ready` describes deterministic outputs for an outer shell to
perform.
