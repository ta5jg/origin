/* =============================================================================
 * File:           crates/origin-core/src/language/catalog.rs
 * Project:        Origin
 * Author:         USDTG GROUP TECHNOLOGY LLC
 * Developer:      Irfan Gedik
 * Created Date:   2026-08-06
 * Version:        0.1.0
 *
 * Description:
 *   Provides deterministic storage, indexing, filtering, duplicate detection,
 *   and integrity validation for Origin language-root records.
 *
 * License:
 *   Origin License v1.0 — see LICENSE in the repository root.
 * ============================================================================= */

//! Deterministic in-memory catalog for Origin language-root records.
//!
//! The catalog validates every inserted [`LanguageRoot`], maintains stable
//! identifier and normalized-root indexes, rejects ambiguous duplicate records,
//! and exposes deterministic query operations for generation campaigns.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::{LanguageId, LanguageRoot, LanguageRootError, MeaningCategory};

/// Current serialized schema version for language catalogs.
pub const LANGUAGE_CATALOG_SCHEMA_VERSION: u16 = 1;

/// Deterministic collection of validated language roots.
///
/// Records are stored in [`BTreeMap`] indexes so iteration and export ordering
/// remain stable across executions and platforms.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageCatalog {
    /// Serialized catalog schema version.
    schema_version: u16,

    /// Primary index keyed by stable root identifier.
    roots: BTreeMap<String, LanguageRoot>,

    /// Secondary index mapping normalized root forms to root identifiers.
    normalized_index: BTreeMap<String, String>,
}

impl LanguageCatalog {
    /// Creates an empty language catalog.
    #[must_use]
    pub fn new() -> Self {
        Self {
            schema_version: LANGUAGE_CATALOG_SCHEMA_VERSION,
            roots: BTreeMap::new(),
            normalized_index: BTreeMap::new(),
        }
    }

    /// Creates a catalog from an iterator of language roots.
    ///
    /// Records are inserted in iterator order, while stored indexes remain
    /// deterministically sorted.
    ///
    /// # Errors
    ///
    /// Returns [`LanguageCatalogError`] when:
    ///
    /// - any root record is internally invalid,
    /// - a duplicate root identifier is encountered,
    /// - or a duplicate normalized root form is encountered.
    pub fn from_roots<I>(roots: I) -> Result<Self, LanguageCatalogError>
    where
        I: IntoIterator<Item = LanguageRoot>,
    {
        let mut catalog = Self::new();

        for root in roots {
            catalog.insert(root)?;
        }

        Ok(catalog)
    }

    /// Inserts a validated language root into the catalog.
    ///
    /// Root identifiers and normalized forms must both be globally unique
    /// within one catalog.
    ///
    /// # Errors
    ///
    /// Returns [`LanguageCatalogError`] when:
    ///
    /// - the root fails internal validation,
    /// - its identifier already exists,
    /// - or its normalized form already belongs to another root.
    pub fn insert(&mut self, root: LanguageRoot) -> Result<(), LanguageCatalogError> {
        root.validate()
            .map_err(|source| LanguageCatalogError::InvalidRoot {
                root_id: root.id.clone(),
                source,
            })?;

        if self.roots.contains_key(&root.id) {
            return Err(LanguageCatalogError::DuplicateId {
                id: root.id.clone(),
            });
        }

        if let Some(existing_id) = self.normalized_index.get(&root.normalized) {
            return Err(LanguageCatalogError::DuplicateNormalizedRoot {
                normalized: root.normalized.clone(),
                existing_id: existing_id.clone(),
                incoming_id: root.id.clone(),
            });
        }

        self.normalized_index
            .insert(root.normalized.clone(), root.id.clone());

        self.roots.insert(root.id.clone(), root);

        Ok(())
    }

    /// Removes a root by identifier.
    ///
    /// Returns the removed root when the identifier existed.
    pub fn remove(&mut self, id: &str) -> Option<LanguageRoot> {
        let root = self.roots.remove(id)?;

        self.normalized_index.remove(&root.normalized);

        Some(root)
    }

