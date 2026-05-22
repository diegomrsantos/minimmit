mod common;

use common::{committee, m_notarization};
use minimmit_core::{
    Block, BlockId, Committee, Event, MNotarization, Network, Nullification, Nullify, Processor,
    ProcessorError, Proposal, ProposalInput, Ready, Storage, TransactionId, ValidatorId,
    ViewNumber, Vote,
};

fn replay(processor: &mut Processor, inputs: Vec<Event>) -> Vec<Ready> {
    inputs
        .into_iter()
        .map(|event| processor.step(event))
        .collect()
}

fn processor() -> Processor {
    Processor::new(ValidatorId::new(0), committee()).expect("local validator is a member")
}

fn leader_processor_for_view_1() -> Processor {
    Processor::new(ValidatorId::new(1), committee())
        .expect("validator 1 leads view 1 in the test committee")
}

fn observed_proposal_blocks(processor: &Processor, view: ViewNumber) -> Vec<BlockId> {
    processor
        .observed_proposals(view)
        .map(|proposal| proposal.block().id())
        .collect()
}

fn observed_proposal_transactions(
    processor: &Processor,
    view: ViewNumber,
) -> Vec<Vec<TransactionId>> {
    processor
        .observed_proposals(view)
        .map(|proposal| proposal.block().transactions().to_vec())
        .collect()
}

fn observed_m_notarizations(processor: &Processor) -> Vec<(ViewNumber, BlockId)> {
    processor
        .observed_m_notarizations()
        .map(|notarization| (notarization.view(), notarization.block()))
        .collect()
}

fn observed_nullifications(processor: &Processor) -> Vec<ViewNumber> {
    processor
        .observed_nullifications()
        .map(Nullification::view)
        .collect()
}

fn proposal(block_id: BlockId, view: ViewNumber) -> Proposal {
    proposal_with_transaction(block_id, view, TransactionId::new(block_id.get()))
}

fn proposal_input(block_id: BlockId) -> ProposalInput {
    ProposalInput::new(block_id, [TransactionId::new(block_id.get())])
}

/// Asserts leader proposal ready output contains proposal work followed by vote work.
///
/// Returns the persisted proposal so tests can assert the proposal fields
/// without repeating the ready output shape.
fn assert_leader_proposal_ready(ready: Ready) -> Proposal {
    assert_eq!(ready.storage.len(), 2, "expected two storage outputs");
    assert_eq!(ready.network.len(), 2, "expected two network outputs");

    let Storage::PersistProposal(persisted) = &ready.storage[0] else {
        panic!("expected proposal storage output");
    };
    let Network::BroadcastProposal(broadcast) = &ready.network[0] else {
        panic!("expected proposal network output");
    };

    assert_eq!(persisted, broadcast);
    let expected_vote = Vote::new(
        persisted.proposer(),
        persisted.block().id(),
        persisted.block().view(),
    );
    assert_eq!(ready.storage[1], Storage::PersistVote(expected_vote));
    assert_eq!(ready.network[1], Network::BroadcastVote(expected_vote));

    persisted.clone()
}

fn assert_ready_vote(ready: Ready, vote: Vote) {
    assert_eq!(ready.storage, [Storage::PersistVote(vote)]);
    assert_eq!(ready.network, [Network::BroadcastVote(vote)]);
}

fn assert_view_1_leader_proposal(proposal: &Proposal) {
    assert_eq!(proposal.proposer(), ValidatorId::new(1));
    assert_eq!(proposal.block().id(), BlockId::new(10));
    assert_eq!(proposal.block().view(), ViewNumber::new(1));
    assert_eq!(proposal.block().parent(), BlockId::GENESIS);
    assert_eq!(proposal.block().transactions(), &[TransactionId::new(10)]);
    assert_eq!(proposal.parent_notarization().block(), BlockId::GENESIS);
    assert_eq!(proposal.parent_notarization().view(), ViewNumber::GENESIS);
    assert_eq!(proposal.nullifications().count(), 0);
}

