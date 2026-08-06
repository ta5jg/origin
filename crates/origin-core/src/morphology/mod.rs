/* =============================================================================
 * File:           crates/origin-core/src/morphology/mod.rs
 * Project:        Origin
 * Author:         USDTG GROUP TECHNOLOGY LLC
 * Developer:      Irfan Gedik
 * Created Date:   2026-08-06
 * Version:        0.1.0
 *
 * Description:
 *   Defines the public morphology and root-transformation interface.
 *
 * License:
 *   Origin License v1.0 — see LICENSE in the repository root.
 * ============================================================================= */

//! Deterministic normalization and morphology infrastructure.

mod collapse;
mod merge;
mod mutate;
mod normalize;

pub use collapse::{
    CollapseChange, CollapseError, CollapseKind, CollapsePolicy, CollapseReport,
    MAX_BOUNDARY_OVERLAP, collapse_roots, collapse_roots_with_policy,
};

pub use merge::{
    MergeError, MergePolicy, MergeReport, MergeStage, merge_roots, merge_roots_with_policy,
};

pub use mutate::{
    DEFAULT_MAXIMUM_MUTATION_LENGTH, DEFAULT_MINIMUM_MUTATION_LENGTH, MorphMutationKind,
    MutationChange, MutationError, MutationPolicy, MutationReport, mutate_root,
    mutate_root_with_kind, mutate_root_with_policy,
};

pub use normalize::{
    NormalizationChange, NormalizationChangeKind, NormalizationError, NormalizationMode,
    NormalizationPolicy, NormalizationReport, is_canonical_root, normalize_root,
    normalize_root_with_policy,
};
