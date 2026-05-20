# Testing Doctrine

This document is the source-backed testing doctrine for Minimmit. It preserves
research lessons that are useful for the project without turning `AGENTS.md`,
skills, or assurance ledgers into long research notes.

`crates/core/assurance.yaml` remains the claim and evidence ledger. It should
not carry testing roadmaps, source surveys, or confidence essays.

## Current Doctrine

The preferred Minimmit testing loop is:

```text
paper claim -> explicit input -> core step -> semantic effect -> trace/replay -> evidence
```

Use the smallest deterministic test that gives useful confidence. For current
`minimmit-core` work, this usually means pure Rust tests over typed protocol
values. Once the deterministic core exposes an input boundary, protocol
scenario tests should drive that boundary directly.

When a core output depends on shell work becoming durable, tests should model
that progress as an explicit lifecycle input. Do not let tests assume that a
storage command completed unless the scenario feeds the corresponding
completion input back into the core. See
[Core And Shell Lifecycle](core-shell-lifecycle.md) for the lifecycle boundary.

When sync, store, simulation, or shell work models overload, tests should assert
bounded degradation through deterministic observations. See
[Bounded Degradation](bounded-degradation.md) for overload vocabulary and
testing implications.

Each test should protect behavior that Minimmit owns: protocol obligations,
project policy, validation, state transitions, or observable semantic output.
Do not add, move, or preserve tests whose failure would primarily report a
change in language, library, dependency, derived, or otherwise mechanical
behavior rather than a change in Minimmit behavior.

Test helpers should name meaningful domain setup, not scalar construction.
Use direct public constructors such as `BlockId::new`, `ViewNumber::new`,
`ValidatorId::new`, and `TransactionId::new` at the call site instead of
wrapping them in test-only helpers.

## Test Layers

- Small claim tests are the default. They cover thresholds, typed
  construction, duplicate rejection, distinct sender counting, deterministic
  ordering, and precise errors.
- Input-step tests should drive one explicit protocol event or lifecycle input
  through the core and inspect the returned protocol-visible output.
- Scenario tests should feed a short ordered input trace, record step outcomes,
  and assert a named protocol story.
- Overload scenario tests should assert public deterministic observations such
  as admission, rejection, expiry, queue age, or tracked-state pressure, not
  private queue internals.
- Replay fixtures should be added only after scenario shape stabilizes. A
  replayed failure must include enough metadata to reproduce it exactly.
- Model conformance should map named model actions into Rust events and compare
  a small semantic projection, not every transient implementation field.
- Bounded event search should come after local scenario tests are useful. Start
  with seeded shuffle or small exhaustive event bags before richer scheduling.

Only input-step and scenario testing are active Codex skill guidance for now.
Replay fixtures, property packs, model conformance, and bounded search should
get separate skills when the implementation reaches those layers.

## Properties

Use two property classes:

- `always`: safety properties. A violation is a candidate bug when the property
  is high-level, protocol-meaningful, and independently confirmable.
- `sometimes`: reachability or coverage signals. Reaching one shows the harness
  exercised useful protocol territory, but does not prove correctness.

Good `always` properties for Minimmit should be stated over protocol facts such
as notarization, nullification, finalization, view progression, signer
membership, and distinct sender counting. Avoid primary bug oracles based on
private buffers, queue depths, scheduler internals, or implementation-specific
interpretation.

Good `sometimes` properties include observing an M-notarization,
L-notarization, nullification, higher view, non-genesis finalization, timeout
path, or rejected conflicting evidence.

When the implementation reaches property-pack testing, keep the first pack
small, high-level, and confirmable. `always` properties should be the only bug
threshold. `sometimes` properties should stay coverage signals.

## Replay Metadata

Any randomized, scheduled, or searched run that reports a candidate must record:

- scenario family or test name
- ordered input trace, or enough scheduler data to reconstruct it
- seed when randomness is used
- scheduler name and version when scheduling is used
- triggered `always` properties
- reached `sometimes` properties
- implementation commit when the result leaves a local test failure report

Unreplayable failures are not acceptable evidence.

Start replay and search work with short explicit traces. Move next to seeded
shuffle or small exhaustive event bags. Richer scheduling, broad simulation,
chaos, Antithesis integration, Jepsen-style external testing, and VOPR-style
infrastructure should wait until local `Event`/`Lifecycle` -> `Core` -> `Ready`
scenarios prove the need.

For Quint or model conformance, map named model actions into Rust events at a
protocol-local boundary. Compare semantic projected state, not every transient
implementation field. Do not count model-only checks as Rust implementation
evidence unless the trace drives Rust behavior.

## Apply Now

- Keep `minimmit-core` deterministic and free of hidden IO, wall-clock time,
  randomness, async scheduling, networking, storage engines, and production
  shell behavior.
- Represent shell persistence completion as explicit input when a protocol
  output depends on it.
- Assert persistence-gated behavior through protocol-visible outputs, not DB
  mechanics or shell queues.
- Assert bounded degradation through explicit observations when overload policy
  belongs to the crate under test.
- Prefer behavior tests through public APIs and protocol-facing outputs.
- Keep tests readable and actionable from the test name plus assertion output.
- Remove mechanical coverage instead of relocating it when cleanup scope
  permits.
- Let `crates/core/assurance.yaml` drive protocol test priority.
- Treat model-only checks as design evidence unless they are connected to Rust
  behavior.
- Use explicit evidence gaps rather than implying coverage from partial tests.

## Defer

Do not add these until the local `Event`/`Lifecycle` -> `Core` -> `Ready` shape
and scenario tests justify them:

- broad simulation or chaos framework
- Antithesis integration
- Jepsen-style external system testing
- full TigerBeetle VOPR-style infrastructure
- generic deterministic runtime
- coverage gates or flake dashboards
- broad property-test or fuzz dependencies in `minimmit-core`

