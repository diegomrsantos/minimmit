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
    voted_current_view: Option<Vote>,
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
            voted_current_view: None,
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
    /// processor's committee. A current-view proposal also emits vote work when
    /// the vote guard allows it. A local proposal trigger records the proposal
    /// immediately and emits storage plus network work; when both outputs refer
    /// to that proposal, the shell must persist before broadcasting it.
    #[must_use]
    pub fn step(&mut self, event: Event) -> Ready {
        match event {
            Event::Noop => Ready::default(),
            Event::Propose(input) => self.propose(input),
            Event::Proposal(proposal) => self.observe_proposal(proposal),
            Event::Nullification(nullification) => {
                self.record_nullification(nullification);
                Ready::default()
            }
            Event::MNotarization(notarization) => {
                self.record_m_notarization(notarization);
                Ready::default()
            }
        }
    }

    /// Records an observed proposal and votes when the current-view guard allows it.
    fn observe_proposal(&mut self, proposal: Proposal) -> Ready {
        if !self.record_proposal(proposal.clone()) {
            return Ready::default();
        }

        self.vote_for_current_view_proposal(&proposal)
    }

    /// Starts the local leader proposal transition when the current view allows it.
    fn propose(&mut self, input: ProposalInput) -> Ready {
        if self.proposed_current_view
            || self.voted_current_view.is_some()
            || self.current_view_is_nullified()
            || self.committee.leader(self.current_view) != self.local_validator
        {
            return Ready::default();
        }

        let Some(proposal) = self.build_proposal(input) else {
            return Ready::default();
        };

        self.proposed_current_view = true;
        self.record_proposal(proposal.clone());
        let vote = Vote::new(
            self.local_validator,
            proposal.block().id(),
            self.current_view,
        );
        self.voted_current_view = Some(vote);

        Ready {
            storage: vec![
                Storage::PersistProposal(proposal.clone()),
                Storage::PersistVote(vote),
            ],
            network: vec![
                Network::BroadcastProposal(proposal),
                Network::BroadcastVote(vote),
            ],
        }
    }

    /// Builds the proposal that the local leader records and asks the shell to handle.
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
    /// choosing a later replacement policy. A valid local proposal for the
    /// current view consumes the local leader's one-proposal slot, even when it
    /// arrives through the artifact path.
    ///
    /// Returns whether the proposal was valid for this processor. Duplicate
    /// observations still return true because a valid duplicate can trigger a
    /// later transition without replacing the first stored proposal.
    fn record_proposal(&mut self, proposal: Proposal) -> bool {
        if !self.valid_proposal_observation(&proposal) {
            return false;
        }

        let consumes_local_proposal_slot = proposal.proposer() == self.local_validator
            && proposal.block().view() == self.current_view;
        let view = proposal.block().view();
        let block = proposal.block().id();

        self.observed_proposals
            .entry(view)
            .or_default()
            .entry(block)
            .or_insert(proposal);

        if consumes_local_proposal_slot {
            self.proposed_current_view = true;
        }

        true
    }

    /// Emits a local vote for an observed current-view proposal when allowed.
    fn vote_for_current_view_proposal(&mut self, proposal: &Proposal) -> Ready {
        if proposal.block().view() != self.current_view
            || self.voted_current_view.is_some()
            || self.current_view_is_nullified()
        {
            return Ready::default();
        }

        let vote = Vote::new(
            self.local_validator,
            proposal.block().id(),
            self.current_view,
        );
        self.voted_current_view = Some(vote);

        Ready {
            storage: vec![Storage::PersistVote(vote)],
            network: vec![Network::BroadcastVote(vote)],
        }
    }

    /// Returns whether this processor has observed nullification of its current view.
    fn current_view_is_nullified(&self) -> bool {
        self.observed_nullifications
            .contains_key(&self.current_view)
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
/// against the processor's committee. A current-view proposal can emit vote
/// work; other downstream behavior such as forwarding and view advancement is
/// implemented by separate transition paths.
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

/// Deterministic output produced by a [`Processor`] transition.
///
/// `Ready` describes work an outer shell should perform. Storage outputs are
/// listed separately from network outputs so a shell can make protocol
/// artifacts durable before releasing dependent messages. When a storage output
/// and a network output refer to the same protocol artifact, the shell persists
/// it first and only then broadcasts it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Ready {
    /// Storage work the shell should complete.
    pub storage: Vec<Storage>,
    /// Network work the shell should release after required storage work.
    pub network: Vec<Network>,
}

