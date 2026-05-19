use std::{collections::BTreeSet, fmt};

use crate::{BlockId, TransactionId, ValidatorId, ViewNumber};

/// Baseline block data for views after genesis.
///
/// A `Block` carries the fields later protocol rules compare or validate for a
/// proposed block: its identity, view, parent identity, and ordered transaction
/// identifiers. View 0 is reserved for genesis, so [`Block::new`] rejects it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    id: BlockId,
    view: ViewNumber,
    parent: BlockId,
    transactions: Vec<TransactionId>,
}

impl Block {
    /// Creates a block for a non-genesis view.
    ///
    /// The parent is required, and transactions keep caller-provided order
    /// because block contents are order-sensitive. Duplicate transaction
    /// identifiers are rejected so a constructed block has one canonical
    /// transaction list.
    pub fn new<I>(
        id: BlockId,
        view: ViewNumber,
        parent: BlockId,
        transactions: I,
    ) -> Result<Self, BlockError>
    where
        I: IntoIterator<Item = TransactionId>,
    {
        if view == ViewNumber::GENESIS {
            return Err(BlockError::GenesisView { view });
        }

        let transactions = distinct_transactions(transactions)?;

        Ok(Self {
            id,
            view,
            parent,
            transactions,
        })
    }

    /// Returns the block identity.
    #[must_use]
    pub fn id(&self) -> BlockId {
        self.id
    }

    /// Returns the block view.
    #[must_use]
    pub fn view(&self) -> ViewNumber {
        self.view
    }

    /// Returns the parent block identity.
    #[must_use]
    pub fn parent(&self) -> BlockId {
        self.parent
    }

    /// Returns the modeled transactions in block order.
    #[must_use]
    pub fn transactions(&self) -> &[TransactionId] {
        &self.transactions
    }
}

/// Modeled signed block proposal input.
///
/// Real cryptographic verification stays outside the core crate. This type
/// records the identity that authenticated a block so later pure proposal
/// validity rules can check whether the signer is the view leader.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedBlock {
    signer: ValidatorId,
    block: Block,
}

impl SignedBlock {
    /// Creates a signed block input.
    #[must_use]
    pub fn new(signer: ValidatorId, block: Block) -> Self {
        Self { signer, block }
    }

    /// Returns the validator identity that signed the block.
    #[must_use]
    pub fn signer(&self) -> ValidatorId {
        self.signer
    }

    /// Returns the signed block.
    #[must_use]
    pub fn block(&self) -> &Block {
        &self.block
    }
}

/// Block construction errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockError {
    /// The block view was reserved for genesis.
    GenesisView {
        /// View used for the attempted block construction.
        view: ViewNumber,
    },
    /// The block listed a transaction more than once.
    DuplicateTransaction {
        /// Duplicated transaction identity.
        transaction: TransactionId,
    },
}

impl fmt::Display for BlockError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GenesisView { view } => {
                write!(formatter, "{view} is reserved for genesis")
            }
            Self::DuplicateTransaction { transaction } => {
                write!(
                    formatter,
                    "{transaction} appears more than once in the block"
                )
            }
        }
    }
}

impl std::error::Error for BlockError {}

fn distinct_transactions<I>(transactions: I) -> Result<Vec<TransactionId>, BlockError>
where
    I: IntoIterator<Item = TransactionId>,
{
    let mut transaction_set = BTreeSet::new();
    let mut transaction_list = Vec::new();

    for transaction in transactions {
        if !transaction_set.insert(transaction) {
            return Err(BlockError::DuplicateTransaction { transaction });
        }

        transaction_list.push(transaction);
    }

    Ok(transaction_list)
}

#[cfg(test)]
mod tests {
    use super::{Block, BlockError};
    use crate::{BlockId, TransactionId, ViewNumber};

    #[test]
    fn preserves_parent_and_transaction_order() {
        let built = Block::new(
            BlockId::new(10),
            ViewNumber::new(2),
            BlockId::new(5),
            [TransactionId::new(3), TransactionId::new(1)],
        )
        .expect("block is valid");

        assert_eq!(built.id(), BlockId::new(10));
        assert_eq!(built.view(), ViewNumber::new(2));
        assert_eq!(built.parent(), BlockId::new(5));
        assert_eq!(
            built.transactions(),
            &[TransactionId::new(3), TransactionId::new(1)]
        );
    }

    #[test]
    fn rejects_genesis_view() {
        assert_eq!(
            Block::new(BlockId::new(10), ViewNumber::new(0), BlockId::new(5), []),
            Err(BlockError::GenesisView {
                view: ViewNumber::new(0),
            })
        );
    }

    #[test]
    fn rejects_duplicate_transactions() {
        assert_eq!(
            Block::new(
                BlockId::new(10),
                ViewNumber::new(2),
                BlockId::new(5),
                [
                    TransactionId::new(3),
                    TransactionId::new(1),
                    TransactionId::new(3),
                ],
            ),
            Err(BlockError::DuplicateTransaction {
                transaction: TransactionId::new(3),
            })
        );
    }
}
