use std::{cell::Cell, collections::BTreeMap};

use super::{
    AvailabilityError, AvailabilityProvider, AvailabilityResult, AvailabilityStatus,
    AvailabilityTarget,
};

/// Fixture-backed provider for deterministic tests and offline CLI scaffolding.
#[derive(Debug, Default)]
pub struct MockAvailabilityProvider {
    responses: BTreeMap<(AvailabilityTarget, String), AvailabilityResult>,
    calls: Cell<usize>,
}

impl MockAvailabilityProvider {
    /// Creates an empty provider that reports unknown for unconfigured requests.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a deterministic response and returns the provider for chaining.
    #[must_use]
    pub fn with_result(mut self, result: AvailabilityResult) -> Self {
        self.responses.insert(
            (
                result.target.clone(),
                result.name.trim().to_ascii_lowercase(),
            ),
            result,
        );
        self
    }

    /// Number of provider calls received.
    #[must_use]
    pub fn call_count(&self) -> usize {
        self.calls.get()
    }
}

impl AvailabilityProvider for MockAvailabilityProvider {
    fn check(
        &self,
        target: &AvailabilityTarget,
        name: &str,
    ) -> Result<AvailabilityResult, AvailabilityError> {
        self.calls.set(self.calls.get() + 1);
        Ok(self
            .responses
            .get(&(target.clone(), name.trim().to_ascii_lowercase()))
            .cloned()
            .unwrap_or_else(|| {
                AvailabilityResult::new(
                    target.clone(),
                    name,
                    AvailabilityStatus::Unknown,
                    "mock-offline",
                )
            }))
    }
}
