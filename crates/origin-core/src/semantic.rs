//! Curated semantic roots and explainable brand composition.

use std::fmt;

use serde::Serialize;

use crate::{
    BrandReport, ConfidenceBasis, LanguageCatalog, LanguageId, LanguageRoot, MeaningCategory,
    MergeError, MergePolicy, MergeReport, RootConfidence, RootMeaning, RootSource, SourceKind,
    analyze_brand, merge_roots_with_policy,
};

/// Deterministic output of composing two curated semantic roots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SemanticComposition {
    /// Stable identifier of the left root.
    pub left_root_id: String,
    /// Stable identifier of the right root.
    pub right_root_id: String,
    /// Result of the morphology merge pipeline.
    pub merge: MergeReport,
    /// Explainable quality analysis of the resulting candidate.
    pub analysis: BrandReport,
    /// Short semantic explanation assembled from the selected roots.
    pub meaning: String,
}

/// Errors returned by semantic composition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticError {
    /// The requested root identifier is absent from the selected catalog.
    UnknownRoot {
        /// Requested stable root identifier.
        id: String,
    },
    /// Morphology could not merge the selected root forms.
    Merge(MergeError),
}

impl fmt::Display for SemanticError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownRoot { id } => write!(formatter, "semantic root `{id}` was not found"),
            Self::Merge(error) => write!(formatter, "semantic composition failed: {error}"),
        }
    }
}

impl std::error::Error for SemanticError {}

