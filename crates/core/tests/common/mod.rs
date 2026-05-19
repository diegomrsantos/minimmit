use minimmit_core::{BlockId, Committee, MNotarization, ValidatorId, ViewNumber, Vote};

const ONE_FAULT: usize = 1;
const MIN_VALIDATORS_WITH_ONE_FAULT: u64 = 6;

pub(crate) fn committee() -> Committee {
    Committee::new(
        (0..MIN_VALIDATORS_WITH_ONE_FAULT)
            .map(ValidatorId::new)
            .collect::<Vec<_>>(),
        ONE_FAULT,
    )
    .expect("committee satisfies n >= 5f + 1")
}

// Shared integration fixtures compile once per integration test crate; the
// processor transition tests use only the committee fixture.
#[allow(dead_code)]
pub(crate) fn m_notarization(block: BlockId, view: ViewNumber) -> MNotarization {
    MNotarization::from_votes(
        &committee(),
        [
            Vote::new(ValidatorId::new(0), block, view),
            Vote::new(ValidatorId::new(1), block, view),
            Vote::new(ValidatorId::new(2), block, view),
        ],
    )
    .expect("votes form an M-notarization")
}
