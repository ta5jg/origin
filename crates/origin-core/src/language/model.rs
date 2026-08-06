/* =============================================================================
 * File:           crates/origin-core/src/language/model.rs
 * Project:        Origin
 * Author:         USDTG GROUP TECHNOLOGY LLC
 * Developer:      Irfan Gedik
 * Created Date:   2026-08-06
 * Version:        0.1.0
 *
 * Description:
 *   Defines canonical language identifiers, linguistic roots, meanings,
 *   transliterations, source provenance, confidence, and dataset validation.
 *
 * License:
 *   Origin License v1.0 — see LICENSE in the repository root.
 * ============================================================================= */

//! Canonical linguistic data models used by the Origin language engine.
//!
//! This module contains no dataset loading or name-generation algorithms.
//! It defines the stable, serializable records exchanged between language
//! catalogs, morphology, generation, evidence, and reporting components.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::RootLanguage;

/// Current serialized schema version for language-root records.
pub const LANGUAGE_ROOT_SCHEMA_VERSION: u16 = 1;

/// Canonical identifier for a language or synthetic language family.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LanguageId {
    /// Sumerian language.
    Sumerian,

    /// Akkadian language.
    Akkadian,

    /// Old Turkic language family.
    OldTurkic,

    /// Latin language.
    Latin,

    /// Sanskrit language.
    Sanskrit,

    /// Origin-generated artificial morphemes.
    Synthetic,

    /// A language not represented by a built-in fixed variant.
    Other(String),
}

impl LanguageId {
    /// Returns the stable machine-readable language identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Sumerian => "sumerian",
            Self::Akkadian => "akkadian",
            Self::OldTurkic => "old-turkic",
            Self::Latin => "latin",
            Self::Sanskrit => "sanskrit",
            Self::Synthetic => "synthetic",
            Self::Other(identifier) => identifier.as_str(),
        }
    }

    /// Returns whether the language is generated rather than historical.
    #[must_use]
    pub const fn is_synthetic(&self) -> bool {
        matches!(self, Self::Synthetic)
    }

    /// Returns whether this identifier represents a built-in language.
    #[must_use]
    pub const fn is_builtin(&self) -> bool {
        !matches!(self, Self::Other(_))
    }
}

impl std::fmt::Display for LanguageId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl From<&LanguageId> for RootLanguage {
    fn from(language: &LanguageId) -> Self {
        match language {
            LanguageId::Sumerian => Self::Sumerian,
            LanguageId::Akkadian => Self::Akkadian,
            LanguageId::OldTurkic => Self::OldTurkic,
            LanguageId::Latin => Self::Latin,
            LanguageId::Sanskrit => Self::Sanskrit,
            LanguageId::Synthetic => Self::Synthetic,
            LanguageId::Other(_) => Self::Other,
        }
    }
}

impl From<LanguageId> for RootLanguage {
    fn from(language: LanguageId) -> Self {
        Self::from(&language)
    }
}

/// Semantic category associated with a root meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MeaningCategory {
    /// Travel, path, motion, transport, or direction.
    Movement,

    /// Connection, network, union, or coordination.
    Connection,

    /// Intelligence, knowledge, learning, or thought.
    Intelligence,

    /// Trust, truth, reliability, or protection.
    Trust,

    /// Strength, authority, endurance, or resilience.
    Strength,

    /// Light, vision, clarity, or discovery.
    Light,

    /// Origin, creation, beginning, or foundation.
    Origin,

    /// Trade, exchange, value, or prosperity.
    Commerce,

    /// Time, continuity, permanence, or longevity.
    Time,

    /// Place, land, city, world, or environment.
    Place,

    /// Life, vitality, growth, or renewal.
    Life,

    /// A general or currently uncategorized meaning.
    Other,
}

