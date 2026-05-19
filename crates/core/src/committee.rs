use std::{collections::BTreeSet, fmt};

use crate::{Config, ConfigError, ValidatorId, ViewNumber};

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

#[cfg(test)]
mod tests {
    use super::{Committee, CommitteeError};
    use crate::{ConfigError, ValidatorId, ViewNumber};

    const ONE_FAULT: usize = 1;
    const BELOW_MIN_VALIDATORS_WITH_ONE_FAULT: usize = 5;
    const MIN_VALIDATORS_WITH_ONE_FAULT: usize = 6;

    fn committee() -> Committee {
        Committee::new(validators(MIN_VALIDATORS_WITH_ONE_FAULT), ONE_FAULT)
            .expect("committee satisfies n >= 5f + 1")
    }

    fn validator(id: u64) -> ValidatorId {
        ValidatorId::new(id)
    }

    fn validators(count: usize) -> Vec<ValidatorId> {
        (0..count as u64).map(validator).collect()
    }

    fn view(number: u64) -> ViewNumber {
        ViewNumber::new(number)
    }

    #[test]
    fn rejects_duplicate_validators() {
        assert_eq!(
            Committee::new(
                [validator(0), validator(1), validator(1), validator(2)],
                ONE_FAULT,
            ),
            Err(CommitteeError::DuplicateValidator {
                validator: validator(1),
            })
        );
    }

    #[test]
    fn uses_config_validation() {
        assert_eq!(
            Committee::new(validators(BELOW_MIN_VALIDATORS_WITH_ONE_FAULT), ONE_FAULT),
            Err(CommitteeError::InvalidConfig(
                ConfigError::TooFewValidators {
                    validator_count: BELOW_MIN_VALIDATORS_WITH_ONE_FAULT,
                    fault_bound: ONE_FAULT,
                    minimum_validator_count: MIN_VALIDATORS_WITH_ONE_FAULT,
                }
            ))
        );
    }

    #[test]
    fn iterates_validators_in_identity_order() {
        let committee = Committee::new(
            [
                validator(2),
                validator(0),
                validator(4),
                validator(1),
                validator(5),
                validator(3),
            ],
            ONE_FAULT,
        )
        .expect("committee satisfies n >= 5f + 1");

        assert_eq!(
            committee.validators().collect::<Vec<_>>(),
            validators(MIN_VALIDATORS_WITH_ONE_FAULT)
        );
    }

    #[test]
    fn leader_uses_view_modulo_validator_identity_order() {
        let committee = Committee::new(
            [
                validator(2),
                validator(0),
                validator(4),
                validator(1),
                validator(5),
                validator(3),
            ],
            ONE_FAULT,
        )
        .expect("committee satisfies n >= 5f + 1");

        assert_eq!(committee.leader(view(0)), validator(0));
        assert_eq!(committee.leader(view(1)), validator(1));
        assert_eq!(committee.leader(view(5)), validator(5));
        assert_eq!(committee.leader(view(6)), validator(0));
    }

    #[test]
    fn duplicate_senders_count_once() {
        let committee = committee();

        let count = committee.count_distinct_valid_senders([
            validator(0),
            validator(0),
            validator(1),
            validator(2),
        ]);

        assert_eq!(count, committee.config().m_threshold());
    }

    #[test]
    fn unknown_senders_do_not_count() {
        let committee = committee();

        let count =
            committee.count_distinct_valid_senders([validator(0), validator(1), validator(99)]);

        assert_eq!(count, 2);
    }

    #[test]
    fn exact_threshold_requires_distinct_valid_senders() {
        let committee = committee();

        let below_threshold = committee.count_distinct_valid_senders([validator(0), validator(1)]);
        let at_threshold =
            committee.count_distinct_valid_senders([validator(0), validator(1), validator(2)]);

        assert_eq!(below_threshold, committee.config().m_threshold() - 1);
        assert_eq!(at_threshold, committee.config().m_threshold());
    }

    #[test]
    fn mixed_valid_invalid_and_duplicate_senders_count_only_distinct_members() {
        let committee = committee();

        let count = committee.count_distinct_valid_senders([
            validator(0),
            validator(0),
            validator(1),
            validator(6),
            validator(99),
            validator(2),
            validator(2),
            validator(3),
        ]);

        assert_eq!(count, 4);
    }
}
