use serde::Serialize;

use super::{AvailabilityResult, AvailabilityStatus};

/// Decision derived from all requested availability sources.
///
/// A provisional result is intentionally not a clearance: one or more sources
/// did not return usable evidence. This makes partial public data visible
/// without treating it as availability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ClearanceRecommendation {
    /// Every requested source returned `Available`.
    Clear,
    /// No source reported a conflict, but at least one source is unknown.
    Provisional,
    /// At least one requested source reported a conflict.
    Reject,
}

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

    /// Returns the count of sources that could not supply evidence.
    #[must_use]
    pub fn unknown_count(&self) -> usize {
        self.results
            .iter()
            .filter(|result| result.status == AvailabilityStatus::Unknown)
            .count()
    }

    /// Returns the number of sources that reported no conflict.
    #[must_use]
    pub fn available_count(&self) -> usize {
        self.results
            .iter()
            .filter(|result| result.status == AvailabilityStatus::Available)
            .count()
    }

    /// Returns the percentage of requested sources with positive evidence.
    ///
    /// This score measures evidence coverage, not legal or trademark safety.
    #[must_use]
    pub fn evidence_score(&self) -> u8 {
        let total = self.results.len();
        if total == 0 {
            return 0;
        }
        u8::try_from((self.available_count() * 100) / total).unwrap_or(100)
    }

    /// Returns the conservative recommendation for this report.
    #[must_use]
    pub fn recommendation(&self) -> ClearanceRecommendation {
        if self.taken_count() > 0 {
            ClearanceRecommendation::Reject
        } else if self.is_clear() {
            ClearanceRecommendation::Clear
        } else {
            ClearanceRecommendation::Provisional
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AvailabilityTarget;

    fn result(status: AvailabilityStatus) -> AvailabilityResult {
        AvailabilityResult::new(AvailabilityTarget::GitHub, "candidate", status, "fixture")
    }

    #[test]
    fn complete_positive_evidence_is_clear() {
        let report =
            AvailabilityReport::new("candidate", vec![result(AvailabilityStatus::Available)]);
        assert_eq!(report.evidence_score(), 100);
        assert_eq!(report.recommendation(), ClearanceRecommendation::Clear);
    }

    #[test]
    fn missing_evidence_is_provisional_not_clear() {
        let report = AvailabilityReport::new(
            "candidate",
            vec![
                result(AvailabilityStatus::Available),
                result(AvailabilityStatus::Unknown),
            ],
        );
        assert_eq!(report.unknown_count(), 1);
        assert_eq!(report.evidence_score(), 50);
        assert_eq!(
            report.recommendation(),
            ClearanceRecommendation::Provisional
        );
    }

    #[test]
    fn a_conflict_rejects_candidate() {
        let report = AvailabilityReport::new(
            "candidate",
            vec![
                result(AvailabilityStatus::Available),
                result(AvailabilityStatus::Taken),
            ],
        );
        assert_eq!(report.recommendation(), ClearanceRecommendation::Reject);
    }
}
