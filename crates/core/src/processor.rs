use std::{collections::BTreeMap, fmt};

use crate::{
    validate_proposal, BlockId, Committee, MNotarization, Nullification, Nullify, Proposal,
    SignedBlock, ValidatorId, ViewNumber, Vote,
};

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
    observed_proposals: BTreeMap<ViewNumber, BTreeMap<BlockId, Proposal>>,
    observed_m_notarizations: BTreeMap<ViewNumber, BTreeMap<BlockId, MNotarization>>,
    observed_nullifications: BTreeMap<ViewNumber, Nullification>,
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
            observed_proposals: BTreeMap::new(),
            observed_m_notarizations: BTreeMap::new(),
            observed_nullifications: BTreeMap::new(),
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

    /// Iterates observed proposals for `view` in deterministic block order.
    pub fn observed_proposals(&self, view: ViewNumber) -> impl Iterator<Item = &Proposal> + '_ {
        self.observed_proposals
            .get(&view)
            .into_iter()
            .flat_map(|by_block| by_block.values())
    }

    /// Iterates observed M-notarizations in deterministic view then block order.
    pub fn observed_m_notarizations(&self) -> impl Iterator<Item = &MNotarization> + '_ {
        self.observed_m_notarizations
            .values()
            .flat_map(|by_block| by_block.values())
    }

    /// Iterates observed nullifications in deterministic view order.
    pub fn observed_nullifications(&self) -> impl Iterator<Item = &Nullification> + '_ {
        self.observed_nullifications.values()
    }

    /// Applies one deterministic protocol event and returns ready output.
    ///
    /// Artifact events record local protocol evidence that is valid for this
    /// processor's committee without implying any downstream vote, proposal,
    /// forwarding, view advancement, or persistence behavior.
    #[must_use]
    pub fn step(&mut self, event: Event) -> Ready {
        match event {
            Event::Noop => Ready::None,
            Event::Proposal(proposal) => {
                self.record_proposal(proposal);
                Ready::None
            }
            Event::Nullification(nullification) => {
                self.record_nullification(nullification);
                Ready::None
            }
            Event::MNotarization(notarization) => {
                self.record_m_notarization(notarization);
                Ready::None
            }
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

    fn record_proposal(&mut self, proposal: Proposal) {
        if !self.valid_proposal_observation(&proposal) {
            return;
        }

        let view = proposal.block().view();
        let block = proposal.block().id();

        self.observed_proposals
            .entry(view)
            .or_default()
            .entry(block)
            .or_insert(proposal);
    }

    fn record_m_notarization(&mut self, notarization: MNotarization) {
        let view = notarization.view();
        if view == ViewNumber::GENESIS {
            return;
        }
        if !self.valid_m_notarization(&notarization) {
            return;
        }

        let block = notarization.block();
        self.observed_m_notarizations
            .entry(view)
            .or_default()
            .entry(block)
            .or_insert(notarization);
    }

    fn record_nullification(&mut self, nullification: Nullification) {
        let view = nullification.view();
        if view == ViewNumber::GENESIS {
            return;
        }
        if !self.valid_nullification(&nullification) {
            return;
        }

        self.observed_nullifications
            .entry(view)
            .or_insert(nullification);
    }

    fn valid_proposal_observation(&self, proposal: &Proposal) -> bool {
        if !self.valid_m_notarization(proposal.parent_notarization())
            || proposal
                .nullifications()
                .any(|nullification| !self.valid_nullification(nullification))
        {
            return false;
        }

        let signed_block = SignedBlock::new(proposal.proposer(), proposal.block().clone());
        validate_proposal(
            &self.committee,
            proposal.block().view(),
            [&signed_block],
            [proposal.parent_notarization()],
            proposal.nullifications(),
        )
        .is_ok()
    }

    fn valid_m_notarization(&self, notarization: &MNotarization) -> bool {
        MNotarization::from_votes(
            &self.committee,
            notarization
                .signers()
                .map(|signer| Vote::new(signer, notarization.block(), notarization.view())),
        )
        .is_ok()
    }

    fn valid_nullification(&self, nullification: &Nullification) -> bool {
        Nullification::from_nullifies(
            &self.committee,
            nullification
                .signers()
                .map(|signer| Nullify::new(signer, nullification.view())),
        )
        .is_ok()
    }
}

/// Deterministic protocol event observed by [`Processor`].
///
/// Artifact events are admitted only when their evidence and proposal predicate
/// validate against the processor's committee. Recording artifact events does
/// not by itself imply voting, proposing, forwarding, advancement, or
/// persistence.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// Explicit event with no protocol effect.
    Noop,
    /// A proposal artifact observed by this processor.
    Proposal(Proposal),
    /// A nullification artifact observed by this processor.
    Nullification(Nullification),
    /// An M-notarization artifact observed by this processor.
    MNotarization(MNotarization),
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