/// Returns Origin's small, attributable built-in Latin semantic catalog.
///
/// The catalog intentionally starts small: every included record has a source
/// rather than treating a generated association as a historical fact.
///
/// # Panics
///
/// Panics only if a hard-coded built-in record violates the catalog contract;
/// this is guarded by the module's catalog test.
#[must_use]
#[allow(clippy::too_many_lines)] // The reviewed built-in dataset is intentionally local and auditable.
pub fn built_in_catalog() -> LanguageCatalog {
    LanguageCatalog::from_roots([
        latin_root("latin-lux", "lūx", "lux", "light", MeaningCategory::Light),
        latin_root("latin-via", "via", "via", "path", MeaningCategory::Movement),
        latin_root("latin-ver", "vērus", "ver", "true", MeaningCategory::Trust),
        latin_root(
            "latin-terra",
            "terra",
            "terra",
            "earth",
            MeaningCategory::Place,
        ),
        latin_root("latin-vita", "vīta", "vita", "life", MeaningCategory::Life),
        latin_root(
            "latin-temp",
            "tempus",
            "temp",
            "time",
            MeaningCategory::Time,
        ),
        latin_root("latin-loc", "locus", "loc", "place", MeaningCategory::Place),
        latin_root(
            "latin-fort",
            "fortis",
            "fort",
            "strong",
            MeaningCategory::Strength,
        ),
        latin_root(
            "latin-nex",
            "nexus",
            "nex",
            "connection",
            MeaningCategory::Connection,
        ),
        latin_root("latin-nov", "novus", "nov", "new", MeaningCategory::Origin),
        historical_root(
            "sumerian-eme",
            LanguageId::Sumerian,
            "eme",
            "eme",
            "language",
            MeaningCategory::Connection,
            "epsd",
            "Electronic Pennsylvania Sumerian Dictionary",
            "ePSD",
        ),
        historical_root(
            "sumerian-dub",
            LanguageId::Sumerian,
            "dub",
            "dub",
            "record",
            MeaningCategory::Intelligence,
            "epsd",
            "Electronic Pennsylvania Sumerian Dictionary",
            "ePSD",
        ),
        historical_root(
            "sumerian-lugal",
            LanguageId::Sumerian,
            "lugal",
            "lugal",
            "king",
            MeaningCategory::Strength,
            "epsd",
            "Electronic Pennsylvania Sumerian Dictionary",
            "ePSD",
        ),
        historical_root(
            "sumerian-uru",
            LanguageId::Sumerian,
            "uru",
            "uru",
            "city",
            MeaningCategory::Place,
            "epsd",
            "Electronic Pennsylvania Sumerian Dictionary",
            "ePSD",
        ),
        historical_root(
            "sumerian-edin",
            LanguageId::Sumerian,
            "edin",
            "edin",
            "plain",
            MeaningCategory::Place,
            "epsd",
            "Electronic Pennsylvania Sumerian Dictionary",
            "ePSD",
        ),
        historical_root(
            "akkadian-sarru",
            LanguageId::Akkadian,
            "šarru",
            "sarru",
            "king",
            MeaningCategory::Strength,
            "cda",
            "A Concise Dictionary of Akkadian",
            "Black, George, and Postgate (2000)",
        ),
        historical_root(
            "akkadian-babu",
            LanguageId::Akkadian,
            "bābu",
            "babu",
            "gate",
            MeaningCategory::Connection,
            "cda",
            "A Concise Dictionary of Akkadian",
            "Black, George, and Postgate (2000)",
        ),
        historical_root(
            "akkadian-naru",
            LanguageId::Akkadian,
            "nāru",
            "naru",
            "river",
            MeaningCategory::Movement,
            "cda",
            "A Concise Dictionary of Akkadian",
            "Black, George, and Postgate (2000)",
        ),
        historical_root(
            "old-turkic-kut",
            LanguageId::OldTurkic,
            "kut",
            "kut",
            "fortune",
            MeaningCategory::Trust,
            "clauson",
            "An Etymological Dictionary of Pre-Thirteenth-Century Turkish",
            "Clauson (1972)",
        ),
        historical_root(
            "old-turkic-il",
            LanguageId::OldTurkic,
            "il",
            "il",
            "realm",
            MeaningCategory::Place,
            "clauson",
            "An Etymological Dictionary of Pre-Thirteenth-Century Turkish",
            "Clauson (1972)",
        ),
        historical_root(
            "old-turkic-yurt",
            LanguageId::OldTurkic,
            "yurt",
            "yurt",
            "homeland",
            MeaningCategory::Place,
            "clauson",
            "An Etymological Dictionary of Pre-Thirteenth-Century Turkish",
            "Clauson (1972)",
        ),
        historical_root(
            "old-turkic-orun",
            LanguageId::OldTurkic,
            "orun",
            "orun",
            "place",
            MeaningCategory::Place,
            "clauson",
            "An Etymological Dictionary of Pre-Thirteenth-Century Turkish",
            "Clauson (1972)",
        ),
        historical_root(
            "old-turkic-yol",
            LanguageId::OldTurkic,
            "yol",
            "yol",
            "path",
            MeaningCategory::Movement,
            "clauson",
            "An Etymological Dictionary of Pre-Thirteenth-Century Turkish",
            "Clauson (1972)",
        ),
        historical_root(
            "old-turkic-yel",
            LanguageId::OldTurkic,
            "yel",
            "yel",
            "wind",
            MeaningCategory::Movement,
            "clauson",
            "An Etymological Dictionary of Pre-Thirteenth-Century Turkish",
            "Clauson (1972)",
        ),
        historical_root(
            "old-turkic-alp",
            LanguageId::OldTurkic,
            "alp",
            "alp",
            "hero",
            MeaningCategory::Strength,
            "clauson",
            "An Etymological Dictionary of Pre-Thirteenth-Century Turkish",
            "Clauson (1972)",
        ),
        historical_root(
            "old-turkic-erk",
            LanguageId::OldTurkic,
            "erk",
            "erk",
            "power",
            MeaningCategory::Strength,
            "clauson",
            "An Etymological Dictionary of Pre-Thirteenth-Century Turkish",
            "Clauson (1972)",
        ),
        historical_root(
            "sanskrit-dhara",
            LanguageId::Sanskrit,
            "dhārā",
            "dhara",
            "stream",
            MeaningCategory::Movement,
            "monier-williams",
            "A Sanskrit-English Dictionary",
            "Monier-Williams (1899)",
        ),
        historical_root(
            "sanskrit-artha",
            LanguageId::Sanskrit,
            "artha",
            "artha",
            "purpose",
            MeaningCategory::Commerce,
            "monier-williams",
            "A Sanskrit-English Dictionary",
            "Monier-Williams (1899)",
        ),
        historical_root(
            "sanskrit-veda",
            LanguageId::Sanskrit,
            "veda",
            "veda",
            "knowledge",
            MeaningCategory::Intelligence,
            "monier-williams",
            "A Sanskrit-English Dictionary",
            "Monier-Williams (1899)",
        ),
        historical_root(
            "sanskrit-yatra",
            LanguageId::Sanskrit,
            "yātrā",
            "yatra",
            "journey",
            MeaningCategory::Movement,
            "monier-williams",
            "A Sanskrit-English Dictionary",
            "Monier-Williams (1899)",
        ),
        historical_root(
            "sanskrit-sutra",
            LanguageId::Sanskrit,
            "sūtra",
            "sutra",
            "thread",
            MeaningCategory::Connection,
            "monier-williams",
            "A Sanskrit-English Dictionary",
            "Monier-Williams (1899)",
        ),
    ])
    .expect("the built-in semantic catalog must remain internally valid")
}

