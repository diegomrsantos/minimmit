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
