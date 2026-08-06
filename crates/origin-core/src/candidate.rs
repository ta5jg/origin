/* =============================================================================
 * File:           crates/origin-core/src/candidate.rs
 * Project:        Origin
 * Author:         USDTG GROUP TECHNOLOGY LLC
 * Developer:      Irfan Gedik
 * Created Date:   2026-08-06
 * Version:        0.1.0
 *
 * Description:
 *   Defines the canonical candidate, provenance, score, rejection, and
 *   validation data models used by Origin naming campaigns.
 *
 * License:
 *   Origin License v1.0 — see LICENSE in the repository root.
 * ============================================================================= */

//! Canonical candidate models for the Origin brand discovery engine.
//!
//! This module defines the stable data structures exchanged between candidate
//! generation, morphology, scoring, selection, validation, and export stages.
//! It intentionally contains no generation or scoring algorithms.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Current schema version of serialized candidate records.
pub const CANDIDATE_SCHEMA_VERSION: u16 = 1;

/// Identifies the strategy that produced a candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GenerationMode {
    /// Candidate created entirely from synthetic phonemes or morphemes.
    Synthetic,

    /// Candidate derived from one historical-language root.
    Ancient,

    /// Candidate produced by combining transformed roots from multiple sources.
    Hybrid,

    /// Candidate produced by mutating an existing name or root.
    Mutation,
}

impl GenerationMode {
    /// Returns the stable machine-readable identifier for this mode.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Synthetic => "synthetic",
            Self::Ancient => "ancient",
            Self::Hybrid => "hybrid",
            Self::Mutation => "mutation",
        }
    }
}

impl std::fmt::Display for GenerationMode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Historical or synthetic language family associated with a source root.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RootLanguage {
    /// Sumerian or Sumerian-inspired source material.
    Sumerian,

    /// Akkadian or Akkadian-inspired source material.
    Akkadian,

    /// Old Turkic source material.
    OldTurkic,

    /// Latin source material.
    Latin,

    /// Sanskrit source material.
    Sanskrit,

    /// Artificial morpheme created by Origin.
    Synthetic,

    /// Source language not covered by the current fixed set.
    Other,
}

impl RootLanguage {
    /// Returns the stable machine-readable identifier for this language.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sumerian => "sumerian",
            Self::Akkadian => "akkadian",
            Self::OldTurkic => "old-turkic",
            Self::Latin => "latin",
            Self::Sanskrit => "sanskrit",
            Self::Synthetic => "synthetic",
            Self::Other => "other",
        }
    }
}

impl std::fmt::Display for RootLanguage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Describes one root or morpheme that contributed to a generated candidate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootProvenance {
    /// Language family of the source root.
    pub language: RootLanguage,

    /// Original source form before Origin normalization.
    pub original: String,

    /// Canonical normalized form consumed by the generation engine.
    pub normalized: String,

    /// Short English gloss or semantic description.
    pub meaning: String,

    /// Optional public source identifier, citation, or dataset reference.
    pub source: Option<String>,

    /// Confidence in the linguistic attribution, from zero to one hundred.
    pub confidence: u8,

    /// Ordered transformations applied to this root.
    pub transformations: Vec<String>,
}

impl RootProvenance {
    /// Creates validated root provenance.
    ///
    /// Confidence values greater than one hundred are clamped to one hundred.
    #[must_use]
    pub fn new(
        language: RootLanguage,
        original: impl Into<String>,
        normalized: impl Into<String>,
        meaning: impl Into<String>,
        confidence: u8,
    ) -> Self {
        Self {
            language,
            original: original.into(),
            normalized: normalized.into(),
            meaning: meaning.into(),
            source: None,
            confidence: confidence.min(100),
            transformations: Vec::new(),
        }
    }

    /// Attaches a public source or dataset reference.
    #[must_use]
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Records an applied transformation.
    #[must_use]
    pub fn with_transformation(mut self, transformation: impl Into<String>) -> Self {
        self.transformations.push(transformation.into());
        self
    }
}

