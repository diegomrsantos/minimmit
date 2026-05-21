# Core And Shell Boundary

This document describes how `minimmit-core` communicates shell-owned work
without owning a database, runtime, network, or storage engine.

`minimmit-core` remains a deterministic state machine. The outer shell owns IO
execution and operational policy. The core decides protocol state and emits
explicit work for the shell to perform.

## Boundary

The core boundary is event-driven:

```text
Event -> Processor -> Ready
```

`Event` inputs represent protocol artifacts, local triggers, and observations.
`Ready` output is a deterministic batch of shell work. The current batch shape
separates storage work from network work:

```text
Ready {
  storage: [Storage],
  network: [Network],
}
```

The current leader-proposal flow is:

```text
Event::Propose(input)
  -> core builds and records proposal p
  -> Ready {
       storage: [Storage::PersistProposal(p)],
       network: [Network::BroadcastProposal(p)],
     }
  -> shell persists p
  -> shell broadcasts p
```

When storage and network outputs refer to the same proposal, the shell must
complete the storage output first and only then release the network output. The
core returns both outputs together so the transition remains replayable as one
deterministic step.

## Ownership

The core owns deterministic protocol semantics:

- the current protocol projection needed to validate and advance state
- protocol-visible decisions such as voting, nullifying, proposing, forwarding,
  and finalizing
- deterministic `Ready` output contents
- deterministic ordering and replay behavior

The shell owns execution and operational state:

- database transactions, schemas, indexes, and storage engine behavior
- full artifact bodies and lookup structures that are not needed in memory by
  the core
- networking, timers, fetch tasks, async scheduling, and production orchestration
- snapshot storage, restart loading, metrics, logs, and tracing
- persistence failure, retry, and shutdown policy

This split means the persisted DB can be much larger than core memory. The core
should hold only the small projection needed for protocol decisions and
deterministic replay. On startup, the shell hydrates that projection from
durable storage.

## State Vocabulary

Use this vocabulary when designing the core API, tests, and milestone issues.
The names do not require one struct per term, but the distinction should remain
visible.

- `DurableState`: protocol projection loaded from storage at startup or known
  by shell policy to have been persisted.
- `RuntimeState`: deterministic volatile state needed for current processing,
  such as current-view bookkeeping or already-observed in-memory artifacts.
- `ReadyWork`: explicit storage, network, timer, or other shell work produced by
  a core transition.

Protocol code should not hide database work, networking, timers, or retry
policy behind state mutation. If shell completion later becomes
protocol-visible input, introduce a concrete core API for that behavior with
tests that prove why the core needs it.

## Storage And Network Outputs

Storage outputs are shell actions that make protocol artifacts or projections
durable. Network outputs are shell actions that release protocol artifacts to
other validators.

For proposals, the core emits storage and broadcast outputs in the same
`Ready` batch. The batch does not grant permission to broadcast before storage
is durable. It gives the shell both pieces of work and the ordering rule:
persist first, broadcast second.

If persistence fails, the shell must not broadcast the dependent proposal. The
first implementation may stop at an explicit shell error or shutdown path
rather than inventing recovery policy in the core.

## Restart

Restart is a shell responsibility with a deterministic core contract.

At startup, the shell loads the latest durable protocol projection from the DB
and constructs the core from it. If the system supports replay after a snapshot,
the replayed history must include the same protocol inputs and any future
core-visible shell completion inputs, so restarted behavior can be compared
with uninterrupted behavior.

The core should not query the whole DB to make ordinary protocol decisions.
When it needs an artifact that is not in its current projection, the surrounding
sync or shell layer should provide it as an explicit input after validation and
fetch policy are handled outside core.

## Concurrency

The shell may run DB writes, network sends, timers, and fetches concurrently.
The core should remain single-writer and deterministic.

Shell completion that matters to protocol behavior must re-enter the core
through an explicit, ordered input. Current leader proposal persistence has no
core completion input; its dependency is enforced by shell ordering between the
storage and network outputs in one `Ready` batch.

## Testing Implications

Tests should drive the same boundary production will use:

```text
protocol input -> ready storage/network output -> shell-ordered execution
```

Useful tests assert Minimmit-owned behavior:

- local leader proposals are recorded immediately
- proposal `Ready` output contains the matching storage and network work
- empty transitions return `Ready::default()`
- restarted and uninterrupted cores behave the same for the supported
  projection
- future shell completion inputs are added only when completion affects protocol
  state

Avoid tests that inspect DB mechanics, async task queues, or private shell
buffers from `minimmit-core`. Those belong in shell, store, sync, or simulation
work once those crates exist.

## Source Map

- [proto-core-lab deterministic cores and shells](https://github.com/diegomrsantos/proto-core-lab/blob/main/docs/decisions/0007-framework-direction-deterministic-cores-and-shells.md)
