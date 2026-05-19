mod common;

use common::{committee, m_notarization};
use minimmit_core::{
    validate_proposal, Block, BlockId, Nullification, Nullify, ProposalValidationError,
    SignedBlock, TransactionId, ValidatorId, ViewNumber,
};

fn nullification(view: ViewNumber) -> Nullification {
    Nullification::from_nullifies(
        &committee(),
        [
            Nullify::new(ValidatorId::new(0), view),
            Nullify::new(ValidatorId::new(1), view),
            Nullify::new(ValidatorId::new(2), view),
        ],
    )
    .expect("nullify messages form a nullification")
}

fn proposed_block(id: BlockId, view: ViewNumber, parent: BlockId) -> Block {
    Block::new(id, view, parent, [TransactionId::new(id.get())]).expect("block is valid")
}

fn signed_by_leader(committee: &minimmit_core::Committee, block: Block) -> SignedBlock {
    SignedBlock::new(committee.leader(block.view()), block)
}

#[test]
fn accepts_proposal_with_m_notarized_parent_and_skipped_view_nullifications() {
    let committee = committee();
    let view_5_leader_signed_block = signed_by_leader(
        &committee,
        proposed_block(BlockId::new(50), ViewNumber::new(5), BlockId::new(20)),
    );
    let view_1_parent = m_notarization(BlockId::new(10), ViewNumber::new(1));
    let view_2_parent = m_notarization(BlockId::new(20), ViewNumber::new(2));
    let view_3_nullification = nullification(ViewNumber::new(3));
    let view_4_nullification = nullification(ViewNumber::new(4));

    let validated = validate_proposal(
        &committee,
        ViewNumber::new(5),
        [&view_5_leader_signed_block],
        [view_1_parent, view_2_parent],
        [view_3_nullification, view_4_nullification],
    )
    .expect("proposal satisfies MM-VALID-PROPOSAL");

    assert_eq!(validated.block(), BlockId::new(50));
    assert_eq!(validated.view(), ViewNumber::new(5));
    assert_eq!(validated.parent().block(), BlockId::new(20));
    assert_eq!(validated.parent().view(), ViewNumber::new(2));
}

#[test]
fn accepts_proposal_when_unrelated_same_view_block_has_non_leader_signature() {
    let committee = committee();
    let view_5_non_leader_signed_block = SignedBlock::new(
        ValidatorId::new(0),
        proposed_block(BlockId::new(51), ViewNumber::new(5), BlockId::new(20)),
    );
    let view_5_leader_signed_block = signed_by_leader(
        &committee,
        proposed_block(BlockId::new(50), ViewNumber::new(5), BlockId::new(20)),
    );

    let validated = validate_proposal(
        &committee,
        ViewNumber::new(5),
        [&view_5_non_leader_signed_block, &view_5_leader_signed_block],
        [m_notarization(BlockId::new(20), ViewNumber::new(2))],
        [
            nullification(ViewNumber::new(3)),
            nullification(ViewNumber::new(4)),
        ],
    )
    .expect("the leader-signed proposal block satisfies MM-VALID-PROPOSAL");

    assert_eq!(validated.block(), BlockId::new(50));
    assert_eq!(validated.view(), ViewNumber::new(5));
    assert_eq!(validated.parent().block(), BlockId::new(20));
    assert_eq!(validated.parent().view(), ViewNumber::new(2));
}

#[test]
fn rejects_only_same_view_block_when_not_signed_by_view_leader() {
    let committee = committee();
    let view_5_block_signed_by_validator_0 = SignedBlock::new(
        ValidatorId::new(0),
        proposed_block(BlockId::new(50), ViewNumber::new(5), BlockId::new(20)),
    );

    assert_eq!(
        validate_proposal(
            &committee,
            ViewNumber::new(5),
            [&view_5_block_signed_by_validator_0],
            [m_notarization(BlockId::new(20), ViewNumber::new(2))],
            [
                nullification(ViewNumber::new(3)),
                nullification(ViewNumber::new(4)),
            ],
        ),
        Err(ProposalValidationError::WrongLeader {
            view: ViewNumber::new(5),
            expected: ValidatorId::new(5),
            actual: ValidatorId::new(0),
        })
    );
}

