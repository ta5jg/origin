//! Core generation, phonotactic analysis and scoring primitives for ORIGIN.

mod analysis;
mod availability;
mod candidate;
mod design;
mod evolution;
mod fuzzy;
mod language;
mod morphology;
mod phonotactics;
mod portfolio;
mod rejection;
mod semantic;
mod similarity;
mod trademark;

use serde::Serialize;

pub use candidate::{
    CANDIDATE_SCHEMA_VERSION, CandidateInvariantError, CandidateRecord, CandidateScores,
    GenerationMode, RejectionCode, RejectionReason, RootLanguage, RootProvenance,
    ValidationEvidence, ValidationStatus,
};
pub use design::{
    DesignOptions, DesignStrategy, DesignedCandidate, MAX_DESIGN_CANDIDATES, design_brands,
};

pub use language::{
    ConfidenceBasis, LANGUAGE_CATALOG_SCHEMA_VERSION, LANGUAGE_ROOT_SCHEMA_VERSION,
    LanguageCatalog, LanguageCatalogError, LanguageId, LanguageRoot, LanguageRootError,
    MeaningCategory, MeaningInterpretation, RootConfidence, RootMeaning, RootQuery, RootSource,
    SourceKind, Transliteration,
};

pub use morphology::{
    CollapseChange, CollapseError, CollapseKind, CollapsePolicy, CollapseReport,
    DEFAULT_MAXIMUM_MUTATION_LENGTH, DEFAULT_MINIMUM_MUTATION_LENGTH, MAX_BOUNDARY_OVERLAP,
    MergeError, MergePolicy, MergeReport, MergeStage, MorphMutationKind, MutationChange,
    MutationError, MutationPolicy, MutationReport, NormalizationChange, NormalizationChangeKind,
    NormalizationError, NormalizationMode, NormalizationPolicy, NormalizationReport,
    collapse_roots, collapse_roots_with_policy, is_canonical_root, merge_roots,
    merge_roots_with_policy, mutate_root, mutate_root_with_kind, mutate_root_with_policy,
    normalize_root, normalize_root_with_policy,
};

pub use analysis::{
    BrandReport, INTERNATIONAL_TECH_V1, LanguageProfile, ScoreBreakdown, analyze_brand,
    analyze_brand_with_policy, analyze_brand_with_profile,
};
pub use availability::{
    AvailabilityCache, AvailabilityChecker, AvailabilityError, AvailabilityProvider,
    AvailabilityReport, AvailabilityResult, AvailabilityStatus, AvailabilityTarget, CacheKey,
    ClearanceRecommendation, MockAvailabilityProvider,
};
pub use evolution::{
    BeamCandidate, BeamSearchOptions, BeamSearchReport, ImproveOptions, ImprovementCandidate,
    ImprovementReport, MutationKind, MutationStep, beam_search, improve,
};
pub use fuzzy::{FuzzyInputs, FuzzyReport, LinguisticQuality, Membership, evaluate_fuzzy};
pub use phonotactics::{PhonotacticReport, analyze_name};
pub use portfolio::{
    PortfolioCandidate, PortfolioConflict, PortfolioOptions, PortfolioReference, PortfolioReport,
    build_portfolio, famous_mark_context,
};
pub use rejection::{RejectReason, RejectionPolicy, RejectionResult, evaluate_rejection};
pub use semantic::{
    SemanticComposition, SemanticError, built_in_catalog, compose, compose_builtin,
};
pub use similarity::{
    Analyzer, SimilarityAnalyzer, SimilarityReport, SimilarityRisk, SimilarityWeights,
    analyze_similarity, analyze_similarity_with_weights,
};
pub use trademark::{
    MarkStrength, TrademarkAnalyzer, TrademarkContext, TrademarkFactor, TrademarkRecommendation,
    TrademarkReport, TrademarkRisk, analyze_trademark, analyze_trademark_risk,
};

const ONSETS: &[u8; 20] = b"bdfgklmnprstvwxyzchj";
const VOWELS: &[u8; 5] = b"aeiou";
const SYLLABLE_RADIX: usize = ONSETS.len() * VOWELS.len();
const MAX_CANDIDATES_U64: u64 = 1_000_000;

/// Maximum number of unique three-syllable candidates in the current model.
pub const MAX_CANDIDATES: usize = SYLLABLE_RADIX * SYLLABLE_RADIX * SYLLABLE_RADIX;

/// A generated brand-name candidate and its explainable quality scores.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Candidate {
    /// Candidate text in lowercase ASCII.
    pub name: String,
    /// Weighted overall score from zero to one hundred.
    pub score: u8,
    /// Ease of pronunciation according to the phonotactic engine.
    pub pronounceability: u8,
    /// Regularity of vowel and consonant alternation.
    pub rhythm: u8,
    /// Balance between vowels and consonants.
    pub vowel_balance: u8,
    /// Resistance to mechanical letter and bigram repetition.
    pub repetition: u8,
    /// Smoothness of adjacent sound-class transitions.
    pub transition_quality: u8,
    /// Whether the candidate passes the active profile threshold.
    pub accepted: bool,
}

