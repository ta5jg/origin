//! Deterministic trademark-conflict risk assessment built on name similarity.
//!
//! This module provides an explainable screening signal. It is not a legal
//! clearance opinion and does not replace jurisdiction-specific trademark
//! searches or advice from a qualified professional.

use serde::Serialize;

use crate::similarity::{SimilarityReport, SimilarityRisk, analyze_similarity};

/// Commercial strength of the earlier mark being compared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkStrength {
    /// Highly descriptive or otherwise commercially weak.
    Weak,
    /// Ordinary distinctiveness and market recognition.
    Average,
    /// Inherently distinctive or commercially well established.
    Strong,
    /// Famous or exceptionally well-known mark.
    Famous,
}

/// Context used to translate name resemblance into conflict risk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TrademarkContext {
    /// Whether the names are intended for the same broad industry.
    pub same_industry: bool,
    /// Whether the goods or services directly overlap.
    pub overlapping_goods: bool,
    /// Whether both marks target the same geographic or customer market.
    pub same_market: bool,
    /// Estimated strength of the earlier/reference mark.
    pub prior_mark_strength: MarkStrength,
}

impl Default for TrademarkContext {
    fn default() -> Self {
        Self {
            same_industry: true,
            overlapping_goods: true,
            same_market: true,
            prior_mark_strength: MarkStrength::Average,
        }
    }
}

/// Overall screening classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TrademarkRisk {
    /// The supplied names could not be assessed reliably.
    Inconclusive,
    /// No material conflict signal was detected by this screening model.
    Minimal,
    /// Some resemblance exists, but contextual conflict indicators are weak.
    Low,
    /// Material resemblance or market overlap warrants manual review.
    Moderate,
    /// Strong conflict indicators make clearance doubtful without deeper review.
    High,
    /// Exact or near-exact resemblance combined with strong commercial overlap.
    Critical,
}

/// Recommended next action for a candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TrademarkRecommendation {
    /// Correct the inputs and repeat the assessment.
    SupplyValidNames,
    /// Continue normal brand validation and registry searching.
    ProceedToSearch,
    /// Perform an expanded registry, marketplace, domain, and common-law search.
    ConductEnhancedSearch,
    /// Obtain professional clearance before adoption or filing.
    SeekLegalReview,
    /// Do not proceed with this candidate without a compelling, reviewed reason.
    AvoidCandidate,
}

/// One explainable factor contributing to the result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TrademarkFactor {
    /// Stable machine-readable factor identifier.
    pub code: String,
    /// Signed contribution to the final risk score.
    pub impact: i8,
    /// Human-readable explanation.
    pub explanation: String,
}

/// Complete deterministic trademark-risk report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TrademarkReport {
    /// Pairwise name-similarity evidence used by the assessment.
    pub similarity: SimilarityReport,
    /// Caller-provided commercial context.
    pub context: TrademarkContext,
    /// Aggregate trademark screening score from zero to one hundred.
    pub risk_score: u8,
    /// Classification derived from the aggregate score and override rules.
    pub risk: TrademarkRisk,
    /// Recommended next action.
    pub recommendation: TrademarkRecommendation,
    /// Whether automated screening considers the candidate provisionally clear.
    pub provisionally_clear: bool,
    /// Ordered, explainable scoring factors.
    pub factors: Vec<TrademarkFactor>,
    /// Important limitations and cautions.
    pub warnings: Vec<String>,
}

/// Stateless trademark-risk analyzer using a fixed context.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TrademarkAnalyzer {
    context: TrademarkContext,
}

impl TrademarkAnalyzer {
    /// Creates an analyzer with caller-provided commercial context.
    #[must_use]
    pub const fn with_context(context: TrademarkContext) -> Self {
        Self { context }
    }

    /// Analyzes one candidate against an earlier/reference mark.
    #[must_use]
    pub fn analyze(self, candidate: &str, reference: &str) -> TrademarkReport {
        analyze_trademark_risk(candidate, reference, self.context)
    }
}

