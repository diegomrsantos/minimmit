mod common;

use common::{
    block, committee, m_notarization, nullification, nullify, transaction, validator, view, vote,
};
use minimmit_core::{
    Block, BlockError, EvidenceError, LNotarization, MNotarization, Nullification, Proposal,
    ProposalError,
};

#[test]
fn block_preserves_parent_and_transaction_order() {
    let built = Block::new(
        block(10),
        view(2),
        block(5),
        [transaction(3), transaction(1)],
    )
    .expect("block is valid");

    assert_eq!(built.id(), block(10));
    assert_eq!(built.view(), view(2));
    assert_eq!(built.parent(), block(5));
    assert_eq!(built.transactions(), &[transaction(3), transaction(1)]);
}

#[test]
fn block_rejects_genesis_view() {
    assert_eq!(
        Block::new(block(10), view(0), block(5), []),
        Err(BlockError::GenesisView { view: view(0) })
    );
}

#[test]
fn block_rejects_duplicate_transactions() {
    assert_eq!(
        Block::new(
            block(10),
            view(2),
            block(5),
            [transaction(3), transaction(1), transaction(3)],
        ),
        Err(BlockError::DuplicateTransaction {
            transaction: transaction(3),
        })
    );
}

#[test]
fn m_notarization_accepts_distinct_valid_threshold_votes_for_one_block() {
    let committee = committee();
    let notarization =
        MNotarization::from_votes(&committee, [vote(2, 10, 3), vote(0, 10, 3), vote(1, 10, 3)])
            .expect("three valid distinct votes meet the M threshold when f = 1");

    assert_eq!(notarization.block(), block(10));
    assert_eq!(notarization.view(), view(3));
    assert_eq!(
        notarization.signers().collect::<Vec<_>>(),
        [validator(0), validator(1), validator(2)]
    );
}

#[test]
fn m_notarization_rejects_below_threshold_votes() {
    let committee = committee();

    assert_eq!(
        MNotarization::from_votes(&committee, [vote(0, 10, 3), vote(1, 10, 3)]),
        Err(EvidenceError::BelowThreshold {
            signer_count: 2,
            threshold: committee.config().m_threshold(),
        })
    );
}

#[test]
fn l_notarization_accepts_n_minus_f_threshold_votes_for_one_block() {
    let committee = committee();
    let notarization = LNotarization::from_votes(
        &committee,
        [
            vote(4, 10, 3),
            vote(0, 10, 3),
            vote(2, 10, 3),
            vote(1, 10, 3),
            vote(3, 10, 3),
        ],
    )
    .expect("five valid distinct votes meet the L threshold when n = 6 and f = 1");

    assert_eq!(notarization.block(), block(10));
    assert_eq!(notarization.view(), view(3));
    assert_eq!(
        notarization.signers().collect::<Vec<_>>(),
        [
            validator(0),
            validator(1),
            validator(2),
            validator(3),
            validator(4),
        ]
    );
}

#[test]
fn l_notarization_rejects_below_threshold_votes() {
    let committee = committee();

    assert_eq!(
        LNotarization::from_votes(&committee, [vote(0, 10, 3), vote(1, 10, 3), vote(2, 10, 3)]),
        Err(EvidenceError::BelowThreshold {
            signer_count: 3,
            threshold: committee.config().l_threshold(),
        })
    );
}

#[test]
fn nullification_accepts_distinct_valid_threshold_messages_for_one_view() {
    let committee = committee();
    let nullification =
        Nullification::from_nullifies(&committee, [nullify(2, 3), nullify(0, 3), nullify(1, 3)])
            .expect("three valid distinct nullify messages meet the threshold when f = 1");

    assert_eq!(nullification.view(), view(3));
    assert_eq!(
        nullification.signers().collect::<Vec<_>>(),
        [validator(0), validator(1), validator(2)]
    );
}

#[test]
fn nullification_rejects_below_threshold_messages() {
    let committee = committee();

    assert_eq!(
        Nullification::from_nullifies(&committee, [nullify(0, 3), nullify(1, 3)]),
        Err(EvidenceError::BelowThreshold {
            signer_count: 2,
            threshold: committee.config().nullification_threshold(),
        })
    );
}

