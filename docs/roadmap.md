# Roadmap

This roadmap describes planned implementation and evidence milestones. It is a
planning aid, not a release promise. Milestones are named by concern so they can
move independently from SemVer crate releases.

## Current Direction

Minimmit should grow from a deterministic protocol core into a workspace of
small crates that can be reviewed together:

```text
crates/
  types/
  core/
  sync/
  store/
  sim/
  shell-commonware/
```

Keep the protocol core deterministic and reviewable. Runtime concerns such as
networking, storage engines, wall-clock timers, and production orchestration
belong outside `minimmit-core`.

When core behavior depends on shell completion, model that as a concrete
core-visible input only when the protocol needs it. Ready outputs may include
storage and network work in one batch; when both refer to the same artifact,
the shell persists first and only then releases the network output. See
[Core And Shell Boundary](core-shell-boundary.md) for the canonical boundary.

When sync, store, simulation, or shell work can be overloaded, make degradation
bounded, observable, and replayable instead of hiding debt in queues. See
[Bounded Degradation](bounded-degradation.md) for the canonical overload
vocabulary.

## Milestones

### baseline-core-v0

Purpose: establish the first assured `minimmit-core` groundwork.

Includes:

- `n >= 5f + 1` configuration validation
- M-notarization and nullification threshold `2f + 1`
- L-notarization threshold `n - f`
- deterministic validator identity and committee membership
- distinct valid sender counting
- baseline block, transaction, vote, nullify, notarization, and proposal data
- deterministic construction and ordering rules
- parent selection from M-notarized prior views
- proposal validation for parent and skipped-view evidence
- minimal deterministic core boundary

Excludes:

- async runtime integration
- networking
- storage engines
- wall-clock timers
- fetch policy
- real cryptography
- full Algorithm 1 state transitions

Done when:

- the milestone issues are closed
- `cargo test` passes from a clean checkout
- touched claims in `crates/core/assurance.yaml` are current
- no runtime, network, storage, or timer behavior has entered core

### release-policy-v0

Purpose: document how milestones, releases, crate versions, and publishing
relate before the first release exists.

Includes:

- release policy
- a changelog with an `Unreleased` section
- README links to release, dependency, and assurance policy
- explicit guidance that useful crates, not closed milestones, trigger releases

Done when:

- docs explain why current crates remain `0.0.0` and `publish = false`
- docs explain how crate-specific release tags will be named
- docs define the `1.0.0` bar

### core-machine-v0

Purpose: turn `minimmit-core` from validated data and pure rules into a
deterministic state machine.

Includes:

- `Processor`, `Event`, and `Ready`
- storage and network ready output queues
- documented persist-before-broadcast contract for shell-ordered proposal output
- local view state
- observed protocol artifacts
- deterministic ready outputs
- leader proposal trigger
- vote on valid proposal
- timeout modeled as input
- receiving current-view M-notarization
- receiving current-view nullification
- per-view state reset on advancement
- replay tests showing the same ordered input trace produces the same outputs

Excludes:

- durable snapshots
- restart persistence
- database-backed persistence
- shell-side persistence policy
- real network messages
- wall-clock scheduling
- sync or fetch policy
- production node orchestration

Release gate impact:

- This is the earliest plausible point for `minimmit-core-v0.1.0`.
- Release only if the state machine is useful and documented, not merely
  present.

### core-baseline-evidence-v0

Purpose: close the baseline Algorithm 1 evidence loop for the documented core
scope.

Includes:

- one vote per view
- forwarding outputs for new nullifications and M-notarizations
- condition-B nullification
- M-notarization advancement and required vote path
- L-notarization finalization
- deterministic handling of multiple M-notarized blocks in one view
- consistency-related regressions
- view progression evidence
- persistence-sensitive behavior, such as persist-before-dependent-output paths,
  driven through explicit ready outputs and concrete core inputs when shell
  completion must change later core behavior
- explicit liveness evidence gaps where executable evidence is not yet present

Done when:

- every supported baseline claim is evidenced or explicitly deferred
- evidence entries link to executable Rust tests or model conformance checks
- the processor API remains replayable and deterministic
- tests do not assume shell persistence has released dependent network output
  before the ready-output ordering contract is satisfied

### types-boundary-v0

Purpose: extract shared protocol and sync types only after more than one crate
needs them.

Candidate package: `minimmit-types`.

Candidate contents:

- `View`
- `ValidatorId`
- `Digest`
- quorum configuration
- artifact identifiers
- artifact dependencies
- range kinds
- validation wrappers
- invalid reason taxonomy
- verified artifact wrappers

Excludes:

