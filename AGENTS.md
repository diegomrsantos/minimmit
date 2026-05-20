# Codex Guidance

This repository is intentionally review-first. Keep future changes small,
literal, and easy to audit.

## Git And Review Flow

- After the bootstrap commit, do not commit directly to protected `main`.
- Use a branch and pull request for every code change.
- Before creating a new work branch, fetch the latest `origin/main`, update the
  local base branch from it, and create the work branch only from that updated
  base. Do not branch from stale `main` or from an old feature branch unless
  the user explicitly asks for that base.
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
- Before opening or merging a pull request, fetch the latest `origin/main` and
  bring the branch up to date so conflicts are resolved locally before GitHub
  merge. Push that update as a normal follow-up commit by default.
- Before opening a pull request, after implementation and local verification,
  spawn a fresh review agent to review the implementation. Address any
  actionable findings, or record why they are deferred, before creating the PR.
- After opening a pull request, do not amend commits or force-push that pull
  request branch unless the user explicitly asks for a history rewrite. Add
  follow-up commits to open pull request branches by default.
- After opening a pull request, do not merge it until the user explicitly asks
  to merge that pull request. A request such as "merge this PR" is sufficient
  when the current PR is unambiguous. General requests to continue, or earlier
  merge approval for another pull request, do not authorize merging a later
  pull request.
- When creating or refining issues, follow `docs/issues.md`; use issue
  templates as forms, not as the canonical policy.
- When creating or refining milestones, follow `docs/milestones.md`.

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
- Use the repo-scoped `minimmit-protocol-scenario-testing` skill for
  event-driven protocol scenarios and semantic-effect tests.
- Identify the paper claim and add a failing test or explicit evidence gap
  before implementing protocol behavior.
- Do not mark a protocol obligation satisfied without executable evidence.

## Rust Style

- Prefer simple, literal names.
- Keep public APIs clear and narrow.
- Avoid unused placeholders.
- Use deterministic data structures when observable ordering matters.
- For tests, follow `docs/testing.md` and the repo-scoped
  `minimmit-test-quality` skill; do not add or preserve mechanical coverage.
- Use the repo-scoped `minimmit-rust-quality` skill when creating or modifying
  Rust implementation code.
- Use the repo-scoped `minimmit-invariant-review` skill when changing
  constructors, state transitions, protocol-facing types, validation, or
  trusted assumptions.
- Use the repo-scoped `minimmit-test-quality` skill when adding or modifying
  tests, test helpers, or regression coverage.
- Use the repo-scoped `minimmit-doc-quality` skill when creating or modifying
  Rust code, public APIs, protocol-facing types, tests, or non-obvious helpers.