impl MeaningCategory {
    /// Returns the stable machine-readable category identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Movement => "movement",
            Self::Connection => "connection",
            Self::Intelligence => "intelligence",
            Self::Trust => "trust",
            Self::Strength => "strength",
            Self::Light => "light",
            Self::Origin => "origin",
            Self::Commerce => "commerce",
            Self::Time => "time",
            Self::Place => "place",
            Self::Life => "life",
            Self::Other => "other",
        }
    }
}

impl std::fmt::Display for MeaningCategory {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Describes one normalized semantic meaning associated with a root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootMeaning {
    /// Short canonical English gloss.
    pub gloss: String,

    /// Optional longer explanation or contextual meaning.
    pub description: Option<String>,

    /// Broad semantic categories associated with the meaning.
    pub categories: BTreeSet<MeaningCategory>,

    /// Whether the meaning is literal, reconstructed, inferred, or synthetic.
    pub interpretation: MeaningInterpretation,
}

impl RootMeaning {
    /// Creates a literal root meaning with no categories.
    #[must_use]
    pub fn literal(gloss: impl Into<String>) -> Self {
        Self {
            gloss: gloss.into(),
            description: None,
            categories: BTreeSet::new(),
            interpretation: MeaningInterpretation::Literal,
        }
    }

    /// Creates a synthetic meaning assigned by Origin.
    #[must_use]
    pub fn synthetic(gloss: impl Into<String>) -> Self {
        Self {
            gloss: gloss.into(),
            description: None,
            categories: BTreeSet::new(),
            interpretation: MeaningInterpretation::Synthetic,
        }
    }

    /// Attaches a longer semantic description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Adds a semantic category.
    #[must_use]
    pub fn with_category(mut self, category: MeaningCategory) -> Self {
        self.categories.insert(category);
        self
    }

    /// Sets the interpretation classification.
    #[must_use]
    pub const fn with_interpretation(mut self, interpretation: MeaningInterpretation) -> Self {
        self.interpretation = interpretation;
        self
    }
}

/// Describes how confidently a meaning is attributed to a root.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MeaningInterpretation {
    /// Meaning is directly attested by the cited source.
    Literal,

    /// Meaning is reconstructed by linguistic scholarship.
    Reconstructed,

    /// Meaning is inferred from context or related forms.
    Inferred,

    /// Meaning is intentionally assigned to a synthetic morpheme.
    Synthetic,
}

/// One transliteration or normalized representation of a source root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transliteration {
    /// Transliteration system or convention.
    pub system: String,

    /// Value represented using the transliteration system.
    pub value: String,

    /// Whether this form is preferred for Origin processing.
    pub preferred: bool,

    /// Optional note describing uncertainty or normalization.
    pub note: Option<String>,
}

impl Transliteration {
    /// Creates a transliteration record.
    #[must_use]
    pub fn new(system: impl Into<String>, value: impl Into<String>, preferred: bool) -> Self {
        Self {
            system: system.into(),
            value: value.into(),
            preferred,
            note: None,
        }
    }

    /// Attaches an explanatory note.
    #[must_use]
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }
}

/// Classification of a linguistic source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceKind {
    /// Academic dictionary or lexicon.
    Lexicon,

    /// Peer-reviewed or scholarly publication.
    AcademicPublication,

    /// Corpus, inscription, tablet, manuscript, or primary text.
    PrimarySource,

    /// Museum, university, or recognized institutional database.
    InstitutionalDatabase,

    /// Secondary reference work.
    SecondaryReference,

    /// Origin-controlled synthetic dataset.
    SyntheticDataset,
}

/// Attributable source supporting a linguistic record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootSource {
    /// Stable source identifier within the Origin dataset.
    pub id: String,

    /// Human-readable title.
    pub title: String,

    /// Source classification.
    pub kind: SourceKind,

    /// Author, editor, institution, or dataset owner.
    pub contributor: Option<String>,

    /// Publication year, when known.
    pub year: Option<u16>,

    /// Public location or bibliographic reference.
    pub reference: String,

    /// Optional license or usage terms.
    pub license: Option<String>,

    /// Optional page, entry, line, tablet, or record locator.
    pub locator: Option<String>,
}

