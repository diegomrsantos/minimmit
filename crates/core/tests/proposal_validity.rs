use minimmit_core::{
    validate_proposal, Block, BlockId, Committee, MNotarization, Nullification, Nullify,
    ProposalValidationError, SignedBlock, TransactionId, ValidatorId, ViewNumber, Vote,
};

const ONE_FAULT: usize = 1;
const MIN_VALIDATORS_WITH_ONE_FAULT: u64 = 6;

fn block(id: u64) -> BlockId {
    BlockId::new(id)
}

fn committee() -> Committee {
    Committee::new(validators(MIN_VALIDATORS_WITH_ONE_FAULT), ONE_FAULT)
        .expect("committee satisfies n >= 5f + 1")
}

fn transaction(id: u64) -> TransactionId {
    TransactionId::new(id)
}

fn validator(id: u64) -> ValidatorId {
    ValidatorId::new(id)
}

fn validators(count: u64) -> Vec<ValidatorId> {
    (0..count).map(validator).collect()
}

fn view(number: u64) -> ViewNumber {
    ViewNumber::new(number)
}

fn vote(signer: u64, block_id: u64, view_number: u64) -> Vote {
    Vote::new(validator(signer), block(block_id), view(view_number))
}

fn nullify(signer: u64, view_number: u64) -> Nullify {
    Nullify::new(validator(signer), view(view_number))
}

fn m_notarization(block_id: u64, view_number: u64) -> MNotarization {
    MNotarization::from_votes(
        &committee(),
        [
            vote(0, block_id, view_number),
            vote(1, block_id, view_number),
            vote(2, block_id, view_number),
        ],
    )
    .expect("votes form an M-notarization")
}

fn nullification(view_number: u64) -> Nullification {
    Nullification::from_nullifies(
        &committee(),
        [
            nullify(0, view_number),
            nullify(1, view_number),
            nullify(2, view_number),
        ],
    )
    .expect("nullify messages form a nullification")
}

fn proposed_block(id: u64, view_number: u64, parent: u64) -> Block {
    Block::new(
        block(id),
        view(view_number),
        block(parent),
        [transaction(id)],
    )
    .expect("block is valid")
}

fn signed_by_leader(committee: &Committee, block: Block) -> SignedBlock {
    SignedBlock::new(committee.leader(block.view()), block)
}

#[test]
fn accepts_proposal_with_m_notarized_parent_and_skipped_view_nullifications() {
    let committee = committee();
    let view_5_leader_signed_block = signed_by_leader(&committee, proposed_block(50, 5, 20));
    let view_1_parent = m_notarization(10, 1);
    let view_2_parent = m_notarization(20, 2);
    let view_3_nullification = nullification(3);
    let view_4_nullification = nullification(4);

    let validated = validate_proposal(
        &committee,
        view(5),
        [&view_5_leader_signed_block],
        [view_1_parent, view_2_parent],
        [view_3_nullification, view_4_nullification],
    )
    .expect("proposal satisfies MM-VALID-PROPOSAL");

    assert_eq!(validated.block(), block(50));
    assert_eq!(validated.view(), view(5));
    assert_eq!(validated.parent().block(), block(20));
    assert_eq!(validated.parent().view(), view(2));
}

#[test]
fn accepts_proposal_when_unrelated_same_view_block_has_non_leader_signature() {
    let committee = committee();
    let view_5_non_leader_signed_block = SignedBlock::new(validator(0), proposed_block(51, 5, 20));
    let view_5_leader_signed_block = signed_by_leader(&committee, proposed_block(50, 5, 20));

    let validated = validate_proposal(
        &committee,
        view(5),
        [&view_5_non_leader_signed_block, &view_5_leader_signed_block],
        [m_notarization(20, 2)],
        [nullification(3), nullification(4)],
    )
    .expect("the leader-signed proposal block satisfies MM-VALID-PROPOSAL");

    assert_eq!(validated.block(), block(50));
    assert_eq!(validated.view(), view(5));
    assert_eq!(validated.parent().block(), block(20));
    assert_eq!(validated.parent().view(), view(2));
}

