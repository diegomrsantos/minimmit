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

## Dependency Policy

- Follow `docs/dependencies.md` for dependency changes.
- Treat any new `minimmit-core` dependency as an explicit review exception
  that needs justification.

## Duplication

- Prefer one canonical source for each policy, rule, helper, or behavior.
- Do not duplicate full rule lists, algorithms, fixtures, or explanatory text
  across code, tests, docs, or agent instructions.
- When another location needs the same guidance, link to the canonical source
  or add the smallest local reminder.
- Repeat logic or text only when the target must stand alone for a distinct
  audience or when explicit protocol evidence is clearer than indirection.

## Protocol TDD And Evidence

- Use the repo-scoped `minimmit-protocol-tdd` skill for protocol behavior.
- Identify the paper claim and add a failing test or explicit evidence gap
  before implementing protocol behavior.
- Do not mark a protocol obligation satisfied without executable evidence.

## Rust Style

- Prefer simple, literal names.
- Keep public APIs clear and narrow.
- Avoid unused placeholders.
- Use deterministic data structures when observable ordering matters.