impl RootSource {
    /// Creates a source record.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        kind: SourceKind,
        reference: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            kind,
            contributor: None,
            year: None,
            reference: reference.into(),
            license: None,
            locator: None,
        }
    }

    /// Attaches an author, editor, institution, or dataset owner.
    #[must_use]
    pub fn with_contributor(mut self, contributor: impl Into<String>) -> Self {
        self.contributor = Some(contributor.into());
        self
    }

    /// Attaches a publication year.
    #[must_use]
    pub const fn with_year(mut self, year: u16) -> Self {
        self.year = Some(year);
        self
    }

    /// Attaches license or usage information.
    #[must_use]
    pub fn with_license(mut self, license: impl Into<String>) -> Self {
        self.license = Some(license.into());
        self
    }

    /// Attaches a source-specific record locator.
    #[must_use]
    pub fn with_locator(mut self, locator: impl Into<String>) -> Self {
        self.locator = Some(locator.into());
        self
    }
}

/// Confidence assigned to a linguistic root or attribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootConfidence {
    /// Confidence score in the inclusive range `0..=100`.
    score: u8,

    /// Basis for the confidence assignment.
    pub basis: ConfidenceBasis,
}

impl RootConfidence {
    /// Creates a confidence value, clamped to `0..=100`.
    #[must_use]
    pub const fn new(score: u8, basis: ConfidenceBasis) -> Self {
        Self {
            score: clamp_score(score),
            basis,
        }
    }

    /// Returns the numeric confidence score.
    #[must_use]
    pub const fn score(self) -> u8 {
        self.score
    }

    /// Returns whether the confidence meets a supplied threshold.
    #[must_use]
    pub const fn meets(self, minimum: u8) -> bool {
        self.score >= minimum
    }
}

impl Default for RootConfidence {
    fn default() -> Self {
        Self::new(0, ConfidenceBasis::Unreviewed)
    }
}

/// Basis used to assign linguistic confidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfidenceBasis {
    /// Supported directly by a reliable cited source.
    Attested,

    /// Supported by multiple independent reliable sources.
    Corroborated,

    /// Scholarly reconstruction with explicit uncertainty.
    Reconstructed,

    /// Internal synthetic value with deterministic provenance.
    Synthetic,

    /// Record has not yet received linguistic review.
    Unreviewed,
}

/// Canonical language-root record consumed by Origin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageRoot {
    /// Serialized record schema version.
    pub schema_version: u16,

    /// Stable dataset-scoped identifier.
    pub id: String,

    /// Language associated with the root.
    pub language: LanguageId,

    /// Original historical or synthetic representation.
    pub original: String,

    /// Canonical lowercase ASCII form consumed by Origin algorithms.
    pub normalized: String,

    /// Semantic meanings associated with the root.
    pub meanings: Vec<RootMeaning>,

    /// Known transliterations or alternative forms.
    pub transliterations: Vec<Transliteration>,

    /// Sources supporting the root and meanings.
    pub sources: Vec<RootSource>,

    /// Confidence in the root attribution.
    pub confidence: RootConfidence,

    /// Whether the root may currently be used by generators.
    pub enabled: bool,

    /// Optional tags used for campaign selection.
    pub tags: BTreeSet<String>,
}

