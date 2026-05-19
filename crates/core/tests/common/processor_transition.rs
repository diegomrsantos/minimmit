use minimmit_core::{Committee, ValidatorId, ViewNumber};

const ONE_FAULT: usize = 1;
const MIN_VALIDATORS_WITH_ONE_FAULT: u64 = 6;

pub(crate) fn committee() -> Committee {
    Committee::new(validators(MIN_VALIDATORS_WITH_ONE_FAULT), ONE_FAULT)
        .expect("committee satisfies n >= 5f + 1")
}

pub(crate) fn validator(id: u64) -> ValidatorId {
    ValidatorId::new(id)
}

fn validators(count: u64) -> Vec<ValidatorId> {
    (0..count).map(validator).collect()
}

pub(crate) fn view(number: u64) -> ViewNumber {
    ViewNumber::new(number)
}