/// Composes two roots from the built-in catalog using the standard morphology policy.
///
/// # Errors
///
/// Returns an error when either identifier is absent or morphology rejects a root.
pub fn compose_builtin(
    left_id: &str,
    right_id: &str,
) -> Result<SemanticComposition, SemanticError> {
    let catalog = built_in_catalog();
    compose(&catalog, left_id, right_id)
}

/// Composes two roots from a caller-supplied catalog.
///
/// # Errors
///
/// Returns an error when either identifier is absent or morphology rejects a root.
pub fn compose(
    catalog: &LanguageCatalog,
    left_id: &str,
    right_id: &str,
) -> Result<SemanticComposition, SemanticError> {
    let left = catalog
        .get(left_id)
        .ok_or_else(|| SemanticError::UnknownRoot { id: left_id.into() })?;
    let right = catalog
        .get(right_id)
        .ok_or_else(|| SemanticError::UnknownRoot {
            id: right_id.into(),
        })?;
    let merge =
        merge_roots_with_policy(&left.normalized, &right.normalized, MergePolicy::default())
            .map_err(SemanticError::Merge)?;
    let candidate = merge.merged();

    Ok(SemanticComposition {
        left_root_id: left.id.clone(),
        right_root_id: right.id.clone(),
        analysis: analyze_brand(candidate),
        meaning: format!("{} + {}", primary_gloss(left), primary_gloss(right)),
        merge,
    })
}

fn latin_root(
    id: &str,
    original: &str,
    normalized: &str,
    gloss: &str,
    category: MeaningCategory,
) -> LanguageRoot {
    LanguageRoot::new(
        id,
        LanguageId::Latin,
        original,
        normalized,
        RootMeaning::literal(gloss).with_category(category),
        RootConfidence::new(90, ConfidenceBasis::Attested),
    )
    .with_source(RootSource::new(
        "lewis-short-1879",
        "A Latin Dictionary",
        SourceKind::SecondaryReference,
        "Lewis and Short (1879), Perseus Digital Library",
    ))
    .with_tag(category.as_str())
}

#[allow(clippy::too_many_arguments)]
fn historical_root(
    id: &str,
    language: LanguageId,
    original: &str,
    normalized: &str,
    gloss: &str,
    category: MeaningCategory,
    source_id: &str,
    source_title: &str,
    reference: &str,
) -> LanguageRoot {
    LanguageRoot::new(
        id,
        language,
        original,
        normalized,
        RootMeaning::literal(gloss).with_category(category),
        RootConfidence::new(80, ConfidenceBasis::Attested),
    )
    .with_source(RootSource::new(
        source_id,
        source_title,
        SourceKind::Lexicon,
        reference,
    ))
    .with_tag(category.as_str())
}

fn primary_gloss(root: &LanguageRoot) -> &str {
    root.meanings
        .first()
        .map_or("unknown", |meaning| meaning.gloss.as_str())
}

#[cfg(test)]
mod tests {
    use super::{SemanticError, built_in_catalog, compose_builtin};

    #[test]
    fn built_in_catalog_is_small_valid_and_attributable() {
        let catalog = built_in_catalog();

        assert_eq!(catalog.len(), 31);
        assert!(catalog.iter().all(|root| !root.sources.is_empty()));
    }

    #[test]
    fn composition_preserves_meaning_provenance_and_analysis() {
        let composition = compose_builtin("latin-lux", "latin-via").expect("known roots");

        assert_eq!(composition.merge.merged(), "luxvia");
        assert_eq!(composition.meaning, "light + path");
        assert_eq!(composition.analysis.normalized, "luxvia");
    }

    #[test]
    fn unknown_root_is_explicit() {
        assert!(matches!(
            compose_builtin("missing", "latin-via"),
            Err(SemanticError::UnknownRoot { .. })
        ));
    }
}