impl LanguageRoot {
    /// Creates a language-root record.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        language: LanguageId,
        original: impl Into<String>,
        normalized: impl Into<String>,
        meaning: RootMeaning,
        confidence: RootConfidence,
    ) -> Self {
        Self {
            schema_version: LANGUAGE_ROOT_SCHEMA_VERSION,
            id: id.into(),
            language,
            original: original.into(),
            normalized: normalized.into(),
            meanings: vec![meaning],
            transliterations: Vec::new(),
            sources: Vec::new(),
            confidence,
            enabled: true,
            tags: BTreeSet::new(),
        }
    }

    /// Adds an additional semantic meaning.
    #[must_use]
    pub fn with_meaning(mut self, meaning: RootMeaning) -> Self {
        self.meanings.push(meaning);
        self
    }

    /// Adds a transliteration.
    #[must_use]
    pub fn with_transliteration(mut self, transliteration: Transliteration) -> Self {
        self.transliterations.push(transliteration);
        self
    }

    /// Adds a supporting source.
    #[must_use]
    pub fn with_source(mut self, source: RootSource) -> Self {
        self.sources.push(source);
        self
    }

    /// Adds a campaign-selection tag.
    #[must_use]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        let tag = normalize_tag(&tag.into());
        if !tag.is_empty() {
            self.tags.insert(tag);
        }
        self
    }

    /// Sets whether generators may use this root.
    #[must_use]
    pub const fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Returns the preferred transliteration, when one exists.
    #[must_use]
    pub fn preferred_transliteration(&self) -> Option<&Transliteration> {
        self.transliterations
            .iter()
            .find(|transliteration| transliteration.preferred)
            .or_else(|| self.transliterations.first())
    }

    /// Returns whether the root has the supplied semantic category.
    #[must_use]
    pub fn has_category(&self, category: MeaningCategory) -> bool {
        self.meanings
            .iter()
            .any(|meaning| meaning.categories.contains(&category))
    }

    /// Returns whether the root carries the supplied normalized tag.
    #[must_use]
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&normalize_tag(tag))
    }

    /// Validates the internal consistency of this root record.
    ///
    /// # Errors
    ///
    /// Returns [`LanguageRootError`] when:
    ///
    /// - the schema version is unsupported,
    /// - the identifier, original form, or normalized form is empty,
    /// - the identifier or normalized form is not canonical,
    /// - the record has no meanings,
    /// - a non-synthetic record has no source,
    /// - multiple transliterations are marked preferred,
    /// - or a source record is incomplete.
    pub fn validate(&self) -> Result<(), LanguageRootError> {
        if self.schema_version != LANGUAGE_ROOT_SCHEMA_VERSION {
            return Err(LanguageRootError::UnsupportedSchemaVersion {
                found: self.schema_version,
                supported: LANGUAGE_ROOT_SCHEMA_VERSION,
            });
        }

        if self.id.trim().is_empty() {
            return Err(LanguageRootError::EmptyId);
        }

        if !is_canonical_identifier(&self.id) {
            return Err(LanguageRootError::InvalidId {
                value: self.id.clone(),
            });
        }

        if self.original.trim().is_empty() {
            return Err(LanguageRootError::EmptyOriginal);
        }

        if self.normalized.is_empty() {
            return Err(LanguageRootError::EmptyNormalized);
        }

        if !is_canonical_root(&self.normalized) {
            return Err(LanguageRootError::InvalidNormalized {
                value: self.normalized.clone(),
            });
        }

        if self.meanings.is_empty() {
            return Err(LanguageRootError::MissingMeaning);
        }

        if self
            .meanings
            .iter()
            .any(|meaning| meaning.gloss.trim().is_empty())
        {
            return Err(LanguageRootError::EmptyMeaningGloss);
        }

        if !self.language.is_synthetic() && self.sources.is_empty() {
            return Err(LanguageRootError::MissingSource);
        }

        let preferred_count = self
            .transliterations
            .iter()
            .filter(|transliteration| transliteration.preferred)
            .count();

        if preferred_count > 1 {
            return Err(LanguageRootError::MultiplePreferredTransliterations);
        }

        for source in &self.sources {
            if source.id.trim().is_empty()
                || source.title.trim().is_empty()
                || source.reference.trim().is_empty()
            {
                return Err(LanguageRootError::IncompleteSource {
                    source_id: source.id.clone(),
                });
            }
        }

        Ok(())
    }
}

