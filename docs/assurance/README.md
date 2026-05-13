# Assurance

This directory tracks the assurance case for the Rust core implementation of
baseline Minimmit.

The goal is to show that the Rust core implements baseline Minimmit as
specified by the paper, without overstating what has actually been proven.

## Layers

- The paper defines the protocol model and proves theorem-level properties of
  that model.
- The Rust core implements part of that model.
- Linked evidence supports claims about the Rust implementation.

Paper proofs are source anchors, not implementation evidence. A model-only
check is also not implementation evidence unless it is connected to Rust
behavior.

## Ledger

`baseline.yaml` is the structured claim ledger for baseline Minimmit from the
paper's formal specification and Algorithm 1.

The ledger tracks evidence existence, not evidence quality or completeness.
`evidenced` means that at least one implementation evidence artifact is linked
for the claim. It does not mean the protocol theorem has been re-proven for the
Rust implementation.

Keep entries small:

- precise claim
- precise paper anchor
- implementation status
- linked evidence artifacts, when they exist

Do not add target evidence plans, coverage essays, or subjective confidence
scores.