#[test]
fn evidence_rejects_empty_inputs() {
    let committee = committee();

    assert_eq!(
        MNotarization::from_votes(&committee, []),
        Err(EvidenceError::Empty)
    );
    assert_eq!(
        Nullification::from_nullifies(&committee, []),
        Err(EvidenceError::Empty)
    );
}

#[test]
fn evidence_rejects_duplicate_signers() {
    let committee = committee();

    assert_eq!(
        MNotarization::from_votes(&committee, [vote(0, 10, 3), vote(1, 10, 3), vote(0, 10, 3)],),
        Err(EvidenceError::DuplicateSigner {
            signer: validator(0),
        })
    );
    assert_eq!(
        Nullification::from_nullifies(&committee, [nullify(0, 3), nullify(1, 3), nullify(0, 3)],),
        Err(EvidenceError::DuplicateSigner {
            signer: validator(0),
        })
    );
}

#[test]
fn evidence_rejects_unknown_signers() {
    let committee = committee();

    assert_eq!(
        MNotarization::from_votes(
            &committee,
            [vote(0, 10, 3), vote(1, 10, 3), vote(99, 10, 3)],
        ),
        Err(EvidenceError::UnknownSigner {
            signer: validator(99),
        })
    );
    assert_eq!(
        Nullification::from_nullifies(&committee, [nullify(0, 3), nullify(1, 3), nullify(99, 3)],),
        Err(EvidenceError::UnknownSigner {
            signer: validator(99),
        })
    );
}

#[test]
fn evidence_rejects_mixed_vote_targets() {
    let committee = committee();

    assert_eq!(
        MNotarization::from_votes(&committee, [vote(0, 10, 3), vote(1, 11, 3), vote(2, 10, 3)],),
        Err(EvidenceError::ConflictingVoteTarget {
            expected_block: block(10),
            expected_view: view(3),
            actual_block: block(11),
            actual_view: view(3),
        })
    );
}

#[test]
fn evidence_rejects_mixed_nullification_views() {
    let committee = committee();

    assert_eq!(
        Nullification::from_nullifies(&committee, [nullify(0, 3), nullify(1, 4), nullify(2, 3)],),
        Err(EvidenceError::ConflictingNullificationView {
            expected_view: view(3),
            actual_view: view(4),
        })
    );
}

#[test]
fn proposal_preserves_fields_and_orders_nullifications_by_view() {
    let parent_notarization = m_notarization(10, 3);
    let skipped_view_5 = nullification(5);
    let skipped_view_4 = nullification(4);
    let proposed_block =
        Block::new(block(12), view(6), block(10), [transaction(1)]).expect("block is valid");

    let proposal = Proposal::new(
        validator(3),
        proposed_block.clone(),
        parent_notarization.clone(),
        [skipped_view_5, skipped_view_4],
    )
    .expect("nullification views are unique");

    assert_eq!(proposal.proposer(), validator(3));
    assert_eq!(proposal.block(), &proposed_block);
    assert_eq!(proposal.parent_notarization(), &parent_notarization);
    assert_eq!(
        proposal
            .nullifications()
            .map(Nullification::view)
            .collect::<Vec<_>>(),
        [view(4), view(5)]
    );
}

#[test]
fn proposal_rejects_duplicate_nullification_views() {
    let parent_notarization = m_notarization(10, 3);
    let first = nullification(5);
    let second =
        Nullification::from_nullifies(&committee(), [nullify(3, 5), nullify(4, 5), nullify(5, 5)])
            .expect("view 5 is nullified");
    let proposed_block =
        Block::new(block(12), view(6), block(10), [transaction(1)]).expect("block is valid");

    assert_eq!(
        Proposal::new(
            validator(3),
            proposed_block,
            parent_notarization,
            [first, second]
        ),
        Err(ProposalError::DuplicateNullification { view: view(5) })
    );
}