/// Storage work produced by a [`Processor`] transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Storage {
    /// Persist a proposal before releasing dependent network output.
    PersistProposal(Proposal),
    /// Persist a vote before releasing dependent network output.
    PersistVote(Vote),
}

/// Network work produced by a [`Processor`] transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Network {
    /// Broadcast a proposal after its matching storage output is durable.
    BroadcastProposal(Proposal),
    /// Broadcast a vote after its matching storage output is durable.
    BroadcastVote(Vote),
}

impl Ready {
    /// Returns true when the transition produced no ready outputs.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.storage.is_empty() && self.network.is_empty()
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
    use super::{Event, Network, Processor, ProposalInput, Ready, Storage};
    use crate::{
        Block, BlockId, Committee, MNotarization, Nullification, Nullify, Proposal, TransactionId,
        ValidatorId, ViewNumber, Vote,
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

    /// Builds a proposal signed by the view leader with caller supplied parent evidence.
    ///
    /// Use this helper when a test needs a proposal whose parent view, parent
    /// block, or skipped view nullifications differ from the simple previous
    /// view proposal shape.
    fn proposal_extending(
        block: BlockId,
        view: ViewNumber,
        parent: BlockId,
        parent_view: ViewNumber,
        nullifications: impl IntoIterator<Item = Nullification>,
    ) -> Proposal {
        let block = Block::new(block, view, parent, [TransactionId::new(block.get())])
            .expect("proposal block is valid");

        Proposal::new(
            committee().leader(view),
            block,
            m_notarization(parent, parent_view),
            nullifications,
        )
        .expect("proposal nullification views are unique")
    }

    fn proposal_input(block: BlockId) -> ProposalInput {
        ProposalInput::new(block, [TransactionId::new(block.get())])
    }

    fn observe_m_notarizations<const N: usize>(
        processor: &mut Processor,
        notarizations: [(BlockId, ViewNumber); N],
    ) {
        for (block, view) in notarizations {
            assert_eq!(
                processor.step(Event::MNotarization(m_notarization(block, view))),
                Ready::default(),
                "observing an M-notarization should not emit ready work"
            );
        }
    }

    fn observe_nullifications<const N: usize>(
        processor: &mut Processor,
        nullifications: [ViewNumber; N],
    ) {
        for view in nullifications {
            assert_eq!(
                processor.step(Event::Nullification(nullification(view))),
                Ready::default(),
                "observing a nullification should not emit ready work"
            );
        }
    }

    fn observed_proposal(processor: &Processor, view: ViewNumber) -> &crate::Proposal {
        processor
            .observed_proposals(view)
            .next()
            .expect("leader proposal is recorded immediately")
    }

    fn assert_proposal_extends_parent(
        proposal: &crate::Proposal,
        parent: BlockId,
        parent_view: ViewNumber,
        skipped_views: &[ViewNumber],
    ) {
        assert_eq!(proposal.block().parent(), parent);
        assert_eq!(
            (
                proposal.parent_notarization().block(),
                proposal.parent_notarization().view(),
            ),
            (parent, parent_view)
        );
        assert_eq!(
            proposal
                .nullifications()
                .map(Nullification::view)
                .collect::<Vec<_>>(),
            skipped_views
        );
    }

    #[test]
    fn leader_proposal_uses_selected_non_genesis_parent_and_skipped_nullifications() {
        // Given
        let mut processor = processor_at_view(ValidatorId::new(5), ViewNumber::new(5));
        // Current-view advancement is implemented later; seed the private view
        // so this production branch has evidence before advancement reaches it.
        observe_m_notarizations(
            &mut processor,
            [
                (BlockId::new(30), ViewNumber::new(3)),
                (BlockId::new(20), ViewNumber::new(3)),
                (BlockId::new(10), ViewNumber::new(2)),
            ],
        );
        observe_nullifications(&mut processor, [ViewNumber::new(4)]);

        // When
        let ready = processor.step(Event::Propose(proposal_input(BlockId::new(50))));

        // Then
        assert_eq!(ready.storage.len(), 2, "expected two storage outputs");
        assert_eq!(ready.network.len(), 2, "expected two network outputs");

        let Storage::PersistProposal(persisted) = &ready.storage[0] else {
            panic!("expected proposal storage output");
        };
        let Network::BroadcastProposal(broadcast) = &ready.network[0] else {
            panic!("expected proposal network output");
        };

        assert_eq!(persisted, broadcast);
        let expected_vote = Vote::new(ValidatorId::new(5), BlockId::new(50), ViewNumber::new(5));
        assert_eq!(ready.storage[1], Storage::PersistVote(expected_vote));
        assert_eq!(ready.network[1], Network::BroadcastVote(expected_vote));

        let proposal = observed_proposal(&processor, ViewNumber::new(5));
        assert_eq!(proposal, persisted);
        assert_proposal_extends_parent(
            proposal,
            BlockId::new(20),
            ViewNumber::new(3),
            &[ViewNumber::new(4)],
        );
    }

    #[test]
    fn leader_proposal_waits_for_all_skipped_view_nullifications() {
        // Given
        let mut processor = processor_at_view(ValidatorId::new(5), ViewNumber::new(5));
        observe_m_notarizations(&mut processor, [(BlockId::new(20), ViewNumber::new(2))]);
        observe_nullifications(&mut processor, [ViewNumber::new(3)]);

        // When
        let ready = processor.step(Event::Propose(proposal_input(BlockId::new(50))));

        // Then
        assert_eq!(ready, Ready::default());

        // Given
        observe_nullifications(&mut processor, [ViewNumber::new(4)]);

        // When
        let ready = processor.step(Event::Propose(proposal_input(BlockId::new(50))));

        // Then
        assert_eq!(ready.storage.len(), 2, "expected two storage outputs");
        assert_eq!(ready.network.len(), 2, "expected two network outputs");

        let Storage::PersistProposal(persisted) = &ready.storage[0] else {
            panic!("expected proposal storage output");
        };
        let Network::BroadcastProposal(broadcast) = &ready.network[0] else {
            panic!("expected proposal network output");
        };

        assert_eq!(persisted, broadcast);
        let expected_vote = Vote::new(ValidatorId::new(5), BlockId::new(50), ViewNumber::new(5));
        assert_eq!(ready.storage[1], Storage::PersistVote(expected_vote));
        assert_eq!(ready.network[1], Network::BroadcastVote(expected_vote));
    }

    #[test]
    fn stale_valid_proposal_does_not_emit_vote() {
        // Claim: MM-VOTE-VALID-PROPOSAL (votecheck/vote1).
        // Story: a valid stale proposal is observed, but it is not a current
        // view proposal and cannot receive the local vote.
        // Given
        let mut processor = processor_at_view(ValidatorId::new(5), ViewNumber::new(5));
        // Current-view advancement is implemented later; seed the private view
        // so stale proposal handling has executable evidence before then.
        let stale_proposal = proposal_extending(
            BlockId::new(40),
            ViewNumber::new(4),
            BlockId::new(30),
            ViewNumber::new(3),
            [],
        );

        // When
        let ready = processor.step(Event::Proposal(stale_proposal));

        // Then
        assert_eq!(ready, Ready::default());
        assert_eq!(
            processor
                .observed_proposals(ViewNumber::new(4))
                .map(|proposal| proposal.block().id())
                .collect::<Vec<_>>(),
            [BlockId::new(40)]
        );
    }

    #[test]
    fn current_view_proposal_missing_skipped_nullification_does_not_emit_vote() {
        // Claim: MM-VOTE-VALID-PROPOSAL (votecheck/vote1).
        // Story: a current view proposal with incomplete skipped view evidence
        // is rejected before the vote guard can consume the local vote.
        // Given
        let mut processor = processor_at_view(ValidatorId::new(5), ViewNumber::new(5));
        // Current-view advancement is implemented later; seed the private view
        // so current-view validation has evidence before advancement reaches it.
        let invalid_proposal = proposal_extending(
            BlockId::new(50),
            ViewNumber::new(5),
            BlockId::new(20),
            ViewNumber::new(2),
            [nullification(ViewNumber::new(3))],
        );

        // When
        let ready = processor.step(Event::Proposal(invalid_proposal));

        // Then
        assert_eq!(ready, Ready::default());
        assert_eq!(
            processor
                .observed_proposals(ViewNumber::new(5))
                .map(|proposal| proposal.block().id())
                .collect::<Vec<_>>(),
            []
        );
    }
}