    /// Returns a root by its stable identifier.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&LanguageRoot> {
        self.roots.get(id)
    }

    /// Returns a mutable root by identifier.
    ///
    /// Direct mutation may invalidate secondary indexes. Call
    /// [`LanguageCatalog::rebuild_indexes`] after changing an identifier or
    /// normalized root.
    pub fn get_mut(&mut self, id: &str) -> Option<&mut LanguageRoot> {
        self.roots.get_mut(id)
    }

    /// Returns the root matching a normalized root form.
    #[must_use]
    pub fn get_by_normalized(&self, normalized: &str) -> Option<&LanguageRoot> {
        let id = self.normalized_index.get(normalized)?;
        self.roots.get(id)
    }

    /// Returns whether the catalog contains a root identifier.
    #[must_use]
    pub fn contains_id(&self, id: &str) -> bool {
        self.roots.contains_key(id)
    }

    /// Returns whether the catalog contains a normalized root form.
    #[must_use]
    pub fn contains_normalized(&self, normalized: &str) -> bool {
        self.normalized_index.contains_key(normalized)
    }

    /// Returns the number of roots in the catalog.
    #[must_use]
    pub fn len(&self) -> usize {
        self.roots.len()
    }

    /// Returns whether the catalog contains no roots.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.roots.is_empty()
    }

    /// Returns the number of enabled roots.
    #[must_use]
    pub fn enabled_count(&self) -> usize {
        self.roots.values().filter(|root| root.enabled).count()
    }

    /// Returns all roots in deterministic identifier order.
    #[must_use]
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &LanguageRoot> {
        self.roots.values()
    }

    /// Returns all enabled roots in deterministic identifier order.
    #[must_use]
    pub fn enabled_roots(&self) -> Vec<&LanguageRoot> {
        self.roots.values().filter(|root| root.enabled).collect()
    }

    /// Returns all disabled roots in deterministic identifier order.
    #[must_use]
    pub fn disabled_roots(&self) -> Vec<&LanguageRoot> {
        self.roots.values().filter(|root| !root.enabled).collect()
    }

    /// Returns roots belonging to the supplied language.
    #[must_use]
    pub fn by_language(&self, language: &LanguageId) -> Vec<&LanguageRoot> {
        self.roots
            .values()
            .filter(|root| &root.language == language)
            .collect()
    }

    /// Returns enabled roots belonging to the supplied language.
    #[must_use]
    pub fn enabled_by_language(&self, language: &LanguageId) -> Vec<&LanguageRoot> {
        self.roots
            .values()
            .filter(|root| root.enabled && &root.language == language)
            .collect()
    }

    /// Returns roots carrying the supplied semantic category.
    #[must_use]
    pub fn by_category(&self, category: MeaningCategory) -> Vec<&LanguageRoot> {
        self.roots
            .values()
            .filter(|root| root.has_category(category))
            .collect()
    }

    /// Returns enabled roots carrying the supplied semantic category.
    #[must_use]
    pub fn enabled_by_category(&self, category: MeaningCategory) -> Vec<&LanguageRoot> {
        self.roots
            .values()
            .filter(|root| root.enabled && root.has_category(category))
            .collect()
    }

    /// Returns roots carrying the supplied tag.
    ///
    /// Tag normalization is delegated to [`LanguageRoot::has_tag`].
    #[must_use]
    pub fn by_tag(&self, tag: &str) -> Vec<&LanguageRoot> {
        self.roots
            .values()
            .filter(|root| root.has_tag(tag))
            .collect()
    }

    /// Returns enabled roots carrying the supplied tag.
    #[must_use]
    pub fn enabled_by_tag(&self, tag: &str) -> Vec<&LanguageRoot> {
        self.roots
            .values()
            .filter(|root| root.enabled && root.has_tag(tag))
            .collect()
    }

    /// Returns enabled roots satisfying all supplied filters.
    ///
    /// Empty filter collections impose no restriction.
    #[must_use]
    pub fn select(&self, query: &RootQuery) -> Vec<&LanguageRoot> {
        self.roots
            .values()
            .filter(|root| {
                if query.enabled_only && !root.enabled {
                    return false;
                }

                if !query.languages.is_empty() && !query.languages.contains(&root.language) {
                    return false;
                }

                if !query.categories.is_empty()
                    && !query
                        .categories
                        .iter()
                        .all(|category| root.has_category(*category))
                {
                    return false;
                }

                if !query.tags.is_empty() && !query.tags.iter().all(|tag| root.has_tag(tag)) {
                    return false;
                }

                root.confidence.score() >= query.minimum_confidence
            })
            .collect()
    }

    /// Returns the distinct languages represented by the catalog.
    #[must_use]
    pub fn languages(&self) -> BTreeSet<LanguageId> {
        self.roots
            .values()
            .map(|root| root.language.clone())
            .collect()
    }

    /// Returns the distinct semantic categories represented by the catalog.
    #[must_use]
    pub fn categories(&self) -> BTreeSet<MeaningCategory> {
        self.roots
            .values()
            .flat_map(|root| {
                root.meanings
                    .iter()
                    .flat_map(|meaning| meaning.categories.iter().copied())
            })
            .collect()
    }

    /// Returns all root identifiers in deterministic order.
    #[must_use]
    pub fn ids(&self) -> Vec<&str> {
        self.roots.keys().map(String::as_str).collect()
    }

    /// Rebuilds secondary indexes after controlled mutable changes.
    ///
    /// The primary identifier keys are also rebuilt from the records
    /// themselves. No changes are retained when rebuilding fails.
    ///
    /// # Errors
    ///
    /// Returns [`LanguageCatalogError`] when:
    ///
    /// - a root is invalid,
    /// - multiple records now contain the same identifier,
    /// - or multiple records now contain the same normalized form.
    pub fn rebuild_indexes(&mut self) -> Result<(), LanguageCatalogError> {
        let roots = self.roots.values().cloned().collect::<Vec<_>>();
        let rebuilt = Self::from_roots(roots)?;

        *self = rebuilt;

        Ok(())
    }

    /// Validates all catalog records and index invariants.
    ///
    /// # Errors
    ///
    /// Returns [`LanguageCatalogError`] when:
    ///
    /// - the catalog schema version is unsupported,
    /// - any root record is invalid,
    /// - the primary key differs from the record identifier,
    /// - a normalized index entry is missing or incorrect,
    /// - or the primary and secondary index sizes differ.
    pub fn validate(&self) -> Result<(), LanguageCatalogError> {
        if self.schema_version != LANGUAGE_CATALOG_SCHEMA_VERSION {
            return Err(LanguageCatalogError::UnsupportedSchemaVersion {
                found: self.schema_version,
                supported: LANGUAGE_CATALOG_SCHEMA_VERSION,
            });
        }

        if self.roots.len() != self.normalized_index.len() {
            return Err(LanguageCatalogError::IndexSizeMismatch {
                roots: self.roots.len(),
                normalized: self.normalized_index.len(),
            });
        }

        for (key, root) in &self.roots {
            root.validate()
                .map_err(|source| LanguageCatalogError::InvalidRoot {
                    root_id: root.id.clone(),
                    source,
                })?;

            if key != &root.id {
                return Err(LanguageCatalogError::PrimaryIndexMismatch {
                    index_key: key.clone(),
                    root_id: root.id.clone(),
                });
            }

            match self.normalized_index.get(&root.normalized) {
                Some(indexed_id) if indexed_id == &root.id => {}
                Some(indexed_id) => {
                    return Err(LanguageCatalogError::NormalizedIndexMismatch {
                        normalized: root.normalized.clone(),
                        expected_id: root.id.clone(),
                        indexed_id: indexed_id.clone(),
                    });
                }
                None => {
                    return Err(LanguageCatalogError::MissingNormalizedIndex {
                        normalized: root.normalized.clone(),
                        root_id: root.id.clone(),
                    });
                }
            }
        }

        for (normalized, root_id) in &self.normalized_index {
            let Some(root) = self.roots.get(root_id) else {
                return Err(LanguageCatalogError::DanglingNormalizedIndex {
                    normalized: normalized.clone(),
                    root_id: root_id.clone(),
                });
            };

            if &root.normalized != normalized {
                return Err(LanguageCatalogError::NormalizedIndexKeyMismatch {
                    index_key: normalized.clone(),
                    root_normalized: root.normalized.clone(),
                    root_id: root.id.clone(),
                });
            }
        }

        Ok(())
    }
}

