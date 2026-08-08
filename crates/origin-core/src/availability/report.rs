use serde::Serialize;

use super::{AvailabilityResult, AvailabilityStatus};

/// Aggregated availability evidence for one candidate name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AvailabilityReport {
    /// Candidate name screened by every result in the report.
    pub name: String,
    /// Individual target observations, in requested order.
    pub results: Vec<AvailabilityResult>,
}

impl AvailabilityReport {
    /// Creates a report from target observations.
    #[must_use]
    pub fn new(name: impl Into<String>, results: Vec<AvailabilityResult>) -> Self {
        Self {
            name: name.into(),
            results,
        }
    }

    /// Returns true when every checked target is available.
    #[must_use]
    pub fn is_clear(&self) -> bool {
        !self.results.is_empty()
            && self
                .results
                .iter()
                .all(|result| result.status == AvailabilityStatus::Available)
    }

    /// Returns the count of registrations reported as taken.
    #[must_use]
    pub fn taken_count(&self) -> usize {
        self.results
            .iter()
            .filter(|result| result.status == AvailabilityStatus::Taken)
            .count()
    }
}
