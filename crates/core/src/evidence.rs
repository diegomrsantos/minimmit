use std::{collections::BTreeSet, fmt};

use crate::{BlockId, Committee, ValidatorId, ViewNumber};

/// Modeled vote message from one validator for one block in one view.
///
/// A `Vote` is the raw input later evidence constructors aggregate. It records
/// the signer identity that authenticated the message, the voted block, and
/// the view in which the vote was cast. Protocol validity is contextual: it
/// depends on the active committee, distinct signer counting, quorum threshold,
/// and state-machine rules. This type stores the signed fields; evidence and
/// state-machine constructors apply those checks with the necessary context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vote {
    signer: ValidatorId,
    block: BlockId,
    view: ViewNumber,
}

impl Vote {
    /// Creates a vote message.
    #[must_use]
    pub const fn new(signer: ValidatorId, block: BlockId, view: ViewNumber) -> Self {
        Self {
            signer,
            block,
            view,
        }
    }

    /// Returns the validator identity that signed the vote.
    #[must_use]
    pub const fn signer(self) -> ValidatorId {
        self.signer
    }

    /// Returns the block this vote targets.
    #[must_use]
    pub const fn block(self) -> BlockId {
        self.block
    }

    /// Returns the view in which this vote was cast.
    #[must_use]
    pub const fn view(self) -> ViewNumber {
        self.view
    }
}

/// Modeled nullify message from one validator for one view.
///
/// A `Nullify` is the raw input later nullification evidence constructors
/// aggregate. It records signer intent for a view. Protocol validity is
/// contextual: it depends on the active committee, distinct signer counting,
/// quorum threshold, and state-machine rules. This type stores the signed
/// fields; evidence and state-machine constructors apply those checks with the
/// necessary context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Nullify {
    signer: ValidatorId,
    view: ViewNumber,
}

impl Nullify {
    /// Creates a nullify message.
    #[must_use]
    pub const fn new(signer: ValidatorId, view: ViewNumber) -> Self {
        Self { signer, view }
    }

    /// Returns the validator identity that signed the nullify message.
    #[must_use]
    pub const fn signer(self) -> ValidatorId {
        self.signer
    }

    /// Returns the view this nullify message targets.
    #[must_use]
    pub const fn view(self) -> ViewNumber {
        self.view
    }
}

/// Evidence that a block has the M-notarization vote threshold in one view.
///
/// `MNotarization` values are constructed from votes plus the active
/// committee. Construction checks that every signer is a committee member, each
/// signer appears once, all votes target the same block and view, and the
/// number of distinct valid signers meets the `2f + 1` M threshold. Proposal
/// validity and state-machine transition rules are checked outside this
/// evidence type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MNotarization {
    block: BlockId,
    view: ViewNumber,
    signers: BTreeSet<ValidatorId>,
}

impl MNotarization {
    /// Creates an M-notarization from votes and the active committee.
    ///
    /// Returns [`EvidenceError`] when the vote set is empty, includes a
    /// non-member or duplicate signer, mixes block/view targets, or has fewer
    /// than `2f + 1` distinct valid signers.
    pub fn from_votes<I>(committee: &Committee, votes: I) -> Result<Self, EvidenceError>
    where
        I: IntoIterator<Item = Vote>,
    {
        let ((block, view), signers) =
            collect_evidence(committee, votes, committee.config().m_threshold())?;

        Ok(Self {
            block,
            view,
            signers,
        })
    }

    /// Returns the notarized block.
    #[must_use]
    pub fn block(&self) -> BlockId {
        self.block
    }

    /// Returns the notarized view.
    #[must_use]
    pub fn view(&self) -> ViewNumber {
        self.view
    }

    /// Iterates signer identities in deterministic order.
    pub fn signers(&self) -> impl Iterator<Item = ValidatorId> + '_ {
        self.signers.iter().copied()
    }
}

/// Evidence that a block has the L-notarization vote threshold in one view.
///
/// `LNotarization` has the same fields as [`MNotarization`] but is constructed
/// with the `n - f` L threshold from the active committee configuration.
/// Proposal validity, finalization, and state-machine transition rules are
/// checked outside this evidence type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LNotarization {
    block: BlockId,
    view: ViewNumber,
    signers: BTreeSet<ValidatorId>,
}

impl LNotarization {
    /// Creates an L-notarization from votes and the active committee.
    ///
    /// Returns [`EvidenceError`] when the vote set is empty, includes a
    /// non-member or duplicate signer, mixes block/view targets, or has fewer
    /// than `n - f` distinct valid signers.
    pub fn from_votes<I>(committee: &Committee, votes: I) -> Result<Self, EvidenceError>
    where
        I: IntoIterator<Item = Vote>,
    {
        let ((block, view), signers) =
            collect_evidence(committee, votes, committee.config().l_threshold())?;

        Ok(Self {
            block,
            view,
            signers,
        })
    }

    /// Returns the notarized block.
    #[must_use]
    pub fn block(&self) -> BlockId {
        self.block
    }

    /// Returns the notarized view.
    #[must_use]
    pub fn view(&self) -> ViewNumber {
        self.view
    }

    /// Iterates signer identities in deterministic order.
    pub fn signers(&self) -> impl Iterator<Item = ValidatorId> + '_ {
        self.signers.iter().copied()
    }
}

/// Evidence that a view has the nullification threshold.
///
/// `Nullification` is constructed from nullify messages plus the active
/// committee. Construction checks that every signer is a committee member, each
/// signer appears once, all messages target the same view, and the number of
/// distinct valid signers meets the `2f + 1` nullification threshold. Timeout,
/// condition-b, and state-machine transition rules are checked outside this
/// evidence type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nullification {
    view: ViewNumber,
    signers: BTreeSet<ValidatorId>,
}

