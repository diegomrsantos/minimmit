//! Core protocol state machine crate for Minimmit.
//!
//! Protocol behavior in this crate should stay deterministic and reviewable
//! from the core state machine.

mod block;
mod committee;
mod config;
mod evidence;
mod identity;

pub use block::{Block, BlockError, SignedBlock};
pub use committee::{Committee, CommitteeError};
pub use config::{Config, ConfigError};
pub use evidence::{EvidenceError, LNotarization, MNotarization, Nullification, Nullify, Vote};
pub use identity::{BlockId, TransactionId, ValidatorId, ViewNumber};

use std::{
    borrow::Borrow,
    collections::{
        btree_map::Entry::{Occupied, Vacant},
        BTreeMap, BTreeSet,
    },
    fmt,
};

const FIRST_NON_GENESIS_VIEW: ViewNumber = ViewNumber::new(1);

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

/// Protocol parent block with its associated view.
///
/// Genesis is represented explicitly because the initial core state assumes a
/// genesis block with genesis notarizations even when callers provide no
/// non-genesis M-notarization evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectedParent {
    block: BlockId,
    view: ViewNumber,
}

impl SelectedParent {
    /// Returns the implicit genesis parent.
    #[must_use]
    pub const fn genesis() -> Self {
        Self {
            block: BlockId::GENESIS,
            view: ViewNumber::GENESIS,
        }
    }

    /// Returns the parent block.
    #[must_use]
    pub const fn block(self) -> BlockId {
        self.block
    }

    /// Returns the parent view.
    #[must_use]
    pub const fn view(self) -> ViewNumber {
        self.view
    }
}

/// Successful proposal validation result.
///
/// This is the protocol-visible projection of the proposal predicate: the
/// unique valid proposal block for the view and the M-notarized parent that
/// proposal extends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidatedProposal {
    block: BlockId,
    view: ViewNumber,
    parent: SelectedParent,
}

impl ValidatedProposal {
    /// Returns the valid proposal block identity.
    #[must_use]
    pub const fn block(self) -> BlockId {
        self.block
    }

    /// Returns the proposal view.
    #[must_use]
    pub const fn view(self) -> ViewNumber {
        self.view
    }

    /// Returns the M-notarized parent the proposal extends.
    #[must_use]
    pub const fn parent(self) -> SelectedParent {
        self.parent
    }
}

/// Deterministic local processor state.
///
/// A `Processor` is one local participant executing the baseline Minimmit
/// protocol. The implementation identifies processors by [`ValidatorId`]
/// because authenticated protocol messages are modeled as signer identities
/// from the validator committee.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Processor {
    local_validator: ValidatorId,
    committee: Committee,
    current_view: ViewNumber,
}

impl Processor {
    /// Creates local processor state for a committee member.
    ///
    /// The initial state starts after genesis in view 1. The genesis block and
    /// genesis notarizations are implicit in the baseline protocol model.
    ///
    /// # Errors
    ///
    /// Returns [`ProcessorError::UnknownLocalValidator`] when `local_validator` is
    /// not a member of `committee`.
    pub fn new(local_validator: ValidatorId, committee: Committee) -> Result<Self, ProcessorError> {
        if !committee.contains(local_validator) {
            return Err(ProcessorError::UnknownLocalValidator {
                validator: local_validator,
            });
        }

        Ok(Self {
            local_validator,
            committee,
            current_view: FIRST_NON_GENESIS_VIEW,
        })
    }

    /// Returns the local validator identity represented by this processor.
    #[must_use]
    pub fn local_validator(&self) -> ValidatorId {
        self.local_validator
    }

    /// Returns the active validator committee.
    #[must_use]
    pub fn committee(&self) -> &Committee {
        &self.committee
    }

    /// Returns the current local view.
    #[must_use]
    pub fn current_view(&self) -> ViewNumber {
        self.current_view
    }

    /// Applies one deterministic input event and returns ready output.
    ///
    /// This initial transition boundary only supports [`Event::Noop`], which
    /// lets tests and later scenario harnesses drive and replay the public
    /// `Event -> Processor -> Ready` shape before protocol behaviors are added.
    #[must_use]
    pub fn step(&mut self, event: Event) -> Ready {
        match event {
            Event::Noop => Ready::none(),
        }
    }
}

/// Deterministic input observed by [`Processor`].
///
/// Protocol message, timeout, and evidence events are intentionally not
/// modeled here until their behavior is implemented with claim-specific
/// executable evidence.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// Explicit event with no protocol effect.
    Noop,
}

/// Deterministic output produced by a [`Processor`] transition.
///
/// The type is intentionally empty for the first transition-boundary slice.
/// Later protocol behavior can add ready outputs without exposing runtime
/// machinery through the processor API.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Ready {
    _private: (),
}

impl Ready {
    /// Returns an empty ready output.
    #[must_use]
    pub const fn none() -> Self {
        Self { _private: () }
    }

    /// Returns true when the transition produced no ready outputs.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        true
    }
}

