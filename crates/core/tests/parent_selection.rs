mod common;

use common::m_notarization;
use minimmit_core::{
    select_parent, BlockId, MNotarization, ParentSelectionError, SelectedParent, ViewNumber,
};

#[test]
fn selects_genesis_when_no_prior_m_notarization_exists() {
    let selected = select_parent(std::iter::empty::<MNotarization>(), ViewNumber::new(1))
        .expect("view after genesis can select a parent");

    assert_eq!(selected, SelectedParent::genesis());
}

#[test]
fn ignores_current_and_future_m_notarizations() {
    let selected = select_parent(
        [
            m_notarization(BlockId::new(10), ViewNumber::new(2)),
            m_notarization(BlockId::new(11), ViewNumber::new(3)),
        ],
        ViewNumber::new(2),
    )
    .expect("view after genesis can select a parent");

    assert_eq!(selected, SelectedParent::genesis());
}

#[test]
fn prefers_greatest_prior_m_notarized_view() {
    let selected = select_parent(
        [
            m_notarization(BlockId::new(10), ViewNumber::new(1)),
            m_notarization(BlockId::new(11), ViewNumber::new(3)),
            m_notarization(BlockId::new(12), ViewNumber::new(2)),
        ],
        ViewNumber::new(4),
    )
    .expect("view after genesis can select a parent");

    assert_eq!(selected.block(), BlockId::new(11));
    assert_eq!(selected.view(), ViewNumber::new(3));
}

#[test]
fn breaks_same_view_ties_by_least_block() {
    let selected = select_parent(
        [
            m_notarization(BlockId::new(30), ViewNumber::new(3)),
            m_notarization(BlockId::new(20), ViewNumber::new(3)),
            m_notarization(BlockId::new(25), ViewNumber::new(3)),
        ],
        ViewNumber::new(4),
    )
    .expect("view after genesis can select a parent");

    assert_eq!(selected.block(), BlockId::new(20));
    assert_eq!(selected.view(), ViewNumber::new(3));
}

#[test]
fn is_independent_of_m_notarization_order() {
    let first = select_parent(
        [
            m_notarization(BlockId::new(30), ViewNumber::new(3)),
            m_notarization(BlockId::new(10), ViewNumber::new(2)),
            m_notarization(BlockId::new(20), ViewNumber::new(3)),
        ],
        ViewNumber::new(4),
    )
    .expect("view after genesis can select a parent");
    let second = select_parent(
        [
            m_notarization(BlockId::new(20), ViewNumber::new(3)),
            m_notarization(BlockId::new(10), ViewNumber::new(2)),
            m_notarization(BlockId::new(30), ViewNumber::new(3)),
        ],
        ViewNumber::new(4),
    )
    .expect("view after genesis can select a parent");

    assert_eq!(first, second);
    assert_eq!(first.block(), BlockId::new(20));
    assert_eq!(first.view(), ViewNumber::new(3));
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
