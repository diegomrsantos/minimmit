//! Core protocol state machine crate for Minimmit.
//!
//! Protocol behavior in this crate should stay deterministic and reviewable
//! from the core state machine.

use std::{
    borrow::Borrow,
    collections::{
        btree_map::Entry::{Occupied, Vacant},
        BTreeMap, BTreeSet,
    },
    fmt,
};

const MIN_VALIDATOR_FAULT_FACTOR: usize = 5;
const M_AND_NULLIFICATION_FAULT_FACTOR: usize = 2;
const THRESHOLD_BASE: usize = 1;
const FIRST_NON_GENESIS_VIEW: ViewNumber = ViewNumber::new(1);

/// Validator identity used by the protocol core.
///
/// This is a deterministic stand-in for authenticated signer identity. Real
/// cryptographic verification stays outside the core crate; protocol rules
/// count only identities that are members of the configured committee.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ValidatorId(u64);

impl ValidatorId {
    /// Creates a validator identity.
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the underlying deterministic identity value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ValidatorId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "validator {}", self.0)
    }
}

/// Baseline Minimmit view number.
///
/// The wrapped value is the protocol's scalar view index. Ordering follows
/// that numeric index so later state-machine rules can compare views without
/// carrying raw integers through the public API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ViewNumber(u64);

impl ViewNumber {
    /// The genesis view.
    pub const GENESIS: Self = Self(0);

    /// Creates a view number from its deterministic counter value.
    #[must_use]
    pub const fn new(number: u64) -> Self {
        Self(number)
    }

    /// Returns the underlying deterministic view number.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ViewNumber {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "view {}", self.0)
    }
}

/// Deterministic block identity used by the protocol core.
///
/// The paper compares blocks by their hashes. The core models that hash as an
/// opaque ordered scalar so tests can express block identity and tie-breaking
/// without introducing hashing or serialization concerns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockId(u64);

impl BlockId {
    /// The genesis block identity.
    pub const GENESIS: Self = Self(0);

    /// Creates a block identity from its modeled hash value.
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the underlying deterministic block identity value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for BlockId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "block {}", self.0)
    }
}

/// Opaque transaction identity carried by modeled blocks.
///
/// The core only needs stable transaction identity and ordering for baseline
/// block data. Transaction payloads and application semantics are intentionally
/// outside this protocol crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TransactionId(u64);

impl TransactionId {
    /// Creates a transaction identity from its deterministic value.
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the underlying deterministic transaction identity value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for TransactionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "transaction {}", self.0)
    }
}

/// Protocol configuration for baseline Minimmit.
///
/// A configuration fixes the paper parameters `n` and `f`, where `n` is the
/// validator set size and `f` is the maximum number of Byzantine processors.
/// It is valid only when `n >= 5f + 1`.
///
/// The configuration precomputes the protocol thresholds:
///
/// - M-notarization threshold: `2f + 1`
/// - nullification threshold: `2f + 1`
/// - L-notarization threshold: `n - f`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    validator_count: usize,
    fault_bound: usize,
    m_threshold: usize,
    l_threshold: usize,
}

impl Config {
    /// Creates a configuration from `n` and `f`.
    ///
    /// Returns [`ConfigError::TooFewValidators`] when `n < 5f + 1`.
    /// Returns [`ConfigError::ThresholdOverflow`] when a threshold formula
    /// cannot fit in `usize`.
    pub fn new(validator_count: usize, fault_bound: usize) -> Result<Self, ConfigError> {
        let minimum_validator_count = minimum_validator_count(fault_bound)?;

        if validator_count < minimum_validator_count {
            return Err(ConfigError::TooFewValidators {
                validator_count,
                fault_bound,
                minimum_validator_count,
            });
        }

        let m_threshold = threshold(M_AND_NULLIFICATION_FAULT_FACTOR, fault_bound)?;
        let l_threshold = validator_count - fault_bound;

        Ok(Self {
            validator_count,
            fault_bound,
            m_threshold,
            l_threshold,
        })
    }

    /// Returns `n`, the validator set size.
    #[must_use]
    pub fn validator_count(&self) -> usize {
        self.validator_count
    }