/// Selects a parent block for `view` from prior M-notarizations.
///
/// This implements the baseline `MM-PARENT-SELECTION` claim: choose a block
/// from the greatest prior view with an M-notarization, breaking ties in that
/// view by the least [`BlockId`]. Genesis is implicit, so the result is
/// [`SelectedParent::genesis`] when no non-genesis prior M-notarization exists.
///
/// # Errors
///
/// Returns [`ParentSelectionError::GenesisView`] when asked to select a parent
/// for genesis.
pub fn select_parent<I, N>(
    m_notarizations: I,
    view: ViewNumber,
) -> Result<SelectedParent, ParentSelectionError>
where
    I: IntoIterator<Item = N>,
    N: Borrow<MNotarization>,
{
    if view == ViewNumber::GENESIS {
        return Err(ParentSelectionError::GenesisView { view });
    }

    let mut selected = SelectedParent::genesis();

    for notarization in m_notarizations {
        let notarization = notarization.borrow();
        let notarized_view = notarization.view();
        if notarized_view <= ViewNumber::GENESIS || notarized_view >= view {
            continue;
        }

        let notarized_block = notarization.block();
        if notarized_view > selected.view
            || (notarized_view == selected.view && notarized_block < selected.block)
        {
            selected = SelectedParent {
                block: notarized_block,
                view: notarized_view,
            };
        }
    }

    Ok(selected)
}

/// Validates the baseline proposal predicate for `view`.
///
/// This implements the `MM-VALID-PROPOSAL` claim over modeled local `S`
/// contents: exactly one view-`view` block signed by the deterministic leader,
/// an M-notarization for that block's parent, and at least one nullification
/// for every skipped view after that parent. Receiver-side proposal validity
/// does not require the parent to match the leader's local `SelectParent(S, v)`
/// result.
///
/// # Errors
///
/// Returns [`ProposalValidationError`] when the modeled local contents do not
/// satisfy the proposal-validity predicate.
pub fn validate_proposal<Blocks, MNotarizations, Nullifications>(
    committee: &Committee,
    view: ViewNumber,
    signed_blocks: Blocks,
    m_notarizations: MNotarizations,
    nullifications: Nullifications,
) -> Result<ValidatedProposal, ProposalValidationError>
where
    Blocks: IntoIterator,
    Blocks::Item: Borrow<SignedBlock>,
    MNotarizations: IntoIterator,
    MNotarizations::Item: Borrow<MNotarization>,
    Nullifications: IntoIterator,
    Nullifications::Item: Borrow<Nullification>,
{
    if view == ViewNumber::GENESIS {
        return Err(ProposalValidationError::GenesisView { view });
    }

    let signed_block = unique_leader_signed_block(committee, view, signed_blocks)?;
    let m_notarizations = collect_borrowed(m_notarizations);

    let parent = validate_parent(signed_block.block().parent(), &m_notarizations, view)?;
    validate_skipped_view_nullifications(parent.view(), view, nullifications)?;

    Ok(ValidatedProposal {
        block: signed_block.block().id(),
        view,
        parent,
    })
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

/// Parent selection errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParentSelectionError {
    /// Parent selection was requested for the genesis view.
    GenesisView {
        /// View used for the attempted parent selection.
        view: ViewNumber,
    },
}

impl fmt::Display for ParentSelectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GenesisView { view } => {
                write!(formatter, "cannot select a parent for {view}")
            }
        }
    }
}

impl std::error::Error for ParentSelectionError {}

/// Proposal validation errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalValidationError {
    /// Proposal validation was requested for the genesis view.
    GenesisView {
        /// View used for the attempted proposal validation.
        view: ViewNumber,
    },
    /// No candidate block was present for the requested view.
    MissingBlock {
        /// View without a candidate proposal block.
        view: ViewNumber,
    },
    /// A candidate block for the view was not signed by that view's leader.
    WrongLeader {
        /// Proposal view.
        view: ViewNumber,
        /// Expected deterministic leader.
        expected: ValidatorId,
        /// Actual signer.
        actual: ValidatorId,
    },
    /// More than one leader-signed candidate block was present for the view.
    ConflictingBlocks {
        /// Proposal view.
        view: ViewNumber,
        /// First candidate block identity.
        first: BlockId,
        /// Second candidate block identity.
        second: BlockId,
    },
    /// The proposed parent had no prior M-notarization evidence.
    MissingParentNotarization {
        /// Parent block identity from the proposal.
        parent: BlockId,
    },
    /// A skipped view after the proposed parent lacked nullification evidence.
    MissingNullification {
        /// Missing nullification view.
        view: ViewNumber,
    },
}