/// Explainable component scores assigned to a candidate.
///
/// Every score uses the inclusive range `0..=100`, where a higher value is
/// better. External collision risk is intentionally not included here because
/// it is evidence-based validation rather than an intrinsic name quality.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateScores {
    /// Ease of pronunciation across configured language profiles.
    pub pronunciation: u8,

    /// Likelihood that listeners can spell the name correctly after hearing it.
    pub spelling: u8,

    /// Syllabic rhythm and adjacent phoneme transition quality.
    pub rhythm: u8,

    /// Wordmark balance and typography suitability.
    pub typography: u8,

    /// Estimated recall friendliness and structural distinctiveness.
    pub memorability: u8,

    /// Suitability of the candidate length.
    pub length: u8,

    /// Resistance to offensive, harmful, or undesirable meanings.
    pub semantic_safety: u8,

    /// Local distinctiveness before external collision research.
    pub intrinsic_uniqueness: u8,

    /// Weighted composite score produced by the active campaign policy.
    pub overall: u8,
}

impl CandidateScores {
    /// Creates a score record and clamps every component to `0..=100`.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        pronunciation: u8,
        spelling: u8,
        rhythm: u8,
        typography: u8,
        memorability: u8,
        length: u8,
        semantic_safety: u8,
        intrinsic_uniqueness: u8,
        overall: u8,
    ) -> Self {
        Self {
            pronunciation: clamp_score(pronunciation),
            spelling: clamp_score(spelling),
            rhythm: clamp_score(rhythm),
            typography: clamp_score(typography),
            memorability: clamp_score(memorability),
            length: clamp_score(length),
            semantic_safety: clamp_score(semantic_safety),
            intrinsic_uniqueness: clamp_score(intrinsic_uniqueness),
            overall: clamp_score(overall),
        }
    }

    /// Returns all score components in stable display order.
    #[must_use]
    pub fn components(self) -> [(&'static str, u8); 9] {
        [
            ("pronunciation", self.pronunciation),
            ("spelling", self.spelling),
            ("rhythm", self.rhythm),
            ("typography", self.typography),
            ("memorability", self.memorability),
            ("length", self.length),
            ("semantic_safety", self.semantic_safety),
            ("intrinsic_uniqueness", self.intrinsic_uniqueness),
            ("overall", self.overall),
        ]
    }
}

/// Stable code identifying why a candidate was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RejectionCode {
    /// Candidate is shorter than the configured minimum.
    TooShort,

    /// Candidate is longer than the configured maximum.
    TooLong,

    /// Candidate contains unsupported characters.
    InvalidCharacters,

    /// Candidate is difficult to pronounce.
    LowPronunciation,

    /// Candidate is difficult to spell from speech.
    LowSpelling,

    /// Candidate has undesirable rhythm or phoneme transitions.
    LowRhythm,

    /// Candidate has poor wordmark or glyph balance.
    LowTypography,

    /// Candidate is insufficiently memorable.
    LowMemorability,

    /// Candidate resembles another candidate too closely.
    NearDuplicate,

    /// Candidate matches a forbidden term or pattern.
    ForbiddenPattern,

    /// Candidate has a potentially harmful meaning.
    NegativeMeaning,

    /// Candidate conflicts with an existing public name or mark.
    ExternalCollision,

    /// Candidate failed a campaign-specific rule.
    CampaignRule,
}

impl RejectionCode {
    /// Returns the stable machine-readable identifier for this rejection.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TooShort => "too_short",
            Self::TooLong => "too_long",
            Self::InvalidCharacters => "invalid_characters",
            Self::LowPronunciation => "low_pronunciation",
            Self::LowSpelling => "low_spelling",
            Self::LowRhythm => "low_rhythm",
            Self::LowTypography => "low_typography",
            Self::LowMemorability => "low_memorability",
            Self::NearDuplicate => "near_duplicate",
            Self::ForbiddenPattern => "forbidden_pattern",
            Self::NegativeMeaning => "negative_meaning",
            Self::ExternalCollision => "external_collision",
            Self::CampaignRule => "campaign_rule",
        }
    }
}

