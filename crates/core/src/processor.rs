use std::fmt;

use crate::{Committee, ValidatorId, ViewNumber};

const FIRST_NON_GENESIS_VIEW: ViewNumber = ViewNumber::new(1);

/// Deterministic local processor state.
///
/// A `Processor` is one local participant executing the baseline Minimmit
/// protocol. The implementation identifies processors by [`ValidatorId`]
/// because authenticated protocol messages are modeled as signer identities
/// from the validator committee.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Processor {
    local_validator: ValidatorId,
    committee: Committee,
    current_view: ViewNumber,
}

impl Processor {
    /// Creates local processor state for a committee member.
    ///
    /// The initial state starts after genesis in view 1. The genesis block and
    /// genesis notarizations are implicit in the baseline protocol model.
    ///
    /// # Errors
    ///
    /// Returns [`ProcessorError::UnknownLocalValidator`] when `local_validator` is
    /// not a member of `committee`.
    pub fn new(local_validator: ValidatorId, committee: Committee) -> Result<Self, ProcessorError> {
        if !committee.contains(local_validator) {
            return Err(ProcessorError::UnknownLocalValidator {
                validator: local_validator,
            });
        }

        Ok(Self {
            local_validator,
            committee,
            current_view: FIRST_NON_GENESIS_VIEW,
        })
    }

    /// Returns the local validator identity represented by this processor.
    #[must_use]
    pub fn local_validator(&self) -> ValidatorId {
        self.local_validator
    }

    /// Returns the active validator committee.
    #[must_use]
    pub fn committee(&self) -> &Committee {
        &self.committee
    }

    /// Returns the current local view.
    #[must_use]
    pub fn current_view(&self) -> ViewNumber {
        self.current_view
    }

    /// Applies one deterministic protocol event and returns ready output.
    ///
    /// Claim-specific proposal, vote, nullification, forwarding, and
    /// finalization events are added only with their own executable evidence.
    #[must_use]
    pub fn step(&mut self, event: Event) -> Ready {
        match event {
            Event::Noop => Ready::None,
        }
    }

    /// Applies shell lifecycle feedback and returns ready output.
    ///
    /// Persistence acknowledgements are explicit lifecycle inputs. They are a
    /// no-op until a real protocol transition emits persistence work and records
    /// pending state that can be matched by [`PersistenceId`].
    #[must_use]
    pub fn lifecycle(&mut self, event: Lifecycle) -> Ready {
        match event {
            Lifecycle::Persisted(_) => Ready::None,
        }
    }
}

/// Deterministic protocol event observed by [`Processor`].
///
/// Protocol message, timeout, and evidence events are intentionally not modeled
/// here until their behavior is implemented with claim-specific executable
/// evidence.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// Explicit event with no protocol effect.
    Noop,
}

/// Shell lifecycle feedback observed by [`Processor`].
///
/// Lifecycle input reports completion of shell-owned work without moving
/// storage, networking, or runtime behavior into the core.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lifecycle {
    /// The shell completed persistence for work identified by the persistence id.
    Persisted(PersistenceId),
}

/// Deterministic output produced by a [`Processor`] transition.
///
/// `Ready` describes work an outer shell should perform. Network,
/// storage-engine, timer, and runtime mechanics remain outside the core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Ready {
    /// The transition produced no ready output.
    #[default]
    None,
    /// The shell should persist protocol state identified by `id`.
    Persist {
        /// Persistence correlation id the shell reports back after completion.
        id: PersistenceId,
    },
}

impl Ready {
    /// Returns true when the transition produced no ready outputs.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        matches!(self, Self::None)
    }
}

/// Identifier that correlates persistence output with lifecycle completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PersistenceId(u64);

impl PersistenceId {
    /// Creates a persistence identifier.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the numeric persistence identifier.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for PersistenceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "persistence {}", self.0)
    }
}

/// Processor state construction errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessorError {
    /// The local validator is not in the active committee.
    UnknownLocalValidator {
        /// Non-member local validator identity.
        validator: ValidatorId,
    },
}

impl fmt::Display for ProcessorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownLocalValidator { validator } => {
                write!(formatter, "{validator} is not a committee member")
            }
        }
    }
}

impl std::error::Error for ProcessorError {}