#[test]
fn rejects_missing_parent_notarization() {
    let committee = committee();
    let view_5_leader_signed_block = signed_by_leader(
        &committee,
        proposed_block(BlockId::new(50), ViewNumber::new(5), BlockId::new(20)),
    );

    assert_eq!(
        validate_proposal(
            &committee,
            ViewNumber::new(5),
            [&view_5_leader_signed_block],
            [m_notarization(BlockId::new(10), ViewNumber::new(1))],
            [
                nullification(ViewNumber::new(3)),
                nullification(ViewNumber::new(4)),
            ],
        ),
        Err(ProposalValidationError::MissingParentNotarization {
            parent: BlockId::new(20),
        })
    );
}

#[test]
fn accepts_m_notarized_parent_even_when_select_parent_would_choose_later_block() {
    let committee = committee();
    let view_5_leader_signed_block = signed_by_leader(
        &committee,
        proposed_block(BlockId::new(50), ViewNumber::new(5), BlockId::new(10)),
    );

    let validated = validate_proposal(
        &committee,
        ViewNumber::new(5),
        [&view_5_leader_signed_block],
        [
            m_notarization(BlockId::new(10), ViewNumber::new(2)),
            m_notarization(BlockId::new(20), ViewNumber::new(3)),
        ],
        [
            nullification(ViewNumber::new(3)),
            nullification(ViewNumber::new(4)),
        ],
    )
    .expect("receiver-side validity does not require SelectParent(S, v)");

    assert_eq!(validated.block(), BlockId::new(50));
    assert_eq!(validated.view(), ViewNumber::new(5));
    assert_eq!(validated.parent().block(), BlockId::new(10));
    assert_eq!(validated.parent().view(), ViewNumber::new(2));
}

#[test]
fn rejects_missing_skipped_view_nullification() {
    let committee = committee();
    let view_5_leader_signed_block = signed_by_leader(
        &committee,
        proposed_block(BlockId::new(50), ViewNumber::new(5), BlockId::new(20)),
    );

    assert_eq!(
        validate_proposal(
            &committee,
            ViewNumber::new(5),
            [&view_5_leader_signed_block],
            [m_notarization(BlockId::new(20), ViewNumber::new(2))],
            [nullification(ViewNumber::new(3))],
        ),
        Err(ProposalValidationError::MissingNullification {
            view: ViewNumber::new(4),
        })
    );
}

#[test]
fn rejects_multiple_leader_signed_blocks_for_view() {
    let committee = committee();
    let first_view_5_leader_signed_block = signed_by_leader(
        &committee,
        proposed_block(BlockId::new(50), ViewNumber::new(5), BlockId::new(20)),
    );
    let second_view_5_leader_signed_block = signed_by_leader(
        &committee,
        proposed_block(BlockId::new(51), ViewNumber::new(5), BlockId::new(20)),
    );

    assert_eq!(
        validate_proposal(
            &committee,
            ViewNumber::new(5),
            [
                &first_view_5_leader_signed_block,
                &second_view_5_leader_signed_block,
            ],
            [m_notarization(BlockId::new(20), ViewNumber::new(2))],
            [
                nullification(ViewNumber::new(3)),
                nullification(ViewNumber::new(4)),
            ],
        ),
        Err(ProposalValidationError::ConflictingBlocks {
            view: ViewNumber::new(5),
            first: BlockId::new(50),
            second: BlockId::new(51),
        })
    );
}

#[test]
fn accepts_duplicate_nullification_evidence_for_skipped_view() {
    let committee = committee();
    let view_5_leader_signed_block = signed_by_leader(
        &committee,
        proposed_block(BlockId::new(50), ViewNumber::new(5), BlockId::new(20)),
    );

    let validated = validate_proposal(
        &committee,
        ViewNumber::new(5),
        [&view_5_leader_signed_block],
        [m_notarization(BlockId::new(20), ViewNumber::new(2))],
        [
            nullification(ViewNumber::new(3)),
            nullification(ViewNumber::new(3)),
            nullification(ViewNumber::new(4)),
        ],
    )
    .expect("duplicate local nullification evidence still covers the skipped view");

    assert_eq!(validated.block(), BlockId::new(50));
    assert_eq!(validated.view(), ViewNumber::new(5));
    assert_eq!(validated.parent().block(), BlockId::new(20));
    assert_eq!(validated.parent().view(), ViewNumber::new(2));
}
