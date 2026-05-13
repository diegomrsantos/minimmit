## Summary

- 

## Scope

- One concern:
- Out of scope:

## Checks

- [ ] `cargo check`
- [ ] `cargo test`

## Protocol And Evidence

- [ ] Protocol behavior names its paper claim or explicit evidence gap.
- [ ] Evidence manifest or PR text reflects the current support level.
- [ ] No production shell concerns added to `crates/core`.

## Review Focus

- Protocol behavior is explicit and reviewable.
- Core logic is not mixed with async, networking, storage, timers, or production
  node concerns.
- Thresholds and sender-counting rules are easy to audit when touched.
