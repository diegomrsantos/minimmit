## Summary

- 

## Scope

- One concern:
- Out of scope:
- Dedicated worktree from updated `main`:

## Checks

- [ ] `cargo check`
- [ ] `cargo test`

## Protocol And Evidence

- [ ] Protocol behavior names its paper claim or explicit evidence gap.
- [ ] If this PR changes protocol state, events, validation, or ready outputs,
      it includes a state-fact matrix or states why none applies.
- [ ] Evidence manifest or PR text reflects the current support level.
- [ ] No production shell concerns added to `crates/core`.

## Review Focus

- Protocol behavior is explicit and reviewable.
- Cross-event paths that establish or consume the same protocol fact are tested.
- Core logic is not mixed with async, networking, storage, timers, or production
  node concerns.
- Thresholds and sender-counting rules are easy to audit when touched.
