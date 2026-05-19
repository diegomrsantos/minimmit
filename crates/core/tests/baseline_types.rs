mod common;

use common::{
    block, committee, m_notarization, nullification, nullify, transaction, validator, view,
};
use minimmit_core::{Block, Nullification, Proposal, ProposalError};

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