/// Screens two names using a conservative same-market default context.
#[must_use]
pub fn analyze_trademark(candidate: &str, reference: &str) -> TrademarkReport {
    TrademarkAnalyzer::default().analyze(candidate, reference)
}

/// Screens two names with explicit commercial context.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn analyze_trademark_risk(
    candidate: &str,
    reference: &str,
    context: TrademarkContext,
) -> TrademarkReport {
    let similarity = analyze_similarity(candidate, reference);
    let valid = !similarity.candidate.is_empty() && !similarity.reference.is_empty();

    if !valid {
        return TrademarkReport {
            similarity,
            context,
            risk_score: 0,
            risk: TrademarkRisk::Inconclusive,
            recommendation: TrademarkRecommendation::SupplyValidNames,
            provisionally_clear: false,
            factors: Vec::new(),
            warnings: vec![String::from(
                "both names must contain at least one alphanumeric character",
            )],
        };
    }

    let mut factors = Vec::new();
    // Reserve forty points for commercial context so stronger prior marks
    // remain distinguishable instead of disappearing through score saturation.
    let similarity_component = u16::from(similarity.overall_similarity).saturating_mul(60) / 100;
    let mut score = i16::try_from(similarity_component).unwrap_or(60);

    push_factor(
        &mut factors,
        "name_similarity",
        0,
        format!(
            "the similarity engine produced an overall resemblance score of {}",
            similarity.overall_similarity
        ),
    );

    score += apply_bool_factor(
        &mut factors,
        context.same_industry,
        "same_industry",
        8,
        "the marks operate in the same broad industry",
        "different industries reduce the immediate conflict signal",
        -6,
    );
    score += apply_bool_factor(
        &mut factors,
        context.overlapping_goods,
        "overlapping_goods",
        12,
        "the goods or services directly overlap",
        "non-overlapping goods or services reduce likely confusion",
        -10,
    );
    score += apply_bool_factor(
        &mut factors,
        context.same_market,
        "same_market",
        7,
        "the marks target the same market or customer group",
        "different markets reduce likely customer exposure",
        -5,
    );

    let strength_impact = match context.prior_mark_strength {
        MarkStrength::Weak => -6,
        MarkStrength::Average => 0,
        MarkStrength::Strong => 8,
        MarkStrength::Famous => 15,
    };
    push_factor(
        &mut factors,
        "prior_mark_strength",
        strength_impact,
        match context.prior_mark_strength {
            MarkStrength::Weak => "a weak earlier mark receives a smaller contextual uplift",
            MarkStrength::Average => "the earlier mark has ordinary assumed strength",
            MarkStrength::Strong => "a strong earlier mark increases conflict exposure",
            MarkStrength::Famous => "a famous earlier mark materially increases conflict exposure",
        },
    );
    score += i16::from(strength_impact);

    let exact_match = similarity.candidate == similarity.reference;
    // A single insertion, deletion, substitution, or transposition in a
    // short brand name commonly produces a Damerau score near eighty.
    let near_exact = similarity.damerau_score >= 80;
    let strong_phonetic = similarity.phonetic_similarity >= 90;
    let strong_visual = similarity.visual_similarity >= 90;

    if near_exact && context.overlapping_goods {
        score += 8;
        push_factor(
            &mut factors,
            "near_exact_overlap",
            8,
            "near-exact spelling combined with overlapping goods raises confusion risk",
        );
    }
    if strong_phonetic && similarity.overall_similarity < 80 {
        score += 6;
        push_factor(
            &mut factors,
            "phonetic_override",
            6,
            "very strong phonetic resemblance raises risk beyond the aggregate score",
        );
    }
    if strong_visual && similarity.overall_similarity < 80 {
        score += 4;
        push_factor(
            &mut factors,
            "visual_override",
            4,
            "very strong visual resemblance raises risk beyond the aggregate score",
        );
    }

    let risk_score = u8::try_from(score.clamp(0, 100)).unwrap_or_default();
    let risk = classify(
        risk_score,
        exact_match,
        near_exact,
        context.overlapping_goods,
        context.same_market,
        context.prior_mark_strength,
    );
    let recommendation = recommendation_for(risk);
    let provisionally_clear = matches!(risk, TrademarkRisk::Minimal | TrademarkRisk::Low);
    let warnings = build_warnings(&similarity, context, risk);

    TrademarkReport {
        similarity,
        context,
        risk_score,
        risk,
        recommendation,
        provisionally_clear,
        factors,
        warnings,
    }
}

