use minimmit_core::{BlockId, Committee, MNotarization, ValidatorId, ViewNumber, Vote};

const ONE_FAULT: usize = 1;
const MIN_VALIDATORS_WITH_ONE_FAULT: u64 = 6;

pub(crate) fn block(id: u64) -> BlockId {
    BlockId::new(id)
}

fn committee() -> Committee {
    Committee::new(validators(MIN_VALIDATORS_WITH_ONE_FAULT), ONE_FAULT)
        .expect("committee satisfies n >= 5f + 1")
}

pub(crate) fn m_notarization(block_id: u64, view_number: u64) -> MNotarization {
    MNotarization::from_votes(
        &committee(),
        [
            vote(0, block_id, view_number),
            vote(1, block_id, view_number),
            vote(2, block_id, view_number),
        ],
    )
    .expect("votes form an M-notarization")
}

fn validator(id: u64) -> ValidatorId {
    ValidatorId::new(id)
}

fn validators(count: u64) -> Vec<ValidatorId> {
    (0..count).map(validator).collect()
}

pub(crate) fn view(number: u64) -> ViewNumber {
    ViewNumber::new(number)
}

fn vote(signer: u64, block_id: u64, view_number: u64) -> Vote {
    Vote::new(validator(signer), block(block_id), view(view_number))
}
