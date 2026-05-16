use minimmit_core::{Committee, CommitteeError, ConfigError, ValidatorId};

const ONE_FAULT: usize = 1;
const MIN_VALIDATORS_WITH_ONE_FAULT: u64 = 6;
const BELOW_MIN_VALIDATORS_WITH_ONE_FAULT: u64 = 5;

fn validator(id: u64) -> ValidatorId {
    ValidatorId::new(id)
}

fn validators(count: u64) -> Vec<ValidatorId> {
    (0..count).map(validator).collect()
}

#[test]
fn committee_rejects_duplicate_validators() {
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
fn committee_uses_config_validation() {
    assert_eq!(
        Committee::new(validators(BELOW_MIN_VALIDATORS_WITH_ONE_FAULT), ONE_FAULT),
        Err(CommitteeError::InvalidConfig(
            ConfigError::TooFewValidators {
                validator_count: BELOW_MIN_VALIDATORS_WITH_ONE_FAULT as usize,
                fault_bound: ONE_FAULT,
                minimum_validator_count: MIN_VALIDATORS_WITH_ONE_FAULT as usize,
            }
        ))
    );
}

#[test]
fn committee_iterates_validators_in_identity_order() {
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
fn duplicate_senders_count_once() {
    let committee = Committee::new(validators(MIN_VALIDATORS_WITH_ONE_FAULT), ONE_FAULT)
        .expect("committee satisfies n >= 5f + 1");

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
    let committee = Committee::new(validators(MIN_VALIDATORS_WITH_ONE_FAULT), ONE_FAULT)
        .expect("committee satisfies n >= 5f + 1");

    let count = committee.count_distinct_valid_senders([validator(0), validator(1), validator(99)]);

    assert_eq!(count, 2);
}

#[test]
fn exact_threshold_requires_distinct_valid_senders() {
    let committee = Committee::new(validators(MIN_VALIDATORS_WITH_ONE_FAULT), ONE_FAULT)
        .expect("committee satisfies n >= 5f + 1");

    let below_threshold = committee.count_distinct_valid_senders([validator(0), validator(1)]);
    let at_threshold =
        committee.count_distinct_valid_senders([validator(0), validator(1), validator(2)]);

    assert_eq!(below_threshold, committee.config().m_threshold() - 1);
    assert_eq!(at_threshold, committee.config().m_threshold());
}

#[test]
fn mixed_valid_invalid_and_duplicate_senders_count_only_distinct_members() {
    let committee = Committee::new(validators(MIN_VALIDATORS_WITH_ONE_FAULT), ONE_FAULT)
        .expect("committee satisfies n >= 5f + 1");

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
