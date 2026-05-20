mod common;

use common::{committee, m_notarization};
use minimmit_core::{
    Block, BlockId, Committee, Event, Lifecycle, MNotarization, Nullification, Nullify,
    PersistenceId, Processor, ProcessorError, Proposal, Ready, TransactionId, ValidatorId,
    ViewNumber, Vote,
};

#[derive(Clone)]
enum TraceInput {
    Event(Event),
    Lifecycle(Lifecycle),
}

fn replay(processor: &mut Processor, inputs: Vec<TraceInput>) -> Vec<Ready> {
    inputs
        .into_iter()
        .map(|input| match input {
            TraceInput::Event(event) => processor.step(event),
            TraceInput::Lifecycle(event) => processor.lifecycle(event),
        })
        .collect()
}

fn processor() -> Processor {
    Processor::new(ValidatorId::new(0), committee()).expect("local validator is a member")
}

fn step_no_ready(processor: &mut Processor, event: Event) {
    let ready = processor.step(event);

    assert_eq!(ready, Ready::None);
    assert!(ready.is_empty());
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

fn observed_m_notarization_signers(processor: &Processor) -> Vec<Vec<ValidatorId>> {
    processor
        .observed_m_notarizations()
        .map(|notarization| notarization.signers().collect())
        .collect()
}

fn observed_nullification_signers(processor: &Processor) -> Vec<Vec<ValidatorId>> {
    processor
        .observed_nullifications()
        .map(|nullification| nullification.signers().collect())
        .collect()
}

fn proposal(block_id: BlockId, view: ViewNumber) -> Proposal {
    proposal_with_transaction(block_id, view, TransactionId::new(block_id.get()))
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

fn m_notarization_with_signers(
    block_id: BlockId,
    view: ViewNumber,
    signers: [u64; 3],
) -> MNotarization {
    MNotarization::from_votes(
        &committee(),
        signers.map(|signer| Vote::new(ValidatorId::new(signer), block_id, view)),
    )
    .expect("votes form an M-notarization")
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

    assert_eq!(ready, Ready::None);
    assert!(ready.is_empty());
    assert_eq!(processor, initial);
}

#[test]
fn proposal_event_records_proposal_without_ready_output_or_view_change() {
    let mut processor = processor();
    let proposal = proposal(BlockId::new(20), ViewNumber::new(2));

    let ready = processor.step(Event::Proposal(proposal));

    assert_eq!(ready, Ready::None);
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

    assert_eq!(ready, Ready::None);
    assert!(ready.is_empty());
    assert_eq!(processor.current_view(), ViewNumber::new(1));
    assert_eq!(observed_nullifications(&processor), [ViewNumber::new(1)]);
}

#[test]
fn m_notarization_event_records_current_view_notarization_without_ready_output() {
    let mut processor = processor();
    let notarization = m_notarization(BlockId::new(20), ViewNumber::new(1));

    let ready = processor.step(Event::MNotarization(notarization));

    assert_eq!(ready, Ready::None);
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

    step_no_ready(
        &mut processor,
        Event::Proposal(proposal(BlockId::new(30), ViewNumber::new(2))),
    );
    step_no_ready(
        &mut processor,
        Event::Proposal(proposal(BlockId::new(20), ViewNumber::new(2))),
    );

    assert_eq!(
        observed_proposal_blocks(&processor, ViewNumber::new(2)),
        [BlockId::new(20), BlockId::new(30)]
    );
}

#[test]
fn observed_m_notarizations_iterate_by_view_then_block_id() {
    let mut processor = processor();

    step_no_ready(
        &mut processor,
        Event::MNotarization(m_notarization(BlockId::new(30), ViewNumber::new(2))),
    );
    step_no_ready(
        &mut processor,
        Event::MNotarization(m_notarization(BlockId::new(40), ViewNumber::new(1))),
    );
    step_no_ready(
        &mut processor,
        Event::MNotarization(m_notarization(BlockId::new(20), ViewNumber::new(2))),
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

    step_no_ready(&mut processor, Event::Proposal(first));
    step_no_ready(&mut processor, Event::Proposal(conflicting));

    assert_eq!(
        observed_proposal_transactions(&processor, ViewNumber::new(2)),
        [vec![TransactionId::new(1)]]
    );
}

#[test]
fn same_key_evidence_keeps_lexicographically_least_signer_set() {
    let mut processor = processor();
    let higher_m_notarization =
        m_notarization_with_signers(BlockId::new(20), ViewNumber::new(2), [3, 4, 5]);
    let lower_m_notarization =
        m_notarization_with_signers(BlockId::new(20), ViewNumber::new(2), [0, 1, 2]);
    let higher_nullification = nullification_with_signers(ViewNumber::new(2), [3, 4, 5]);
    let lower_nullification = nullification_with_signers(ViewNumber::new(2), [0, 1, 2]);

    step_no_ready(&mut processor, Event::MNotarization(higher_m_notarization));
    step_no_ready(&mut processor, Event::MNotarization(lower_m_notarization));
    step_no_ready(&mut processor, Event::Nullification(higher_nullification));
    step_no_ready(&mut processor, Event::Nullification(lower_nullification));

    assert_eq!(
        observed_m_notarization_signers(&processor),
        [vec![
            ValidatorId::new(0),
            ValidatorId::new(1),
            ValidatorId::new(2)
        ]]
    );
    assert_eq!(
        observed_nullification_signers(&processor),
        [vec![
            ValidatorId::new(0),
            ValidatorId::new(1),
            ValidatorId::new(2)
        ]]
    );
}

#[test]
fn future_observations_are_stored_without_advancing_view() {
    let mut processor = processor();

    step_no_ready(
        &mut processor,
        Event::Proposal(proposal(BlockId::new(40), ViewNumber::new(4))),
    );
    step_no_ready(
        &mut processor,
        Event::MNotarization(m_notarization(BlockId::new(30), ViewNumber::new(3))),
    );
    step_no_ready(
        &mut processor,
        Event::Nullification(nullification(ViewNumber::new(5))),
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

    step_no_ready(
        &mut processor,
        Event::MNotarization(notarization_valid_for_another_committee),
    );
    step_no_ready(
        &mut processor,
        Event::Nullification(nullification_valid_for_another_committee),
    );
    step_no_ready(
        &mut processor,
        Event::Proposal(proposal_with_parent_from_another_committee),
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

    step_no_ready(&mut processor, Event::Proposal(wrong_leader_proposal));

    assert_eq!(observed_proposal_blocks(&processor, ViewNumber::new(1)), []);
}

#[test]
fn genesis_notarization_and_nullification_observations_are_ignored() {
    let mut processor = processor();

    step_no_ready(
        &mut processor,
        Event::MNotarization(m_notarization(BlockId::GENESIS, ViewNumber::GENESIS)),
    );
    step_no_ready(
        &mut processor,
        Event::Nullification(nullification(ViewNumber::GENESIS)),
    );

    assert_eq!(observed_m_notarizations(&processor), []);
    assert_eq!(observed_nullifications(&processor), []);
}

#[test]
fn persisted_lifecycle_without_pending_work_returns_no_ready_output() {
    let mut processor = processor();
    let initial = processor.clone();

    let ready = processor.lifecycle(Lifecycle::Persisted(PersistenceId::new(7)));

    assert_eq!(ready, Ready::None);
    assert!(ready.is_empty());
    assert_eq!(processor, initial);
}

#[test]
fn persist_ready_output_is_not_empty() {
    let ready = Ready::Persist {
        id: PersistenceId::new(42),
    };

    assert!(!ready.is_empty());
}

#[test]
fn same_event_and_lifecycle_sequence_replays_to_same_ready_outputs() {
    let inputs = vec![
        TraceInput::Event(Event::Noop),
        TraceInput::Event(Event::Proposal(proposal(
            BlockId::new(20),
            ViewNumber::new(2),
        ))),
        TraceInput::Event(Event::MNotarization(m_notarization(
            BlockId::new(10),
            ViewNumber::new(1),
        ))),
        TraceInput::Lifecycle(Lifecycle::Persisted(PersistenceId::new(7))),
        TraceInput::Event(Event::Nullification(nullification(ViewNumber::new(3)))),
    ];
    let mut first = processor();
    let mut second = processor();

    let first_outputs = replay(&mut first, inputs.clone());
    let second_outputs = replay(&mut second, inputs);

    assert_eq!(first_outputs, second_outputs);
    assert_eq!(first, second);
}