    /// Returns `f`, the maximum number of Byzantine processors.
    #[must_use]
    pub fn fault_bound(&self) -> usize {
        self.fault_bound
    }

    /// Returns the M-notarization threshold, `2f + 1`.
    #[must_use]
    pub fn m_threshold(&self) -> usize {
        self.m_threshold
    }

    /// Returns the nullification threshold, `2f + 1`.
    #[must_use]
    pub fn nullification_threshold(&self) -> usize {
        self.m_threshold
    }

    /// Returns the L-notarization threshold, `n - f`.
    #[must_use]
    pub fn l_threshold(&self) -> usize {
        self.l_threshold
    }
}

/// Configuration errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigError {
    /// `n` is smaller than the required `5f + 1` validator count.
    TooFewValidators {
        /// Provided validator set size.
        validator_count: usize,
        /// Provided fault bound.
        fault_bound: usize,
        /// Minimum validator set size for the provided fault bound.
        minimum_validator_count: usize,
    },
    /// A threshold calculation overflowed `usize`.
    ThresholdOverflow {
        /// Fault bound used in the overflowing threshold calculation.
        fault_bound: usize,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooFewValidators {
                validator_count,
                fault_bound,
                minimum_validator_count,
            } => write!(
                formatter,
                "validator count {validator_count} is below the required minimum {minimum_validator_count} for f = {fault_bound}"
            ),
            Self::ThresholdOverflow { fault_bound } => {
                write!(formatter, "threshold calculation overflowed for f = {fault_bound}")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// Deterministic validator committee for baseline Minimmit.
///
/// The committee defines which signer identities can contribute to threshold
/// evidence. Sender counts derived from this type ignore identities outside
/// the committee and count duplicate committee members only once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Committee {
    config: Config,
    validators: BTreeSet<ValidatorId>,
}

impl Committee {
    /// Creates a committee from unique validator identities and fault bound
    /// `f`.
    ///
    /// The committee size is the configuration's `n`, so the validator set
    /// must be unique and satisfy `n >= 5f + 1`.
    pub fn new<I>(validators: I, fault_bound: usize) -> Result<Self, CommitteeError>
    where
        I: IntoIterator<Item = ValidatorId>,
    {
        let mut validator_set = BTreeSet::new();

        for validator in validators {
            if !validator_set.insert(validator) {
                return Err(CommitteeError::DuplicateValidator { validator });
            }
        }

        let config =
            Config::new(validator_set.len(), fault_bound).map_err(CommitteeError::InvalidConfig)?;

        Ok(Self {
            config,
            validators: validator_set,
        })
    }

    /// Returns the protocol configuration for this committee.
    #[must_use]
    pub fn config(&self) -> Config {
        self.config
    }

    /// Returns true when the validator is a committee member.
    #[must_use]
    pub fn contains(&self, validator: ValidatorId) -> bool {
        self.validators.contains(&validator)
    }

    /// Iterates validators in deterministic identity order.
    pub fn validators(&self) -> impl Iterator<Item = ValidatorId> + '_ {
        self.validators.iter().copied()
    }

    /// Returns the deterministic leader for `view`.
    ///
    /// The paper defines `lead(v)` by indexing processors modulo `n`. The core
    /// uses committee identity order as the deterministic processor order.
    #[must_use]
    pub fn leader(&self, view: ViewNumber) -> ValidatorId {
        let validator_count = self.validators.len() as u64;
        let leader_index = (view.get() % validator_count) as usize;

        self.validators
            .iter()
            .copied()
            .nth(leader_index)
            .expect("validated committees are non-empty")
    }

    /// Counts distinct senders that are members of this committee.
    ///
    /// Duplicate senders count once and non-members do not contribute.
    pub fn count_distinct_valid_senders<I>(&self, senders: I) -> usize
    where
        I: IntoIterator<Item = ValidatorId>,
    {
        let mut distinct = BTreeSet::new();

        for sender in senders {
            if self.contains(sender) {
                distinct.insert(sender);
            }
        }

        distinct.len()
    }
}

