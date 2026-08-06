/* =============================================================================
 * File:           crates/origin-core/src/morphology/merge.rs
 * Project:        Origin
 * Author:         USDTG GROUP TECHNOLOGY LLC
 * Developer:      Irfan Gedik
 * Created Date:   2026-08-07
 * Version:        0.1.0
 *
 * Description:
 *   Coordinates root normalization and boundary collapse as one deterministic
 *   morphology merge pipeline with unified provenance and typed errors.
 *
 * License:
 *   Origin License v1.0 — see LICENSE in the repository root.
 * ============================================================================= */

//! Canonical root-merging pipeline for Origin.
//!
//! This module combines the normalization and collapse stages behind one
//! stable API. Raw historical or synthetic roots are normalized first, then
//! merged according to the active collapse policy.
//!
//! The resulting report preserves the outputs and provenance of every stage.

use serde::{Deserialize, Serialize};

use super::{
    CollapseError, CollapsePolicy, CollapseReport, NormalizationError, NormalizationPolicy,
    NormalizationReport, collapse_roots_with_policy, normalize_root_with_policy,
};

/// Configuration controlling the complete morphology merge pipeline.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergePolicy {
    /// Policy applied independently to both input roots.
    pub normalization: NormalizationPolicy,

    /// Policy applied when joining the normalized roots.
    pub collapse: CollapsePolicy,
}

impl MergePolicy {
    /// Returns Origin's standard merge policy.
    #[must_use]
    pub const fn standard() -> Self {
        Self {
            normalization: NormalizationPolicy::standard(),
            collapse: CollapsePolicy::standard(),
        }
    }

    /// Sets the normalization policy.
    #[must_use]
    pub const fn with_normalization(mut self, normalization: NormalizationPolicy) -> Self {
        self.normalization = normalization;
        self
    }

    /// Sets the collapse policy.
    #[must_use]
    pub const fn with_collapse(mut self, collapse: CollapsePolicy) -> Self {
        self.collapse = collapse;
        self
    }
}

/// Complete result of a two-root merge operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeReport {
    /// Original left input.
    pub left_original: String,

    /// Original right input.
    pub right_original: String,

    /// Normalization report for the left root.
    pub left_normalization: NormalizationReport,

    /// Normalization report for the right root.
    pub right_normalization: NormalizationReport,

    /// Collapse report for the normalized roots.
    pub collapse: CollapseReport,
}

impl MergeReport {
    /// Returns the final canonical merged root.
    #[must_use]
    pub fn merged(&self) -> &str {
        &self.collapse.collapsed
    }

    /// Returns whether any normalization or collapse transformation occurred.
    #[must_use]
    pub fn changed(&self) -> bool {
        self.left_normalization.is_lossy()
            || self.right_normalization.is_lossy()
            || self.collapse.changed()
    }

    /// Returns all normalization and collapse provenance in stable order.
    #[must_use]
    pub fn provenance_steps(&self) -> Vec<String> {
        let mut steps = Vec::new();

        steps.extend(
            self.left_normalization
                .provenance_steps()
                .into_iter()
                .map(|step| format!("left:{step}")),
        );

        steps.extend(
            self.right_normalization
                .provenance_steps()
                .into_iter()
                .map(|step| format!("right:{step}")),
        );

        steps.extend(
            self.collapse
                .provenance_steps()
                .into_iter()
                .map(|step| format!("merge:{step}")),
        );

        steps
    }

    /// Returns the total number of source characters removed.
    #[must_use]
    pub fn removed_character_count(&self) -> usize {
        self.left_normalization.removed_count()
            + self.right_normalization.removed_count()
            + self.collapse.removed_character_count()
    }
}

/// Stage at which a merge operation failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MergeStage {
    /// Normalization of the left root.
    LeftNormalization,

    /// Normalization of the right root.
    RightNormalization,

    /// Collapse of the two normalized roots.
    Collapse,
}

/// Error produced by the complete merge pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeError {
    /// Left-root normalization failed.
    LeftNormalization {
        /// Underlying normalization error.
        source: NormalizationError,
    },

    /// Right-root normalization failed.
    RightNormalization {
        /// Underlying normalization error.
        source: NormalizationError,
    },

    /// Boundary collapse failed.
    Collapse {
        /// Underlying collapse error.
        source: CollapseError,
    },
}

impl MergeError {
    /// Returns the stage that produced this error.
    #[must_use]
    pub const fn stage(&self) -> MergeStage {
        match self {
            Self::LeftNormalization { .. } => MergeStage::LeftNormalization,
            Self::RightNormalization { .. } => MergeStage::RightNormalization,
            Self::Collapse { .. } => MergeStage::Collapse,
        }
    }
}

impl std::fmt::Display for MergeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LeftNormalization { source } => {
                write!(formatter, "left-root normalization failed: {source}")
            }
            Self::RightNormalization { source } => {
                write!(formatter, "right-root normalization failed: {source}")
            }
            Self::Collapse { source } => {
                write!(formatter, "root collapse failed: {source}")
            }
        }
    }
}

impl std::error::Error for MergeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::LeftNormalization { source } | Self::RightNormalization { source } => {
                Some(source)
            }
            Self::Collapse { source } => Some(source),
        }
    }
}

/// Merges two roots using [`MergePolicy::standard`].
///
/// # Errors
///
/// Returns [`MergeError`] when either root cannot be normalized or the
/// normalized roots cannot be collapsed.
pub fn merge_roots(left: &str, right: &str) -> Result<String, MergeError> {
    merge_roots_with_policy(left, right, MergePolicy::standard())
        .map(|report| report.collapse.collapsed)
}

