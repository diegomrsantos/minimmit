# Protocol Claims Ledger

This document is the canonical place for paper-derived Minimmit protocol claims
before executable evidence exists. It is intentionally not an implementation
plan. Each protocol PR should name the claim it touches, add executable
evidence, and update the status here or in a future evidence manifest.

Source of truth: the Minimmit paper, arXiv:2508.10862.

## Ledger Rules

- Keep claim ids stable once implementation or tests refer to them.
- Do not mark a claim satisfied without executable evidence.
- Record unsupported behavior as `deferred`; do not remove it silently.
- Prefer paper section, definition, lemma, or algorithm labels as source
  anchors.
- Treat Commonware and `proto-core-lab` as comparison material, not as
  authority over the paper.

## Scope

This initial ledger covers the baseline Minimmit protocol from the paper's
formal specification and Algorithm 1. It does not cover E-Minimmit, erasure
coding, threshold signatures, compressed nullifications, or other optimizations.
Those should get separate claims when they become implementation scope.

## Assumptions

| Id | Assumption | Source Anchor | Status |
| --- | --- | --- | --- |
| `ASM-BASELINE` | The implementation target is baseline Minimmit, not E-Minimmit or later optimizations. | Paper sections `formal`, `alg1`; excludes `erasurespec`, `alg2`, `opt`. | Active |
| `ASM-FAULTS` | The validator set has size `n`, at most `f` Byzantine processors, and `n >= 5f + 1`. | Setup; Introduction. | Deferred evidence |
| `ASM-CRYPTO` | Messages are authenticated and signed by processors in the validator set. Initial core tests may model signatures as signer identities, but membership and distinctness still matter. | Setup cryptographic assumptions; formal message definitions. | Deferred evidence |
| `ASM-DETERMINISTIC-CORE` | Async runtime, networking, storage, wall clocks, and real cryptography stay outside the protocol core. Timers and message receipt are modeled as inputs. | Repository design constraint; Algorithm 1 timer/message inputs. | Active |
| `ASM-GENESIS` | Local state starts with the genesis block and M/L notarizations for genesis. | Formal specification, local variable `S`. | Deferred evidence |

## Protocol Claims

| Id | Claim | Source Anchor | Target Evidence | Status |
| --- | --- | --- | --- | --- |
| `MM-THRESHOLDS` | M-notarization and nullification thresholds are `2f + 1`; L-notarization threshold is `n - f`; configuration must satisfy `n >= 5f + 1`. | Intuition; formal definitions of M-notarizations, L-notarizations, and nullifications. | Unit tests for config validation and threshold values. | Deferred |
| `MM-DISTINCT-VALID-SENDERS` | Votes and nullify messages count only distinct valid processors from the validator set. Duplicates and non-members must not inflate quorums. | Formal definitions of M-notarizations, L-notarizations, and nullifications. | Unit tests for quorum collection. | Deferred |
| `MM-PARENT-SELECTION` | `SelectParent(S, v)` returns a block from the greatest prior view with an M-notarization, breaking ties by lexicographically least block. | Formal definition of `SelectParent`. | Unit tests with multiple M-notarized blocks and views. | Deferred |
| `MM-VALID-PROPOSAL` | A valid proposal for view `v` contains precisely one leader-signed view-`v` block, an M-notarization for its parent, and nullifications for every skipped view. | Formal definition of valid proposal. | Transition tests for valid and invalid proposals. | Deferred |
| `MM-FORWARDING` | Processors forward newly received nullifications and M-notarizations. L-notarization forwarding is not required for liveness. | Formal definition of new nullifications/notarizations; Algorithm 1 `forwardN`, `forwardML`. | Transition tests for ready outputs on new evidence. | Deferred |
| `MM-LEADER-PROPOSE` | A leader that has not proposed in its current view proposes a child of `SelectParent(S, v)` and records that it has proposed. | Algorithm 1 `sendblock`. | Transition tests for leader proposal readiness. | Deferred |
| `MM-VOTE-VALID-PROPOSAL` | A processor votes for a valid proposal only if it has not already voted and has not nullified the current view. | Algorithm 1 `votecheck`, `vote1`. | Transition tests for voting and suppression after vote/nullify. | Deferred |
| `MM-TIMEOUT-NULLIFY` | On timeout in a view, a processor that has neither voted nor nullified sends `nullify(v)`. | Algorithm 1 `timeout1`, `timeout2`. | Transition tests with timer events. | Deferred |
| `MM-ADVANCE-NULLIFICATION` | Receiving a nullification for the current view advances to the next view and resets per-view state. | Algorithm 1 `newview1`. | Transition tests for view advance and reset state. | Deferred |
| `MM-ADVANCE-M-NOTARIZATION` | Receiving an M-notarization for a current-view block advances to the next view; if the processor has not voted or nullified, it votes for that block before advancing. | Algorithm 1 `vote2`, `newview2`; intuition "Adding an extra instruction to send votes". | Transition tests for vote-before-advance behavior. | Deferred |
| `MM-CONDITION-B-NULLIFY` | After voting, a processor can nullify the current view upon `2f + 1` distinct messages that are each either `nullify(v)` or a vote for a different current-view block. | Algorithm 1 `beginN`, `endN`, `sendN`. | Transition tests for mixed nullify/conflicting-vote evidence. | Deferred |
| `MM-FINALIZE-L` | A new L-notarization finalizes the notarized block. | Algorithm 1 finalization step; formal L-notarization definition. | Transition tests for finalization ready output. | Deferred |

## Safety And Liveness Claims

| Id | Claim | Source Anchor | Target Evidence | Status |
| --- | --- | --- | --- | --- |
| `MM-ONE-VOTE-PER-VIEW` | A correct processor votes for at most one block in each view. | Lemma `singlevote`. | Unit and transition tests over all vote paths. | Deferred |
| `MM-X1-L-EXCLUDES-CONFLICTING-M` | If a block receives an L-notarization, no distinct block in the same view receives an M-notarization. | Lemma `X1`. | Quorum intersection tests and model scenarios. | Deferred |
| `MM-X2-L-EXCLUDES-NULLIFICATION` | If a block receives an L-notarization, its view does not receive a nullification. | Lemma `X2`. | Scenario tests for nullification conditions after L evidence. | Deferred |
| `MM-CONSISTENCY` | Inconsistent blocks cannot both be finalized. | Consistency lemma. | Cluster/model scenarios and replay fixtures. | Deferred |
| `MM-VIEW-PROGRESSION` | Correct processors progress through views by receiving either a nullification or an M-notarization. | Liveness section, progression-through-views lemma. | Cluster scenario tests. | Deferred |
| `MM-CORRECT-LEADER-L` | A correct leader whose view begins after GST disseminates a block that receives an L-notarization. | Lemma `L1`. | Model/spec comparison scenarios. | Deferred |
| `MM-LIVENESS` | The baseline protocol satisfies liveness under the paper assumptions. | Lemma `L2`. | Model/spec comparison and long-run scenario tests. | Deferred |

## Evidence Status

All protocol claims in this initial document are deferred because the repository
does not yet contain protocol behavior or executable protocol tests. Future PRs
should move claims from `Deferred` only when they add executable evidence in the
same change.