/// Deterministic filtering criteria for language-root selection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootQuery {
    /// Restricts results to these languages when non-empty.
    pub languages: BTreeSet<LanguageId>,

    /// Requires every listed semantic category.
    pub categories: BTreeSet<MeaningCategory>,

    /// Requires every listed normalized tag.
    pub tags: BTreeSet<String>,

    /// Minimum accepted linguistic-confidence score.
    pub minimum_confidence: u8,

    /// Whether disabled roots must be excluded.
    pub enabled_only: bool,
}

impl RootQuery {
    /// Creates a query that selects all enabled roots.
    #[must_use]
    pub fn enabled() -> Self {
        Self {
            languages: BTreeSet::new(),
            categories: BTreeSet::new(),
            tags: BTreeSet::new(),
            minimum_confidence: 0,
            enabled_only: true,
        }
    }

    /// Creates a query that may include disabled roots.
    #[must_use]
    pub fn all() -> Self {
        Self {
            enabled_only: false,
            ..Self::enabled()
        }
    }

    /// Adds a required language.
    #[must_use]
    pub fn with_language(mut self, language: LanguageId) -> Self {
        self.languages.insert(language);
        self
    }

    /// Adds a required semantic category.
    #[must_use]
    pub fn with_category(mut self, category: MeaningCategory) -> Self {
        self.categories.insert(category);
        self
    }