fn proposal_with_transaction(
    block_id: BlockId,
    view: ViewNumber,
    transaction: TransactionId,
) -> Proposal {
    proposal_with_proposer(block_id, view, transaction, committee().leader(view))
}

fn proposal_with_proposer(
    block_id: BlockId,
    view: ViewNumber,
    transaction: TransactionId,
    proposer: ValidatorId,
) -> Proposal {
    let parent = if view == ViewNumber::new(1) {
        BlockId::GENESIS
    } else {
        BlockId::new(block_id.get() - 1)
    };
    let parent_view = if view == ViewNumber::new(1) {
        ViewNumber::GENESIS
    } else {
        ViewNumber::new(view.get() - 1)
    };
    let block = Block::new(block_id, view, parent, [transaction]).expect("proposal block is valid");

    Proposal::new(proposer, block, m_notarization(parent, parent_view), [])
        .expect("proposal nullification views are unique")
}

fn nullification(view: ViewNumber) -> Nullification {
    nullification_with_signers(view, [0, 1, 2])
}

fn nullification_with_signers(view: ViewNumber, signers: [u64; 3]) -> Nullification {
    Nullification::from_nullifies(
        &committee(),
        signers.map(|signer| Nullify::new(ValidatorId::new(signer), view)),
    )
    .expect("nullify messages form a nullification")
}

fn other_committee() -> Committee {
    Committee::new((10..16).map(ValidatorId::new).collect::<Vec<_>>(), 1)
        .expect("other committee satisfies n >= 5f + 1")
}

fn m_notarization_from_other_committee(block_id: BlockId, view: ViewNumber) -> MNotarization {
    MNotarization::from_votes(
        &other_committee(),
        [
            Vote::new(ValidatorId::new(10), block_id, view),
            Vote::new(ValidatorId::new(11), block_id, view),
            Vote::new(ValidatorId::new(12), block_id, view),
        ],
    )
    .expect("other-committee votes form an M-notarization")
}

fn nullification_from_other_committee(view: ViewNumber) -> Nullification {
    Nullification::from_nullifies(
        &other_committee(),
        [
            Nullify::new(ValidatorId::new(10), view),
            Nullify::new(ValidatorId::new(11), view),
            Nullify::new(ValidatorId::new(12), view),
        ],
    )
    .expect("other-committee nullifies form a nullification")
}

fn proposal_with_parent_notarization_from_other_committee(
    block_id: BlockId,
    view: ViewNumber,
) -> Proposal {
    let block = Block::new(
        block_id,
        view,
        BlockId::GENESIS,
        [TransactionId::new(block_id.get())],
    )
    .expect("proposal block is valid");

    Proposal::new(
        committee().leader(view),
        block,
        m_notarization_from_other_committee(BlockId::GENESIS, ViewNumber::GENESIS),
        [],
    )
    .expect("proposal nullification views are unique")
}

#[test]
fn processor_starts_after_genesis_for_local_committee_member() {
    let committee = committee();
    let processor = Processor::new(ValidatorId::new(2), committee.clone())
        .expect("local validator is a member");

    assert_eq!(processor.local_validator(), ValidatorId::new(2));
    assert_eq!(processor.committee(), &committee);
    assert_eq!(processor.current_view(), ViewNumber::new(1));
}

#[test]
fn processor_rejects_local_validator_outside_committee() {
    assert_eq!(
        Processor::new(ValidatorId::new(99), committee()),
        Err(ProcessorError::UnknownLocalValidator {
            validator: ValidatorId::new(99),
        })
    );
}

#[test]
fn noop_event_returns_no_ready_output_and_preserves_state() {
    let mut processor = processor();
    let initial = processor.clone();

    let ready = processor.step(Event::Noop);

    assert_eq!(ready, Ready::default());
    assert!(ready.is_empty());
    assert_eq!(processor, initial);
}