/// One explainable reason that prevented a candidate from advancing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RejectionReason {
    /// Stable machine-readable rejection code.
    pub code: RejectionCode,

    /// Human-readable explanation.
    pub message: String,

    /// Optional measured value that triggered the rule.
    pub observed: Option<i64>,

    /// Optional campaign threshold associated with the rule.
    pub threshold: Option<i64>,
}

impl RejectionReason {
    /// Creates a rejection reason without numeric measurements.
    #[must_use]
    pub fn new(code: RejectionCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            observed: None,
            threshold: None,
        }
    }

    /// Attaches observed and threshold values.
    #[must_use]
    pub const fn with_measurement(mut self, observed: i64, threshold: i64) -> Self {
        self.observed = Some(observed);
        self.threshold = Some(threshold);
        self
    }
}

/// Status returned by one external validation adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ValidationStatus {
    /// Validation has not yet been attempted.
    NotChecked,

    /// No collision was found in the performed search.
    NoCollisionFound,

    /// A possible collision requires review.
    PossibleCollision,

    /// A strong exact or near-exact collision was found.
    CollisionFound,

    /// The source could not be queried or interpreted.
    Inconclusive,

    /// Validation failed because of a transport or internal error.
    Error,
}

impl ValidationStatus {
    /// Returns whether this result should block automatic final selection.
    #[must_use]
    pub const fn blocks_automatic_selection(self) -> bool {
        matches!(
            self,
            Self::PossibleCollision | Self::CollisionFound | Self::Inconclusive | Self::Error
        )
    }
}

/// One attributable result from an external validation source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationEvidence {
    /// Stable source identifier such as `domain:com`, `github`, or `crates.io`.
    pub source: String,

    /// Exact query submitted to the source.
    pub query: String,

    /// Normalized validation outcome.
    pub status: ValidationStatus,

    /// Human-readable finding.
    pub summary: String,

    /// Optional public evidence location.
    pub evidence_url: Option<String>,

    /// Timestamp supplied by the validation runtime.
    ///
    /// The core model deliberately stores an RFC 3339 string rather than
    /// depending on a particular clock or date-time crate.
    pub checked_at: Option<String>,

    /// Confidence in the normalized finding, from zero to one hundred.
    pub confidence: u8,

    /// Optional source-specific metadata in deterministic key order.
    pub metadata: BTreeMap<String, String>,
}

impl ValidationEvidence {
    /// Creates a validation record.
    #[must_use]
    pub fn new(
        source: impl Into<String>,
        query: impl Into<String>,
        status: ValidationStatus,
        summary: impl Into<String>,
        confidence: u8,
    ) -> Self {
        Self {
            source: source.into(),
            query: query.into(),
            status,
            summary: summary.into(),
            evidence_url: None,
            checked_at: None,
            confidence: confidence.min(100),
            metadata: BTreeMap::new(),
        }
    }

    /// Attaches a public evidence URL.
    #[must_use]
    pub fn with_url(mut self, evidence_url: impl Into<String>) -> Self {
        self.evidence_url = Some(evidence_url.into());
        self
    }

    /// Attaches an externally supplied RFC 3339 timestamp.
    #[must_use]
    pub fn checked_at(mut self, checked_at: impl Into<String>) -> Self {
        self.checked_at = Some(checked_at.into());
        self
    }

