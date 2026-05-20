# Core And Shell Lifecycle

This document describes how `minimmit-core` communicates persistence-sensitive
progress without owning a database, runtime, network, or storage engine.

`minimmit-core` remains a deterministic state machine. The outer shell owns
execution of IO work. When protocol behavior depends on shell work becoming
durable, the shell reports that completion back to the core as an explicit
lifecycle input.

## Boundary

The core boundary is event-driven:

```text
input -> core step -> ready output
```

Inputs include protocol events and lifecycle inputs. Protocol events represent
Minimmit artifacts, timeouts, local triggers, or observations. Lifecycle inputs
represent completion of work the core previously emitted, such as durable
persistence.

Ready outputs describe work the shell should perform. They must be explicit and
correlatable. When an output is a hard dependency for later protocol behavior,
the persistence request needs an identifier that can be acknowledged later by a
lifecycle input.

The exact Rust names can evolve with the state-machine API, but the contract is:

```text
Protocol input
  -> Ready::Persist { id, proposal }
  -> shell commits durable storage
  -> Lifecycle::Persisted(id)
  -> core records durable completion
  -> shell releases the persisted artifact
```

The current processor uses this boundary for local leader proposals.
`Event::Propose` emits `Ready::Persist` with the proposal payload the shell
must make durable before releasing it. The matching `Lifecycle::Persisted`
records durable completion inside the core and returns `Ready::None`. Unknown,
duplicate, or stale persistence acknowledgements remain deterministic no-ops.

The core must not secretly assume that a storage command completed. The shell
must not hide completion of a protocol-relevant persistence request from the
core.

## Ownership

The core owns deterministic protocol semantics:

- the current protocol projection needed to validate and advance state
- protocol-visible decisions such as voting, nullifying, proposing, forwarding,
  and finalizing
- pending protocol work whose completion changes what the core may safely emit
- deterministic ordering and replay behavior

The shell owns execution and operational state:

- database transactions, schemas, indexes, and storage engine behavior
- full artifact bodies and lookup structures that are not needed in memory by
  the core
- networking, timers, fetch tasks, async scheduling, and production orchestration
- snapshot storage, restart loading, metrics, logs, and tracing

This split means the persisted DB can be much larger than core memory. The core
should hold only the small projection needed for protocol decisions and
deterministic replay. On startup, the shell may hydrate that projection from
durable storage, but the running core still learns about new persistence
completion through lifecycle inputs.

## State Vocabulary

Use this vocabulary when designing the core API, tests, and milestone issues.
The names do not require one struct per term, but the distinction should remain
visible.

- `DurableState`: protocol projection the core knows is durable because it was
  loaded at startup or acknowledged by a lifecycle event.
- `RuntimeState`: deterministic volatile state needed for current processing,
  such as current-view bookkeeping or already-observed in-memory artifacts.
- `PendingPersistence`: protocol-relevant work emitted to the shell but not yet
  acknowledged as durable.

Dependent protocol output may rely on `DurableState`, but must not rely on
`PendingPersistence` until the matching lifecycle input arrives.

## Hard And Soft Outputs

Hard outputs are shell actions whose completion affects later protocol behavior.
Persistence is the first hard output class Minimmit should model. A
persist-before-send rule is the main example: if the core decides to vote,
nullify, propose, or advance in a way that must survive restart before being
broadcast, it should emit persistence first and require matching lifecycle
completion before the shell releases the dependent work or the core treats it
as durable.

Soft outputs are shell actions that do not by themselves become durable
protocol facts. Network sends, timer scheduling, metrics, and logs can usually
be handled by the shell without changing the core's durable protocol state.
They may still get lifecycle inputs later if a concrete protocol story needs
them, but they should not be modeled speculatively.

## Example Flow

A persistence-sensitive transition should look like this:

```text
1. Protocol event enters the core.
2. Core records the transition as pending and emits Ready::Persist { id: 42, ... }:
     storage: persist the protocol projection or artifact
3. Shell commits the DB transaction.
4. Shell feeds Lifecycle::Persisted(42).
5. Core moves the pending transition into durable state and returns Ready::None.
6. Shell may release the persisted artifact.
```

If persistence fails, the first implementation may stop at an explicit error or
evidence gap rather than inventing recovery policy. What matters is that the
core does not silently treat failed or incomplete persistence as durable.

## Restart

Restart is a shell responsibility with a deterministic core contract.

At startup, the shell loads the latest durable protocol projection from the DB
and constructs the core from it. If the system supports replay after a snapshot,
the replayed history must include lifecycle inputs as well as protocol inputs,
so restarted behavior can be compared with uninterrupted behavior.

The core should not query the whole DB to make ordinary protocol decisions.
When it needs an artifact that is not in its current projection, the surrounding
sync or shell layer should provide it as an explicit input after validation and
fetch policy are handled outside core.

## Concurrency

The shell may run DB writes, network sends, timers, and fetches concurrently.
The core should remain single-writer and deterministic.

Concurrent shell completions re-enter the core through an ordered input stream.
The core may process unrelated inputs while a persistence request is pending only
when doing so does not cross the dependency guarded by that request. A dependent
vote, nullification, proposal, forwarding output, or advancement must wait for
the lifecycle completion that makes its prerequisite durable.

This keeps concurrency outside the protocol core while still making persistence
visible to the protocol logic that depends on it.

## Testing Implications

Tests should drive the same boundary production will use:

```text
protocol input -> ready persistence output -> lifecycle input -> durable completion
```

Useful tests assert Minimmit-owned behavior:

- hard outputs are persistence-id-correlated
- pending durable state changes only on the matching persistence lifecycle input
- restarted and uninterrupted cores behave the same for the supported
  projection
- unrelated inputs do not accidentally complete persistence-gated behavior

Avoid tests that inspect DB mechanics, async task queues, or private shell
buffers from `minimmit-core`. Those belong in shell, store, sync, or simulation
work once those crates exist.

## Source Map

- [proto-core-lab explicit output lifecycle](https://github.com/diegomrsantos/proto-core-lab/blob/main/docs/decisions/0006-explicit-output-lifecycle.md)
- [proto-core-lab deterministic cores and shells](https://github.com/diegomrsantos/proto-core-lab/blob/main/docs/decisions/0007-framework-direction-deterministic-cores-and-shells.md)
- [proto-core-lab Minimmit restart test](https://github.com/diegomrsantos/proto-core-lab/blob/main/protocols/minimmit_core/tests/restart.rs)
- [proto-core-lab persisted lifecycle trace](https://github.com/diegomrsantos/proto-core-lab/blob/main/examples/raft_core/fixtures/persisted_lifecycle_trace.json)
