//! Core protocol state machine crate for Minimmit.
//!
//! Protocol behavior in this crate should stay deterministic and reviewable
//! from the core state machine.

mod block;
mod committee;
mod config;
mod evidence;
mod identity;
mod processor;
mod proposal;
mod proposal_validation;
mod ready;

pub use block::{Block, BlockError, SignedBlock};
pub use committee::{Committee, CommitteeError};
pub use config::{Config, ConfigError};
pub use evidence::{EvidenceError, LNotarization, MNotarization, Nullification, Nullify, Vote};
pub use identity::{BlockId, TransactionId, ValidatorId, ViewNumber};
pub use processor::{Event, Processor, ProcessorError};
pub use proposal::{Proposal, ProposalError};
pub use proposal_validation::{
    select_parent, validate_proposal, ParentSelectionError, ProposalValidationError,
    SelectedParent, ValidatedProposal,
};
pub use ready::{Ready, ReadyBatch, ReadyBatchError, ReadyBatchId, ReadyError, ReadyOutput};
