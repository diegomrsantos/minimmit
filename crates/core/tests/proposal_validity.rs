#[path = "common/proposal_validity.rs"]
mod common;

use common::{
    block, committee, m_notarization, nullification, proposed_block, signed_by_leader, validator,
    view,
};
use minimmit_core::{validate_proposal, ProposalValidationError, SignedBlock};

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
