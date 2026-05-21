# Minimmit

Minimmit is an independent, unofficial experimental implementation of the
Minimmit BFT protocol. It is not an official Commonware project and is not
production-ready today; the current focus is deterministic core behavior and
explicit assurance before production node integration.

## What Is Minimmit?

Minimmit is a Byzantine fault tolerant consensus protocol for a validator set
of size `n` that may contain up to `f` Byzantine validators. Validators exchange
votes across views, and threshold evidence determines when a proposal can be
notarized, when a view can be nullified, and when a value can be finalized.

The baseline protocol assumes `n >= 5f + 1`: the validator set must contain at
least five times the tolerated Byzantine fault count, plus one. This repo
implements the protocol in stages, with exact claim status kept in the core
assurance ledger.

## Approach

Minimmit starts with the part of a consensus implementation that should be most
directly reviewable: the protocol decision logic. The core owns deterministic
protocol semantics, including the local state needed to validate observations,
advance views, select outputs, and make consensus-relevant decisions.

That core is intended as the foundation for later production work, not as a
replacement for it. A production node still needs the surrounding shell, but
the protocol decisions should remain isolated enough to inspect and test
directly.

The shell owns execution concerns: networking, storage, timers, async tasks,
metrics, logging, sync, restart loading, and production orchestration. Those
systems are necessary for a real node, but they should not quietly become part
of the protocol state machine.

The intended boundary is compact:

```text
Event -> Processor -> Ready
```

`Event` is protocol input. `Processor` is the deterministic state machine for
one validator identity. `Ready` is a deterministic batch of shell work, with
storage and network outputs listed separately so the shell can enforce
persist-before-broadcast ordering where an artifact requires it.

If shell completion later changes what the protocol may safely do, that
completion should re-enter the core through a concrete, ordered input with
tests showing why the protocol needs it. Current leader proposal persistence is
handled by shell ordering between matching storage and network outputs in one
`Ready` batch.

This separation makes tests easier to write, understand, replay, and maintain.
A deterministic rule should not require an async runtime just to be exercised.
Ordered input traces can be replayed directly, and failures are easier to place:
protocol bug, shell bug, storage issue, or scheduling artifact.

## Tradeoffs

This approach delays broad production integration until core behavior and the
shell boundary have enough evidence to carry it. Networking, durable storage,
sync, restart behavior, metrics, operations, and runtime policy all still have
to be built and tested around the core.

The boundary also requires discipline up front. Shell work must stay explicit:
the core should emit deterministic `Ready` output, and shell completion should
become core input only when it affects protocol behavior. Unsupported protocol
claims remain visible until the implementation has executable evidence for
them.

The payoff is a smaller review surface for protocol behavior and a clearer
assurance trail. Reviewers can inspect the deterministic core, the tests that
drive it, and the ledger that says which claims are evidenced, in progress, or
still planned.

## Status

Minimmit is experimental and is not for production use today. Deterministic
core groundwork exists, including foundational protocol data types, validation
rules, parent selection, observed artifacts, leader proposal output, and the
initial processor boundary. Full Algorithm 1 behavior, broader consistency and
liveness evidence, production shell integration, and production-readiness
evidence remain planned or in progress.

Exact evidence status is canonical in
[`crates/core/assurance.yaml`](crates/core/assurance.yaml). The surrounding
project guidance lives in [`docs/assurance/`](docs/assurance/),
[`docs/testing.md`](docs/testing.md), [`docs/roadmap.md`](docs/roadmap.md),
[`docs/core-shell-boundary.md`](docs/core-shell-boundary.md),
[`docs/dependencies.md`](docs/dependencies.md),
[`docs/releases.md`](docs/releases.md), and [`AGENTS.md`](AGENTS.md).

## License And Attribution

This implementation is licensed under either of:

- Apache License, Version 2.0 ([`LICENSE-APACHE`](LICENSE-APACHE))
- MIT license ([`LICENSE-MIT`](LICENSE-MIT))

at your option.

The protocol source is
[Minimmit: Fast Finality with Even Faster Blocks](https://arxiv.org/abs/2508.10862)
and Commonware's published
[Minimmit specification](https://github.com/commonwarexyz/monorepo/blob/main/pipeline/minimmit/minimmit.md).
The arXiv paper is distributed under CC BY 4.0, and Commonware's
[Minimmit announcement](https://commonware.xyz/blogs/minimmit) states that
Minimmit is released under both MIT and Apache-2.0. This repository follows
that licensing signal while remaining an independent implementation.