    /// Adds source-specific metadata.
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Canonical candidate exchanged between Origin pipeline stages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateRecord {
    /// Serialized schema version.
    pub schema_version: u16,

    /// Canonical lowercase ASCII representation used for comparison.
    pub name: String,

    /// Preferred display representation.
    pub display_name: String,

    /// Generation strategy that produced this candidate.
    pub mode: GenerationMode,

    /// Seed used by the deterministic generator.
    pub seed: u64,

    /// Stable generation index within the campaign.
    pub generation_index: u64,

    /// Provenance for roots and morphemes used by the generator.
    pub roots: Vec<RootProvenance>,

    /// Ordered generation and morphology operations.
    pub derivation: Vec<String>,

    /// Explainable intrinsic quality scores.
    pub scores: CandidateScores,

    /// Whether the candidate passed intrinsic campaign filtering.
    pub accepted: bool,

    /// Reasons that prevented intrinsic acceptance.
    pub rejection_reasons: Vec<RejectionReason>,

    /// External validation evidence collected after generation.
    pub validations: Vec<ValidationEvidence>,

    /// Campaign-defined labels, warnings, or review notes.
    pub tags: Vec<String>,

    /// Deterministic extension fields in stable key order.
    pub metadata: BTreeMap<String, String>,
}

impl CandidateRecord {
    /// Creates an unscored candidate record.
    ///
    /// The name is normalized to lowercase ASCII. The display name uses a
    /// capitalized first character and lowercase remaining characters.
    #[must_use]
    pub fn new(
        name: impl AsRef<str>,
        mode: GenerationMode,
        seed: u64,
        generation_index: u64,
    ) -> Self {
        let normalized = normalize_candidate(name.as_ref());

        Self {
            schema_version: CANDIDATE_SCHEMA_VERSION,
            display_name: display_candidate(&normalized),
            name: normalized,
            mode,
            seed,
            generation_index,
            roots: Vec::new(),
            derivation: Vec::new(),
            scores: CandidateScores::default(),
            accepted: false,
            rejection_reasons: Vec::new(),
            validations: Vec::new(),
            tags: Vec::new(),
            metadata: BTreeMap::new(),
        }
    }

    /// Attaches one source root.
    #[must_use]
    pub fn with_root(mut self, root: RootProvenance) -> Self {
        self.roots.push(root);
        self
    }

    /// Records one generation or morphology operation.
    #[must_use]
    pub fn with_derivation(mut self, step: impl Into<String>) -> Self {
        self.derivation.push(step.into());
        self
    }

    /// Assigns the complete intrinsic score record.
    #[must_use]
    pub const fn with_scores(mut self, scores: CandidateScores) -> Self {
        self.scores = scores;
        self
    }

    /// Marks the candidate as accepted when it has no rejection reasons.
    #[must_use]
    pub fn accept(mut self) -> Self {
        self.accepted = self.rejection_reasons.is_empty();
        self
    }

    /// Rejects the candidate and records the reason.
    #[must_use]
    pub fn reject(mut self, reason: RejectionReason) -> Self {
        self.accepted = false;
        self.rejection_reasons.push(reason);
        self
    }

    /// Attaches one external validation result.
    #[must_use]
    pub fn with_validation(mut self, validation: ValidationEvidence) -> Self {
        self.validations.push(validation);
        self
    }

    /// Adds a campaign or review tag if it is not already present.
    #[must_use]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
        self
    }

    /// Adds deterministic extension metadata.
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Returns whether all recorded validations are non-blocking.
    ///
    /// A candidate without external validation is not externally clear.
    #[must_use]
    pub fn externally_clear(&self) -> bool {
        !self.validations.is_empty()
            && self
                .validations
                .iter()
                .all(|evidence| !evidence.status.blocks_automatic_selection())
    }

    /// Returns whether the candidate may enter automatic finalist selection.
    #[must_use]
    pub fn eligible_for_finalist_selection(&self) -> bool {
        self.accepted && self.externally_clear()
    }

    /// Returns the number of blocking validation results.
    #[must_use]
    pub fn blocking_validation_count(&self) -> usize {
        self.validations
            .iter()
            .filter(|evidence| evidence.status.blocks_automatic_selection())
            .count()
    }

