//! Offline-first availability screening contracts and orchestration.

mod cache;
mod checker;
mod errors;
mod mock_provider;
mod report;
mod traits;
mod types;

pub use cache::{AvailabilityCache, CacheKey};
pub use checker::AvailabilityChecker;
pub use errors::AvailabilityError;
pub use mock_provider::MockAvailabilityProvider;
pub use report::AvailabilityReport;
pub use traits::AvailabilityProvider;
pub use types::{AvailabilityResult, AvailabilityStatus, AvailabilityTarget};

#[cfg(test)]
mod tests {
    use super::{
        AvailabilityChecker, AvailabilityError, AvailabilityProvider, AvailabilityResult,
        AvailabilityStatus, AvailabilityTarget, MockAvailabilityProvider,
    };

    #[test]
    fn checker_caches_normalized_duplicate_requests() {
        let provider = MockAvailabilityProvider::new().with_result(AvailabilityResult::new(
            AvailabilityTarget::GitHub,
            "qarvan",
            AvailabilityStatus::Available,
            "fixture",
        ));
        let checker = AvailabilityChecker::new(provider);

        let first = checker
            .check(" Qarvan ", &AvailabilityTarget::GitHub)
            .expect("fixture response should resolve");
        let second = checker
            .check("qarvan", &AvailabilityTarget::GitHub)
            .expect("cached response should resolve");

        assert_eq!(first, second);
        assert_eq!(checker.cache_len(), 1);
    }

    #[test]
    fn checker_builds_a_report_in_requested_order() {
        let checker = AvailabilityChecker::new(MockAvailabilityProvider::new());
        let report = checker
            .check_all(
                "qarvan",
                [
                    AvailabilityTarget::Npm,
                    AvailabilityTarget::Domain { tld: "com".into() },
                ],
            )
            .expect("offline responses should resolve");

        assert_eq!(report.name, "qarvan");
        assert_eq!(report.results.len(), 2);
        assert_eq!(report.results[0].target, AvailabilityTarget::Npm);
        assert_eq!(report.results[1].target.code(), "domain.com");
        assert!(!report.is_clear());
    }

    #[test]
    fn checker_rejects_blank_names_without_calling_provider() {
        let checker = AvailabilityChecker::new(MockAvailabilityProvider::new());

        assert_eq!(
            checker.check(" \t", &AvailabilityTarget::GitHub),
            Err(AvailabilityError::EmptyName)
        );
        assert_eq!(checker.cache_len(), 0);
    }

    #[test]
    fn provider_convenience_methods_route_to_expected_targets() {
        let provider = MockAvailabilityProvider::new();

        assert_eq!(
            provider.github("qarvan").expect("github mock").target,
            AvailabilityTarget::GitHub
        );
        assert_eq!(
            provider.crates_io("qarvan").expect("crates mock").target,
            AvailabilityTarget::CratesIo
        );
        assert_eq!(
            provider.npm("qarvan").expect("npm mock").target,
            AvailabilityTarget::Npm
        );
        assert_eq!(
            provider.pypi("qarvan").expect("pypi mock").target,
            AvailabilityTarget::PyPi
        );
        assert_eq!(
            provider
                .domain("qarvan", "COM")
                .expect("domain mock")
                .target,
            AvailabilityTarget::Domain { tld: "com".into() }
        );
    }

    #[test]
    fn report_is_clear_only_when_every_target_is_available() {
        let provider = MockAvailabilityProvider::new()
            .with_result(AvailabilityResult::new(
                AvailabilityTarget::GitHub,
                "qarvan",
                AvailabilityStatus::Available,
                "fixture",
            ))
            .with_result(AvailabilityResult::new(
                AvailabilityTarget::Npm,
                "qarvan",
                AvailabilityStatus::Available,
                "fixture",
            ));
        let report = AvailabilityChecker::new(provider)
            .check_all(
                "qarvan",
                [AvailabilityTarget::GitHub, AvailabilityTarget::Npm],
            )
            .expect("fixture responses should resolve");

        assert!(report.is_clear());
        assert_eq!(report.taken_count(), 0);
    }

    #[test]
    fn report_counts_taken_results() {
        let provider = MockAvailabilityProvider::new().with_result(AvailabilityResult::new(
            AvailabilityTarget::Npm,
            "qarvan",
            AvailabilityStatus::Taken,
            "fixture",
        ));
        let report = AvailabilityChecker::new(provider)
            .check_all(
                "qarvan",
                [AvailabilityTarget::GitHub, AvailabilityTarget::Npm],
            )
            .expect("fixture responses should resolve");

        assert_eq!(report.taken_count(), 1);
        assert!(!report.is_clear());
    }
}
