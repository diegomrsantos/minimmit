mod common;

use common::committee;
use minimmit_core::{
    Event, Lifecycle, PersistenceId, Processor, ProcessorError, Ready, ValidatorId, ViewNumber,
};

#[derive(Clone, Copy)]
enum TraceInput {
    Event(Event),
    Lifecycle(Lifecycle),
}

fn replay(processor: &mut Processor, inputs: &[TraceInput]) -> Vec<Ready> {
    inputs
        .iter()
        .map(|input| match *input {
            TraceInput::Event(event) => processor.step(event),
            TraceInput::Lifecycle(event) => processor.lifecycle(event),
        })
        .collect()
}

fn processor() -> Processor {
    Processor::new(ValidatorId::new(0), committee()).expect("local validator is a member")
}

#[test]
fn processor_starts_after_genesis_for_local_committee_member() {
    let committee = committee();
    let processor = Processor::new(ValidatorId::new(2), committee.clone())
        .expect("local validator is a member");

    assert_eq!(processor.local_validator(), ValidatorId::new(2));
    assert_eq!(processor.committee(), &committee);
    assert_eq!(processor.current_view(), ViewNumber::new(1));
}

#[test]
fn processor_rejects_local_validator_outside_committee() {
    assert_eq!(
        Processor::new(ValidatorId::new(99), committee()),
        Err(ProcessorError::UnknownLocalValidator {
            validator: ValidatorId::new(99),
        })
    );
}

#[test]
fn noop_event_returns_no_ready_output_and_preserves_state() {
    let mut processor = processor();
    let initial = processor.clone();

    let ready = processor.step(Event::Noop);

    assert_eq!(ready, Ready::None);
    assert!(ready.is_empty());
    assert_eq!(processor, initial);
}

#[test]
fn persisted_lifecycle_without_pending_work_returns_no_ready_output() {
    let mut processor = processor();
    let initial = processor.clone();

    let ready = processor.lifecycle(Lifecycle::Persisted(PersistenceId::new(7)));

    assert_eq!(ready, Ready::None);
    assert!(ready.is_empty());
    assert_eq!(processor, initial);
}

#[test]
fn persist_ready_output_is_not_empty() {
    let ready = Ready::Persist {
        id: PersistenceId::new(42),
    };

    assert!(!ready.is_empty());
}

#[test]
fn same_event_and_lifecycle_sequence_replays_to_same_ready_outputs() {
    let inputs = [
        TraceInput::Event(Event::Noop),
        TraceInput::Lifecycle(Lifecycle::Persisted(PersistenceId::new(7))),
        TraceInput::Event(Event::Noop),
    ];
    let mut first = processor();
    let mut second = processor();

    let first_outputs = replay(&mut first, &inputs);
    let second_outputs = replay(&mut second, &inputs);

    assert_eq!(first_outputs, second_outputs);
    assert_eq!(first, second);
}