#[test]
fn rejects_only_same_view_block_when_not_signed_by_view_leader() {
    let committee = committee();
    let view_5_block_signed_by_validator_0 =
        SignedBlock::new(validator(0), proposed_block(50, 5, 20));

    assert_eq!(
        validate_proposal(
            &committee,
            view(5),
            [&view_5_block_signed_by_validator_0],
            [m_notarization(20, 2)],
            [nullification(3), nullification(4)],
        ),
        Err(ProposalValidationError::WrongLeader {
            view: view(5),
            expected: validator(5),
            actual: validator(0),
        })
    );
}

#[test]
fn rejects_missing_parent_notarization() {
    let committee = committee();
    let view_5_leader_signed_block = signed_by_leader(&committee, proposed_block(50, 5, 20));

    assert_eq!(
        validate_proposal(
            &committee,
            view(5),
            [&view_5_leader_signed_block],
            [m_notarization(10, 1)],
            [nullification(3), nullification(4)],
        ),
        Err(ProposalValidationError::MissingParentNotarization { parent: block(20) })
    );
}

#[test]
fn accepts_m_notarized_parent_even_when_select_parent_would_choose_later_block() {
    let committee = committee();
    let view_5_leader_signed_block = signed_by_leader(&committee, proposed_block(50, 5, 10));

    let validated = validate_proposal(
        &committee,
        view(5),
        [&view_5_leader_signed_block],
        [m_notarization(10, 2), m_notarization(20, 3)],
        [nullification(3), nullification(4)],
    )
    .expect("receiver-side validity does not require SelectParent(S, v)");

    assert_eq!(validated.block(), block(50));
    assert_eq!(validated.view(), view(5));
    assert_eq!(validated.parent().block(), block(10));
    assert_eq!(validated.parent().view(), view(2));
}

#[test]
fn rejects_missing_skipped_view_nullification() {
    let committee = committee();
    let view_5_leader_signed_block = signed_by_leader(&committee, proposed_block(50, 5, 20));

    assert_eq!(
        validate_proposal(
            &committee,
            view(5),
            [&view_5_leader_signed_block],
            [m_notarization(20, 2)],
            [nullification(3)],
        ),
        Err(ProposalValidationError::MissingNullification { view: view(4) })
    );
}

#[test]
fn rejects_multiple_leader_signed_blocks_for_view() {
    let committee = committee();
    let first_view_5_leader_signed_block = signed_by_leader(&committee, proposed_block(50, 5, 20));
    let second_view_5_leader_signed_block = signed_by_leader(&committee, proposed_block(51, 5, 20));

    assert_eq!(
        validate_proposal(
            &committee,
            view(5),
            [
                &first_view_5_leader_signed_block,
                &second_view_5_leader_signed_block,
            ],
            [m_notarization(20, 2)],
            [nullification(3), nullification(4)],
        ),
        Err(ProposalValidationError::ConflictingBlocks {
            view: view(5),
            first: block(50),
            second: block(51),
        })
    );
}

#[test]
fn accepts_duplicate_nullification_evidence_for_skipped_view() {
    let committee = committee();
    let view_5_leader_signed_block = signed_by_leader(&committee, proposed_block(50, 5, 20));

    let validated = validate_proposal(
        &committee,
        view(5),
        [&view_5_leader_signed_block],
        [m_notarization(20, 2)],
        [nullification(3), nullification(3), nullification(4)],
    )
    .expect("duplicate local nullification evidence still covers the skipped view");

    assert_eq!(validated.block(), block(50));
    assert_eq!(validated.view(), view(5));
    assert_eq!(validated.parent().block(), block(20));
    assert_eq!(validated.parent().view(), view(2));
}
