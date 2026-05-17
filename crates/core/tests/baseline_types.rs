use minimmit_core::{BlockId, TransactionId, ViewNumber};

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