#[test]
fn future_proposal_event_records_proposal_without_ready_output_or_view_change() {
    let mut processor = processor();
    let proposal = proposal(BlockId::new(20), ViewNumber::new(2));

    let ready = processor.step(Event::Proposal(proposal));

    assert_eq!(ready, Ready::default());
    assert!(ready.is_empty());
    assert_eq!(processor.current_view(), ViewNumber::new(1));
    assert_eq!(
        observed_proposal_blocks(&processor, ViewNumber::new(2)),
        [BlockId::new(20)]
    );
}

#[test]
fn nullification_event_records_current_view_nullification_without_ready_output() {
    let mut processor = processor();
    let nullification = nullification(ViewNumber::new(1));

    let ready = processor.step(Event::Nullification(nullification));

    assert_eq!(ready, Ready::default());
    assert!(ready.is_empty());
    assert_eq!(processor.current_view(), ViewNumber::new(1));
    assert_eq!(observed_nullifications(&processor), [ViewNumber::new(1)]);
}

// Protocol state fact: current view vote.
//
// Claim: MM-VOTE-VALID-PROPOSAL, Algorithm 1 votecheck and vote1.
//
// Meaning:
// The processor has cast its one vote for its current view.
//
// Established by:
// - successful local Event::Propose, which emits proposal work and matching vote
//   work
// - observed valid current view Event::Proposal, when the processor has not
//   voted or observed current view nullification
//
// Not established by:
// - invalid proposal evidence
// - stale or future proposals
// - proposal not signed by the view leader
// - any proposal after the current view already has a vote or nullification
//
// Used by:
// Later proposal, timeout, nullification, and M-notarization transitions must
// preserve one vote per view and avoid illegal nullification after voting.
//
// Reset:
// View advancement will reset this fact. That executable evidence remains
// deferred to #73 and #74.
//
// Evidence:
// The tests below cover the establishing and blocked paths available before
// view advancement lands.
#[test]
fn current_view_proposal_emits_vote_ready_output() {
    let mut processor = processor();
    let proposal = proposal(BlockId::new(10), ViewNumber::new(1));

    let ready = processor.step(Event::Proposal(proposal));

    assert_ready_vote(
        ready,
        Vote::new(ValidatorId::new(0), BlockId::new(10), ViewNumber::new(1)),
    );
    assert_eq!(
        observed_proposal_blocks(&processor, ViewNumber::new(1)),
        [BlockId::new(10)]
    );
}

#[test]
fn processor_that_already_voted_does_not_vote_again() {
    let mut processor = processor();
    let first = proposal(BlockId::new(10), ViewNumber::new(1));
    let different_block = proposal(BlockId::new(11), ViewNumber::new(1));

    assert_ready_vote(
        processor.step(Event::Proposal(first)),
        Vote::new(ValidatorId::new(0), BlockId::new(10), ViewNumber::new(1)),
    );

    let ready = processor.step(Event::Proposal(different_block));

    assert_eq!(ready, Ready::default());
    assert_eq!(
        observed_proposal_blocks(&processor, ViewNumber::new(1)),
        [BlockId::new(10), BlockId::new(11)]
    );
}

#[test]
fn leader_that_proposed_does_not_vote_for_conflicting_proposal() {
    let mut processor = leader_processor_for_view_1();

    assert_leader_proposal_ready(processor.step(Event::Propose(proposal_input(BlockId::new(10)))));

    let ready = processor.step(Event::Proposal(proposal(
        BlockId::new(11),
        ViewNumber::new(1),
    )));

    assert_eq!(ready, Ready::default());
    assert_eq!(
        observed_proposal_blocks(&processor, ViewNumber::new(1)),
        [BlockId::new(10), BlockId::new(11)]
    );
}