    /// Adds a required tag.
    #[must_use]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        let tag = normalize_query_tag(&tag.into());

        if !tag.is_empty() {
            self.tags.insert(tag);
        }

        self
    }

    /// Sets the minimum confidence score.
    #[must_use]
    pub const fn with_minimum_confidence(mut self, minimum_confidence: u8) -> Self {
        self.minimum_confidence = clamp_score(minimum_confidence);
        self
    }

    /// Sets whether only enabled roots are eligible.
    #[must_use]
    pub const fn with_enabled_only(mut self, enabled_only: bool) -> Self {
        self.enabled_only = enabled_only;
        self
    }
}

impl Default for RootQuery {
    fn default() -> Self {
        Self::enabled()
    }
}

/// Error produced while constructing or validating a language catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LanguageCatalogError {
    /// Catalog schema version is unsupported.
    UnsupportedSchemaVersion {
        /// Version found in the catalog.
        found: u16,

        /// Version supported by this implementation.
        supported: u16,
    },

    /// A language root failed its own internal validation.
    InvalidRoot {
        /// Identifier of the invalid root.
        root_id: String,

        /// Underlying root validation error.
        source: LanguageRootError,
    },

    /// Multiple roots use the same stable identifier.
    DuplicateId {
        /// Duplicate identifier.
        id: String,
    },

    /// Multiple roots use the same normalized form.
    DuplicateNormalizedRoot {
        /// Duplicate normalized form.
        normalized: String,

        /// Identifier already owning the normalized form.
        existing_id: String,

        /// Identifier of the incoming root.
        incoming_id: String,
    },

    /// Primary index key differs from the contained root identifier.
    PrimaryIndexMismatch {
        /// Primary index key.
        index_key: String,

        /// Identifier stored inside the root.
        root_id: String,
    },

    /// A root is missing from the normalized-form index.
    MissingNormalizedIndex {
        /// Missing normalized form.
        normalized: String,

        /// Root identifier.
        root_id: String,
    },

    /// A normalized-form index points to an unexpected root.
    NormalizedIndexMismatch {
        /// Normalized form.
        normalized: String,

        /// Expected root identifier.
        expected_id: String,

        /// Actual indexed root identifier.
        indexed_id: String,
    },

    /// A normalized index entry points to a missing root.
    DanglingNormalizedIndex {
        /// Indexed normalized form.
        normalized: String,

        /// Missing root identifier.
        root_id: String,
    },

    /// A normalized index key differs from the referenced root value.
    NormalizedIndexKeyMismatch {
        /// Normalized index key.
        index_key: String,

        /// Normalized form stored in the root.
        root_normalized: String,

        /// Referenced root identifier.
        root_id: String,
    },

    /// Primary and secondary index sizes differ.
    IndexSizeMismatch {
        /// Number of primary root records.
        roots: usize,

        /// Number of normalized index entries.
        normalized: usize,
    },
}