fn apply_bool_factor(
    factors: &mut Vec<TrademarkFactor>,
    condition: bool,
    code: &str,
    positive_impact: i8,
    positive_explanation: &str,
    negative_explanation: &str,
    negative_impact: i8,
) -> i16 {
    let (impact, explanation) = if condition {
        (positive_impact, positive_explanation)
    } else {
        (negative_impact, negative_explanation)
    };
    push_factor(factors, code, impact, explanation);
    i16::from(impact)
}

fn push_factor(
    factors: &mut Vec<TrademarkFactor>,
    code: &str,
    impact: i8,
    explanation: impl Into<String>,
) {
    factors.push(TrademarkFactor {
        code: String::from(code),
        impact,
        explanation: explanation.into(),
    });
}

#[allow(clippy::fn_params_excessive_bools)]
fn classify(
    score: u8,
    exact_match: bool,
    near_exact: bool,
    overlapping_goods: bool,
    same_market: bool,
    strength: MarkStrength,
) -> TrademarkRisk {
    if exact_match && (overlapping_goods || same_market) {
        return TrademarkRisk::Critical;
    }
    if near_exact
        && overlapping_goods
        && matches!(strength, MarkStrength::Strong | MarkStrength::Famous)
    {
        return TrademarkRisk::Critical;
    }

    match score {
        0..=24 => TrademarkRisk::Minimal,
        25..=44 => TrademarkRisk::Low,
        45..=64 => TrademarkRisk::Moderate,
        65..=84 => TrademarkRisk::High,
        _ => TrademarkRisk::Critical,
    }
}

const fn recommendation_for(risk: TrademarkRisk) -> TrademarkRecommendation {
    match risk {
        TrademarkRisk::Inconclusive => TrademarkRecommendation::SupplyValidNames,
        TrademarkRisk::Minimal | TrademarkRisk::Low => TrademarkRecommendation::ProceedToSearch,
        TrademarkRisk::Moderate => TrademarkRecommendation::ConductEnhancedSearch,
        TrademarkRisk::High => TrademarkRecommendation::SeekLegalReview,
        TrademarkRisk::Critical => TrademarkRecommendation::AvoidCandidate,
    }
}