#[test]
fn current_view_nullification_prevents_later_vote() {
    let mut processor = processor();

    assert_eq!(
        processor.step(Event::Nullification(nullification(ViewNumber::new(1)))),
        Ready::default()
    );

    let ready = processor.step(Event::Proposal(proposal(
        BlockId::new(10),
        ViewNumber::new(1),
    )));

    assert_eq!(ready, Ready::default());
    assert_eq!(observed_nullifications(&processor), [ViewNumber::new(1)]);
    assert_eq!(
        observed_proposal_blocks(&processor, ViewNumber::new(1)),
        [BlockId::new(10)]
    );
}

#[test]
fn current_view_nullification_prevents_later_leader_proposal() {
    let mut processor = leader_processor_for_view_1();

    assert_eq!(
        processor.step(Event::Nullification(nullification(ViewNumber::new(1)))),
        Ready::default()
    );

    let ready = processor.step(Event::Propose(proposal_input(BlockId::new(10))));

    assert_eq!(ready, Ready::default());
    assert_eq!(observed_nullifications(&processor), [ViewNumber::new(1)]);
    assert_eq!(observed_proposal_blocks(&processor, ViewNumber::new(1)), []);
}

#[test]
fn future_proposal_does_not_consume_current_view_vote_slot() {
    let mut processor = processor();

    assert_eq!(
        processor.step(Event::Proposal(proposal(
            BlockId::new(20),
            ViewNumber::new(2)
        ))),
        Ready::default()
    );

    assert_ready_vote(
        processor.step(Event::Proposal(proposal(
            BlockId::new(10),
            ViewNumber::new(1),
        ))),
        Vote::new(ValidatorId::new(0), BlockId::new(10), ViewNumber::new(1)),
    );
}

#[test]
fn m_notarization_event_records_current_view_notarization_without_ready_output() {
    let mut processor = processor();
    let notarization = m_notarization(BlockId::new(20), ViewNumber::new(1));

    let ready = processor.step(Event::MNotarization(notarization));

    assert_eq!(ready, Ready::default());
    assert!(ready.is_empty());
    assert_eq!(processor.current_view(), ViewNumber::new(1));
    assert_eq!(
        observed_m_notarizations(&processor),
        [(ViewNumber::new(1), BlockId::new(20))]
    );
}

#[test]
fn observed_proposals_iterate_by_block_id_within_view() {
    let mut processor = processor();

    // Insert out of order so the final assertion proves iteration order.
    assert_eq!(
        processor.step(Event::Proposal(proposal(
            BlockId::new(30),
            ViewNumber::new(2)
        ))),
        Ready::default()
    );
    assert_eq!(
        processor.step(Event::Proposal(proposal(
            BlockId::new(20),
            ViewNumber::new(2)
        ))),
        Ready::default()
    );

    assert_eq!(
        observed_proposal_blocks(&processor, ViewNumber::new(2)),
        [BlockId::new(20), BlockId::new(30)]
    );
}

#[test]
fn observed_m_notarizations_iterate_by_view_then_block_id() {
    let mut processor = processor();

    // Insert out of order so the final assertion proves iteration order.
    assert_eq!(
        processor.step(Event::MNotarization(m_notarization(
            BlockId::new(30),
            ViewNumber::new(2)
        ))),
        Ready::default()
    );
    assert_eq!(
        processor.step(Event::MNotarization(m_notarization(
            BlockId::new(40),
            ViewNumber::new(1)
        ))),
        Ready::default()
    );
    assert_eq!(
        processor.step(Event::MNotarization(m_notarization(
            BlockId::new(20),
            ViewNumber::new(2)
        ))),
        Ready::default()
    );

    assert_eq!(
        observed_m_notarizations(&processor),
        [
            (ViewNumber::new(1), BlockId::new(40)),
            (ViewNumber::new(2), BlockId::new(20)),
            (ViewNumber::new(2), BlockId::new(30)),
        ]
    );
}

