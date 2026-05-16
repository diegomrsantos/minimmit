use minimmit_core::{Config, ConfigError};

const NO_FAULTS: usize = 0;
const ONE_FAULT: usize = 1;
const TWO_FAULTS: usize = 2;

const NO_VALIDATORS: usize = 0;
const MIN_VALIDATORS_WITH_NO_FAULTS: usize = 1;
const MIN_VALIDATORS_WITH_ONE_FAULT: usize = 6;
const BELOW_MIN_VALIDATORS_WITH_ONE_FAULT: usize = 5;
const EXTRA_VALIDATOR: usize = 1;
const MIN_VALIDATORS_WITH_TWO_FAULTS: usize = 11;

const MINIMUM_CASE_L_FAULT_FACTOR: usize = 4;
const THRESHOLD_BASE: usize = 1;

#[test]
fn accepts_minimum_validator_count() {
    let config =
        Config::new(MIN_VALIDATORS_WITH_ONE_FAULT, ONE_FAULT).expect("minimum n = 5f + 1 is valid");

    assert_eq!(config.validator_count(), MIN_VALIDATORS_WITH_ONE_FAULT);
    assert_eq!(config.fault_bound(), ONE_FAULT);
    assert_eq!(config.m_threshold(), 3);
    assert_eq!(config.nullification_threshold(), 3);
    assert_eq!(config.l_threshold(), 5);
}

#[test]
fn accepts_larger_validator_count() {
    let larger_validator_count = MIN_VALIDATORS_WITH_ONE_FAULT + EXTRA_VALIDATOR;
    let config =
        Config::new(larger_validator_count, ONE_FAULT).expect("n greater than 5f + 1 is valid");

    assert_eq!(config.m_threshold(), 2 * config.fault_bound() + 1);
    assert_eq!(
        config.nullification_threshold(),
        2 * config.fault_bound() + 1
    );
    assert_eq!(
        config.l_threshold(),
        config.validator_count() - config.fault_bound()
    );
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
    let config =
        Config::new(MIN_VALIDATORS_WITH_NO_FAULTS, NO_FAULTS).expect("n = 1 is valid when f = 0");

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
    let larger_validator_count = MIN_VALIDATORS_WITH_TWO_FAULTS + EXTRA_VALIDATOR;
    let minimum = Config::new(MIN_VALIDATORS_WITH_TWO_FAULTS, TWO_FAULTS)
        .expect("minimum n = 5f + 1 is valid");
    let larger = Config::new(larger_validator_count, TWO_FAULTS).expect("larger n is valid");
    let minimum_case_l_threshold =
        MINIMUM_CASE_L_FAULT_FACTOR * minimum.fault_bound() + THRESHOLD_BASE;
    let larger_case_l_threshold =
        MINIMUM_CASE_L_FAULT_FACTOR * larger.fault_bound() + THRESHOLD_BASE;

    assert_eq!(minimum.l_threshold(), minimum_case_l_threshold);
    assert_ne!(larger.l_threshold(), larger_case_l_threshold);
    assert_eq!(
        larger.l_threshold(),
        larger.validator_count() - larger.fault_bound()
    );
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
