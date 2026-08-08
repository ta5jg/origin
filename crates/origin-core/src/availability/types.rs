use serde::Serialize;

/// A registry or namespace that can be screened for a candidate name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum AvailabilityTarget {
    /// A GitHub organization or user namespace.
    GitHub,
    /// The crates.io package registry.
    CratesIo,
    /// The npm package registry.
    Npm,
    /// The Python Package Index package registry.
    PyPi,
    /// A domain name under the supplied top-level domain.
    Domain {
        /// Top-level domain without a leading dot, for example `com`.
        tld: String,
    },
}

impl AvailabilityTarget {
    /// Returns a stable machine-readable identifier for this target.
    #[must_use]
    pub fn code(&self) -> String {
        match self {
            Self::GitHub => "github".into(),
            Self::CratesIo => "crates.io".into(),
            Self::Npm => "npm".into(),
            Self::PyPi => "pypi".into(),
            Self::Domain { tld } => format!("domain.{tld}"),
        }
    }
}

/// The observed state of a candidate in one target namespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AvailabilityStatus {
    /// The provider found no conflicting registration.
    Available,
    /// The provider found an existing registration.
    Taken,
    /// The provider cannot determine a state, including offline mode.
    Unknown,
}

/// One provider response, including an auditable source label.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AvailabilityResult {
    /// The target checked.
    pub target: AvailabilityTarget,
    /// Candidate name as submitted to the provider.
    pub name: String,
    /// Observed availability state.
    pub status: AvailabilityStatus,
    /// Provider identifier or evidence reference.
    pub source: String,
    /// Optional human-readable detail about the observation.
    pub detail: Option<String>,
    /// Unix timestamp in milliseconds when the provider performed the lookup.
    ///
    /// Offline fixture results leave this unset because they are not external
    /// evidence.
    pub checked_at_unix_ms: Option<u128>,
}

impl AvailabilityResult {
    /// Creates a result with no supplemental detail.
    #[must_use]
    pub fn new(
        target: AvailabilityTarget,
        name: impl Into<String>,
        status: AvailabilityStatus,
        source: impl Into<String>,
    ) -> Self {
        Self {
            target,
            name: name.into(),
            status,
            source: source.into(),
            detail: None,
            checked_at_unix_ms: None,
        }
    }

    /// Adds provider detail and a machine-readable evidence timestamp.
    #[must_use]
    pub fn with_evidence(mut self, detail: impl Into<String>, checked_at_unix_ms: u128) -> Self {
        self.detail = Some(detail.into());
        self.checked_at_unix_ms = Some(checked_at_unix_ms);
        self
    }
}