#[test]
fn conflicting_same_block_proposal_keeps_first_observed_proposal() {
    let mut processor = processor();
    let first =
        proposal_with_transaction(BlockId::new(20), ViewNumber::new(2), TransactionId::new(1));
    let conflicting =
        proposal_with_transaction(BlockId::new(20), ViewNumber::new(2), TransactionId::new(2));

    assert_eq!(processor.step(Event::Proposal(first)), Ready::default());
    assert_eq!(
        processor.step(Event::Proposal(conflicting)),
        Ready::default()
    );

    assert_eq!(
        observed_proposal_transactions(&processor, ViewNumber::new(2)),
        [vec![TransactionId::new(1)]]
    );
}

#[test]
fn same_block_m_notarization_is_recorded_once() {
    let mut processor = processor();
    let notarization = m_notarization(BlockId::new(20), ViewNumber::new(2));

    assert_eq!(
        processor.step(Event::MNotarization(notarization.clone())),
        Ready::default()
    );
    assert_eq!(
        processor.step(Event::MNotarization(notarization)),
        Ready::default()
    );

    assert_eq!(
        observed_m_notarizations(&processor),
        [(ViewNumber::new(2), BlockId::new(20))]
    );
}

#[test]
fn same_view_nullification_is_recorded_once() {
    let mut processor = processor();
    let nullification = nullification(ViewNumber::new(2));

    assert_eq!(
        processor.step(Event::Nullification(nullification.clone())),
        Ready::default()
    );
    assert_eq!(
        processor.step(Event::Nullification(nullification)),
        Ready::default()
    );

    assert_eq!(observed_nullifications(&processor), [ViewNumber::new(2)]);
}

#[test]
fn future_observations_are_stored_without_advancing_view() {
    let mut processor = processor();

    assert_eq!(
        processor.step(Event::Proposal(proposal(
            BlockId::new(40),
            ViewNumber::new(4)
        ))),
        Ready::default()
    );
    assert_eq!(
        processor.step(Event::MNotarization(m_notarization(
            BlockId::new(30),
            ViewNumber::new(3)
        ))),
        Ready::default()
    );
    assert_eq!(
        processor.step(Event::Nullification(nullification(ViewNumber::new(5)))),
        Ready::default()
    );

    assert_eq!(processor.current_view(), ViewNumber::new(1));
    assert_eq!(
        observed_proposal_blocks(&processor, ViewNumber::new(4)),
        [BlockId::new(40)]
    );
    assert_eq!(
        observed_m_notarizations(&processor),
        [(ViewNumber::new(3), BlockId::new(30))]
    );
    assert_eq!(observed_nullifications(&processor), [ViewNumber::new(5)]);
}

#[test]
fn artifacts_valid_for_another_committee_are_ignored() {
    let mut processor = processor();
    let notarization_valid_for_another_committee =
        m_notarization_from_other_committee(BlockId::new(20), ViewNumber::new(1));
    let nullification_valid_for_another_committee =
        nullification_from_other_committee(ViewNumber::new(1));
    let proposal_with_parent_from_another_committee =
        proposal_with_parent_notarization_from_other_committee(
            BlockId::new(30),
            ViewNumber::new(1),
        );

    // These artifacts are well-formed, but not for the processor's committee.
    assert_eq!(
        processor.step(Event::MNotarization(
            notarization_valid_for_another_committee
        )),
        Ready::default()
    );
    assert_eq!(
        processor.step(Event::Nullification(
            nullification_valid_for_another_committee
        )),
        Ready::default()
    );
    assert_eq!(
        processor.step(Event::Proposal(proposal_with_parent_from_another_committee)),
        Ready::default()
    );

    assert_eq!(observed_m_notarizations(&processor), []);
    assert_eq!(observed_nullifications(&processor), []);
    assert_eq!(observed_proposal_blocks(&processor, ViewNumber::new(1)), []);
}

#[test]
fn proposal_not_signed_by_view_leader_is_ignored() {
    let mut processor = processor();
    let wrong_leader_proposal = proposal_with_proposer(
        BlockId::new(20),
        ViewNumber::new(1),
        TransactionId::new(20),
        ValidatorId::new(0),
    );

    assert_eq!(
        processor.step(Event::Proposal(wrong_leader_proposal)),
        Ready::default()
    );

    assert_eq!(observed_proposal_blocks(&processor, ViewNumber::new(1)), []);
}