impl fmt::Display for ProposalValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GenesisView { view } => {
                write!(formatter, "cannot validate a proposal for {view}")
            }
            Self::MissingBlock { view } => {
                write!(formatter, "no proposal block is present for {view}")
            }
            Self::WrongLeader {
                view,
                expected,
                actual,
            } => write!(
                formatter,
                "proposal block for {view} was signed by {actual}, expected {expected}"
            ),
            Self::ConflictingBlocks {
                view,
                first,
                second,
            } => write!(
                formatter,
                "proposal for {view} has conflicting leader-signed blocks {first} and {second}"
            ),
            Self::MissingParentNotarization { parent } => {
                write!(
                    formatter,
                    "proposal parent {parent} has no prior M-notarization"
                )
            }
            Self::MissingNullification { view } => {
                write!(formatter, "proposal is missing nullification for {view}")
            }
        }
    }
}

impl std::error::Error for ProposalValidationError {}

/// Processor state construction errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessorError {
    /// The local validator is not in the active committee.
    UnknownLocalValidator {
        /// Non-member local validator identity.
        validator: ValidatorId,
    },
}

impl fmt::Display for ProcessorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownLocalValidator { validator } => {
                write!(formatter, "{validator} is not a committee member")
            }
        }
    }
}

impl std::error::Error for ProcessorError {}

/// Clones borrowed-or-owned modeled inputs into an owned list for validation.
///
/// The public validators accept either owned values or references; owning the
/// list locally lets later checks make multiple deterministic passes.
fn collect_borrowed<T, B, I>(items: I) -> Vec<T>
where
    T: Clone,
    B: Borrow<T>,
    I: IntoIterator<Item = B>,
{
    items
        .into_iter()
        .map(|item| item.borrow().clone())
        .collect()
}

/// Returns the unique leader-signed block for `view` from modeled local input.
///
/// Blocks for other views are ignored because the proposal predicate is scoped
/// to one requested view. Non-leader-signed same-view blocks are unrelated
/// local evidence when exactly one leader-signed block is present; if no
/// leader-signed block exists, the first same-view non-leader signature reports
/// the signer mismatch.
fn unique_leader_signed_block<B, I>(
    committee: &Committee,
    view: ViewNumber,
    signed_blocks: I,
) -> Result<SignedBlock, ProposalValidationError>
where
    B: Borrow<SignedBlock>,
    I: IntoIterator<Item = B>,
{
    let leader = committee.leader(view);
    let mut selected: Option<SignedBlock> = None;
    let mut first_wrong_signer = None;

    for signed_block in signed_blocks {
        let signed_block = signed_block.borrow();
        if signed_block.block().view() != view {
            continue;
        }

        if signed_block.signer() != leader {
            first_wrong_signer.get_or_insert(signed_block.signer());
            continue;
        }

        if let Some(first) = &selected {
            return Err(ProposalValidationError::ConflictingBlocks {
                view,
                first: first.block().id(),
                second: signed_block.block().id(),
            });
        }

        selected = Some(signed_block.clone());
    }

    if let Some(signed_block) = selected {
        return Ok(signed_block);
    }

    if let Some(actual) = first_wrong_signer {
        return Err(ProposalValidationError::WrongLeader {
            view,
            expected: leader,
            actual,
        });
    }

    Err(ProposalValidationError::MissingBlock { view })
}

/// Returns the prior M-notarized parent that the proposal extends.
///
/// Genesis is implicit. For non-genesis parents, any prior M-notarization for
/// the proposed parent is enough; unrelated later M-notarizations do not affect
/// receiver-side proposal validity.
fn validate_parent(
    parent: BlockId,
    m_notarizations: &[MNotarization],
    view: ViewNumber,
) -> Result<SelectedParent, ProposalValidationError> {
    if parent == BlockId::GENESIS {
        return Ok(SelectedParent::genesis());
    }

    let parent_view = m_notarizations
        .iter()
        .filter(|notarization| {
            notarization.block() == parent
                && notarization.view() > ViewNumber::GENESIS
                && notarization.view() < view
        })
        .map(MNotarization::view)
        .max();

    parent_view
        .map(|view| SelectedParent {
            block: parent,
            view,
        })
        .ok_or(ProposalValidationError::MissingParentNotarization { parent })
}

/// Checks that every skipped view after `parent_view` has nullification evidence.
///
/// Extra nullifications and duplicate proofs for an already-covered skipped
/// view are harmless for this receiver-side predicate.
fn validate_skipped_view_nullifications<N, I>(
    parent_view: ViewNumber,
    proposal_view: ViewNumber,
    nullifications: I,
) -> Result<(), ProposalValidationError>
where
    N: Borrow<Nullification>,
    I: IntoIterator<Item = N>,
{
    let mut covered = BTreeSet::new();

    for nullification in nullifications {
        let nullified_view = nullification.borrow().view();
        if parent_view < nullified_view && nullified_view < proposal_view {
            covered.insert(nullified_view);
        }
    }

    for skipped_view in (parent_view.get() + 1)..proposal_view.get() {
        let skipped_view = ViewNumber::new(skipped_view);
        if !covered.contains(&skipped_view) {
            return Err(ProposalValidationError::MissingNullification { view: skipped_view });
        }
    }

    Ok(())
}
