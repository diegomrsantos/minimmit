# Release Policy

Minimmit uses milestones and assurance ledgers for pre-release progress, and
uses SemVer releases only when a crate has a useful public contract.

This repository is currently pre-release. `minimmit-core` is versioned
`0.0.0`, has `publish = false`, and no crate in this workspace should be
published until it is useful as a dependency.

## Terms

- A milestone is a work and evidence checkpoint. Closing a milestone does not
  imply a tag, release, or published crate.
- A tag is an immutable source snapshot. Tags are useful only when the snapshot
  is worth referring to outside normal git history.
- A release is a documented, usable snapshot for downstream readers or users.
- Publishing is making a crate available through a registry such as crates.io.
  Publishing should wait for an actual external dependency need.
- A stable public contract is the set of APIs, documented behavior, formats,
  and compatibility promises that users can rely on.

## Policy

- Keep crates at `0.0.0` and `publish = false` until they have a useful public
  contract.
- Do not create a SemVer release just because a milestone closes.
- Version crates independently. A crate can reach `1.0.0` only when that
  crate's public contract is stable.
- Use crate-specific tags for crate releases, such as
  `minimmit-core-v0.1.0`.
- Avoid a plain repository-wide `v1.0.0` until every supported public surface in
  that release is stable or explicitly excluded.
- Keep breaking changes visible in the changelog and release notes even before
  `1.0.0`.

## Public Contract

For Minimmit crates, the public contract includes more than Rust item
signatures:

- exported types, functions, traits, errors, and feature flags
- documented deterministic state machine behavior
- protocol behavior exposed by `minimmit-core`
- artifact identifiers, dependency rules, verification results, and invalid
  reason taxonomy
- assurance ledger schema and claim status meaning
- replay or simulation formats that are documented as supported
- store retention and prune refusal semantics
- sync retry, prioritization, peer targeting, and delivery guarantees

Private modules, unpublished crates, and docs explicitly marked experimental are
outside the stable contract.

## Current Milestone

`baseline-core-v0` is a pre-release implementation milestone for the first
assured `minimmit-core` slice.

When this milestone closes, do not tag or publish a release. The outcome is
assured groundwork for later core behavior.

## First Useful Releases

`minimmit-core-v0.1.0` may be created when:

- it exposes a usable deterministic `Event -> Core -> Ready` state machine
- it supports a meaningful baseline protocol slice
- docs explain supported and unsupported behavior
- assurance entries for the supported slice are current
- `cargo test` passes from a clean checkout

`minimmit-types-v0.1.0` may be created when:

- at least two crates depend on its public types
- the exported type boundary is narrow and documented
- it is not a speculative dumping ground

`minimmit-store-v0.1.0` may be created when:

- it can serve sync or simulation with deterministic lookup
- retention and prune behavior are tested and documented

`minimmit-sync-v0.1.0` may be created when:

- LBAS runs against the store and type boundaries
- bounded behavior is covered by tests

`minimmit-sim-v0.1.0` may be created when:

- it runs reproducible adversarial scenarios
- it produces evidence that changes review decisions

Do not publish any crate to crates.io until there is an actual external
dependency need.

## 1.0.0 Bar

A crate reaches `1.0.0` only when its public contract is stable under SemVer.

For each `1.0.0` candidate:

- exported APIs and errors are intentional
- documented behavior is stable
- public dependencies are acceptable for a stable contract
- tests and evidence cover the documented scope
- known release-blocking evidence gaps are closed or out of scope
- migration notes exist for users coming from `0.x`
- breaking changes after release are expected to require a major version bump

Production-node readiness is not automatically required for every library
crate's `1.0.0`. A top-level repository `v1.0.0` must not imply unsupported
production readiness.

## Release Checklist

Before creating any release tag:

- confirm the crate is useful as a dependency or audit snapshot
- confirm `cargo test` passes from a clean checkout
- confirm assurance links are current for supported protocol behavior
- confirm `CHANGELOG.md` has an entry for the release
- confirm release notes state supported behavior, experimental behavior, and
  known gaps
- confirm breaking changes are called out explicitly
- confirm `publish = false` remains in place unless publishing is intentional

## External References

- Semantic Versioning: <https://semver.org/>
- Cargo package publishing: <https://doc.rust-lang.org/cargo/reference/publishing.html>
- Cargo manifest fields: <https://doc.rust-lang.org/cargo/reference/manifest.html>
- Cargo SemVer compatibility: <https://doc.rust-lang.org/cargo/reference/semver.html>