    /// Validates the internal invariants of this candidate record.
    ///
    /// This validation does not perform external collision research.
    ///
    /// # Errors
    ///
    /// Returns [`CandidateInvariantError`] when:
    ///
    /// - the schema version is unsupported,
    /// - the canonical name is empty or contains unsupported characters,
    /// - the display name is empty,
    /// - an accepted candidate contains rejection reasons,
    /// - or validation evidence contains invalid confidence.
    pub fn validate(&self) -> Result<(), CandidateInvariantError> {
        if self.schema_version != CANDIDATE_SCHEMA_VERSION {
            return Err(CandidateInvariantError::UnsupportedSchemaVersion {
                found: self.schema_version,
                supported: CANDIDATE_SCHEMA_VERSION,
            });
        }

        if self.name.is_empty() {
            return Err(CandidateInvariantError::EmptyName);
        }

        if !self
            .name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        {
            return Err(CandidateInvariantError::NonCanonicalName {
                name: self.name.clone(),
            });
        }

        if self.display_name.is_empty() {
            return Err(CandidateInvariantError::EmptyDisplayName);
        }

        if self.accepted && !self.rejection_reasons.is_empty() {
            return Err(CandidateInvariantError::AcceptedWithRejections);
        }

        if self
            .validations
            .iter()
            .any(|evidence| evidence.confidence > 100)
        {
            return Err(CandidateInvariantError::InvalidValidationConfidence);
        }

        Ok(())
    }
}

/// Internal consistency error for a candidate record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CandidateInvariantError {
    /// Candidate contains no canonical name.
    EmptyName,

    /// Candidate contains no display representation.
    EmptyDisplayName,

    /// Canonical candidate name contains unsupported characters.
    NonCanonicalName {
        /// Invalid canonical value.
        name: String,
    },

    /// Candidate is marked accepted while retaining rejection reasons.
    AcceptedWithRejections,

    /// A validation record contains a confidence outside `0..=100`.
    InvalidValidationConfidence,

    /// Serialized schema version is not supported by this implementation.
    UnsupportedSchemaVersion {
        /// Version found in the record.
        found: u16,

        /// Version supported by this implementation.
        supported: u16,
    },
}

impl std::fmt::Display for CandidateInvariantError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyName => formatter.write_str("candidate name must not be empty"),
            Self::EmptyDisplayName => {
                formatter.write_str("candidate display name must not be empty")
            }
            Self::NonCanonicalName { name } => {
                write!(
                    formatter,
                    "candidate name must contain only lowercase ASCII letters or digits: {name}"
                )
            }
            Self::AcceptedWithRejections => {
                formatter.write_str("an accepted candidate must not contain rejection reasons")
            }
            Self::InvalidValidationConfidence => {
                formatter.write_str("validation confidence must be between zero and one hundred")
            }
            Self::UnsupportedSchemaVersion { found, supported } => write!(
                formatter,
                "unsupported candidate schema version {found}; supported version is {supported}"
            ),
        }
    }
}

impl std::error::Error for CandidateInvariantError {}

const fn clamp_score(score: u8) -> u8 {
    if score > 100 { 100 } else { score }
}

fn normalize_candidate(input: &str) -> String {
    input
        .trim()
        .bytes()
        .filter_map(|byte| {
            if byte.is_ascii_alphanumeric() {
                Some(byte.to_ascii_lowercase() as char)
            } else {
                None
            }
        })
        .collect()
}