/// Runs the complete normalization and collapse pipeline.
///
/// # Errors
///
/// Returns [`MergeError`] when:
///
/// - normalization of the left root fails,
/// - normalization of the right root fails,
/// - or collapse of the normalized roots fails.
pub fn merge_roots_with_policy(
    left: &str,
    right: &str,
    policy: MergePolicy,
) -> Result<MergeReport, MergeError> {
    let left_normalization = normalize_root_with_policy(left, policy.normalization)
        .map_err(|source| MergeError::LeftNormalization { source })?;

    let right_normalization = normalize_root_with_policy(right, policy.normalization)
        .map_err(|source| MergeError::RightNormalization { source })?;

    let collapse = collapse_roots_with_policy(
        &left_normalization.normalized,
        &right_normalization.normalized,
        policy.collapse,
    )
    .map_err(|source| MergeError::Collapse { source })?;

    Ok(MergeReport {
        left_original: left.to_owned(),
        right_original: right.to_owned(),
        left_normalization,
        right_normalization,
        collapse,
    })
}

#[cfg(test)]
mod tests {
    use super::{MergeError, MergePolicy, MergeStage, merge_roots, merge_roots_with_policy};
    use crate::{CollapsePolicy, NormalizationError, NormalizationMode, NormalizationPolicy};

    #[test]
    fn canonical_roots_merge_successfully() {
        assert_eq!(
            merge_roots("kut", "tora").expect("roots must merge"),
            "kutora"
        );
    }

    #[test]
    fn raw_unicode_roots_are_normalized_before_merge() {
        let report = merge_roots_with_policy("Šar", "Ōra", MergePolicy::standard())
            .expect("roots must merge");

        assert_eq!(report.left_normalization.normalized, "sar");
        assert_eq!(report.right_normalization.normalized, "ora");
        assert_eq!(report.merged(), "sarora");
        assert!(report.changed());
    }

    #[test]
    fn boundary_overlap_is_applied_after_normalization() {
        let report = merge_roots_with_policy("Vör", "Ōrbis", MergePolicy::standard())
            .expect("roots must merge");

        assert_eq!(report.left_normalization.normalized, "vor");
        assert_eq!(report.right_normalization.normalized, "orbis");
        assert_eq!(report.merged(), "vorbis");
    }

    #[test]
    fn merge_is_deterministic() {
        let policy = MergePolicy::standard();

        let first =
            merge_roots_with_policy("Šar", "Ōrbis", policy).expect("first merge must succeed");

        let second =
            merge_roots_with_policy("Šar", "Ōrbis", policy).expect("second merge must succeed");

        assert_eq!(first, second);
    }

    #[test]
    fn provenance_order_is_stable() {
        let report = merge_roots_with_policy("vor", "Ōrbis", MergePolicy::standard())
            .expect("roots must merge");

        assert_eq!(report.merged(), "vorbis");
        assert_eq!(
            report.provenance_steps(),
            [
                "right:normalize:0:Ō>o",
                "merge:collapse:boundary-overlap:3:or",
            ]
        );
    }

    #[test]
    fn unchanged_roots_report_no_transformation() {
        let report = merge_roots_with_policy("kut", "ora", MergePolicy::standard())
            .expect("roots must merge");

        assert_eq!(report.merged(), "kutora");
        assert!(!report.changed());
        assert!(report.provenance_steps().is_empty());
    }

    #[test]
    fn removed_character_count_combines_all_stages() {
        let report = merge_roots_with_policy("vör-", "Ōrbis", MergePolicy::standard())
            .expect("roots must merge");

        assert_eq!(report.left_normalization.normalized, "vor");
        assert_eq!(report.right_normalization.normalized, "orbis");
        assert_eq!(report.merged(), "vorbis");

        // One separator removed during normalization and two overlapping
        // characters removed during collapse.
        assert_eq!(report.removed_character_count(), 3);
    }

    #[test]
    fn collapse_policy_can_be_disabled() {
        let policy = MergePolicy::standard().with_collapse(CollapsePolicy::disabled());

        let report = merge_roots_with_policy("vor", "orbis", policy).expect("roots must merge");

        assert_eq!(report.merged(), "vororbis");
        assert!(!report.collapse.changed());
    }

    #[test]
    fn normalization_policy_can_be_strict() {
        let policy = MergePolicy::standard()
            .with_normalization(NormalizationPolicy::strict().with_mode(NormalizationMode::Strict));

        let result = merge_roots_with_policy("via☃", "ora", policy);

        assert_eq!(
            result,
            Err(MergeError::LeftNormalization {
                source: NormalizationError::UnsupportedCharacter {
                    character: '☃',
                    byte_offset: 3,
                },
            })
        );
    }

    #[test]
    fn empty_left_root_reports_left_stage() {
        let error = merge_roots_with_policy("", "ora", MergePolicy::standard())
            .expect_err("empty left root must fail");

        assert_eq!(error.stage(), MergeStage::LeftNormalization);
        assert_eq!(
            error,
            MergeError::LeftNormalization {
                source: NormalizationError::EmptyInput,
            }
        );
    }

    #[test]
    fn empty_right_root_reports_right_stage() {
        let error = merge_roots_with_policy("via", "", MergePolicy::standard())
            .expect_err("empty right root must fail");

        assert_eq!(error.stage(), MergeStage::RightNormalization);
        assert_eq!(
            error,
            MergeError::RightNormalization {
                source: NormalizationError::EmptyInput,
            }
        );
    }

    #[test]
    fn standard_policy_equals_default_policy() {
        assert_eq!(MergePolicy::standard(), MergePolicy::default());
    }
}
