use std::{collections::BTreeMap, fmt};

use crate::{
    select_parent, validate_proposal, Block, BlockId, Committee, MNotarization, Nullification,
    Nullify, Proposal, SignedBlock, TransactionId, ValidatorId, ViewNumber, Vote,
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
    proposed_current_view: bool,
    pending_proposal: Option<PendingProposal>,
    next_persistence_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingProposal {
    persistence_id: PersistenceId,
    proposal: Proposal,
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
            proposed_current_view: false,
            pending_proposal: None,
            next_persistence_id: 1,
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
    /// processor's committee without implying any downstream vote, forwarding,
    /// view advancement, or persistence behavior. A local proposal trigger is
    /// persistence-gated: it emits persistence work before the shell may
    /// release the proposal.
    #[must_use]
    pub fn step(&mut self, event: Event) -> Ready {
        match event {
            Event::Noop => Ready::None,
            Event::Propose(input) => self.propose(input),
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
    /// Persistence acknowledgements are explicit lifecycle inputs. Unknown,
    /// duplicate, or stale acknowledgements are deterministic no-ops.
    #[must_use]
    pub fn lifecycle(&mut self, event: Lifecycle) -> Ready {
        match event {
            Lifecycle::Persisted(id) => self.acknowledge_persisted(id),
        }
    }

    /// Starts the local leader proposal transition when the current view allows it.
    fn propose(&mut self, input: ProposalInput) -> Ready {
        if self.proposed_current_view
            || self.committee.leader(self.current_view) != self.local_validator
        {
            return Ready::None;
        }

        let Some(proposal) = self.build_proposal(input) else {
            return Ready::None;
        };
        let Some(persistence_id) = self.allocate_persistence_id() else {
            return Ready::None;
        };

        self.proposed_current_view = true;
        self.pending_proposal = Some(PendingProposal {
            persistence_id,
            proposal: proposal.clone(),
        });

        Ready::Persist {
            id: persistence_id,
            proposal,
        }
    }

    /// Marks persisted proposal work complete after the matching acknowledgement.
    fn acknowledge_persisted(&mut self, id: PersistenceId) -> Ready {
        let Some(pending) = self.pending_proposal.take() else {
            return Ready::None;
        };

        if pending.persistence_id != id {
            self.pending_proposal = Some(pending);
            return Ready::None;
        }

        self.record_proposal(pending.proposal);

        Ready::None
    }

    /// Allocates the next persistence id without wrapping.
    fn allocate_persistence_id(&mut self) -> Option<PersistenceId> {
        let next = self.next_persistence_id.checked_add(1)?;
        let id = PersistenceId::new(self.next_persistence_id);
        self.next_persistence_id = next;
        Some(id)
    }

    /// Builds the proposal that the local leader asks the shell to persist.
    fn build_proposal(&self, input: ProposalInput) -> Option<Proposal> {
        let parent = select_parent(self.observed_m_notarizations(), self.current_view).ok()?;
        let parent_notarization = self.parent_notarization(parent.block(), parent.view())?;
        let nullifications = self.skipped_view_nullifications(parent.view())?;
        let block = Block::new(
            input.block,
            self.current_view,
            parent.block(),
            input.transactions,
        )
        .ok()?;

        Proposal::new(
            self.local_validator,
            block,
            parent_notarization,
            nullifications,
        )
        .ok()
    }

    /// Returns the parent M-notarization carried by a local proposal.
    fn parent_notarization(&self, block: BlockId, view: ViewNumber) -> Option<MNotarization> {
        if block == BlockId::GENESIS && view == ViewNumber::GENESIS {
            return self.implicit_genesis_notarization();
        }

        self.observed_m_notarizations
            .get(&view)
            .and_then(|by_block| by_block.get(&block))
            .cloned()
    }

    /// Builds the implicit genesis M-notarization from deterministic committee order.
    fn implicit_genesis_notarization(&self) -> Option<MNotarization> {
        MNotarization::from_votes(
            &self.committee,
            self.committee
                .validators()
                .take(self.committee.config().m_threshold())
                .map(|signer| Vote::new(signer, BlockId::GENESIS, ViewNumber::GENESIS)),
        )
        .ok()
    }

    /// Collects skipped-view nullifications required by a local proposal.
    fn skipped_view_nullifications(&self, parent_view: ViewNumber) -> Option<Vec<Nullification>> {
        let start = parent_view.get().checked_add(1)?;
        let mut nullifications = Vec::new();

        for view in start..self.current_view.get() {
            let view = ViewNumber::new(view);
            nullifications.push(self.observed_nullifications.get(&view)?.clone());
        }

        Some(nullifications)
    }

    /// Records a proposal when it validates against this processor's committee.
    ///
    /// Duplicate observations for the same `(view, block)` keep the first
    /// admitted proposal so observation storage stays deterministic without
    /// choosing a later replacement policy.
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

    /// Records a non-genesis M-notarization valid for this processor's committee.
    ///
    /// Duplicate observations for the same `(view, block)` keep the first
    /// admitted notarization. Forwarding can add a separate proof-selection
    /// policy when forwarding output exists.
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

    /// Records a non-genesis nullification valid for this processor's committee.
    ///
    /// Duplicate observations for the same view keep the first admitted
    /// nullification.
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

    /// Returns whether a proposal can be admitted into this processor's state.
    ///
    /// Admission requires the carried evidence to validate against this
    /// processor's committee and the proposal predicate to hold for the
    /// proposal's view.
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

    /// Returns whether an M-notarization certificate belongs to this committee.
    fn valid_m_notarization(&self, notarization: &MNotarization) -> bool {
        MNotarization::from_votes(
            &self.committee,
            notarization
                .signers()
                .map(|signer| Vote::new(signer, notarization.block(), notarization.view())),
        )
        .is_ok()
    }

    /// Returns whether a nullification certificate belongs to this committee.
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
/// Events include local triggers and observed protocol artifacts. Artifact
/// events are admitted only when their evidence and proposal predicate validate
/// against the processor's committee. Recording artifact events does not by
/// itself imply voting, proposing, forwarding, advancement, or persistence.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// Explicit event with no protocol effect.
    Noop,
    /// Local trigger to propose a block in the current view if this processor
    /// is the leader.
    Propose(ProposalInput),
    /// A proposal artifact observed by this processor.
    Proposal(Proposal),
    /// A nullification artifact observed by this processor.
    Nullification(Nullification),
    /// An M-notarization artifact observed by this processor.
    MNotarization(MNotarization),
}

/// Local block contents supplied to the leader proposal transition.
///
/// The processor chooses the current view and parent from local protocol state.
/// Mempool policy, transaction payloads, hashing, and real signing stay outside
/// `minimmit-core`, so the trigger carries only the modeled block identity and
/// ordered transaction identifiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalInput {
    block: BlockId,
    transactions: Vec<TransactionId>,
}

impl ProposalInput {
    /// Creates local proposal input from a block identity and transaction ids.
    pub fn new<I>(block: BlockId, transactions: I) -> Self
    where
        I: IntoIterator<Item = TransactionId>,
    {
        Self {
            block,
            transactions: transactions.into_iter().collect(),
        }
    }

    /// Returns the requested block identity.
    #[must_use]
    pub fn block(&self) -> BlockId {
        self.block
    }

    /// Returns the requested transaction identifiers in block order.
    #[must_use]
    pub fn transactions(&self) -> &[TransactionId] {
        &self.transactions
    }
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
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Ready {
    /// The transition produced no ready output.
    #[default]
    None,
    /// The shell should persist protocol state identified by `id`.
    Persist {
        /// Persistence correlation id the shell reports back after completion.
        id: PersistenceId,
        /// Proposal artifact the shell should persist before releasing it.
        proposal: Proposal,
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

#[cfg(test)]
mod tests {
    use super::{Event, Lifecycle, PersistenceId, Processor, ProposalInput, Ready};
    use crate::{
        BlockId, Committee, MNotarization, Nullification, Nullify, TransactionId, ValidatorId,
        ViewNumber, Vote,
    };

    fn committee() -> Committee {
        Committee::new((0..6).map(ValidatorId::new).collect::<Vec<_>>(), 1)
            .expect("committee satisfies n >= 5f + 1")
    }

    fn processor_at_view(local_validator: ValidatorId, current_view: ViewNumber) -> Processor {
        let mut processor =
            Processor::new(local_validator, committee()).expect("local validator is a member");
        processor.current_view = current_view;
        processor
    }

    fn m_notarization(block: BlockId, view: ViewNumber) -> MNotarization {
        MNotarization::from_votes(
            &committee(),
            [
                Vote::new(ValidatorId::new(0), block, view),
                Vote::new(ValidatorId::new(1), block, view),
                Vote::new(ValidatorId::new(2), block, view),
            ],
        )
        .expect("votes form an M-notarization")
    }

    fn nullification(view: ViewNumber) -> Nullification {
        Nullification::from_nullifies(
            &committee(),
            [
                Nullify::new(ValidatorId::new(0), view),
                Nullify::new(ValidatorId::new(1), view),
                Nullify::new(ValidatorId::new(2), view),
            ],
        )
        .expect("nullifies form a nullification")
    }

    fn proposal_input(block: BlockId) -> ProposalInput {
        ProposalInput::new(block, [TransactionId::new(block.get())])
    }

    fn persisted_proposal(ready: Ready) -> (PersistenceId, crate::Proposal) {
        let Ready::Persist { id, proposal } = ready else {
            panic!("expected persistence output, got {ready:?}");
        };
        (id, proposal)
    }

    #[test]
    fn leader_proposal_uses_selected_non_genesis_parent_and_skipped_nullifications() {
        let mut processor = processor_at_view(ValidatorId::new(5), ViewNumber::new(5));

        // Current-view advancement is implemented later; seed the private view
        // so this production branch has evidence before advancement reaches it.
        assert_eq!(
            processor.step(Event::MNotarization(m_notarization(
                BlockId::new(30),
                ViewNumber::new(3),
            ))),
            Ready::None
        );
        assert_eq!(
            processor.step(Event::MNotarization(m_notarization(
                BlockId::new(20),
                ViewNumber::new(3),
            ))),
            Ready::None
        );
        assert_eq!(
            processor.step(Event::MNotarization(m_notarization(
                BlockId::new(10),
                ViewNumber::new(2),
            ))),
            Ready::None
        );
        assert_eq!(
            processor.step(Event::Nullification(nullification(ViewNumber::new(4)))),
            Ready::None
        );

        let (id, persisted) =
            persisted_proposal(processor.step(Event::Propose(proposal_input(BlockId::new(50)))));
        assert_eq!(id, PersistenceId::new(1));
        assert_eq!(
            processor.lifecycle(Lifecycle::Persisted(PersistenceId::new(1))),
            Ready::None
        );

        let proposal = processor
            .observed_proposals(ViewNumber::new(5))
            .next()
            .expect("acknowledged proposal is recorded");
        assert_eq!(proposal, &persisted);
        assert_eq!(proposal.block().parent(), BlockId::new(20));
        assert_eq!(proposal.parent_notarization().block(), BlockId::new(20));
        assert_eq!(proposal.parent_notarization().view(), ViewNumber::new(3));
        assert_eq!(
            proposal
                .nullifications()
                .map(Nullification::view)
                .collect::<Vec<_>>(),
            [ViewNumber::new(4)]
        );
    }

    #[test]
    fn leader_proposal_waits_for_all_skipped_view_nullifications() {
        let mut processor = processor_at_view(ValidatorId::new(5), ViewNumber::new(5));

        assert_eq!(
            processor.step(Event::MNotarization(m_notarization(
                BlockId::new(20),
                ViewNumber::new(2),
            ))),
            Ready::None
        );
        assert_eq!(
            processor.step(Event::Nullification(nullification(ViewNumber::new(3)))),
            Ready::None
        );

        assert_eq!(
            processor.step(Event::Propose(proposal_input(BlockId::new(50)))),
            Ready::None
        );
        assert_eq!(
            processor.step(Event::Nullification(nullification(ViewNumber::new(4)))),
            Ready::None
        );

        let (id, _) =
            persisted_proposal(processor.step(Event::Propose(proposal_input(BlockId::new(50)))));
        assert_eq!(id, PersistenceId::new(1));
    }
}