#[test]
fn genesis_notarization_and_nullification_observations_are_ignored() {
    let mut processor = processor();

    assert_eq!(
        processor.step(Event::MNotarization(m_notarization(
            BlockId::GENESIS,
            ViewNumber::GENESIS
        ))),
        Ready::default()
    );
    assert_eq!(
        processor.step(Event::Nullification(nullification(ViewNumber::GENESIS))),
        Ready::default()
    );

    assert_eq!(observed_m_notarizations(&processor), []);
    assert_eq!(observed_nullifications(&processor), []);
}

#[test]
fn leader_proposal_trigger_records_and_returns_proposal_then_vote_work() {
    let mut processor = leader_processor_for_view_1();

    let ready = processor.step(Event::Propose(proposal_input(BlockId::new(10))));

    let persisted = assert_leader_proposal_ready(ready);
    assert_view_1_leader_proposal(&persisted);
    assert_eq!(
        observed_proposal_blocks(&processor, ViewNumber::new(1)),
        [BlockId::new(10)]
    );
    assert_eq!(
        processor
            .observed_proposals(ViewNumber::new(1))
            .next()
            .expect("leader proposal is recorded immediately"),
        &persisted
    );
}

#[test]
fn non_leader_proposal_trigger_returns_no_ready_output() {
    let mut processor = processor();

    let ready = processor.step(Event::Propose(proposal_input(BlockId::new(10))));

    assert_eq!(ready, Ready::default());
    assert!(ready.is_empty());
    assert_eq!(observed_proposal_blocks(&processor, ViewNumber::new(1)), []);
}

// State-fact transition matrix: local leader current-view proposal slot.
//
// Fact: the local leader's current-view proposal slot is consumed.
// Claim: MM-LEADER-PROPOSE / Algorithm 1 sendblock.
// Establishing events: successful local Event::Propose; observed valid local
// current-view Event::Proposal.
// Must not establish: invalid local current-view proposal, or valid local
// future-view proposal.
// Depends: a later local Event::Propose for the same current view must not emit
// a second proposal.
// Reset: view advancement will reset the slot; current-view advancement is a
// later evidence gap.
// Evidence: the proposal-slot tests below cover the current cross-event paths.
#[test]
fn leader_that_has_already_proposed_does_not_start_second_proposal() {
    let mut processor = leader_processor_for_view_1();

    // Given
    assert_leader_proposal_ready(processor.step(Event::Propose(proposal_input(BlockId::new(10)))));

    // When
    let ready = processor.step(Event::Propose(proposal_input(BlockId::new(11))));

    // Then
    assert_eq!(ready, Ready::default());
    assert!(ready.is_empty());
    assert_eq!(processor.current_view(), ViewNumber::new(1));
    assert_eq!(
        observed_proposal_blocks(&processor, ViewNumber::new(1)),
        [BlockId::new(10)]
    );
}

#[test]
fn observed_local_current_view_proposal_consumes_proposal_slot() {
    let mut processor = leader_processor_for_view_1();
    let local_proposal = proposal(BlockId::new(10), ViewNumber::new(1));

    // Given
    assert_ready_vote(
        processor.step(Event::Proposal(local_proposal)),
        Vote::new(ValidatorId::new(1), BlockId::new(10), ViewNumber::new(1)),
    );

    // When
    let ready = processor.step(Event::Propose(proposal_input(BlockId::new(11))));

    // Then
    assert_eq!(ready, Ready::default());
    assert!(ready.is_empty());
    assert_eq!(processor.current_view(), ViewNumber::new(1));
    assert_eq!(
        observed_proposal_blocks(&processor, ViewNumber::new(1)),
        [BlockId::new(10)]
    );
}

