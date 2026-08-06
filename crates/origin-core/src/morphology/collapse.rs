/* =============================================================================
 * File:           crates/origin-core/src/morphology/collapse.rs
 * Project:        Origin
 * Author:         USDTG GROUP TECHNOLOGY LLC
 * Developer:      Irfan Gedik
 * Created Date:   2026-08-06
 * Version:        0.1.0
 *
 * Description:
 *   Collapses mechanical repetitions, overlapping root boundaries, repeated
 *   syllables, and awkward adjacent sound sequences during morphology.
 *
 * License:
 *   Origin License v1.0 — see LICENSE in the repository root.
 * ============================================================================= */

//! Deterministic morphology-boundary collapsing for Origin.
//!
//! This module removes mechanical duplication introduced when roots or
//! morphemes are combined. It operates only on canonical lowercase ASCII roots
//! and reports every transformation in stable order.
//!
//! The collapse engine does not score names, mutate arbitrary phonemes, or
//! perform aesthetic beautification.

use serde::{Deserialize, Serialize};

use super::is_canonical_root;

/// Maximum supported overlap length at a root boundary.
pub const MAX_BOUNDARY_OVERLAP: usize = 4;

/// Configuration controlling morphology collapse behavior.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]

pub struct CollapsePolicy {
    /// Removes the longest shared suffix-prefix overlap.
    pub collapse_boundary_overlap: bool,

    /// Collapses identical vowels at the join boundary.
    pub collapse_duplicate_vowels: bool,

    /// Collapses identical consonants at the join boundary.
    pub collapse_duplicate_consonants: bool,

    /// Removes immediately repeated short syllables.
    pub collapse_repeated_syllables: bool,

    /// Maximum overlap length considered at the boundary.
    pub maximum_overlap: usize,
}

impl CollapsePolicy {
    /// Returns Origin's standard morphology-collapse policy.
    #[must_use]
    pub const fn standard() -> Self {
        Self {
            collapse_boundary_overlap: true,
            collapse_duplicate_vowels: true,
            collapse_duplicate_consonants: true,
            collapse_repeated_syllables: true,
            maximum_overlap: MAX_BOUNDARY_OVERLAP,
        }
    }

    /// Disables all collapse behavior.
    #[must_use]
    pub const fn disabled() -> Self {
        Self {
            collapse_boundary_overlap: false,
            collapse_duplicate_vowels: false,
            collapse_duplicate_consonants: false,
            collapse_repeated_syllables: false,
            maximum_overlap: 0,
        }
    }

    /// Sets whether suffix-prefix overlaps are collapsed.
    #[must_use]
    pub const fn with_boundary_overlap(mut self, enabled: bool) -> Self {
        self.collapse_boundary_overlap = enabled;
        self
    }

    /// Sets whether duplicate vowels are collapsed.
    #[must_use]
    pub const fn with_duplicate_vowels(mut self, enabled: bool) -> Self {
        self.collapse_duplicate_vowels = enabled;
        self
    }

    /// Sets whether duplicate consonants are collapsed.
    #[must_use]
    pub const fn with_duplicate_consonants(mut self, enabled: bool) -> Self {
        self.collapse_duplicate_consonants = enabled;
        self
    }

    /// Sets whether repeated syllables are collapsed.
    #[must_use]
    pub const fn with_repeated_syllables(mut self, enabled: bool) -> Self {
        self.collapse_repeated_syllables = enabled;
        self
    }

    /// Sets the maximum suffix-prefix overlap length.
    #[must_use]
    pub const fn with_maximum_overlap(mut self, maximum_overlap: usize) -> Self {
        self.maximum_overlap = maximum_overlap;
        self
    }
}

impl Default for CollapsePolicy {
    fn default() -> Self {
        Self::standard()
    }
}