fn build_warnings(
    similarity: &SimilarityReport,
    context: TrademarkContext,
    risk: TrademarkRisk,
) -> Vec<String> {
    let mut warnings = vec![String::from(
        "automated screening is not a substitute for official registry and common-law searches",
    )];

    if similarity.risk >= SimilarityRisk::High {
        warnings.push(String::from(
            "the names have a strong resemblance independent of commercial context",
        ));
    }
    if context.overlapping_goods {
        warnings.push(String::from(
            "overlapping goods or services materially increase likely-confusion exposure",
        ));
    }
    if matches!(context.prior_mark_strength, MarkStrength::Famous) {
        warnings.push(String::from(
            "famous marks may receive broader protection than ordinary marks",
        ));
    }
    if matches!(risk, TrademarkRisk::High | TrademarkRisk::Critical) {
        warnings.push(String::from(
            "do not adopt or file this candidate without jurisdiction-specific professional review",
        ));
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::{
        MarkStrength, TrademarkAnalyzer, TrademarkContext, TrademarkRecommendation, TrademarkRisk,
        analyze_trademark, analyze_trademark_risk,
    };

    #[test]
    fn exact_match_in_same_market_is_critical() {
        let report = analyze_trademark("Nova", "nova");

        assert_eq!(report.risk, TrademarkRisk::Critical);
        assert_eq!(
            report.recommendation,
            TrademarkRecommendation::AvoidCandidate
        );
        assert!(!report.provisionally_clear);
    }

    #[test]
    fn empty_input_is_inconclusive() {
        let report = analyze_trademark("---", "Nova");

        assert_eq!(report.risk, TrademarkRisk::Inconclusive);
        assert_eq!(
            report.recommendation,
            TrademarkRecommendation::SupplyValidNames
        );
        assert_eq!(report.risk_score, 0);
    }

    #[test]
    fn distant_names_can_be_provisionally_clear() {
        let context = TrademarkContext {
            same_industry: false,
            overlapping_goods: false,
            same_market: false,
            prior_mark_strength: MarkStrength::Weak,
        };
        let report = analyze_trademark_risk("danoti", "xelvar", context);

        assert!(matches!(
            report.risk,
            TrademarkRisk::Minimal | TrademarkRisk::Low
        ));
        assert!(report.provisionally_clear);
    }

    #[test]
    fn commercial_overlap_increases_score() {
        let separated = TrademarkContext {
            same_industry: false,
            overlapping_goods: false,
            same_market: false,
            prior_mark_strength: MarkStrength::Average,
        };
        let overlapping = TrademarkContext::default();

        let lower = analyze_trademark_risk("origin", "orign", separated);
        let higher = analyze_trademark_risk("origin", "orign", overlapping);

        assert!(higher.risk_score > lower.risk_score);
    }

    #[test]
    fn famous_mark_increases_score() {
        let average = TrademarkContext::default();
        let famous = TrademarkContext {
            prior_mark_strength: MarkStrength::Famous,
            ..average
        };

        let average_report = analyze_trademark_risk("kargo", "cargo", average);
        let famous_report = analyze_trademark_risk("kargo", "cargo", famous);

        assert!(famous_report.risk_score > average_report.risk_score);
    }

    #[test]
    fn near_exact_strong_mark_can_be_critical() {
        let context = TrademarkContext {
            prior_mark_strength: MarkStrength::Strong,
            ..TrademarkContext::default()
        };
        let report = analyze_trademark_risk("googel", "google", context);

        assert_eq!(report.risk, TrademarkRisk::Critical);
    }

    #[test]
    fn factors_are_machine_readable_and_explainable() {
        let report = analyze_trademark("origin", "orign");

        assert!(!report.factors.is_empty());
        assert!(
            report
                .factors
                .iter()
                .all(|factor| { !factor.code.is_empty() && !factor.explanation.is_empty() })
        );
    }

    #[test]
    fn analyzer_with_context_matches_free_function() {
        let context = TrademarkContext {
            same_industry: true,
            overlapping_goods: false,
            same_market: true,
            prior_mark_strength: MarkStrength::Average,
        };
        let analyzer = TrademarkAnalyzer::with_context(context);

        assert_eq!(
            analyzer.analyze("pixel", "pixxel"),
            analyze_trademark_risk("pixel", "pixxel", context)
        );
    }

    #[test]
    fn screening_is_deterministic() {
        let context = TrademarkContext::default();
        assert_eq!(
            analyze_trademark_risk("spotify", "spotifai", context),
            analyze_trademark_risk("spotify", "spotifai", context)
        );
    }

    #[test]
    fn high_risk_reports_include_professional_review_warning() {
        let report = analyze_trademark("apple", "appl");

        if matches!(report.risk, TrademarkRisk::High | TrademarkRisk::Critical) {
            assert!(
                report
                    .warnings
                    .iter()
                    .any(|warning| { warning.contains("professional review") })
            );
        }
    }
}
