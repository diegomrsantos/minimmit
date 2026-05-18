use std::fmt;

/// Validator identity used by the protocol core.
///
/// This is a deterministic stand-in for authenticated signer identity. Real
/// cryptographic verification stays outside the core crate; protocol rules
/// count only identities that are members of the configured committee.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ValidatorId(u64);

impl ValidatorId {
    /// Creates a validator identity.
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the underlying deterministic identity value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ValidatorId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "validator {}", self.0)
    }
}

/// Baseline Minimmit view number.
///
/// The wrapped value is the protocol's scalar view index. Ordering follows
/// that numeric index so later state-machine rules can compare views without
/// carrying raw integers through the public API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ViewNumber(u64);

impl ViewNumber {
    /// The genesis view.
    pub const GENESIS: Self = Self(0);

    /// Creates a view number from its deterministic counter value.
    #[must_use]
    pub const fn new(number: u64) -> Self {
        Self(number)
    }

    /// Returns the underlying deterministic view number.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ViewNumber {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "view {}", self.0)
    }
}

/// Deterministic block identity used by the protocol core.
///
/// The paper compares blocks by their hashes. The core models that hash as an
/// opaque ordered scalar so tests can express block identity and tie-breaking
/// without introducing hashing or serialization concerns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockId(u64);

impl BlockId {
    /// The genesis block identity.
    pub const GENESIS: Self = Self(0);

    /// Creates a block identity from its modeled hash value.
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the underlying deterministic block identity value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for BlockId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "block {}", self.0)
    }
}

/// Opaque transaction identity carried by modeled blocks.
///
/// The core only needs stable transaction identity and ordering for baseline
/// block data. Transaction payloads and application semantics are intentionally
/// outside this protocol crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TransactionId(u64);

impl TransactionId {
    /// Creates a transaction identity from its deterministic value.
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the underlying deterministic transaction identity value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for TransactionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "transaction {}", self.0)
    }
}
