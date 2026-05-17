//! Core protocol state machine crate for Minimmit.
//!
//! Protocol behavior in this crate should stay deterministic and reviewable
//! from the core state machine.

use std::{collections::BTreeSet, fmt};

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

fn minimum_validator_count(fault_bound: usize) -> Result<usize, ConfigError> {
    threshold(MIN_VALIDATOR_FAULT_FACTOR, fault_bound)
}

fn threshold(multiplier: usize, fault_bound: usize) -> Result<usize, ConfigError> {
    fault_bound
        .checked_mul(multiplier)
        .and_then(|value| value.checked_add(THRESHOLD_BASE))
        .ok_or(ConfigError::ThresholdOverflow { fault_bound })
}
