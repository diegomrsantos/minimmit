---
name: minimmit-doc-quality
description: Use for writing or reviewing Rust docs, public API docs, protocol-facing code comments, test readability, examples, # Errors, # Panics, # Safety, and documentation quality in Minimmit.
metadata:
  short-description: Rust documentation discipline for Minimmit changes
---

# Minimmit Doc Quality

## Use This Skill When

Use this skill whenever creating, modifying, or reviewing Rust code, public
APIs, protocol-facing types, tests, test helpers, examples, or non-obvious
private helpers.

For protocol behavior, also use `minimmit-protocol-tdd`. This skill covers
documentation quality; it is not protocol evidence.

## Before Finishing

Make a small documentation pass over the files you touched:

1. Identify every function, method, type, trait, test helper, and private
   helper you created or changed.
2. Add or update rustdoc for public items, protocol-facing concepts, and any
   non-obvious private helper contract created in the change.
3. Add ordinary comments only for non-obvious invariants, ordering, policy,
   safety reasoning, or protocol assumptions.
4. Remove or avoid comments that restate signatures, assertions, or obvious
   control flow.
5. Update stale nearby docs when behavior, names, or contracts change.

## Rustdoc Rules

- Start public rustdoc with a concise one-line summary.
- Add details only when they help a reviewer or API user understand purpose,
  important arguments, return meaning, invariants, or observable contracts.
- Use `# Errors` for fallible APIs and describe when errors occur.
- Use `# Panics` for intentional panics that callers can trigger.
- Use `# Safety` for unsafe APIs and state the caller obligations.
- Use intra-doc links for relevant local types and methods, but do not
  over-link the item currently being documented.
- Add examples when they clarify why or how an API is used. Keep examples
  small and testable when practical.
- Hide unhelpful implementation details from public docs unless callers need
  them to use the item correctly.

## Comments

- Prefer clear code over comments.
- Comment non-obvious invariants, deterministic ordering, policy choices,
  safety reasoning, and protocol assumptions.
- Do not duplicate full protocol rule lists in comments. Link to canonical
  docs, evidence, or tests when a local reminder is enough.
- Every unsafe block must have a nearby `SAFETY:` comment explaining why the
  block is valid and which invariants it relies on.

## Tests

- Test names should state the behavior under test.
- Test comments should identify the protocol claim, scenario, or regression
  risk when the name and setup are not enough.
- Keep helpers readable enough that assertions remain the focus.

## Context Control

- Keep this pass local to touched code unless stale nearby docs would mislead
  a reviewer.
- Do not add `missing_docs`, CI gates, or broad lint changes for this skill
  unless the task explicitly asks for enforcement.
- Read the full references only for documentation-focused work or when unsure
  about a specific rustdoc convention.

## References

- Rustdoc Book, "How to write documentation":
  <https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html>
- Rust API Guidelines, "Documentation":
  <https://rust-lang.github.io/api-guidelines/documentation.html>
- Standard Library Developers Guide, "Writing documentation":
  <https://std-dev-guide.rust-lang.org/development/how-to-write-documentation.html>
- Standard Library Developers Guide, "Safety comments":
  <https://std-dev-guide.rust-lang.org/policy/safety-comments.html>
- Rustdoc Book, "Linking to items by name":
  <https://doc.rust-lang.org/rustdoc/write-documentation/linking-to-items-by-name.html>
- Rustdoc Book, "Rustdoc-specific lints":
  <https://doc.rust-lang.org/rustdoc/lints.html>