- protocol state machine behavior
- LBAS policy
- storage policy
- speculative abstractions not used by at least two crates

Release gate:

- Release `minimmit-types-v0.1.0` only after at least two crates depend on its
  public types.

### store-v0

Purpose: add deterministic artifact storage for sync and simulation.

Candidate package: `minimmit-store`.

Includes:

- deterministic in-memory artifact lookup
- artifact insertion
- duplicate handling
- missing artifact classification
- artifact count and byte pressure visibility
- retention obligations
- prune refusal while retained artifacts are still required
- tracked artifact cardinality observations
- deterministic iteration where observable ordering matters

Excludes:

- database integration
- filesystem persistence
- async I/O
- production storage tuning
- cache eviction based on wall-clock time

Tests:

- lookup hit and miss
- duplicate artifact insertion
- invalid artifact separation
- retention blocks prune
- prune succeeds after retention is released
- deterministic observable order

Release gate:

- Release `minimmit-store-v0.1.0` only after it can serve sync or simulation as
  a useful deterministic store.

### sync-lbas-v0

Purpose: add the deterministic LBAS synchronizer.

Candidate package: `minimmit-sync`.

Shape:

```text
Observation -> Sync -> Ready
```

Includes:

- missing artifact tracking
- explicit admission outcomes for fetch and delivery work
- bounded request and response queues
- dependency expansion
- verification before delivery
- bounded retry
- work-class priority for current-view and catch-up work
- expiry, rejection, or drop reasons for stale or excess work
- peer targeting
- retention coordination
- pruning coordination
- deterministic overload observations
- deterministic ready outputs for requests and deliveries

Excludes:

- network transport
- runtime timers
- Commonware adapters
- storage engine internals
- protocol safety logic that belongs in `minimmit-core`

Tests:

- missing payload
- skipped proof
- invalid response
- retry bound
- queue item, byte, and age bounds
- expiry and drop semantics
- target selection
- current-view priority
- priority isolation under mixed load
- duplicate delivery
- starvation avoidance
- bounded amplification
- prune coordination

Release gate:

- Release `minimmit-sync-v0.1.0` only after LBAS runs against the store and type
  boundaries with bounded behavior tests.

### sim-v0

Purpose: add deterministic adversarial simulation across core, store, and sync.

Candidate package: `minimmit-sim`.

Includes:

- seeded or serialized scenarios
- deterministic peer/network scheduling
- partition and heal scenarios
- stale timer input
- delayed storage work and persist-before-broadcast scenarios
- invalid-response floods
- missing-artifact storms
- stale backlog expiry
- priority isolation under overload
- hot dependency or resource contention
- equivocation scenarios
- invalid peer responses
- prune-too-early scenarios
- duplicate delivery scenarios
- reproducible failure output

Excludes:

- production networking
- nondeterministic timing
- Commonware shell integration

Release gate:

- Release `minimmit-sim-v0.1.0` only after it runs reproducible adversarial
  scenarios that are useful as evidence.

### comparison-v0

Purpose: justify LBAS against a naive fetch strategy with executable evidence.

Includes:

- naive fetch baseline
- bounded LBAS comparison
- deterministic metrics collection
- scenario-level comparison reports

Metrics:

- fetch count
- bytes requested
- invalid responses
- duplicate deliveries
- rejected, delayed, dropped, and expired work
- maximum queue depth and age
- tracked-state cardinality
- recovery latency
- recovery tail under overload
- starvation
- prune failures

Done when:

- LBAS complexity is supported by executable comparison evidence
- comparison scenarios are reproducible

### api-hardening-v0

Purpose: prepare useful crates for first real releases.

Includes:

- public API review
- accidental export removal
- naming review
- error taxonomy review
- feature flag review
- dependency review
- README and crate docs review
- release-or-stay-private decision for each crate

Done when:

- every useful crate has an explicit release gate decision
- crates that are not useful remain `0.0.0` and `publish = false`
- any crate tagged for release has a changelog entry and release notes

### shell-commonware-v0

Purpose: add Commonware integration after simulator evidence exists.

Candidate package: `minimmit-shell-commonware`.

Includes:

- adapter from runtime events into deterministic crate inputs
- adapter from deterministic ready outputs into runtime actions
- adapter from future shell completion events into concrete core inputs when
  completion affects protocol state
- integration tests that preserve protocol boundaries

Excludes:

- protocol behavior hidden in shell code
- direct DB reads as a substitute for explicit core inputs
- replacing simulator evidence with runtime smoke tests
- production-readiness claims before operational evidence exists

Release gate:

- Keep this crate experimental until its runtime contract is independently
  useful and documented.
