use std::{
    collections::{
        btree_map::Entry::{Occupied, Vacant},
        BTreeMap,
    },
    fmt,
};

use crate::{Block, MNotarization, Nullification, ValidatorId, ViewNumber};

/// Baseline proposal data carried by the protocol core.
///
/// A `Proposal` stores the modeled fields a proposer sends for a block:
/// proposer identity, proposed block, parent M-notarization, and any
/// skipped-view nullifications. Construction keeps nullifications in
/// deterministic view order and rejects duplicate nullifications for the same
/// view. Block validity, parent selection, proposer eligibility, and
/// state-machine transition rules are checked outside this data type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proposal {
    proposer: ValidatorId,
    block: Block,
    parent_notarization: MNotarization,
    nullifications: BTreeMap<ViewNumber, Nullification>,
}

impl Proposal {
    /// Creates a proposal from its carried block and evidence fields.
    ///
    /// Returns [`ProposalError::DuplicateNullification`] when more than one
    /// nullification targets the same view.
    pub fn new<I>(
        proposer: ValidatorId,
        block: Block,
        parent_notarization: MNotarization,
        nullifications: I,
    ) -> Result<Self, ProposalError>
    where
        I: IntoIterator<Item = Nullification>,
    {
        let mut by_view = BTreeMap::new();

        for nullification in nullifications {
            let view = nullification.view();
            match by_view.entry(view) {
                Vacant(entry) => {
                    entry.insert(nullification);
                }
                Occupied(_) => {
                    return Err(ProposalError::DuplicateNullification { view });
                }
            }
        }

        Ok(Self {
            proposer,
            block,
            parent_notarization,
            nullifications: by_view,
        })
    }

    /// Returns the modeled proposer identity.
    #[must_use]
    pub fn proposer(&self) -> ValidatorId {
        self.proposer
    }

    /// Returns the proposed block.
    #[must_use]
    pub fn block(&self) -> &Block {
        &self.block
    }

    /// Returns the parent M-notarization carried by the proposal.
    #[must_use]
    pub fn parent_notarization(&self) -> &MNotarization {
        &self.parent_notarization
    }

    /// Iterates skipped-view nullifications in deterministic view order.
    pub fn nullifications(&self) -> impl Iterator<Item = &Nullification> {
        self.nullifications.values()
    }
}

/// Proposal construction errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalError {
    /// The proposal carried more than one nullification for a view.
    DuplicateNullification {
        /// Duplicated nullification view.
        view: ViewNumber,
    },
}

impl fmt::Display for ProposalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateNullification { view } => {
                write!(
                    formatter,
                    "proposal contains more than one nullification for {view}"
                )
            }
        }
    }
}

impl std::error::Error for ProposalError {}

#[cfg(test)]
mod tests {
    use super::{Proposal, ProposalError};
    use crate::{
        Block, BlockId, Committee, MNotarization, Nullification, Nullify, TransactionId,
        ValidatorId, ViewNumber, Vote,
    };

    const ONE_FAULT: usize = 1;
    const MIN_VALIDATORS_WITH_ONE_FAULT: usize = 6;

    fn committee() -> Committee {
        Committee::new(validators(MIN_VALIDATORS_WITH_ONE_FAULT), ONE_FAULT)
            .expect("committee satisfies n >= 5f + 1")
    }

    fn m_notarization(block_id: BlockId, view: ViewNumber) -> MNotarization {
        MNotarization::from_votes(
            &committee(),
            [
                vote(0, block_id, view),
                vote(1, block_id, view),
                vote(2, block_id, view),
            ],
        )
        .expect("votes meet the M threshold")
    }

    fn nullification(view: ViewNumber) -> Nullification {
        Nullification::from_nullifies(
            &committee(),
            [nullify(0, view), nullify(1, view), nullify(2, view)],
        )
        .expect("nullify messages meet the threshold")
    }

    fn nullify(signer: u64, view: ViewNumber) -> Nullify {
        Nullify::new(ValidatorId::new(signer), view)
    }

    fn validators(count: usize) -> Vec<ValidatorId> {
        (0..count as u64).map(ValidatorId::new).collect()
    }

    fn vote(signer: u64, block_id: BlockId, view: ViewNumber) -> Vote {
        Vote::new(ValidatorId::new(signer), block_id, view)
    }

    #[test]
    fn preserves_fields_and_orders_nullifications_by_view() {
        let parent_notarization = m_notarization(BlockId::new(10), ViewNumber::new(3));
        let skipped_view_5 = nullification(ViewNumber::new(5));
        let skipped_view_4 = nullification(ViewNumber::new(4));
        let proposed_block = Block::new(
            BlockId::new(12),
            ViewNumber::new(6),
            BlockId::new(10),
            [TransactionId::new(1)],
        )
        .expect("block is valid");

        let proposal = Proposal::new(
            ValidatorId::new(3),
            proposed_block.clone(),
            parent_notarization.clone(),
            [skipped_view_5, skipped_view_4],
        )
        .expect("nullification views are unique");

        assert_eq!(proposal.proposer(), ValidatorId::new(3));
        assert_eq!(proposal.block(), &proposed_block);
        assert_eq!(proposal.parent_notarization(), &parent_notarization);
        assert_eq!(
            proposal
                .nullifications()
                .map(Nullification::view)
                .collect::<Vec<_>>(),
            [ViewNumber::new(4), ViewNumber::new(5)]
        );
    }

    #[test]
    fn rejects_duplicate_nullification_views() {
        let parent_notarization = m_notarization(BlockId::new(10), ViewNumber::new(3));
        let first = nullification(ViewNumber::new(5));
        let second = Nullification::from_nullifies(
            &committee(),
            [
                nullify(3, ViewNumber::new(5)),
                nullify(4, ViewNumber::new(5)),
                nullify(5, ViewNumber::new(5)),
            ],
        )
        .expect("view 5 is nullified");
        let proposed_block = Block::new(
            BlockId::new(12),
            ViewNumber::new(6),
            BlockId::new(10),
            [TransactionId::new(1)],
        )
        .expect("block is valid");

        assert_eq!(
            Proposal::new(
                ValidatorId::new(3),
                proposed_block,
                parent_notarization,
                [first, second]
            ),
            Err(ProposalError::DuplicateNullification {
                view: ViewNumber::new(5),
            })
        );
    }
}
