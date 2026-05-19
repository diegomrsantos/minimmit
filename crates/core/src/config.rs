use std::fmt;

const MIN_VALIDATOR_FAULT_FACTOR: usize = 5;
const M_AND_NULLIFICATION_FAULT_FACTOR: usize = 2;
const THRESHOLD_BASE: usize = 1;

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

fn minimum_validator_count(fault_bound: usize) -> Result<usize, ConfigError> {
    threshold(MIN_VALIDATOR_FAULT_FACTOR, fault_bound)
}

fn threshold(multiplier: usize, fault_bound: usize) -> Result<usize, ConfigError> {
    fault_bound
        .checked_mul(multiplier)
        .and_then(|value| value.checked_add(THRESHOLD_BASE))
        .ok_or(ConfigError::ThresholdOverflow { fault_bound })
}

#[cfg(test)]
mod tests {
    use super::{Config, ConfigError};

    const NO_FAULTS: usize = 0;
    const ONE_FAULT: usize = 1;
    const TWO_FAULTS: usize = 2;

    const NO_VALIDATORS: usize = 0;
    const MIN_VALIDATORS_WITH_NO_FAULTS: usize = 1;
    const MIN_VALIDATORS_WITH_ONE_FAULT: usize = 6;
    const BELOW_MIN_VALIDATORS_WITH_ONE_FAULT: usize = 5;
    const LARGER_VALIDATORS_WITH_ONE_FAULT: usize = 7;
    const MIN_VALIDATORS_WITH_TWO_FAULTS: usize = 11;
    const LARGER_VALIDATORS_WITH_TWO_FAULTS: usize = 12;

    #[test]
    fn accepts_minimum_validator_count() {
        let config = Config::new(MIN_VALIDATORS_WITH_ONE_FAULT, ONE_FAULT)
            .expect("minimum n = 5f + 1 is valid");

        assert_eq!(config.validator_count(), MIN_VALIDATORS_WITH_ONE_FAULT);
        assert_eq!(config.fault_bound(), ONE_FAULT);
        assert_eq!(config.m_threshold(), 3);
        assert_eq!(config.nullification_threshold(), 3);
        assert_eq!(config.l_threshold(), 5);
    }

    #[test]
    fn accepts_larger_validator_count() {
        let config = Config::new(LARGER_VALIDATORS_WITH_ONE_FAULT, ONE_FAULT)
            .expect("n greater than 5f + 1 is valid");

        assert_eq!(config.validator_count(), LARGER_VALIDATORS_WITH_ONE_FAULT);
        assert_eq!(config.fault_bound(), ONE_FAULT);
        assert_eq!(config.m_threshold(), 3);
        assert_eq!(config.nullification_threshold(), 3);
        assert_eq!(config.l_threshold(), 6);
    }

    #[test]
    fn rejects_validator_count_below_minimum() {
        let error = Config::new(BELOW_MIN_VALIDATORS_WITH_ONE_FAULT, ONE_FAULT)
            .expect_err("n < 5f + 1 is invalid");

        assert_eq!(
            error,
            ConfigError::TooFewValidators {
                validator_count: BELOW_MIN_VALIDATORS_WITH_ONE_FAULT,
                fault_bound: ONE_FAULT,
                minimum_validator_count: MIN_VALIDATORS_WITH_ONE_FAULT,
            }
        );
    }

    #[test]
    fn fault_bound_zero_requires_at_least_one_validator() {
        let config = Config::new(MIN_VALIDATORS_WITH_NO_FAULTS, NO_FAULTS)
            .expect("n = 1 is valid when f = 0");

        assert_eq!(config.m_threshold(), 1);
        assert_eq!(config.nullification_threshold(), 1);
        assert_eq!(config.l_threshold(), 1);
        assert_eq!(
            Config::new(NO_VALIDATORS, NO_FAULTS),
            Err(ConfigError::TooFewValidators {
                validator_count: NO_VALIDATORS,
                fault_bound: NO_FAULTS,
                minimum_validator_count: MIN_VALIDATORS_WITH_NO_FAULTS,
            })
        );
    }

    #[test]
    fn l_threshold_equals_four_f_plus_one_only_at_minimum_validator_count() {
        let minimum = Config::new(MIN_VALIDATORS_WITH_TWO_FAULTS, TWO_FAULTS)
            .expect("minimum n = 5f + 1 is valid");
        let larger =
            Config::new(LARGER_VALIDATORS_WITH_TWO_FAULTS, TWO_FAULTS).expect("larger n is valid");

        assert_eq!(minimum.l_threshold(), 9);
        assert_eq!(larger.l_threshold(), 10);
    }

    #[test]
    fn rejects_fault_bounds_that_overflow_threshold_calculation() {
        assert_eq!(
            Config::new(usize::MAX, usize::MAX),
            Err(ConfigError::ThresholdOverflow {
                fault_bound: usize::MAX,
            })
        );
    }
}