/// Committee construction errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitteeError {
    /// The validator list contained the same identity more than once.
    DuplicateValidator {
        /// Duplicated validator identity.
        validator: ValidatorId,
    },
    /// The committee size and fault bound did not form a valid configuration.
    InvalidConfig(ConfigError),
}

impl fmt::Display for CommitteeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateValidator { validator } => {
                write!(
                    formatter,
                    "{validator} appears more than once in the committee"
                )
            }
            Self::InvalidConfig(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for CommitteeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::DuplicateValidator { .. } => None,
            Self::InvalidConfig(error) => Some(error),
        }
    }
}

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

/// Modeled vote message from one validator for one block in one view.
///
/// A `Vote` is the raw input later evidence constructors aggregate. It records
/// the signer identity that authenticated the message, the voted block, and
/// the view in which the vote was cast. Protocol validity is contextual: it
/// depends on the active committee, distinct signer counting, quorum threshold,
/// and state-machine rules. This type stores the signed fields; evidence and
/// state-machine constructors apply those checks with the necessary context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vote {
    signer: ValidatorId,
    block: BlockId,
    view: ViewNumber,
}

impl Vote {
    /// Creates a vote message.
    #[must_use]
    pub const fn new(signer: ValidatorId, block: BlockId, view: ViewNumber) -> Self {
        Self {
            signer,
            block,
            view,
        }
    }

    /// Returns the validator identity that signed the vote.
    #[must_use]
    pub const fn signer(self) -> ValidatorId {
        self.signer
    }

    /// Returns the block this vote targets.
    #[must_use]
    pub const fn block(self) -> BlockId {
        self.block
    }

    /// Returns the view in which this vote was cast.
    #[must_use]
    pub const fn view(self) -> ViewNumber {
        self.view
    }
}

/// Modeled nullify message from one validator for one view.
///
/// A `Nullify` is the raw input later nullification evidence constructors
/// aggregate. It records signer intent for a view. Protocol validity is
/// contextual: it depends on the active committee, distinct signer counting,
/// quorum threshold, and state-machine rules. This type stores the signed
/// fields; evidence and state-machine constructors apply those checks with the
/// necessary context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Nullify {
    signer: ValidatorId,
    view: ViewNumber,
}

impl Nullify {
    /// Creates a nullify message.
    #[must_use]
    pub const fn new(signer: ValidatorId, view: ViewNumber) -> Self {
        Self { signer, view }
    }

    /// Returns the validator identity that signed the nullify message.
    #[must_use]
    pub const fn signer(self) -> ValidatorId {
        self.signer
    }

    /// Returns the view this nullify message targets.
    #[must_use]
    pub const fn view(self) -> ViewNumber {
        self.view
    }
}

/// Evidence that a block has the M-notarization vote threshold in one view.
///
/// `MNotarization` values are constructed from votes plus the active
/// committee. Construction checks that every signer is a committee member, each
/// signer appears once, all votes target the same block and view, and the
/// number of distinct valid signers meets the `2f + 1` M threshold. Proposal
/// validity and state-machine transition rules are checked outside this
/// evidence type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MNotarization {
    block: BlockId,
    view: ViewNumber,
    signers: BTreeSet<ValidatorId>,
}

impl MNotarization {
    /// Creates an M-notarization from votes and the active committee.
    ///
    /// Returns [`EvidenceError`] when the vote set is empty, includes a
    /// non-member or duplicate signer, mixes block/view targets, or has fewer
    /// than `2f + 1` distinct valid signers.
    pub fn from_votes<I>(committee: &Committee, votes: I) -> Result<Self, EvidenceError>
    where
        I: IntoIterator<Item = Vote>,
    {
        let ((block, view), signers) =
            collect_evidence(committee, votes, committee.config().m_threshold())?;

        Ok(Self {
            block,
            view,
            signers,
        })
    }

    /// Returns the notarized block.
    #[must_use]
    pub fn block(&self) -> BlockId {
        self.block
    }

    /// Returns the notarized view.
    #[must_use]
    pub fn view(&self) -> ViewNumber {
        self.view
    }