impl std::fmt::Display for LanguageCatalogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { found, supported } => write!(
                formatter,
                "unsupported language-catalog schema version {found}; supported version is {supported}"
            ),
            Self::InvalidRoot { root_id, source } => {
                write!(formatter, "invalid language root {root_id}: {source}")
            }
            Self::DuplicateId { id } => {
                write!(formatter, "duplicate language-root identifier: {id}")
            }
            Self::DuplicateNormalizedRoot {
                normalized,
                existing_id,
                incoming_id,
            } => write!(
                formatter,
                "normalized root {normalized} is already owned by {existing_id}; incoming root is {incoming_id}"
            ),
            Self::PrimaryIndexMismatch { index_key, root_id } => write!(
                formatter,
                "primary index key {index_key} does not match root identifier {root_id}"
            ),
            Self::MissingNormalizedIndex {
                normalized,
                root_id,
            } => write!(
                formatter,
                "normalized index is missing {normalized} for root {root_id}"
            ),
            Self::NormalizedIndexMismatch {
                normalized,
                expected_id,
                indexed_id,
            } => write!(
                formatter,
                "normalized index {normalized} points to {indexed_id}; expected {expected_id}"
            ),
            Self::DanglingNormalizedIndex {
                normalized,
                root_id,
            } => write!(
                formatter,
                "normalized index {normalized} points to missing root {root_id}"
            ),
            Self::NormalizedIndexKeyMismatch {
                index_key,
                root_normalized,
                root_id,
            } => write!(
                formatter,
                "normalized index key {index_key} does not match root value {root_normalized} for {root_id}"
            ),
            Self::IndexSizeMismatch { roots, normalized } => write!(
                formatter,
                "catalog index size mismatch: {roots} roots and {normalized} normalized entries"
            ),
        }
    }
}

impl std::error::Error for LanguageCatalogError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidRoot { source, .. } => Some(source),
            _ => None,
        }
    }
}

const fn clamp_score(score: u8) -> u8 {
    if score > 100 { 100 } else { score }
}

