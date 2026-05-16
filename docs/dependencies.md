# Dependency Policy

Minimmit should keep dependencies small, explicit, and easy to audit. The
default for protocol work is to use `std` and local types.

## Core Crate

`minimmit-core` is dependency-free by default. Add a dependency to the core
crate only when it is narrow, deterministic, and easier to review than the
equivalent local implementation.

Do not add these concerns to `minimmit-core` through dependencies:

- async runtime integration
- networking
- storage engines
- wall-clock timers
- metrics or tracing infrastructure
- production node orchestration
- real cryptography backends
- serialization format commitments

Those concerns belong in outer crates that translate between runtime systems
and the deterministic core input/output model.

## Dev Dependencies

Test-only dependencies are acceptable when they improve assurance evidence or
make protocol tests clearer. Keep them scoped to the crate that needs them, and
do not let test helpers shape public APIs or protocol behavior.

## Review Checklist

Every dependency change should explain:

- why `std` or a small local type is not enough
- whether the dependency affects protocol behavior
- whether behavior remains deterministic and replayable
- the size and relevance of transitive dependencies
- license, maintenance, and minimum supported Rust version impact

Keep dependency changes separate from unrelated protocol, refactor, CI, or
formatting work unless the protocol change cannot be reviewed without the
dependency.

## Lockfile

Do not add a `Cargo.lock` as a standalone policy change while this workspace is
library-only. Revisit this when the repository adds binaries, release artifacts,
or CI rules that require locked dependency resolution.
