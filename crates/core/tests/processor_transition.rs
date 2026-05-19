#[path = "common/processor_transition.rs"]
mod common;

use common::{committee, validator, view};
use minimmit_core::{Event, Processor, ProcessorError, Ready};

fn replay(processor: &mut Processor, events: &[Event]) -> Vec<Ready> {
    events
        .iter()
        .cloned()
        .map(|event| processor.step(event))
        .collect()
}

#[test]
fn processor_starts_after_genesis_for_local_committee_member() {
    let committee = committee();
    let processor =
        Processor::new(validator(2), committee.clone()).expect("local validator is a member");

    assert_eq!(processor.local_validator(), validator(2));
    assert_eq!(processor.committee(), &committee);
    assert_eq!(processor.current_view(), view(1));
}

#[test]
fn processor_rejects_local_validator_outside_committee() {
    assert_eq!(
        Processor::new(validator(99), committee()),
        Err(ProcessorError::UnknownLocalValidator {
            validator: validator(99),
        })
    );
}

#[test]
fn noop_event_returns_no_ready_output_and_preserves_state() {
    let mut processor =
        Processor::new(validator(0), committee()).expect("local validator is a member");
    let initial = processor.clone();

    let ready = processor.step(Event::Noop);

    assert_eq!(ready, Ready::none());
    assert!(ready.is_empty());
    assert_eq!(processor, initial);
}

#[test]
fn same_event_sequence_replays_to_same_ready_outputs() {
    let events = [Event::Noop, Event::Noop, Event::Noop];
    let mut first = Processor::new(validator(0), committee()).expect("local validator is a member");
    let mut second =
        Processor::new(validator(0), committee()).expect("local validator is a member");

    let first_outputs = replay(&mut first, &events);
    let second_outputs = replay(&mut second, &events);

    assert_eq!(first_outputs, second_outputs);
    assert_eq!(first, second);
}
