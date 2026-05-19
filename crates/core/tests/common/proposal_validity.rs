use minimmit_core::{
    Block, BlockId, Committee, MNotarization, Nullification, Nullify, SignedBlock, TransactionId,
    ValidatorId, ViewNumber, Vote,
};

const ONE_FAULT: usize = 1;
const MIN_VALIDATORS_WITH_ONE_FAULT: u64 = 6;

pub(crate) fn block(id: u64) -> BlockId {
    BlockId::new(id)
}

pub(crate) fn committee() -> Committee {
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

pub(crate) fn nullification(view_number: u64) -> Nullification {
    Nullification::from_nullifies(
        &committee(),
        [
            nullify(0, view_number),
            nullify(1, view_number),
            nullify(2, view_number),
        ],
    )
    .expect("nullify messages form a nullification")
}

fn nullify(signer: u64, view_number: u64) -> Nullify {
    Nullify::new(validator(signer), view(view_number))
}

pub(crate) fn proposed_block(id: u64, view_number: u64, parent: u64) -> Block {
    Block::new(
        block(id),
        view(view_number),
        block(parent),
        [transaction(id)],
    )
    .expect("block is valid")
}

pub(crate) fn signed_by_leader(committee: &Committee, block: Block) -> SignedBlock {
    SignedBlock::new(committee.leader(block.view()), block)
}

fn transaction(id: u64) -> TransactionId {
    TransactionId::new(id)
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

fn vote(signer: u64, block_id: u64, view_number: u64) -> Vote {
    Vote::new(validator(signer), block(block_id), view(view_number))
}
