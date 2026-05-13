# Codex Guidance

This repository is intentionally review-first. Keep future changes small,
literal, and easy to audit.

## Git And Review Flow

- After the bootstrap commit, do not commit directly to protected `main`.
- Use a branch and pull request for every code change.
- Do not prefix branch names or pull request titles with `codex` or `[codex]`.
- Use semantic commit messages for commits and pull request titles, such as
  `docs: expand README motivation`, `fix: correct threshold counting`, or
  `feat: add vote handling`.
- Name branches by change type and concern, such as `docs/readme-motivation`,
  `fix/threshold-counting`, or `feat/vote-handling`.
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

## Protocol TDD And Evidence

- For Minimmit protocol behavior, use the repo-scoped
  `minimmit-protocol-tdd` skill.
- Follow paper-driven TDD: identify the protocol claim, add a failing test or
  explicit evidence gap, then implement the smallest deterministic behavior.
- Treat `proto-core-lab` as architecture and test-harness inspiration, not as a
  semantic authority.
- Keep evidence manifests aligned with implemented behavior.
- Do not mark a protocol obligation as satisfied without executable evidence.

## Rust Style

- Prefer simple, literal names.
- Keep public APIs clear and narrow.
- Avoid unused placeholders.
- Use deterministic data structures when observable ordering matters.
