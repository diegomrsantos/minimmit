//! Core protocol state machine crate for Minimmit.
//!
//! Protocol behavior in this crate should stay deterministic and reviewable
//! from the core state machine.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

const MIN_VALIDATOR_FAULT_FACTOR: usize = 5;
const M_AND_NULLIFICATION_FAULT_FACTOR: usize = 2;
const THRESHOLD_BASE: usize = 1;

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

/// View number used by the protocol core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ViewNumber(u64);

impl ViewNumber {
    /// Creates a view number.
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

/// Block identity used by the protocol core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockId(u64);

impl BlockId {
    /// Creates a block identity.
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TransactionId(u64);

impl TransactionId {
    /// Creates a transaction identity.
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

/// Baseline Minimmit block data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    id: BlockId,
    view: ViewNumber,
    parent: Option<BlockId>,
    transactions: Vec<TransactionId>,
}

impl Block {
    /// Creates the genesis block.
    #[must_use]
    pub fn genesis(id: BlockId) -> Self {
        Self {
            id,
            view: ViewNumber::new(0),
            parent: None,
            transactions: Vec::new(),
        }
    }

    /// Creates a non-genesis block with a parent and modeled transactions.
    pub fn new<I>(
        id: BlockId,
        view: ViewNumber,
        parent: BlockId,
        transactions: I,
    ) -> Result<Self, BlockError>
    where
        I: IntoIterator<Item = TransactionId>,
    {
        let transactions = distinct_transactions(transactions)?;

        Ok(Self {
            id,
            view,
            parent: Some(parent),
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

    /// Returns the parent block identity, if this is not genesis.
    #[must_use]
    pub fn parent(&self) -> Option<BlockId> {
        self.parent
    }

    /// Returns the modeled transactions in block order.
    #[must_use]
    pub fn transactions(&self) -> &[TransactionId] {
        &self.transactions
    }
}

/// Block construction errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockError {
    /// The block listed a transaction more than once.
    DuplicateTransaction {
        /// Duplicated transaction identity.
        transaction: TransactionId,
    },
}

impl fmt::Display for BlockError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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

/// Modeled vote message for a block in a view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vote {
    signer: ValidatorId,
    block: BlockId,
    view: ViewNumber,
}

impl Vote {
    /// Creates a modeled vote signed by a validator identity.
    #[must_use]
    pub const fn new(signer: ValidatorId, block: BlockId, view: ViewNumber) -> Self {
        Self {
            signer,
            block,
            view,
        }
    }

    /// Returns the signer identity.
    #[must_use]
    pub const fn signer(self) -> ValidatorId {
        self.signer
    }

    /// Returns the voted block.
    #[must_use]
    pub const fn block(self) -> BlockId {
        self.block
    }

    /// Returns the voted view.
    #[must_use]
    pub const fn view(self) -> ViewNumber {
        self.view
    }
}

/// Modeled nullify message for a view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Nullify {
    signer: ValidatorId,
    view: ViewNumber,
}

impl Nullify {
    /// Creates a modeled nullify message signed by a validator identity.
    #[must_use]
    pub const fn new(signer: ValidatorId, view: ViewNumber) -> Self {
        Self { signer, view }
    }

    /// Returns the signer identity.
    #[must_use]
    pub const fn signer(self) -> ValidatorId {
        self.signer
    }

