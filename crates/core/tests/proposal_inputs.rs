mod common;

use common::{validator, view, ONE_FAULT};
use minimmit_core::Committee;

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
