use minimmit_core::{Block, BlockError, BlockId, TransactionId, ViewNumber};

fn block(id: u64) -> BlockId {
    BlockId::new(id)
}

fn transaction(id: u64) -> TransactionId {
    TransactionId::new(id)
}

fn view(number: u64) -> ViewNumber {
    ViewNumber::new(number)
}

#[test]
fn ids_sort_deterministically() {
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
    .expect("transactions are distinct");

    assert_eq!(built.id(), block(10));
    assert_eq!(built.view(), view(2));
    assert_eq!(built.parent(), Some(block(5)));
    assert_eq!(built.transactions(), &[transaction(3), transaction(1)]);
}

#[test]
fn genesis_block_has_no_parent_or_transactions() {
    let genesis = Block::genesis(block(0));

    assert_eq!(genesis.id(), block(0));
    assert_eq!(genesis.view(), view(0));
    assert_eq!(genesis.parent(), None);
    assert!(genesis.transactions().is_empty());
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