/// Classification of a collapse transformation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CollapseKind {
    /// A shared suffix-prefix boundary was removed.
    BoundaryOverlap,

    /// An identical vowel at the join boundary was removed.
    DuplicateVowel,

    /// An identical consonant at the join boundary was removed.
    DuplicateConsonant,

    /// An immediately repeated short syllable was removed.
    RepeatedSyllable,
}

/// One explainable collapse operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollapseChange {
    /// Transformation classification.
    pub kind: CollapseKind,

    /// Zero-based byte position where the transformation occurred.
    pub position: usize,

    /// Removed text.
    pub removed: String,

    /// Human-readable explanation.
    pub explanation: String,
}

impl CollapseChange {
    fn new(
        kind: CollapseKind,
        position: usize,
        removed: impl Into<String>,
        explanation: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            position,
            removed: removed.into(),
            explanation: explanation.into(),
        }
    }
}

/// Complete result of one collapse operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollapseReport {
    /// Canonical left input root.
    pub left: String,

    /// Canonical right input root.
    pub right: String,

    /// Direct concatenation before collapse.
    pub original: String,

    /// Final collapsed form.
    pub collapsed: String,

    /// Ordered transformation history.
    pub changes: Vec<CollapseChange>,
}

impl CollapseReport {
    /// Returns whether any collapse transformation occurred.
    #[must_use]
    pub fn changed(&self) -> bool {
        !self.changes.is_empty()
    }

    /// Returns the number of removed ASCII characters.
    #[must_use]
    pub fn removed_character_count(&self) -> usize {
        self.changes.iter().map(|change| change.removed.len()).sum()
    }

    /// Converts collapse changes into stable provenance descriptions.
    #[must_use]
    pub fn provenance_steps(&self) -> Vec<String> {
        self.changes
            .iter()
            .map(|change| {
                format!(
                    "collapse:{}:{}:{}",
                    collapse_kind_name(change.kind),
                    change.position,
                    change.removed
                )
            })
            .collect()
    }
}

/// Error produced while collapsing roots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollapseError {
    /// Left root is empty.
    EmptyLeft,

    /// Right root is empty.
    EmptyRight,

    /// Left root is not canonical lowercase ASCII.
    InvalidLeft {
        /// Invalid value.
        value: String,
    },

    /// Right root is not canonical lowercase ASCII.
    InvalidRight {
        /// Invalid value.
        value: String,
    },

    /// Collapse unexpectedly produced an empty result.
    EmptyOutput,
}

impl std::fmt::Display for CollapseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyLeft => formatter.write_str("left root must not be empty"),
            Self::EmptyRight => formatter.write_str("right root must not be empty"),
            Self::InvalidLeft { value } => write!(
                formatter,
                "left root must contain lowercase ASCII letters only: {value}"
            ),
            Self::InvalidRight { value } => write!(
                formatter,
                "right root must contain lowercase ASCII letters only: {value}"
            ),
            Self::EmptyOutput => formatter.write_str("collapse produced an empty output"),
        }
    }
}

impl std::error::Error for CollapseError {}

/// Collapses two canonical roots using [`CollapsePolicy::standard`].
///
/// # Errors
///
/// Returns [`CollapseError`] when either root is empty, non-canonical, or the
/// operation unexpectedly produces an empty result.
pub fn collapse_roots(left: &str, right: &str) -> Result<String, CollapseError> {
    collapse_roots_with_policy(left, right, CollapsePolicy::standard())
        .map(|report| report.collapsed)
}