    /// Iterates signer identities in deterministic order.
    pub fn signers(&self) -> impl Iterator<Item = ValidatorId> + '_ {
        self.signers.iter().copied()
    }
}

/// Evidence that a block has the L-notarization vote threshold in one view.
///
/// `LNotarization` has the same fields as [`MNotarization`] but is constructed
/// with the `n - f` L threshold from the active committee configuration.
/// Proposal validity, finalization, and state-machine transition rules are
/// checked outside this evidence type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LNotarization {
    block: BlockId,
    view: ViewNumber,
    signers: BTreeSet<ValidatorId>,
}

impl LNotarization {
    /// Creates an L-notarization from votes and the active committee.
    ///
    /// Returns [`EvidenceError`] when the vote set is empty, includes a
    /// non-member or duplicate signer, mixes block/view targets, or has fewer
    /// than `n - f` distinct valid signers.
    pub fn from_votes<I>(committee: &Committee, votes: I) -> Result<Self, EvidenceError>
    where
        I: IntoIterator<Item = Vote>,
    {
        let ((block, view), signers) =
            collect_evidence(committee, votes, committee.config().l_threshold())?;

        Ok(Self {
            block,
            view,
            signers,
        })
    }

    /// Returns the notarized block.
    #[must_use]
    pub fn block(&self) -> BlockId {
        self.block
    }

    /// Returns the notarized view.
    #[must_use]
    pub fn view(&self) -> ViewNumber {
        self.view
    }

    /// Iterates signer identities in deterministic order.
    pub fn signers(&self) -> impl Iterator<Item = ValidatorId> + '_ {
        self.signers.iter().copied()
    }
}

/// Evidence that a view has the nullification threshold.
///
/// `Nullification` is constructed from nullify messages plus the active
/// committee. Construction checks that every signer is a committee member, each
/// signer appears once, all messages target the same view, and the number of
/// distinct valid signers meets the `2f + 1` nullification threshold. Timeout,
/// condition-b, and state-machine transition rules are checked outside this
/// evidence type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nullification {
    view: ViewNumber,
    signers: BTreeSet<ValidatorId>,
}

impl Nullification {
    /// Creates a nullification from nullify messages and the active committee.
    ///
    /// Returns [`EvidenceError`] when the message set is empty, includes a
    /// non-member or duplicate signer, mixes target views, or has fewer than
    /// `2f + 1` distinct valid signers.
    pub fn from_nullifies<I>(committee: &Committee, nullifies: I) -> Result<Self, EvidenceError>
    where
        I: IntoIterator<Item = Nullify>,
    {
        let (view, signers) = collect_evidence(
            committee,
            nullifies,
            committee.config().nullification_threshold(),
        )?;

        Ok(Self { view, signers })
    }

    /// Returns the nullified view.
    #[must_use]
    pub fn view(&self) -> ViewNumber {
        self.view
    }

    /// Iterates signer identities in deterministic order.
    pub fn signers(&self) -> impl Iterator<Item = ValidatorId> + '_ {
        self.signers.iter().copied()
    }
}

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

/// Evidence construction errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceError {
    /// No messages were supplied.
    Empty,
    /// The same signer appeared more than once.
    DuplicateSigner {
        /// Duplicated signer identity.
        signer: ValidatorId,
    },
    /// A signer is not in the active committee.
    UnknownSigner {
        /// Non-member signer identity.
        signer: ValidatorId,
    },
    /// Votes did not all target the same block and view.
    ConflictingVoteTarget {
        /// Expected block from the first vote.
        expected_block: BlockId,
        /// Expected view from the first vote.
        expected_view: ViewNumber,
        /// Conflicting vote block.
        actual_block: BlockId,
        /// Conflicting vote view.
        actual_view: ViewNumber,
    },
    /// Nullify messages did not all target the same view.
    ConflictingNullificationView {
        /// Expected view from the first nullify message.
        expected_view: ViewNumber,
        /// Conflicting nullify view.
        actual_view: ViewNumber,
    },
    /// The evidence had fewer distinct valid signers than required.
    BelowThreshold {
        /// Number of distinct valid signers.
        signer_count: usize,
        /// Required threshold.
        threshold: usize,
    },
}