#[test]
fn invalid_observed_local_current_view_proposal_does_not_consume_proposal_slot() {
    let mut processor = leader_processor_for_view_1();
    let invalid_local_proposal = proposal_with_parent_notarization_from_other_committee(
        BlockId::new(10),
        ViewNumber::new(1),
    );

    // Given
    assert_eq!(
        processor.step(Event::Proposal(invalid_local_proposal)),
        Ready::default()
    );
    assert_eq!(observed_proposal_blocks(&processor, ViewNumber::new(1)), []);

    // When
    let proposal = assert_leader_proposal_ready(
        processor.step(Event::Propose(proposal_input(BlockId::new(10)))),
    );

    // Then
    assert_eq!(processor.current_view(), ViewNumber::new(1));
    assert_eq!(proposal.block().id(), BlockId::new(10));
    assert_eq!(
        observed_proposal_blocks(&processor, ViewNumber::new(1)),
        [BlockId::new(10)]
    );
}

#[test]
fn observed_local_future_view_proposal_does_not_consume_current_view_proposal_slot() {
    let mut processor = leader_processor_for_view_1();
    let future_local_proposal = proposal(BlockId::new(70), ViewNumber::new(7));

    // Given
    assert_eq!(
        processor.step(Event::Proposal(future_local_proposal)),
        Ready::default()
    );
    assert_eq!(
        observed_proposal_blocks(&processor, ViewNumber::new(7)),
        [BlockId::new(70)]
    );

    // When
    let proposal = assert_leader_proposal_ready(
        processor.step(Event::Propose(proposal_input(BlockId::new(10)))),
    );

    // Then
    assert_eq!(processor.current_view(), ViewNumber::new(1));
    assert_eq!(proposal.block().id(), BlockId::new(10));
    assert_eq!(
        observed_proposal_blocks(&processor, ViewNumber::new(1)),
        [BlockId::new(10)]
    );
    assert_eq!(
        observed_proposal_blocks(&processor, ViewNumber::new(7)),
        [BlockId::new(70)]
    );
}

#[test]
fn invalid_leader_proposal_input_does_not_record_proposed_state() {
    let mut processor = leader_processor_for_view_1();
    let duplicate_transactions = ProposalInput::new(
        BlockId::new(10),
        [TransactionId::new(1), TransactionId::new(1)],
    );

    // Given
    assert_eq!(
        processor.step(Event::Propose(duplicate_transactions)),
        Ready::default()
    );
    assert_eq!(observed_proposal_blocks(&processor, ViewNumber::new(1)), []);

    // When
    assert_leader_proposal_ready(processor.step(Event::Propose(proposal_input(BlockId::new(10)))));

    // Then
    assert_eq!(
        observed_proposal_blocks(&processor, ViewNumber::new(1)),
        [BlockId::new(10)]
    );
}

#[test]
fn storage_and_network_ready_output_is_not_empty() {
    let proposal = proposal(BlockId::new(20), ViewNumber::new(2));
    let ready = Ready {
        storage: vec![Storage::PersistProposal(proposal.clone())],
        network: vec![Network::BroadcastProposal(proposal)],
    };

    assert!(!ready.is_empty());
}

#[test]
fn same_event_sequence_replays_to_same_ready_outputs() {
    let inputs = vec![
        Event::Noop,
        Event::Propose(proposal_input(BlockId::new(10))),
        Event::Proposal(proposal(BlockId::new(20), ViewNumber::new(2))),
        Event::MNotarization(m_notarization(BlockId::new(10), ViewNumber::new(1))),
        Event::Nullification(nullification(ViewNumber::new(3))),
    ];
    let mut first = leader_processor_for_view_1();
    let mut second = leader_processor_for_view_1();

    let first_outputs = replay(&mut first, inputs.clone());
    let second_outputs = replay(&mut second, inputs);

    assert_eq!(first_outputs, second_outputs);
    assert_eq!(first, second);
}