/// Collapses two canonical roots and returns a complete transformation report.
///
/// # Errors
///
/// Returns [`CollapseError`] when:
///
/// - either input is empty,
/// - either input is not canonical lowercase ASCII,
/// - or collapse unexpectedly produces an empty result.
pub fn collapse_roots_with_policy(
    left: &str,
    right: &str,
    policy: CollapsePolicy,
) -> Result<CollapseReport, CollapseError> {
    validate_inputs(left, right)?;

    let original = format!("{left}{right}");
    let mut collapsed = original.clone();
    let mut changes = Vec::new();
    let mut boundary = left.len();

    if policy.collapse_boundary_overlap {
        let maximum = policy.maximum_overlap.min(left.len()).min(right.len());

        if let Some(overlap) = longest_boundary_overlap(left, right, maximum) {
            let removed_start = boundary;
            let removed = right[..overlap].to_owned();

            collapsed.replace_range(removed_start..removed_start + overlap, "");
            changes.push(CollapseChange::new(
                CollapseKind::BoundaryOverlap,
                removed_start,
                removed.clone(),
                format!(
                    "removed shared boundary overlap {removed:?} between {left:?} and {right:?}"
                ),
            ));
        }
    }

    boundary = collapsed_boundary_position(left, right, &collapsed);

    if boundary > 0 && boundary < collapsed.len() {
        let left_character = collapsed.as_bytes()[boundary - 1];
        let right_character = collapsed.as_bytes()[boundary];

        if left_character == right_character {
            let character = char::from(right_character);

            if is_vowel_byte(right_character) && policy.collapse_duplicate_vowels {
                collapsed.remove(boundary);
                changes.push(CollapseChange::new(
                    CollapseKind::DuplicateVowel,
                    boundary,
                    character.to_string(),
                    format!("removed duplicate boundary vowel {character:?}"),
                ));
            } else if !is_vowel_byte(right_character) && policy.collapse_duplicate_consonants {
                collapsed.remove(boundary);
                changes.push(CollapseChange::new(
                    CollapseKind::DuplicateConsonant,
                    boundary,
                    character.to_string(),
                    format!("removed duplicate boundary consonant {character:?}"),
                ));
            }
        }
    }

    if policy.collapse_repeated_syllables {
        collapse_repeated_short_syllables(&mut collapsed, &mut changes);
    }

    if collapsed.is_empty() {
        return Err(CollapseError::EmptyOutput);
    }

    Ok(CollapseReport {
        left: left.to_owned(),
        right: right.to_owned(),
        original,
        collapsed,
        changes,
    })
}

fn validate_inputs(left: &str, right: &str) -> Result<(), CollapseError> {
    if left.is_empty() {
        return Err(CollapseError::EmptyLeft);
    }

    if right.is_empty() {
        return Err(CollapseError::EmptyRight);
    }

    if !is_canonical_root(left) {
        return Err(CollapseError::InvalidLeft {
            value: left.to_owned(),
        });
    }

    if !is_canonical_root(right) {
        return Err(CollapseError::InvalidRight {
            value: right.to_owned(),
        });
    }

    Ok(())
}

fn longest_boundary_overlap(left: &str, right: &str, maximum: usize) -> Option<usize> {
    (2..=maximum)
        .rev()
        .find(|&length| left[left.len() - length..] == right[..length])
}

fn collapsed_boundary_position(left: &str, right: &str, collapsed: &str) -> usize {
    let direct_length = left.len() + right.len();
    let removed = direct_length.saturating_sub(collapsed.len());

    left.len().saturating_sub(removed.min(right.len()))
}

fn collapse_repeated_short_syllables(value: &mut String, changes: &mut Vec<CollapseChange>) {
    let mut position = 0;

    while position < value.len() {
        let mut collapsed_at_position = false;

        for syllable_length in (2..=3).rev() {
            let repeated_length = syllable_length * 2;

            if position + repeated_length > value.len() {
                continue;
            }

            let first = &value[position..position + syllable_length];
            let second = &value[position + syllable_length..position + repeated_length];

            if first == second && contains_vowel(first) {
                let removed = second.to_owned();
                let remove_start = position + syllable_length;

                value.replace_range(remove_start..remove_start + syllable_length, "");

                changes.push(CollapseChange::new(
                    CollapseKind::RepeatedSyllable,
                    remove_start,
                    removed.clone(),
                    format!("removed immediately repeated syllable {removed:?}"),
                ));

                collapsed_at_position = true;
                break;
            }
        }

        if !collapsed_at_position {
            position += 1;
        }
    }
}