/// Internal consistency error for a language-root record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LanguageRootError {
    /// The record uses an unsupported schema version.
    UnsupportedSchemaVersion {
        /// Version found in the record.
        found: u16,

        /// Version supported by the current implementation.
        supported: u16,
    },

    /// Root identifier is empty.
    EmptyId,

    /// Root identifier is not canonical.
    InvalidId {
        /// Invalid identifier value.
        value: String,
    },

    /// Original root representation is empty.
    EmptyOriginal,

    /// Normalized root representation is empty.
    EmptyNormalized,

    /// Normalized root contains unsupported characters.
    InvalidNormalized {
        /// Invalid normalized value.
        value: String,
    },

    /// Root has no semantic meaning.
    MissingMeaning,

    /// One meaning has an empty canonical gloss.
    EmptyMeaningGloss,

    /// Historical-language root has no supporting source.
    MissingSource,

    /// More than one transliteration is marked preferred.
    MultiplePreferredTransliterations,

    /// Source metadata is incomplete.
    IncompleteSource {
        /// Identifier of the incomplete source.
        source_id: String,
    },
}

impl std::fmt::Display for LanguageRootError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { found, supported } => write!(
                formatter,
                "unsupported language-root schema version {found}; supported version is {supported}"
            ),
            Self::EmptyId => formatter.write_str("language-root identifier must not be empty"),
            Self::InvalidId { value } => write!(
                formatter,
                "language-root identifier must use lowercase ASCII letters, digits, hyphens, or underscores: {value}"
            ),
            Self::EmptyOriginal => {
                formatter.write_str("original root representation must not be empty")
            }
            Self::EmptyNormalized => {
                formatter.write_str("normalized root representation must not be empty")
            }
            Self::InvalidNormalized { value } => write!(
                formatter,
                "normalized root must use lowercase ASCII letters only: {value}"
            ),
            Self::MissingMeaning => {
                formatter.write_str("language root must contain at least one meaning")
            }
            Self::EmptyMeaningGloss => formatter.write_str("root meaning gloss must not be empty"),
            Self::MissingSource => formatter
                .write_str("historical-language root must contain at least one supporting source"),
            Self::MultiplePreferredTransliterations => formatter
                .write_str("language root must not contain multiple preferred transliterations"),
            Self::IncompleteSource { source_id } => {
                write!(formatter, "source metadata is incomplete: {source_id}")
            }
        }
    }
}

impl std::error::Error for LanguageRootError {}

const fn clamp_score(score: u8) -> u8 {
    if score > 100 { 100 } else { score }
}