fn display_candidate(normalized: &str) -> String {
    let mut characters = normalized.chars();

    match characters.next() {
        Some(first) => {
            let mut display = String::with_capacity(normalized.len());
            display.push(first.to_ascii_uppercase());
            display.extend(characters);
            display
        }
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CandidateInvariantError, CandidateRecord, CandidateScores, GenerationMode, RejectionCode,
        RejectionReason, RootLanguage, RootProvenance, ValidationEvidence, ValidationStatus,
    };

    fn high_scores() -> CandidateScores {
        CandidateScores::new(92, 88, 91, 90, 89, 95, 100, 94, 92)
    }

    #[test]
    fn candidate_creation_normalizes_name_and_display_value() {
        let candidate = CandidateRecord::new("  Nyr-Vexa  ", GenerationMode::Synthetic, 42, 7);

        assert_eq!(candidate.name, "nyrvexa");
        assert_eq!(candidate.display_name, "Nyrvexa");
        assert_eq!(candidate.seed, 42);
        assert_eq!(candidate.generation_index, 7);
        assert!(!candidate.accepted);
    }

    #[test]
    fn candidate_can_record_root_provenance() {
        let root = RootProvenance::new(RootLanguage::OldTurkic, "yol", "yol", "road", 95)
            .with_source("data/roots/old_turkic.csv:12")
            .with_transformation("vowel-shift:o>u");

        let candidate =
            CandidateRecord::new("yulora", GenerationMode::Ancient, 1, 0).with_root(root);

        assert_eq!(candidate.roots.len(), 1);
        assert_eq!(candidate.roots[0].language, RootLanguage::OldTurkic);
        assert_eq!(candidate.roots[0].confidence, 95);
        assert_eq!(candidate.roots[0].transformations.len(), 1);
    }

    #[test]
    fn accepting_candidate_requires_no_rejection_reasons() {
        let accepted = CandidateRecord::new("velora", GenerationMode::Hybrid, 8, 3)
            .with_scores(high_scores())
            .accept();

        assert!(accepted.accepted);
        assert!(accepted.validate().is_ok());

        let rejected = accepted.reject(RejectionReason::new(
            RejectionCode::NearDuplicate,
            "Too similar to a reference mark.",
        ));

        assert!(!rejected.accepted);
        assert_eq!(rejected.rejection_reasons.len(), 1);
        assert!(rejected.validate().is_ok());
    }

    #[test]
    fn external_clearance_requires_evidence() {
        let candidate = CandidateRecord::new("orvexa", GenerationMode::Synthetic, 4, 2)
            .with_scores(high_scores())
            .accept();

        assert!(!candidate.externally_clear());
        assert!(!candidate.eligible_for_finalist_selection());

        let candidate = candidate
            .with_validation(ValidationEvidence::new(
                "domain:com",
                "orvexa.com",
                ValidationStatus::NoCollisionFound,
                "No registration was observed.",
                80,
            ))
            .with_validation(ValidationEvidence::new(
                "github",
                "orvexa",
                ValidationStatus::NoCollisionFound,
                "No exact account or repository was observed.",
                75,
            ));

        assert!(candidate.externally_clear());
        assert!(candidate.eligible_for_finalist_selection());
    }

    #[test]
    fn possible_collision_blocks_finalist_selection() {
        let candidate = CandidateRecord::new("qervon", GenerationMode::Synthetic, 4, 3)
            .with_scores(high_scores())
            .accept()
            .with_validation(ValidationEvidence::new(
                "web",
                "\"qervon\"",
                ValidationStatus::PossibleCollision,
                "Existing public use was found.",
                95,
            ));

        assert_eq!(candidate.blocking_validation_count(), 1);
        assert!(!candidate.externally_clear());
        assert!(!candidate.eligible_for_finalist_selection());
    }

    #[test]
    fn tags_are_unique() {
        let candidate = CandidateRecord::new("korvexa", GenerationMode::Hybrid, 9, 1)
            .with_tag("enterprise")
            .with_tag("enterprise")
            .with_tag("logistics");

        assert_eq!(candidate.tags, ["enterprise", "logistics"]);
    }

    #[test]
    fn accepted_candidate_with_rejections_is_invalid() {
        let mut candidate = CandidateRecord::new("avexon", GenerationMode::Synthetic, 10, 2);

        candidate.accepted = true;
        candidate.rejection_reasons.push(RejectionReason::new(
            RejectionCode::CampaignRule,
            "Synthetic invariant violation.",
        ));

        assert_eq!(
            candidate.validate(),
            Err(CandidateInvariantError::AcceptedWithRejections)
        );
    }

    #[test]
    fn score_components_have_stable_order() {
        let scores = high_scores();
        let components = scores.components();

        assert_eq!(components[0], ("pronunciation", 92));
        assert_eq!(components[8], ("overall", 92));
    }

    #[test]
    fn generation_mode_has_stable_display_value() {
        assert_eq!(GenerationMode::Synthetic.to_string(), "synthetic");
        assert_eq!(GenerationMode::Ancient.to_string(), "ancient");
        assert_eq!(GenerationMode::Hybrid.to_string(), "hybrid");
        assert_eq!(GenerationMode::Mutation.to_string(), "mutation");
    }
}
