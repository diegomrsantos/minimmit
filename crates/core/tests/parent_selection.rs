use minimmit_core::{
    select_parent, BlockId, Committee, MNotarization, ParentSelectionError, SelectedParent,
    ValidatorId, ViewNumber, Vote,
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

#[test]
fn selects_genesis_when_no_prior_m_notarization_exists() {
    let selected = select_parent(std::iter::empty::<MNotarization>(), view(1))
        .expect("view after genesis can select a parent");

    assert_eq!(selected, SelectedParent::genesis());
}

#[test]
fn ignores_current_and_future_m_notarizations() {
    let selected = select_parent([m_notarization(10, 2), m_notarization(11, 3)], view(2))
        .expect("view after genesis can select a parent");

    assert_eq!(selected, SelectedParent::genesis());
}

#[test]
fn prefers_greatest_prior_m_notarized_view() {
    let selected = select_parent(
        [
            m_notarization(10, 1),
            m_notarization(11, 3),
            m_notarization(12, 2),
        ],
        view(4),
    )
    .expect("view after genesis can select a parent");

    assert_eq!(selected.block(), block(11));
    assert_eq!(selected.view(), view(3));
}

#[test]
fn breaks_same_view_ties_by_least_block() {
    let selected = select_parent(
        [
            m_notarization(30, 3),
            m_notarization(20, 3),
            m_notarization(25, 3),
        ],
        view(4),
    )
    .expect("view after genesis can select a parent");

    assert_eq!(selected.block(), block(20));
    assert_eq!(selected.view(), view(3));
}

#[test]
fn is_independent_of_m_notarization_order() {
    let first = select_parent(
        [
            m_notarization(30, 3),
            m_notarization(10, 2),
            m_notarization(20, 3),
        ],
        view(4),
    )
    .expect("view after genesis can select a parent");
    let second = select_parent(
        [
            m_notarization(20, 3),
            m_notarization(10, 2),
            m_notarization(30, 3),
        ],
        view(4),
    )
    .expect("view after genesis can select a parent");

    assert_eq!(first, second);
    assert_eq!(first.block(), block(20));
    assert_eq!(first.view(), view(3));
}

#[test]
fn rejects_parent_selection_for_genesis_view() {
    assert_eq!(
        select_parent(std::iter::empty::<MNotarization>(), ViewNumber::GENESIS),
        Err(ParentSelectionError::GenesisView {
            view: ViewNumber::GENESIS,
        })
    );
}