fn normalize_query_tag(input: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::{LanguageCatalog, LanguageCatalogError, RootQuery};
    use crate::{
        ConfidenceBasis, LanguageId, LanguageRoot, MeaningCategory, RootConfidence, RootMeaning,
        RootSource, SourceKind,
    };

    fn source(id: &str) -> RootSource {
        RootSource::new(
            id,
            "Test Lexicon",
            SourceKind::Lexicon,
            "https://example.invalid/lexicon",
        )
    }

    fn latin_via() -> LanguageRoot {
        LanguageRoot::new(
            "latin-via",
            LanguageId::Latin,
            "via",
            "via",
            RootMeaning::literal("road")
                .with_category(MeaningCategory::Movement)
                .with_category(MeaningCategory::Connection),
            RootConfidence::new(95, ConfidenceBasis::Attested),
        )
        .with_source(source("latin-source"))
        .with_tag("logistics")
        .with_tag("enterprise")
    }

    fn old_turkic_yol() -> LanguageRoot {
        LanguageRoot::new(
            "old-turkic-yol",
            LanguageId::OldTurkic,
            "yol",
            "yol",
            RootMeaning::literal("road").with_category(MeaningCategory::Movement),
            RootConfidence::new(93, ConfidenceBasis::Attested),
        )
        .with_source(source("old-turkic-source"))
        .with_tag("logistics")
    }

    fn sumerian_uru() -> LanguageRoot {
        LanguageRoot::new(
            "sumerian-uru",
            LanguageId::Sumerian,
            "uru",
            "uru",
            RootMeaning::literal("city").with_category(MeaningCategory::Place),
            RootConfidence::new(90, ConfidenceBasis::Corroborated),
        )
        .with_source(source("sumerian-source"))
        .with_tag("civilization")
    }

    fn synthetic_vor() -> LanguageRoot {
        LanguageRoot::new(
            "synthetic-vor",
            LanguageId::Synthetic,
            "vor",
            "vor",
            RootMeaning::synthetic("coordinated intelligence")
                .with_category(MeaningCategory::Connection)
                .with_category(MeaningCategory::Intelligence),
            RootConfidence::new(100, ConfidenceBasis::Synthetic),
        )
        .with_tag("technology")
    }

    fn catalog() -> LanguageCatalog {
        LanguageCatalog::from_roots([
            latin_via(),
            old_turkic_yol(),
            sumerian_uru(),
            synthetic_vor(),
        ])
        .expect("fixture catalog must be valid")
    }

    #[test]
    fn empty_catalog_is_valid() {
        let catalog = LanguageCatalog::new();

        assert!(catalog.is_empty());
        assert_eq!(catalog.len(), 0);
        assert!(catalog.validate().is_ok());
    }

    #[test]
    fn roots_are_indexed_by_id_and_normalized_form() {
        let catalog = catalog();

        assert_eq!(
            catalog
                .get("latin-via")
                .map(|root| root.normalized.as_str()),
            Some("via")
        );

        assert_eq!(
            catalog
                .get_by_normalized("yol")
                .map(|root| root.id.as_str()),
            Some("old-turkic-yol")
        );
    }

    #[test]
    fn duplicate_identifier_is_rejected() {
        let root = latin_via();

        let result = LanguageCatalog::from_roots([root.clone(), root]);

        assert_eq!(
            result,
            Err(LanguageCatalogError::DuplicateId {
                id: "latin-via".to_owned(),
            })
        );
    }

    #[test]
    fn duplicate_normalized_form_is_rejected() {
        let duplicate = LanguageRoot::new(
            "synthetic-via",
            LanguageId::Synthetic,
            "via",
            "via",
            RootMeaning::synthetic("synthetic path"),
            RootConfidence::new(100, ConfidenceBasis::Synthetic),
        );

        let result = LanguageCatalog::from_roots([latin_via(), duplicate]);

        assert_eq!(
            result,
            Err(LanguageCatalogError::DuplicateNormalizedRoot {
                normalized: "via".to_owned(),
                existing_id: "latin-via".to_owned(),
                incoming_id: "synthetic-via".to_owned(),
            })
        );
    }

    #[test]
    fn invalid_root_is_rejected_during_insert() {
        let mut root = latin_via();
        root.normalized = "Vía".to_owned();

        let result = LanguageCatalog::from_roots([root]);

        assert!(matches!(
            result,
            Err(LanguageCatalogError::InvalidRoot {
                root_id,
                ..
            }) if root_id == "latin-via"
        ));
    }

    #[test]
    fn iteration_order_is_deterministic_by_identifier() {
        let catalog = catalog();

        assert_eq!(
            catalog.ids(),
            [
                "latin-via",
                "old-turkic-yol",
                "sumerian-uru",
                "synthetic-vor",
            ]
        );
    }

    #[test]
    fn catalog_filters_by_language() {
        let catalog = catalog();

        let roots = catalog.by_language(&LanguageId::OldTurkic);

        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].id, "old-turkic-yol");
    }

    #[test]
    fn catalog_filters_by_category() {
        let catalog = catalog();

        let roots = catalog.by_category(MeaningCategory::Movement);

        assert_eq!(
            roots
                .iter()
                .map(|root| root.id.as_str())
                .collect::<Vec<_>>(),
            ["latin-via", "old-turkic-yol"]
        );
    }

    #[test]
    fn catalog_filters_by_normalized_tag() {
        let catalog = catalog();

        let roots = catalog.by_tag("Enterprise");

        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].id, "latin-via");
    }

    #[test]
    fn disabled_roots_are_excluded_from_enabled_queries() {
        let mut disabled = old_turkic_yol();
        disabled.enabled = false;

        let catalog =
            LanguageCatalog::from_roots([latin_via(), disabled]).expect("catalog must be valid");

        assert_eq!(catalog.len(), 2);
        assert_eq!(catalog.enabled_count(), 1);
        assert_eq!(catalog.enabled_roots()[0].id, "latin-via");
        assert_eq!(catalog.disabled_roots()[0].id, "old-turkic-yol");
    }

    #[test]
    fn compound_query_requires_all_filters() {
        let query = RootQuery::enabled()
            .with_language(LanguageId::Latin)
            .with_category(MeaningCategory::Movement)
            .with_tag("enterprise software")
            .with_minimum_confidence(90);

        let mut matching_root = latin_via();
        matching_root.tags.insert("enterprise-software".to_owned());

        let catalog =
            LanguageCatalog::from_roots([matching_root, old_turkic_yol(), sumerian_uru()])
                .expect("catalog must be valid");

        let results = catalog.select(&query);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "latin-via");
        assert_eq!(catalog.len(), 3);
    }

    #[test]
    fn query_can_include_disabled_roots() {
        let mut disabled = old_turkic_yol();
        disabled.enabled = false;

        let catalog = LanguageCatalog::from_roots([disabled]).expect("catalog must be valid");

        assert!(catalog.select(&RootQuery::enabled()).is_empty());

        assert_eq!(catalog.select(&RootQuery::all()).len(), 1);
    }

    #[test]
    fn catalog_reports_represented_languages_and_categories() {
        let catalog = catalog();

        assert!(catalog.languages().contains(&LanguageId::Latin));
        assert!(catalog.languages().contains(&LanguageId::Synthetic));

        assert!(catalog.categories().contains(&MeaningCategory::Movement));

        assert!(
            catalog
                .categories()
                .contains(&MeaningCategory::Intelligence)
        );
    }

    #[test]
    fn removal_updates_both_indexes() {
        let mut catalog = catalog();

        let removed = catalog.remove("latin-via").expect("root must exist");

        assert_eq!(removed.normalized, "via");
        assert!(!catalog.contains_id("latin-via"));
        assert!(!catalog.contains_normalized("via"));
        assert!(catalog.validate().is_ok());
    }

    #[test]
    fn rebuilding_indexes_applies_mutated_normalized_value() {
        let mut catalog = catalog();

        let root = catalog.get_mut("latin-via").expect("root must exist");

        root.normalized = "viara".to_owned();

        assert!(catalog.get_by_normalized("viara").is_none());

        catalog
            .rebuild_indexes()
            .expect("rebuilt catalog must be valid");

        assert!(!catalog.contains_normalized("via"));
        assert!(catalog.contains_normalized("viara"));
        assert!(catalog.validate().is_ok());
    }
}
