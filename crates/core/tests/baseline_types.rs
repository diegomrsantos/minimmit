use minimmit_core::{
    Block, BlockError, BlockId, Nullify, TransactionId, ValidatorId, ViewNumber, Vote,
};

fn block(id: u64) -> BlockId {
    BlockId::new(id)
}

fn transaction(id: u64) -> TransactionId {
    TransactionId::new(id)
}

fn validator(id: u64) -> ValidatorId {
    ValidatorId::new(id)
}

fn view(number: u64) -> ViewNumber {
    ViewNumber::new(number)
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
