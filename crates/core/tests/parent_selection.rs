mod common;

use common::{block, m_notarization, view};
use minimmit_core::{
    select_parent, MNotarization, ParentSelectionError, SelectedParent, ViewNumber,
};

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