impl fmt::Display for EvidenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(formatter, "evidence is empty"),
            Self::DuplicateSigner { signer } => {
                write!(formatter, "{signer} appears more than once in evidence")
            }
            Self::UnknownSigner { signer } => {
                write!(formatter, "{signer} is not a committee member")
            }
            Self::ConflictingVoteTarget {
                expected_block,
                expected_view,
                actual_block,
                actual_view,
            } => write!(
                formatter,
                "vote targets {actual_block} in {actual_view}, expected {expected_block} in {expected_view}"
            ),
            Self::ConflictingNullificationView {
                expected_view,
                actual_view,
            } => write!(
                formatter,
                "nullify message targets {actual_view}, expected {expected_view}"
            ),
            Self::BelowThreshold {
                signer_count,
                threshold,
            } => write!(
                formatter,
                "evidence has {signer_count} distinct valid signers, below threshold {threshold}"
            ),
        }
    }
}

impl std::error::Error for EvidenceError {}

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

/// Message type that can contribute to threshold evidence.
trait EvidenceMessage {
    /// Target all messages in one evidence value must agree on.
    type Target: Copy + Eq;

    /// Returns the signer identity carried by the message.
    fn signer(&self) -> ValidatorId;

    /// Returns the message target.
    fn target(&self) -> Self::Target;

    /// Builds the target-conflict error for this message kind.
    fn conflicting_target_error(expected: Self::Target, actual: Self::Target) -> EvidenceError;
}

impl EvidenceMessage for Vote {
    type Target = (BlockId, ViewNumber);

    fn signer(&self) -> ValidatorId {
        self.signer
    }

    fn target(&self) -> Self::Target {
        (self.block, self.view)
    }

    fn conflicting_target_error(expected: Self::Target, actual: Self::Target) -> EvidenceError {
        EvidenceError::ConflictingVoteTarget {
            expected_block: expected.0,
            expected_view: expected.1,
            actual_block: actual.0,
            actual_view: actual.1,
        }
    }
}

impl EvidenceMessage for Nullify {
    type Target = ViewNumber;

    fn signer(&self) -> ValidatorId {
        self.signer
    }

    fn target(&self) -> Self::Target {
        self.view
    }

    fn conflicting_target_error(expected: Self::Target, actual: Self::Target) -> EvidenceError {
        EvidenceError::ConflictingNullificationView {
            expected_view: expected,
            actual_view: actual,
        }
    }
}

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

/// Collects one-target threshold evidence from modeled messages.
fn collect_evidence<I, M>(
    committee: &Committee,
    messages: I,
    threshold: usize,
) -> Result<(M::Target, BTreeSet<ValidatorId>), EvidenceError>
where
    I: IntoIterator<Item = M>,
    M: EvidenceMessage,
{
    let mut messages = messages.into_iter();
    let first = messages.next().ok_or(EvidenceError::Empty)?;
    let target = first.target();
    let mut signers = BTreeSet::new();

    for message in std::iter::once(first).chain(messages) {
        let actual_target = message.target();
        if actual_target != target {
            return Err(M::conflicting_target_error(target, actual_target));
        }

        let signer = message.signer();
        if !committee.contains(signer) {
            return Err(EvidenceError::UnknownSigner { signer });
        }

        if !signers.insert(signer) {
            return Err(EvidenceError::DuplicateSigner { signer });
        }
    }

    if signers.len() < threshold {
        return Err(EvidenceError::BelowThreshold {
            signer_count: signers.len(),
            threshold,
        });
    }

    Ok((target, signers))
}

fn minimum_validator_count(fault_bound: usize) -> Result<usize, ConfigError> {
    threshold(MIN_VALIDATOR_FAULT_FACTOR, fault_bound)
}

fn threshold(multiplier: usize, fault_bound: usize) -> Result<usize, ConfigError> {
    fault_bound
        .checked_mul(multiplier)
        .and_then(|value| value.checked_add(THRESHOLD_BASE))
        .ok_or(ConfigError::ThresholdOverflow { fault_bound })
}
