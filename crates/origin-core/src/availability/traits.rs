use super::{AvailabilityError, AvailabilityResult, AvailabilityTarget};

/// Pluggable external availability lookup boundary.
///
/// Implementations may use HTTP, an enterprise database, or fixtures. The core
/// crate makes no network requests itself, so deterministic offline tests remain
/// possible.
pub trait AvailabilityProvider {
    /// Checks one target namespace for the supplied candidate name.
    ///
    /// # Errors
    ///
    /// Returns an error when the provider cannot complete the lookup.
    fn check(
        &self,
        target: &AvailabilityTarget,
        name: &str,
    ) -> Result<AvailabilityResult, AvailabilityError>;

    /// Checks GitHub namespace availability.
    ///
    /// # Errors
    ///
    /// Propagates a provider lookup failure.
    fn github(&self, name: &str) -> Result<AvailabilityResult, AvailabilityError> {
        self.check(&AvailabilityTarget::GitHub, name)
    }

    /// Checks crates.io package availability.
    ///
    /// # Errors
    ///
    /// Propagates a provider lookup failure.
    fn crates_io(&self, name: &str) -> Result<AvailabilityResult, AvailabilityError> {
        self.check(&AvailabilityTarget::CratesIo, name)
    }

    /// Checks npm package availability.
    ///
    /// # Errors
    ///
    /// Propagates a provider lookup failure.
    fn npm(&self, name: &str) -> Result<AvailabilityResult, AvailabilityError> {
        self.check(&AvailabilityTarget::Npm, name)
    }

    /// Checks `PyPI` package availability.
    ///
    /// # Errors
    ///
    /// Propagates a provider lookup failure.
    fn pypi(&self, name: &str) -> Result<AvailabilityResult, AvailabilityError> {
        self.check(&AvailabilityTarget::PyPi, name)
    }

    /// Searches the configured company-register source for an exact-name conflict.
    ///
    /// # Errors
    ///
    /// Propagates a provider lookup failure.
    fn company(&self, name: &str) -> Result<AvailabilityResult, AvailabilityError> {
        self.check(&AvailabilityTarget::Company, name)
    }

    /// Searches the configured public web-presence source for an exact-name conflict.
    ///
    /// # Errors
    ///
    /// Propagates a provider lookup failure.
    fn web(&self, name: &str) -> Result<AvailabilityResult, AvailabilityError> {
        self.check(&AvailabilityTarget::Web, name)
    }

    /// Checks a domain under the supplied top-level domain.
    ///
    /// # Errors
    ///
    /// Propagates a provider lookup failure.
    fn domain(&self, name: &str, tld: &str) -> Result<AvailabilityResult, AvailabilityError> {
        self.check(
            &AvailabilityTarget::Domain {
                tld: tld.to_ascii_lowercase(),
            },
            name,
        )
    }
}
