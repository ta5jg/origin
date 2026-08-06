/* =============================================================================
 * File:           crates/origin-core/src/language/mod.rs
 * Project:        Origin
 * Author:         USDTG GROUP TECHNOLOGY LLC
 * Developer:      Irfan Gedik
 * Created Date:   2026-08-06
 * Version:        0.1.0
 *
 * Description:
 *   Defines the public language-root dataset interface.
 *
 * License:
 *   Origin License v1.0 — see LICENSE in the repository root.
 * ============================================================================= */

//! Language-root models, deterministic catalogs, and dataset infrastructure.

mod catalog;
mod model;

pub use catalog::{
    LANGUAGE_CATALOG_SCHEMA_VERSION, LanguageCatalog, LanguageCatalogError, RootQuery,
};

pub use model::{
    ConfidenceBasis, LANGUAGE_ROOT_SCHEMA_VERSION, LanguageId, LanguageRoot, LanguageRootError,
    MeaningCategory, MeaningInterpretation, RootConfidence, RootMeaning, RootSource, SourceKind,
    Transliteration,
};
