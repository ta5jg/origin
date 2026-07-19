//! Deterministic portfolio generation and conflict-aware ranking.

use serde::Serialize;

use crate::{
    Candidate, GenerateOptions, MarkStrength, TrademarkContext, TrademarkReport, TrademarkRisk,
    analyze_trademark_risk, generate,
};

/// One earlier mark used to screen generated candidates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PortfolioReference {
    /// Earlier or competing brand name.
    pub name: String,
    /// Commercial context used for the comparison.
    pub context: TrademarkContext,
}

impl PortfolioReference {
    /// Creates a conservative same-market reference.
    #[must_use]
    pub fn conservative(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            context: TrademarkContext::default(),
        }
    }

    /// Creates a reference with explicit commercial context.
    #[must_use]
    pub fn with_context(name: impl Into<String>, context: TrademarkContext) -> Self {
        Self {
            name: name.into(),
            context,
        }
    }
}

/// Configuration for deterministic portfolio construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortfolioOptions {
    /// Number of raw generated candidates to inspect.
    pub candidate_count: usize,
    /// Maximum number of ranked candidates to return.
    pub result_count: usize,
    /// Seed used by the deterministic generator.
    pub seed: u64,
    /// Highest trademark risk allowed in the final portfolio.
    pub maximum_risk: TrademarkRisk,
    /// Whether candidates rejected by linguistic analysis may be returned.
    pub include_rejected: bool,
}

impl Default for PortfolioOptions {
    fn default() -> Self {
        Self {
            candidate_count: 1_000,
            result_count: 25,
            seed: 1,
            maximum_risk: TrademarkRisk::Low,
            include_rejected: false,
        }
    }
}

/// Conflict evidence for one candidate/reference pair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PortfolioConflict {
    /// Reference name that produced this result.
    pub reference: String,
    /// Full explainable trademark report.
    pub report: TrademarkReport,
}

/// One ranked candidate in a portfolio.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PortfolioCandidate {
    /// Generated candidate and linguistic score breakdown.
    pub candidate: Candidate,
    /// Highest trademark risk found across all references.
    pub maximum_risk: TrademarkRisk,
    /// Highest numeric trademark-risk score across all references.
    pub maximum_risk_score: u8,
    /// Reference comparisons ordered from highest to lowest risk.
    pub conflicts: Vec<PortfolioConflict>,
    /// Stable human-readable ranking explanations.
    pub explanations: Vec<String>,
}

/// Complete deterministic portfolio result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PortfolioReport {
    /// Number of raw candidates inspected.
    pub inspected: usize,
    /// Number removed by linguistic acceptance rules.
    pub rejected_by_quality: usize,
    /// Number removed by trademark-risk limits.
    pub rejected_by_risk: usize,
    /// Ranked surviving candidates.
    pub results: Vec<PortfolioCandidate>,
}