    /// Returns the nullified view.
    #[must_use]
    pub const fn view(self) -> ViewNumber {
        self.view
    }
}

/// M-notarization backed by threshold vote evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MNotarization {
    block: BlockId,
    view: ViewNumber,
    signers: BTreeSet<ValidatorId>,
}

impl MNotarization {
    /// Creates an M-notarization from votes with `2f + 1` distinct valid
    /// signers for one block in one view.
    pub fn from_votes<I>(committee: &Committee, votes: I) -> Result<Self, EvidenceError>
    where
        I: IntoIterator<Item = Vote>,
    {
        let evidence = validate_votes(committee, votes, committee.config().m_threshold())?;

        Ok(Self {
            block: evidence.block,
            view: evidence.view,
            signers: evidence.signers,
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

/// L-notarization backed by threshold vote evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LNotarization {
    block: BlockId,
    view: ViewNumber,
    signers: BTreeSet<ValidatorId>,
}

impl LNotarization {
    /// Creates an L-notarization from votes with `n - f` distinct valid
    /// signers for one block in one view.
    pub fn from_votes<I>(committee: &Committee, votes: I) -> Result<Self, EvidenceError>
    where
        I: IntoIterator<Item = Vote>,
    {
        let evidence = validate_votes(committee, votes, committee.config().l_threshold())?;

        Ok(Self {
            block: evidence.block,
            view: evidence.view,
            signers: evidence.signers,
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

/// Nullification backed by threshold nullify evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nullification {
    view: ViewNumber,
    signers: BTreeSet<ValidatorId>,
}

impl Nullification {
    /// Creates a nullification from nullify messages with `2f + 1` distinct
    /// valid signers for one view.
    pub fn from_nullifies<I>(committee: &Committee, nullifies: I) -> Result<Self, EvidenceError>
    where
        I: IntoIterator<Item = Nullify>,
    {
        let evidence = validate_nullifies(
            committee,
            nullifies,
            committee.config().nullification_threshold(),
        )?;

        Ok(Self {
            view: evidence.view,
            signers: evidence.signers,
        })
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

/// Evidence validation errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceError {
    /// No messages were supplied.
    Empty,
    /// The same signer appeared more than once.
    DuplicateSigner {
        /// Duplicated signer identity.
        signer: ValidatorId,
    },
    /// A signer is not in the committee.
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

/// Baseline proposal data carried by the protocol core.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proposal {
    proposer: ValidatorId,
    block: Block,
    parent_notarization: MNotarization,
    nullifications: BTreeMap<ViewNumber, Nullification>,
}

impl Proposal {
    /// Creates a proposal from a block, parent M-notarization, and
    /// skipped-view nullifications.
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
            if by_view.insert(view, nullification).is_some() {
                return Err(ProposalError::DuplicateNullification { view });
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

    /// Returns the parent M-notarization carried by this proposal.
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

struct VoteEvidence {
    block: BlockId,
    view: ViewNumber,
    signers: BTreeSet<ValidatorId>,
}

struct NullificationEvidence {
    view: ViewNumber,
    signers: BTreeSet<ValidatorId>,
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

fn validate_votes<I>(
    committee: &Committee,
    votes: I,
    threshold: usize,
) -> Result<VoteEvidence, EvidenceError>
where
    I: IntoIterator<Item = Vote>,
{
    let mut votes = votes.into_iter();
    let first = votes.next().ok_or(EvidenceError::Empty)?;
    let mut signers = BTreeSet::new();

    insert_signer(committee, &mut signers, first.signer())?;

    for vote in votes {
        if vote.block() != first.block() || vote.view() != first.view() {
            return Err(EvidenceError::ConflictingVoteTarget {
                expected_block: first.block(),
                expected_view: first.view(),
                actual_block: vote.block(),
                actual_view: vote.view(),
            });
        }

        insert_signer(committee, &mut signers, vote.signer())?;
    }

    require_threshold(signers.len(), threshold)?;

    Ok(VoteEvidence {
        block: first.block(),
        view: first.view(),
        signers,
    })
}

fn validate_nullifies<I>(
    committee: &Committee,
    nullifies: I,
    threshold: usize,
) -> Result<NullificationEvidence, EvidenceError>
where
    I: IntoIterator<Item = Nullify>,
{
    let mut nullifies = nullifies.into_iter();
    let first = nullifies.next().ok_or(EvidenceError::Empty)?;
    let mut signers = BTreeSet::new();

    insert_signer(committee, &mut signers, first.signer())?;

    for nullify in nullifies {
        if nullify.view() != first.view() {
            return Err(EvidenceError::ConflictingNullificationView {
                expected_view: first.view(),
                actual_view: nullify.view(),
            });
        }

        insert_signer(committee, &mut signers, nullify.signer())?;
    }

    require_threshold(signers.len(), threshold)?;

    Ok(NullificationEvidence {
        view: first.view(),
        signers,
    })
}

fn insert_signer(
    committee: &Committee,
    signers: &mut BTreeSet<ValidatorId>,
    signer: ValidatorId,
) -> Result<(), EvidenceError> {
    if !committee.contains(signer) {
        return Err(EvidenceError::UnknownSigner { signer });
    }

    if !signers.insert(signer) {
        return Err(EvidenceError::DuplicateSigner { signer });
    }

    Ok(())
}

fn require_threshold(signer_count: usize, threshold: usize) -> Result<(), EvidenceError> {
    if signer_count < threshold {
        return Err(EvidenceError::BelowThreshold {
            signer_count,
            threshold,
        });
    }

    Ok(())
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