/// Configuration for deterministic candidate generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GenerateOptions {
    /// Maximum number of unique candidates to return.
    pub count: usize,
    /// Seed used to choose a deterministic traversal through the name space.
    pub seed: u64,
}

impl Default for GenerateOptions {
    fn default() -> Self {
        Self {
            count: 100,
            seed: 1,
        }
    }
}

/// Generates unique candidates using deterministic phoneme composition.
///
/// The current model uses three fixed-width consonant-vowel syllables. This
/// creates exactly one million unique six-letter candidates while preserving
/// reproducibility and constant-time candidate construction.
///
/// Requests larger than [`MAX_CANDIDATES`] are capped at that limit.
#[must_use]
pub fn generate(options: GenerateOptions) -> Vec<Candidate> {
    let count = options.count.min(MAX_CANDIDATES);
    let start = seed_start(options.seed);
    let step = seed_step(options.seed);

    let mut candidates = Vec::with_capacity(count);
    for offset in 0..count {
        let offset = u64::try_from(offset).unwrap_or_default();
        let index = (start + offset * step) % MAX_CANDIDATES_U64;
        let index = usize::try_from(index).unwrap_or_default();
        let name = compose_from_index(index);
        let report = analyze_brand(&name);
        candidates.push(Candidate {
            score: report.overall_score,
            pronounceability: report.scores.pronounceability,
            rhythm: report.scores.rhythm,
            vowel_balance: report.scores.vowel_balance,
            repetition: report.scores.repetition,
            transition_quality: report.scores.transition_quality,
            accepted: report.accepted,
            name,
        });
    }

    candidates.sort_unstable_by(|left, right| {
        right
            .accepted
            .cmp(&left.accepted)
            .then_with(|| right.score.cmp(&left.score))
            .then_with(|| right.repetition.cmp(&left.repetition))
            .then_with(|| left.name.cmp(&right.name))
    });
    candidates
}

fn compose_from_index(mut index: usize) -> String {
    let mut bytes = [0_u8; 6];

    for syllable_position in (0..3).rev() {
        let syllable = index % SYLLABLE_RADIX;
        index /= SYLLABLE_RADIX;

        let onset = ONSETS[syllable / VOWELS.len()];
        let vowel = VOWELS[syllable % VOWELS.len()];
        let byte_position = syllable_position * 2;
        bytes[byte_position] = onset;
        bytes[byte_position + 1] = vowel;
    }

    String::from_utf8(bytes.to_vec()).expect("the phoneme table contains ASCII only")
}

fn seed_start(seed: u64) -> u64 {
    mix(seed) % MAX_CANDIDATES_U64
}

#[allow(clippy::manual_is_multiple_of)]
fn seed_step(seed: u64) -> u64 {
    let mut step = (mix(seed ^ 0xA5A5_A5A5_A5A5_A5A5) % MAX_CANDIDATES_U64).max(1);

    while step % 2 == 0 || step % 5 == 0 {
        step += 1;
        if step >= MAX_CANDIDATES_U64 {
            step = 1;
        }
    }

    step
}

const fn mix(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::{GenerateOptions, MAX_CANDIDATES, generate};
    use std::collections::HashSet;

    #[test]
    fn generation_is_deterministic() {
        let options = GenerateOptions {
            count: 25,
            seed: 42,
        };
        assert_eq!(generate(options), generate(options));
    }

    #[test]
    fn different_seeds_change_the_traversal() {
        assert_ne!(
            generate(GenerateOptions { count: 25, seed: 1 }),
            generate(GenerateOptions { count: 25, seed: 2 })
        );
    }

    #[test]
    fn generation_returns_requested_unique_names() {
        let candidates = generate(GenerateOptions {
            count: 10_000,
            seed: 7,
        });
        let unique = candidates
            .iter()
            .map(|candidate| candidate.name.as_str())
            .collect::<HashSet<_>>();

        assert_eq!(candidates.len(), 10_000);
        assert_eq!(candidates.len(), unique.len());
        assert!(candidates.iter().all(|candidate| candidate.name.len() == 6
            && candidate.name.is_ascii()
            && candidate.score <= 100
            && candidate.pronounceability <= 100
            && candidate.rhythm <= 100
            && candidate.vowel_balance <= 100
            && candidate.repetition <= 100
            && candidate.transition_quality <= 100));
    }

    #[test]
    fn accepted_candidates_are_ranked_before_rejected_candidates() {
        let candidates = generate(GenerateOptions {
            count: 1_000,
            seed: 42,
        });
        let first_rejected = candidates.iter().position(|candidate| !candidate.accepted);

        if let Some(index) = first_rejected {
            assert!(
                candidates[index..]
                    .iter()
                    .all(|candidate| !candidate.accepted)
            );
        }
    }

    #[test]
    fn current_model_exposes_one_million_candidates() {
        assert_eq!(MAX_CANDIDATES, 1_000_000);
    }
}
