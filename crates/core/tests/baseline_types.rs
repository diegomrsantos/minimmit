use minimmit_core::{
    Block, BlockError, BlockId, Committee, EvidenceError, LNotarization, MNotarization,
    Nullification, Nullify, TransactionId, ValidatorId, ViewNumber, Vote,
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

#[test]
fn identifiers_order_by_inner_value() {
    let mut views = [view(2), view(0), view(1)];
    views.sort();

    let mut blocks = [block(7), block(3), block(5)];
    blocks.sort();

    let mut transactions = [transaction(11), transaction(10), transaction(12)];
    transactions.sort();

    assert_eq!(views, [view(0), view(1), view(2)]);
    assert_eq!(blocks, [block(3), block(5), block(7)]);
    assert_eq!(
        transactions,
        [transaction(10), transaction(11), transaction(12)]
    );
}

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
fn vote_records_signer_block_and_view() {
    let vote = Vote::new(validator(2), block(10), view(3));

    assert_eq!(vote.signer(), validator(2));
    assert_eq!(vote.block(), block(10));
    assert_eq!(vote.view(), view(3));
}

#[test]
fn nullify_records_signer_and_view() {
    let nullify = Nullify::new(validator(2), view(3));

    assert_eq!(nullify.signer(), validator(2));
    assert_eq!(nullify.view(), view(3));
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