/// Generates, screens, and ranks a portfolio of candidate brand names.
#[must_use]
pub fn build_portfolio(
    options: PortfolioOptions,
    references: &[PortfolioReference],
) -> PortfolioReport {
    let generated = generate(GenerateOptions {
        count: options.candidate_count,
        seed: options.seed,
    });
    let inspected = generated.len();
    let mut rejected_by_quality = 0;
    let mut rejected_by_risk = 0;
    let mut results = Vec::new();

    for candidate in generated {
        if !options.include_rejected && !candidate.accepted {
            rejected_by_quality += 1;
            continue;
        }

        let mut conflicts = references
            .iter()
            .map(|reference| PortfolioConflict {
                reference: reference.name.clone(),
                report: analyze_trademark_risk(&candidate.name, &reference.name, reference.context),
            })
            .collect::<Vec<_>>();

        conflicts.sort_unstable_by(|left, right| {
            right
                .report
                .risk
                .cmp(&left.report.risk)
                .then_with(|| right.report.risk_score.cmp(&left.report.risk_score))
                .then_with(|| left.reference.cmp(&right.reference))
        });

        let maximum_risk = conflicts
            .first()
            .map_or(TrademarkRisk::Minimal, |conflict| conflict.report.risk);
        let maximum_risk_score = conflicts
            .first()
            .map_or(0, |conflict| conflict.report.risk_score);

        if maximum_risk > options.maximum_risk {
            rejected_by_risk += 1;
            continue;
        }

        let mut explanations = vec![format!(
            "linguistic quality score is {} and acceptance is {}",
            candidate.score, candidate.accepted
        )];
        if references.is_empty() {
            explanations.push(String::from(
                "no trademark references were supplied; conflict screening was not performed",
            ));
        } else {
            explanations.push(format!(
                "highest trademark screening result is {maximum_risk:?} with score {maximum_risk_score}"
            ));
        }

        results.push(PortfolioCandidate {
            candidate,
            maximum_risk,
            maximum_risk_score,
            conflicts,
            explanations,
        });
    }

    results.sort_unstable_by(|left, right| {
        left.maximum_risk
            .cmp(&right.maximum_risk)
            .then_with(|| left.maximum_risk_score.cmp(&right.maximum_risk_score))
            .then_with(|| right.candidate.accepted.cmp(&left.candidate.accepted))
            .then_with(|| right.candidate.score.cmp(&left.candidate.score))
            .then_with(|| right.candidate.repetition.cmp(&left.candidate.repetition))
            .then_with(|| left.candidate.name.cmp(&right.candidate.name))
    });
    results.truncate(options.result_count);

    PortfolioReport {
        inspected,
        rejected_by_quality,
        rejected_by_risk,
        results,
    }
}

/// Convenience context for a famous mark in the same market and goods class.
#[must_use]
pub const fn famous_mark_context() -> TrademarkContext {
    TrademarkContext {
        same_industry: true,
        overlapping_goods: true,
        same_market: true,
        prior_mark_strength: MarkStrength::Famous,
    }
}

#[cfg(test)]
mod tests {
    use super::{PortfolioOptions, PortfolioReference, build_portfolio, famous_mark_context};
    use crate::TrademarkRisk;

    #[test]
    fn portfolio_generation_is_deterministic() {
        let options = PortfolioOptions {
            candidate_count: 250,
            result_count: 20,
            seed: 42,
            maximum_risk: TrademarkRisk::Moderate,
            include_rejected: false,
        };
        let references = [PortfolioReference::conservative("origin")];

        assert_eq!(
            build_portfolio(options, &references),
            build_portfolio(options, &references)
        );
    }

    #[test]
    fn result_limit_is_enforced() {
        let report = build_portfolio(
            PortfolioOptions {
                candidate_count: 500,
                result_count: 7,
                ..PortfolioOptions::default()
            },
            &[],
        );

        assert_eq!(report.inspected, 500);
        assert!(report.results.len() <= 7);
    }

    #[test]
    fn returned_candidates_respect_risk_limit() {
        let references = [PortfolioReference::with_context(
            "danoti",
            famous_mark_context(),
        )];
        let options = PortfolioOptions {
            candidate_count: 1_000,
            result_count: 50,
            seed: 7,
            maximum_risk: TrademarkRisk::Low,
            include_rejected: true,
        };
        let report = build_portfolio(options, &references);

        assert!(
            report
                .results
                .iter()
                .all(|candidate| candidate.maximum_risk <= TrademarkRisk::Low)
        );
    }

    #[test]
    fn empty_reference_set_is_reported_explainably() {
        let report = build_portfolio(
            PortfolioOptions {
                candidate_count: 20,
                result_count: 5,
                include_rejected: true,
                ..PortfolioOptions::default()
            },
            &[],
        );

        assert!(report.results.iter().all(|candidate| {
            candidate
                .explanations
                .iter()
                .any(|explanation| explanation.contains("no trademark references"))
        }));
    }
}
