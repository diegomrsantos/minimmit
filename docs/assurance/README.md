# Assurance

This directory describes how Minimmit crates track assurance.

The goal is to show that each Rust crate implements its scoped part of
Minimmit as specified by the paper, without overstating what has actually been
proven.

## Layers

- The paper defines the protocol model and proves theorem-level properties of
  that model.
- A Rust crate implements part of that model.
- Linked evidence supports claims about the crate implementation.

Paper proofs are source anchors, not implementation evidence. A model-only
check is also not implementation evidence unless it is connected to Rust
behavior.

## Ledger

Each crate owns its own assurance ledger. The core crate ledger is
`crates/core/assurance.yaml`.

The ledger tracks evidence existence, not evidence quality or completeness.
`evidenced` means that at least one implementation evidence artifact is linked
for the claim. It does not mean the protocol theorem has been re-proven for the
Rust crate.

Keep entries small:

- precise claim
- precise paper anchor
- claim status
- linked evidence artifacts, when they exist

Do not add target evidence plans, coverage essays, or subjective confidence
scores.