impl Nullification {
    /// Creates a nullification from nullify messages and the active committee.
    ///
    /// Returns [`EvidenceError`] when the message set is empty, includes a
    /// non-member or duplicate signer, mixes target views, or has fewer than
    /// `2f + 1` distinct valid signers.
    pub fn from_nullifies<I>(committee: &Committee, nullifies: I) -> Result<Self, EvidenceError>
    where
        I: IntoIterator<Item = Nullify>,
    {
        let (view, signers) = collect_evidence(
            committee,
            nullifies,
            committee.config().nullification_threshold(),
        )?;

        Ok(Self { view, signers })
    }

    /// Returns the nullified view.
    #[must_use]
    pub fn view(&self) -> ViewNumber {
        self.view
    }

    /// Iterates signer identities in deterministic order.
    pub fn signers(&self) -> impl Iterator<Item = ValidatorId> + '_ {
        self.signers.iter().copied()
    }
}

/// Evidence construction errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceError {
    /// No messages were supplied.
    Empty,
    /// The same signer appeared more than once.
    DuplicateSigner {
        /// Duplicated signer identity.
        signer: ValidatorId,
    },
    /// A signer is not in the active committee.
    UnknownSigner {
        /// Non-member signer identity.
        signer: ValidatorId,
    },
    /// Votes did not all target the same block and view.
    ConflictingVoteTarget {
        /// Expected block from the first vote.
        expected_block: BlockId,
        /// Expected view from the first vote.
        expected_view: ViewNumber,
        /// Conflicting vote block.
        actual_block: BlockId,
        /// Conflicting vote view.
        actual_view: ViewNumber,
    },
    /// Nullify messages did not all target the same view.
    ConflictingNullificationView {
        /// Expected view from the first nullify message.
        expected_view: ViewNumber,
        /// Conflicting nullify view.
        actual_view: ViewNumber,
    },
    /// The evidence had fewer distinct valid signers than required.
    BelowThreshold {
        /// Number of distinct valid signers.
        signer_count: usize,
        /// Required threshold.
        threshold: usize,
    },
}

impl fmt::Display for EvidenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(formatter, "evidence is empty"),
            Self::DuplicateSigner { signer } => {
                write!(formatter, "{signer} appears more than once in evidence")
            }
            Self::UnknownSigner { signer } => {
                write!(formatter, "{signer} is not a committee member")
            }
            Self::ConflictingVoteTarget {
                expected_block,
                expected_view,
                actual_block,
                actual_view,
            } => write!(
                formatter,
                "vote targets {actual_block} in {actual_view}, expected {expected_block} in {expected_view}"
            ),
            Self::ConflictingNullificationView {
                expected_view,
                actual_view,
            } => write!(
                formatter,
                "nullify message targets {actual_view}, expected {expected_view}"
            ),
            Self::BelowThreshold {
                signer_count,
                threshold,
            } => write!(
                formatter,
                "evidence has {signer_count} distinct valid signers, below threshold {threshold}"
            ),
        }
    }
}

impl std::error::Error for EvidenceError {}

/// Message type that can contribute to threshold evidence.
trait EvidenceMessage {
    /// Target all messages in one evidence value must agree on.
    type Target: Copy + Eq;

    /// Returns the signer identity carried by the message.
    fn signer(&self) -> ValidatorId;

    /// Returns the message target.
    fn target(&self) -> Self::Target;

    /// Builds the target-conflict error for this message kind.
    fn conflicting_target_error(expected: Self::Target, actual: Self::Target) -> EvidenceError;
}

impl EvidenceMessage for Vote {
    type Target = (BlockId, ViewNumber);

    fn signer(&self) -> ValidatorId {
        self.signer
    }

    fn target(&self) -> Self::Target {
        (self.block, self.view)
    }

    fn conflicting_target_error(expected: Self::Target, actual: Self::Target) -> EvidenceError {
        EvidenceError::ConflictingVoteTarget {
            expected_block: expected.0,
            expected_view: expected.1,
            actual_block: actual.0,
            actual_view: actual.1,
        }
    }
}

impl EvidenceMessage for Nullify {
    type Target = ViewNumber;

    fn signer(&self) -> ValidatorId {
        self.signer
    }

    fn target(&self) -> Self::Target {
        self.view
    }

    fn conflicting_target_error(expected: Self::Target, actual: Self::Target) -> EvidenceError {
        EvidenceError::ConflictingNullificationView {
            expected_view: expected,
            actual_view: actual,
        }
    }
}

/// Collects one-target threshold evidence from modeled messages.
fn collect_evidence<I, M>(
    committee: &Committee,
    messages: I,
    threshold: usize,
) -> Result<(M::Target, BTreeSet<ValidatorId>), EvidenceError>
where
    I: IntoIterator<Item = M>,
    M: EvidenceMessage,
{
    let mut messages = messages.into_iter();
    let first = messages.next().ok_or(EvidenceError::Empty)?;
    let target = first.target();
    let mut signers = BTreeSet::new();

    for message in std::iter::once(first).chain(messages) {
        let actual_target = message.target();
        if actual_target != target {
            return Err(M::conflicting_target_error(target, actual_target));
        }

        let signer = message.signer();
        if !committee.contains(signer) {
            return Err(EvidenceError::UnknownSigner { signer });
        }

        if !signers.insert(signer) {
            return Err(EvidenceError::DuplicateSigner { signer });
        }
    }

    if signers.len() < threshold {
        return Err(EvidenceError::BelowThreshold {
            signer_count: signers.len(),
            threshold,
        });
    }

    Ok((target, signers))
}