fn normalize_tag(input: &str) -> String {
    input
        .trim()
        .bytes()
        .filter_map(|byte| {
            if byte.is_ascii_alphanumeric() {
                Some(byte.to_ascii_lowercase() as char)
            } else if byte == b'-' || byte == b'_' || byte.is_ascii_whitespace() {
                Some('-')
            } else {
                None
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn is_canonical_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'_'
        })
}

fn is_canonical_root(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::{
        ConfidenceBasis, LanguageId, LanguageRoot, LanguageRootError, MeaningCategory,
        MeaningInterpretation, RootConfidence, RootMeaning, RootSource, SourceKind,
        Transliteration,
    };
    use crate::RootLanguage;

    fn source() -> RootSource {
        RootSource::new(
            "latin-lexicon-001",
            "Latin Lexicon",
            SourceKind::Lexicon,
            "https://example.invalid/latin",
        )
        .with_contributor("Example Institution")
        .with_year(2026)
        .with_locator("entry:via")
    }

    fn valid_latin_root() -> LanguageRoot {
        LanguageRoot::new(
            "latin-via",
            LanguageId::Latin,
            "via",
            "via",
            RootMeaning::literal("road").with_category(MeaningCategory::Movement),
            RootConfidence::new(95, ConfidenceBasis::Attested),
        )
        .with_transliteration(Transliteration::new("origin-ascii", "via", true))
        .with_source(source())
        .with_tag("Logistics")
    }

    #[test]
    fn language_identifiers_have_stable_values() {
        assert_eq!(LanguageId::Sumerian.to_string(), "sumerian");
        assert_eq!(LanguageId::OldTurkic.to_string(), "old-turkic");
        assert_eq!(
            LanguageId::Other("proto-origin".to_owned()).to_string(),
            "proto-origin"
        );
    }

    #[test]
    fn language_identifier_converts_to_candidate_root_language() {
        assert_eq!(
            RootLanguage::from(LanguageId::OldTurkic),
            RootLanguage::OldTurkic
        );
        assert_eq!(
            RootLanguage::from(LanguageId::Other("elamite".to_owned())),
            RootLanguage::Other
        );
    }

    #[test]
    fn valid_historical_root_passes_validation() {
        let root = valid_latin_root();

        assert!(root.validate().is_ok());
        assert!(root.enabled);
        assert!(root.has_category(MeaningCategory::Movement));
        assert!(root.has_tag("logistics"));
        assert_eq!(
            root.preferred_transliteration()
                .map(|value| value.value.as_str()),
            Some("via")
        );
    }

    #[test]
    fn historical_root_requires_source() {
        let root = LanguageRoot::new(
            "old-turkic-yol",
            LanguageId::OldTurkic,
            "yol",
            "yol",
            RootMeaning::literal("road"),
            RootConfidence::new(90, ConfidenceBasis::Attested),
        );

        assert_eq!(root.validate(), Err(LanguageRootError::MissingSource));
    }

    #[test]
    fn synthetic_root_does_not_require_external_source() {
        let root = LanguageRoot::new(
            "synthetic-vor",
            LanguageId::Synthetic,
            "vor",
            "vor",
            RootMeaning::synthetic("coordinated intelligence")
                .with_category(MeaningCategory::Connection)
                .with_category(MeaningCategory::Intelligence),
            RootConfidence::new(100, ConfidenceBasis::Synthetic),
        );

        assert!(root.validate().is_ok());
    }

    #[test]
    fn normalized_root_must_use_lowercase_ascii_letters() {
        let mut root = valid_latin_root();
        root.normalized = "Vía-1".to_owned();

        assert_eq!(
            root.validate(),
            Err(LanguageRootError::InvalidNormalized {
                value: "Vía-1".to_owned(),
            })
        );
    }

    #[test]
    fn only_one_transliteration_may_be_preferred() {
        let root = valid_latin_root().with_transliteration(Transliteration::new(
            "alternative",
            "wiya",
            true,
        ));

        assert_eq!(
            root.validate(),
            Err(LanguageRootError::MultiplePreferredTransliterations)
        );
    }

    #[test]
    fn root_requires_at_least_one_meaning() {
        let mut root = valid_latin_root();
        root.meanings.clear();

        assert_eq!(root.validate(), Err(LanguageRootError::MissingMeaning));
    }

    #[test]
    fn source_metadata_must_be_complete() {
        let mut root = valid_latin_root();
        root.sources[0].reference.clear();

        assert_eq!(
            root.validate(),
            Err(LanguageRootError::IncompleteSource {
                source_id: "latin-lexicon-001".to_owned(),
            })
        );
    }

    #[test]
    fn confidence_is_bounded_and_comparable() {
        let confidence = RootConfidence::new(255, ConfidenceBasis::Corroborated);

        assert_eq!(confidence.score(), 100);
        assert!(confidence.meets(90));
        assert!(!RootConfidence::default().meets(1));
    }

    #[test]
    fn root_meaning_supports_interpretation_and_categories() {
        let meaning = RootMeaning::literal("light")
            .with_description("Visible light or metaphorical clarity.")
            .with_category(MeaningCategory::Light)
            .with_interpretation(MeaningInterpretation::Reconstructed);

        assert_eq!(meaning.interpretation, MeaningInterpretation::Reconstructed);
        assert!(meaning.categories.contains(&MeaningCategory::Light));
    }

    #[test]
    fn tags_are_normalized_and_deduplicated() {
        let root = valid_latin_root()
            .with_tag("Enterprise Software")
            .with_tag("enterprise-software");

        assert!(root.has_tag("enterprise software"));
        assert_eq!(
            root.tags
                .iter()
                .filter(|tag| tag.as_str() == "enterprise-software")
                .count(),
            1
        );
    }
}