fn contains_vowel(value: &str) -> bool {
    value.bytes().any(is_vowel_byte)
}

const fn is_vowel_byte(byte: u8) -> bool {
    matches!(byte, b'a' | b'e' | b'i' | b'o' | b'u')
}

const fn collapse_kind_name(kind: CollapseKind) -> &'static str {
    match kind {
        CollapseKind::BoundaryOverlap => "boundary-overlap",
        CollapseKind::DuplicateVowel => "duplicate-vowel",
        CollapseKind::DuplicateConsonant => "duplicate-consonant",
        CollapseKind::RepeatedSyllable => "repeated-syllable",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CollapseError, CollapseKind, CollapsePolicy, collapse_roots, collapse_roots_with_policy,
    };

    #[test]
    fn roots_without_overlap_are_concatenated() {
        let report = collapse_roots_with_policy("kut", "ora", CollapsePolicy::standard())
            .expect("roots must collapse");

        assert_eq!(report.original, "kutora");
        assert_eq!(report.collapsed, "kutora");
        assert!(!report.changed());
    }

    #[test]
    fn longest_boundary_overlap_is_removed() {
        let report = collapse_roots_with_policy("vor", "orbis", CollapsePolicy::standard())
            .expect("roots must collapse");

        assert_eq!(report.collapsed, "vorbis");
        assert_eq!(report.changes.len(), 1);
        assert_eq!(report.changes[0].kind, CollapseKind::BoundaryOverlap);
        assert_eq!(report.changes[0].removed, "or");
    }

    #[test]
    fn longer_overlap_is_preferred() {
        let report = collapse_roots_with_policy("velora", "oralis", CollapsePolicy::standard())
            .expect("roots must collapse");

        assert_eq!(report.collapsed, "veloralis");
        assert_eq!(report.changes[0].removed, "ora");
    }

    #[test]
    fn one_character_overlap_is_not_treated_as_boundary_overlap() {
        let report = collapse_roots_with_policy("via", "arena", CollapsePolicy::standard())
            .expect("roots must collapse");

        assert_eq!(report.collapsed, "viarena");
        assert_eq!(report.changes[0].kind, CollapseKind::DuplicateVowel);
        assert_eq!(report.changes[0].removed, "a");
    }

    #[test]
    fn duplicate_boundary_vowel_is_collapsed() {
        assert_eq!(
            collapse_roots("via", "arena").expect("roots must collapse"),
            "viarena"
        );
    }

    #[test]
    fn duplicate_boundary_consonant_is_collapsed() {
        let report = collapse_roots_with_policy("kut", "tora", CollapsePolicy::standard())
            .expect("roots must collapse");

        assert_eq!(report.collapsed, "kutora");
        assert_eq!(report.changes[0].kind, CollapseKind::DuplicateConsonant);
    }

    #[test]
    fn repeated_two_letter_syllable_is_collapsed() {
        let report = collapse_roots_with_policy(
            "nara",
            "raka",
            CollapsePolicy::standard()
                .with_boundary_overlap(false)
                .with_duplicate_vowels(false)
                .with_duplicate_consonants(false),
        )
        .expect("roots must collapse");

        assert_eq!(report.collapsed, "naraka");
        assert!(
            report
                .changes
                .iter()
                .any(|change| change.kind == CollapseKind::RepeatedSyllable)
        );
    }

    #[test]
    fn repeated_three_letter_syllable_is_collapsed() {
        let report = collapse_roots_with_policy(
            "vel",
            "velora",
            CollapsePolicy::standard()
                .with_boundary_overlap(false)
                .with_duplicate_vowels(false)
                .with_duplicate_consonants(false),
        )
        .expect("roots must collapse");

        assert_eq!(report.collapsed, "velora");
        assert_eq!(report.changes[0].kind, CollapseKind::RepeatedSyllable);
    }

    #[test]
    fn repeated_syllable_without_vowel_is_preserved() {
        let report = collapse_roots_with_policy("str", "str", CollapsePolicy::standard())
            .expect("roots must collapse");

        assert_eq!(report.collapsed, "str");
        assert_eq!(report.changes[0].kind, CollapseKind::BoundaryOverlap);
    }

    #[test]
    fn overlap_can_be_disabled() {
        let report = collapse_roots_with_policy(
            "vor",
            "orbis",
            CollapsePolicy::standard()
                .with_boundary_overlap(false)
                .with_repeated_syllables(false),
        )
        .expect("roots must collapse");

        assert_eq!(report.collapsed, "vororbis");
        assert!(!report.changed());
    }

    #[test]
    fn duplicate_vowel_collapse_can_be_disabled() {
        let report = collapse_roots_with_policy(
            "via",
            "arena",
            CollapsePolicy::standard()
                .with_boundary_overlap(false)
                .with_duplicate_vowels(false),
        )
        .expect("roots must collapse");

        assert_eq!(report.collapsed, "viaarena");
    }

    #[test]
    fn duplicate_consonant_collapse_can_be_disabled() {
        let report = collapse_roots_with_policy(
            "kut",
            "tora",
            CollapsePolicy::standard()
                .with_boundary_overlap(false)
                .with_duplicate_consonants(false),
        )
        .expect("roots must collapse");

        assert_eq!(report.collapsed, "kuttora");
    }

    #[test]
    fn disabled_policy_preserves_direct_concatenation() {
        let report = collapse_roots_with_policy("vor", "orbis", CollapsePolicy::disabled())
            .expect("roots must concatenate");

        assert_eq!(report.collapsed, "vororbis");
        assert!(report.changes.is_empty());
    }

    #[test]
    fn maximum_overlap_is_respected() {
        let report = collapse_roots_with_policy(
            "velor",
            "oralis",
            CollapsePolicy::standard()
                .with_maximum_overlap(2)
                .with_duplicate_vowels(false)
                .with_duplicate_consonants(false)
                .with_repeated_syllables(false),
        )
        .expect("roots must collapse");

        assert_eq!(report.collapsed, "veloralis");
        assert_eq!(report.changes.len(), 1);
        assert_eq!(report.changes[0].kind, CollapseKind::BoundaryOverlap);
        assert_eq!(report.changes[0].removed, "or");
    }

    #[test]
    fn provenance_steps_are_stable() {
        let report = collapse_roots_with_policy("vor", "orbis", CollapsePolicy::standard())
            .expect("roots must collapse");

        assert_eq!(
            report.provenance_steps(),
            ["collapse:boundary-overlap:3:or"]
        );
    }

    #[test]
    fn removed_character_count_is_reported() {
        let report = collapse_roots_with_policy("vor", "orbis", CollapsePolicy::standard())
            .expect("roots must collapse");

        assert_eq!(report.removed_character_count(), 2);
    }

    #[test]
    fn empty_left_root_is_rejected() {
        assert_eq!(collapse_roots("", "via"), Err(CollapseError::EmptyLeft));
    }

    #[test]
    fn empty_right_root_is_rejected() {
        assert_eq!(collapse_roots("via", ""), Err(CollapseError::EmptyRight));
    }

    #[test]
    fn invalid_left_root_is_rejected() {
        assert_eq!(
            collapse_roots("Vía", "ora"),
            Err(CollapseError::InvalidLeft {
                value: "Vía".to_owned(),
            })
        );
    }

    #[test]
    fn invalid_right_root_is_rejected() {
        assert_eq!(
            collapse_roots("via", "Or-A"),
            Err(CollapseError::InvalidRight {
                value: "Or-A".to_owned(),
            })
        );
    }
}
