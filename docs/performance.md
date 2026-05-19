# Performance Doctrine

This document is the source-backed performance doctrine for Minimmit. It
preserves performance lessons that are useful for the project without turning
`AGENTS.md`, crate READMEs, or assurance ledgers into benchmark plans or
observability policy.

`crates/core/assurance.yaml` remains the claim and evidence ledger. It should
not carry performance roadmaps, source surveys, benchmark results, or confidence
essays.

## Current Doctrine

Performance matters for Minimmit, but the first performance rule is to preserve
the deterministic protocol boundary:

```text
explicit input -> deterministic step -> protocol-visible output -> measured artifact
```

For current `minimmit-core` work, performance review should focus on public API
shape, algorithmic bounds, allocation behavior, deterministic ordering, and
avoiding accidental unbounded work. Correctness, replayability, and protocol
evidence come before optimization.

Do not add metrics, tracing, wall-clock timing, async runtime hooks, or
observability dependencies to `minimmit-core`. The core should expose
deterministic protocol facts. Outer crates can turn those facts into metrics,
traces, logs, benchmark reports, or operational telemetry.

## Performance Layers

- Design-time review is the default. Review collection choices, loop nesting,
  cloning, allocation, sender counting, artifact lookup, and observable ordering
  when protocol-facing behavior changes.
- Focused microbenchmarks should be added only after a hot path or data
  structure choice is stable enough that the benchmark will remain meaningful.
- Deterministic scenario metrics should come after the `Event`/`Lifecycle` ->
  `Core` -> `Ready` boundary can be driven by replayable input traces.
- Sync and store performance evidence should measure bounded behavior under
  deterministic scenarios before claiming production relevance.
- Runtime observability belongs in shell or integration crates that translate
  deterministic outputs into operational metrics, traces, and logs.

Performance work should produce measured artifacts. A claim such as "faster",
"bounded", "low overhead", or "scales better" needs a scenario, input size,
measurement method, and reproducible command or report.

## Metrics

Keep two metric classes separate:

- Protocol and simulation metrics are deterministic measurements produced by
  tests, simulations, or comparison reports. They can support design decisions
  and review.
- Operational telemetry is runtime measurement from a running node. It can
  support production diagnosis, capacity planning, and alerting, but it should
  not become protocol behavior.

The comparison roadmap currently names the first useful protocol-side metrics:

- fetch count
- bytes requested
- invalid responses
- duplicate deliveries
- recovery latency
- starvation
- prune failures

Those metrics belong in deterministic sync, store, simulation, or comparison
work. Prometheus, OpenTelemetry, tracing spans, log schemas, dashboards, and
alerts belong in outer runtime-facing crates.

## Apply Now

- Keep `minimmit-core` dependency-free and free of hidden IO, wall-clock time,
  async scheduling, metrics emitters, tracing spans, and global telemetry state.
- Prefer clear deterministic data structures over faster nondeterministic ones
  when observable ordering matters.
- Review complexity and allocation behavior when changing public APIs,
  constructors, sender counting, evidence aggregation, artifact lookup, or
  deterministic ordering.
- Use small local tests or hand-checkable examples to guard against accidental
  unbounded behavior before adding benchmark infrastructure.
- Record explicit performance questions as deferred work when they cannot yet be
  measured through a stable deterministic boundary.

## Defer

Do not add these until the local `Event`/`Lifecycle` -> `Core` -> `Ready` shape
and later sync/store/simulation work justify them:

- Criterion or other benchmark dependencies in the workspace
- benchmark CI gates or performance dashboards
- Prometheus, OpenTelemetry, `tracing`, metrics, or logging dependencies in
  `minimmit-core`
- production latency, throughput, or SLO claims
- wall-clock scenario timing as protocol evidence
- broad profiling or optimization passes without a named hot path

## Source Map

Protocol and project context:

- [Minimmit: Fast Finality with Even Faster Blocks](https://arxiv.org/abs/2508.10862):
  the protocol is motivated by low-latency consensus, but implementation
  performance claims should stay tied to reproducible scenarios.
- [Minimmit roadmap](roadmap.md):
  deterministic comparison metrics first appear in the `comparison-v0` milestone
  after core, store, sync, and simulation shape exists.
- [Minimmit dependency policy](dependencies.md):
  metrics, tracing, runtime, wall-clock timers, and production orchestration
  belong outside `minimmit-core`.

Performance engineering:

- [The Rust Performance Book, Profiling](https://nnethercote.github.io/perf-book/profiling.html):
  profile to find hot code before optimizing.
- [The Rust Performance Book, General Tips](https://nnethercote.github.io/perf-book/general-tips.html):
  optimize hot code, and prefer algorithm or data-structure improvements over
  low-level tuning when they apply.
- [Criterion.rs Getting Started](https://bheisler.github.io/criterion.rs/book/getting_started.html):
  use a benchmark harness only when there is a stable target worth measuring.
- [Thinking Methodically About Performance](https://queue.acm.org/detail.cfm?id=2413037):
  diagnose performance with evidence and resource-oriented methods rather than
  guesswork.

Distributed systems and observability:

- [Google SRE, Monitoring Distributed Systems](https://sre.google/sre-book/monitoring-distributed-systems/):
  operational monitoring commonly focuses on latency, traffic, errors, and
  saturation.
- [The Tail at Scale](https://research.google/pubs/the-tail-at-scale/):
  distributed systems need tail-latency awareness, not only average latency.
- [Dapper](https://research.google/pubs/dapper-a-large-scale-distributed-systems-tracing-infrastructure/):
  distributed tracing is valuable for runtime diagnosis but has overhead,
  sampling, and deployment concerns.
- [OpenTelemetry Observability Primer](https://opentelemetry.io/docs/concepts/observability-primer/):
  traces, metrics, and logs are runtime observability signals.
- [OpenTelemetry Semantic Conventions](https://opentelemetry.io/docs/concepts/semantic-conventions/):
  use standard telemetry vocabulary when runtime instrumentation exists.
- [Prometheus Instrumentation](https://prometheus.io/docs/practices/instrumentation/):
  instrumentation should avoid cardinality traps and benchmark hot-path impact.
- [The Zen of Prometheus](https://prometheus.io/docs/practices/the_zen/):
  instrumentation is important, but unbounded labels and cardinality explosions
  make metrics unsafe.
- [FoundationDB Simulation and Testing](https://apple.github.io/foundationdb/testing.html):
  deterministic simulation can coexist with performance testing while keeping
  failures reproducible.