## Source Map

Google testing:

- [Software Engineering at Google, Testing Overview](https://abseil.io/resources/swe-book/html/ch11.html):
  small deterministic tests by default, risk-driven investment, and a bias
  toward fast actionable feedback.
- [Unit Testing](https://abseil.io/resources/swe-book/html/ch12.html):
  test behavior through public APIs and keep tests maintainable.
- [Test Doubles](https://abseil.io/resources/swe-book/html/ch13.html):
  prefer realistic behavior and avoid brittle interaction tests when state
  tests will do.
- [Larger Testing](https://abseil.io/resources/swe-book/html/ch14.html):
  larger tests are useful but expensive, so reserve them for risks small tests
  cannot cover.
- [Continuous Integration](https://abseil.io/resources/swe-book/html/ch23.html):
  preserve signal quality and keep failures actionable.
- [Just Say No to More End-to-End Tests](https://testing.googleblog.com/2015/04/just-say-no-to-more-end-to-end-tests.html):
  broad end-to-end tests should not be the primary correctness strategy.
- [Flaky Tests at Google](https://testing.googleblog.com/2016/05/flaky-tests-at-google-and-how-we.html):
  flaky tests damage trust in the suite.
- [Test Failures Should Be Actionable](https://testing.googleblog.com/2024/05/test-failures-should-be-actionable.html):
  a failing test should point directly at the broken behavior.

Private protocol-event research:

- [protocol-event-lab README](https://github.com/diegomrsantos/protocol-event-lab/blob/main/README.md):
  protocol-visible events, quiescence, semantic properties, and replayable
  traces are the useful middle layer between pure models and whole-system
  fault platforms.
- [protocol-event-lab thesis](https://github.com/diegomrsantos/protocol-event-lab/blob/main/docs/thesis.md):
  schedule protocol-visible events rather than low-level runtime tasks.
- [protocol-event-lab integration challenges](https://github.com/diegomrsantos/protocol-event-lab/blob/main/docs/integration-challenges.md):
  adapters should expose narrow event injection, quiescence, state snapshots,
  and invariant evaluation.
- [protocol-event-lab Antithesis-style properties](https://github.com/diegomrsantos/protocol-event-lab/blob/main/docs/decisions/0011-antithesis-style-simplex-properties-for-m3.md):
  keep first property packs small, high-level, and confirmable.
- [proto-core-lab Minimmit scope](https://github.com/diegomrsantos/proto-core-lab/blob/main/docs/decisions/0002-minimmit-v0-scope-and-test-shape.md):
  start with a small synchronous core and import only deterministic harness
  ideas, not a generic framework.
- [proto-core-lab protocol-core testing direction](https://github.com/diegomrsantos/proto-core-lab/blob/main/docs/decisions/0003-protocol-core-testing-direction.md):
  test through explicit events, explicit transitions, and semantic effects.
- [proto-core-lab composed submachines](https://github.com/diegomrsantos/proto-core-lab/blob/main/docs/decisions/0004-composed-deterministic-submachines.md):
  split complex protocol cores into deterministic submachines with typed
  internal event flow only when the concern is real.
- [proto-core-lab explicit output lifecycle](https://github.com/diegomrsantos/proto-core-lab/blob/main/docs/decisions/0006-explicit-output-lifecycle.md):
  represent shell work completion explicitly instead of hiding output
  lifecycle state inside the core.
- [proto-core-lab deterministic cores and shells](https://github.com/diegomrsantos/proto-core-lab/blob/main/docs/decisions/0007-framework-direction-deterministic-cores-and-shells.md):
  keep core semantics deterministic while shells execute and acknowledge hard
  outputs such as persistence.
- [proto-core-lab Quint boundary](https://github.com/diegomrsantos/proto-core-lab/blob/main/docs/quint-connect-boundary.md):
  keep protocol-specific Quint replay mappings local until more than one
  protocol proves the abstraction.

Model-based and system testing:

- [Quint model-based testing](https://quint.sh/docs/model-based-testing):
  distinguish model validation from implementation conformance, and use traces
  to connect the two.
- [Informal Systems trace generation](https://mbt.informal.systems/docs/tla_basics_tutorials/generating_traces.html):
  generated model traces can seed implementation tests when mapped through a
  clear boundary.
- [Antithesis properties](https://antithesis.com/docs/properties_assertions/properties/):
  `always` and `sometimes` properties are a useful vocabulary for safety and
  reachability.
- [Antithesis deterministic simulation testing](https://antithesis.com/docs/resources/deterministic_simulation_testing/):
  deterministic replay matters more than one-off fault discovery.
- [TigerBeetle VOPR](https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/internals/vopr.md):
  record enough seed, scheduler, and trace metadata to reproduce simulation
  failures exactly.
- [TigerBeetle simulation testing for liveness](https://tigerbeetle.com/blog/2023-07-06-simulation-testing-for-liveness/):
  separate safety exploration from liveness scenarios that heal a correct core.
- [FoundationDB testing](https://apple.github.io/foundationdb/testing.html):
  deterministic simulation and fault injection are powerful after the system
  has a suitable controlled execution boundary.
- [Stateright](https://github.com/stateright/stateright):
  bounded model checking is useful for small protocol models and state spaces.
- [Jepsen](https://jepsen.io/):
  test named correctness claims through histories and checkers, not vague
  system behavior.
- [P](https://p-org.github.io/P/):
  event-oriented state machines are a useful way to model distributed protocol
  behavior.
- [Molly / Lineage-Driven Fault Injection](https://people.ucsc.edu/~palvaro/molly.pdf):
  fault exploration should be guided by the dependencies needed to violate a
  meaningful outcome.
