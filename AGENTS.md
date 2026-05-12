# Codex Guidance

This repository is intentionally review-first. Keep future changes small,
literal, and easy to audit.

## Git And Review Flow

- After the bootstrap commit, do not commit directly to protected `main`.
- Use a branch and pull request for every code change.
- Keep pull requests small and reviewable.
- One pull request must cover one concern.
- Do not mix docs, refactors, protocol behavior, test infrastructure, CI, or
  formatting unless one change truly depends on the other.

## Protocol Boundaries

- Keep protocol logic in the core crate.
- Do not place protocol logic inside async, network, storage, or timer
  machinery.
- Keep protocol behavior explicit and reviewable in the state machine.
- Avoid broad framework work unless the core crate forces it.

## Rust Style

- Prefer simple, literal names.
- Keep public APIs clear and narrow.
- Avoid unused placeholders.
- Use deterministic data structures when observable ordering matters.
