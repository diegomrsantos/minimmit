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

    /// Applies one deterministic input event and returns ready output.
    ///
    /// This initial transition boundary only supports [`Event::Noop`], which
    /// lets tests and later scenario harnesses drive and replay the public
    /// `Event -> Processor -> Ready` shape before protocol behaviors are added.
    #[must_use]
    pub fn step(&mut self, event: Event) -> Ready {
        match event {
            Event::Noop => Ready::none(),
        }
    }
}

/// Deterministic input observed by [`Processor`].
///
/// Protocol message, timeout, and evidence events are intentionally not
/// modeled here until their behavior is implemented with claim-specific
/// executable evidence.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// Explicit event with no protocol effect.
    Noop,
}

/// Deterministic output produced by a [`Processor`] transition.
///
/// The type is intentionally empty for the first transition-boundary slice.
/// Later protocol behavior can add ready outputs without exposing runtime
/// machinery through the processor API.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Ready {
    _private: (),
}

impl Ready {
    /// Returns an empty ready output.
    #[must_use]
    pub const fn none() -> Self {
        Self { _private: () }
    }

    /// Returns true when the transition produced no ready outputs.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        true
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
