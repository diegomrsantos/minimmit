mod common;

use common::{block, transaction, validator, view, ONE_FAULT};
use minimmit_core::{Block, Committee, SignedBlock};

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
fn signed_block_records_signer_and_block() {
    let proposed_block =
        Block::new(block(50), view(5), block(20), [transaction(1)]).expect("block is valid");

    let signed_block = SignedBlock::new(validator(5), proposed_block.clone());

    assert_eq!(signed_block.signer(), validator(5));
    assert_eq!(signed_block.block(), &proposed_block);
}
