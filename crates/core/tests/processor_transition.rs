mod common;

use common::committee;
use minimmit_core::{
    Event, Processor, ProcessorError, Ready, ReadyBatch, ReadyBatchError, ReadyBatchId, ReadyError,
    ReadyOutput, ValidatorId, ViewNumber,
};

fn replay(processor: &mut Processor, events: &[Event]) -> Vec<Ready> {
    events
        .iter()
        .cloned()
        .map(|event| processor.step(event))
        .collect()
}

fn persistence_batch(id: u64) -> ReadyBatch {
    ReadyBatch::new(ReadyBatchId::new(id), [ReadyOutput::PersistProcessorState])
        .expect("persistence output makes the batch non-empty")
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
    let mut processor =
        Processor::new(ValidatorId::new(0), committee()).expect("local validator is a member");
    let initial = processor.clone();

    let ready = processor.step(Event::Noop);

    assert_eq!(ready, Ready::none());
    assert!(ready.is_empty());
    assert_eq!(ready.batches(), &[]);
    assert_eq!(processor, initial);
}

#[test]
fn ready_preserves_batch_identity_and_order() {
    let ready = Ready::from_batches([persistence_batch(2), persistence_batch(1)])
        .expect("batch ids are distinct");

    assert!(!ready.is_empty());
    assert_eq!(
        ready
            .batches()
            .iter()
            .map(ReadyBatch::id)
            .collect::<Vec<_>>(),
        [ReadyBatchId::new(2), ReadyBatchId::new(1)]
    );
    assert_eq!(
        ready.batches()[0].outputs(),
        &[ReadyOutput::PersistProcessorState]
    );
}

#[test]
fn ready_rejects_duplicate_batch_ids() {
    assert_eq!(
        Ready::from_batches([persistence_batch(7), persistence_batch(7)]),
        Err(ReadyError::DuplicateBatch {
            batch: ReadyBatchId::new(7),
        })
    );
}

#[test]
fn ready_batch_rejects_empty_outputs() {
    assert_eq!(
        ReadyBatch::new(ReadyBatchId::new(9), std::iter::empty()),
        Err(ReadyBatchError::EmptyOutputBatch {
            batch: ReadyBatchId::new(9),
        })
    );
}

#[test]
fn same_event_sequence_replays_to_same_ready_outputs() {
    let events = [Event::Noop, Event::Noop, Event::Noop];
    let mut first =
        Processor::new(ValidatorId::new(0), committee()).expect("local validator is a member");
    let mut second =
        Processor::new(ValidatorId::new(0), committee()).expect("local validator is a member");

    let first_outputs = replay(&mut first, &events);
    let second_outputs = replay(&mut second, &events);

    assert_eq!(first_outputs, second_outputs);
    assert_eq!(first, second);
}
