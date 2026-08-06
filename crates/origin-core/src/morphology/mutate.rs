/* =============================================================================
 * File:           crates/origin-core/src/morphology/mutate.rs
 * Project:        Origin
 * Author:         USDTG GROUP TECHNOLOGY LLC
 * Developer:      Irfan Gedik
 * Created Date:   2026-08-07
 * Version:        0.1.0
 *
 * Description:
 *   Applies deterministic, seed-driven, single-step phonetic and structural
 *   mutations to canonical Origin roots with explainable provenance.
 *
 * License:
 *   Origin License v1.0 — see LICENSE in the repository root.
 * ============================================================================= */

//! Deterministic single-step morphology mutation.
//!
//! The mutation engine transforms one canonical lowercase ASCII root through
//! an explicitly selected or seed-selected mutation rule. Every successful
//! mutation changes the input and returns stable provenance.
//!
//! This module does not perform candidate scoring, external validation, or
//! multi-generation evolutionary search.

use serde::{Deserialize, Serialize};

use super::is_canonical_root;

/// Minimum canonical root length accepted by the standard mutation policy.
pub const DEFAULT_MINIMUM_MUTATION_LENGTH: usize = 3;

/// Maximum canonical root length accepted by the standard mutation policy.
pub const DEFAULT_MAXIMUM_MUTATION_LENGTH: usize = 12;

/// Supported deterministic morphology mutations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MorphMutationKind {
    /// Replaces one vowel with a related vowel.
    VowelShift,

    /// Replaces one consonant with a phonetically related consonant.
    ConsonantShift,

    /// Inserts a short synthetic prefix.
    PrefixInsertion,

    /// Appends a short synthetic suffix.
    SuffixInsertion,

    /// Removes one non-initial internal vowel.
    InternalVowelDeletion,

    /// Replaces a consonant with a softer related consonant.
    SoftenConsonant,

    /// Replaces a consonant with a harder related consonant.
    HardenConsonant,
}

impl MorphMutationKind {
    /// Returns the stable machine-readable mutation identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::VowelShift => "vowel-shift",
            Self::ConsonantShift => "consonant-shift",
            Self::PrefixInsertion => "prefix-insertion",
            Self::SuffixInsertion => "suffix-insertion",
            Self::InternalVowelDeletion => "internal-vowel-deletion",
            Self::SoftenConsonant => "soften-consonant",
            Self::HardenConsonant => "harden-consonant",
        }
    }
}

impl std::fmt::Display for MorphMutationKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Configuration controlling deterministic root mutation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutationPolicy {
    /// Mutation kinds eligible for seed-driven selection.
    pub allowed_kinds: Vec<MorphMutationKind>,

    /// Minimum accepted input and output length.
    pub minimum_length: usize,

    /// Maximum accepted input and output length.
    pub maximum_length: usize,

    /// Whether insertion rules may create adjacent identical characters.
    pub allow_adjacent_duplicates: bool,
}

impl MutationPolicy {
    /// Returns Origin's standard mutation policy.
    #[must_use]
    pub fn standard() -> Self {
        Self {
            allowed_kinds: vec![
                MorphMutationKind::VowelShift,
                MorphMutationKind::ConsonantShift,
                MorphMutationKind::PrefixInsertion,
                MorphMutationKind::SuffixInsertion,
                MorphMutationKind::InternalVowelDeletion,
                MorphMutationKind::SoftenConsonant,
                MorphMutationKind::HardenConsonant,
            ],
            minimum_length: DEFAULT_MINIMUM_MUTATION_LENGTH,
            maximum_length: DEFAULT_MAXIMUM_MUTATION_LENGTH,
            allow_adjacent_duplicates: false,
        }
    }

    /// Restricts mutation selection to the supplied kinds.
    #[must_use]
    pub fn with_allowed_kinds(
        mut self,
        allowed_kinds: impl IntoIterator<Item = MorphMutationKind>,
    ) -> Self {
        self.allowed_kinds = allowed_kinds.into_iter().collect();
        self
    }

    /// Sets the minimum accepted root length.
    #[must_use]
    pub const fn with_minimum_length(mut self, minimum_length: usize) -> Self {
        self.minimum_length = minimum_length;
        self
    }

    /// Sets the maximum accepted root length.
    #[must_use]
    pub const fn with_maximum_length(mut self, maximum_length: usize) -> Self {
        self.maximum_length = maximum_length;
        self
    }

    /// Sets whether insertion rules may create adjacent duplicate characters.
    #[must_use]
    pub const fn with_adjacent_duplicates(mut self, allow_adjacent_duplicates: bool) -> Self {
        self.allow_adjacent_duplicates = allow_adjacent_duplicates;
        self
    }

    /// Validates the mutation policy.
    ///
    /// # Errors
    ///
    /// Returns [`MutationError::InvalidPolicy`] when:
    ///
    /// - no mutation kinds are enabled,
    /// - the minimum length is zero,
    /// - or the minimum length is greater than the maximum length.
    pub fn validate(&self) -> Result<(), MutationError> {
        if self.allowed_kinds.is_empty() {
            return Err(MutationError::InvalidPolicy {
                reason: "at least one mutation kind must be enabled".to_owned(),
            });
        }

        if self.minimum_length == 0 {
            return Err(MutationError::InvalidPolicy {
                reason: "minimum length must be greater than zero".to_owned(),
            });
        }

        if self.minimum_length > self.maximum_length {
            return Err(MutationError::InvalidPolicy {
                reason: "minimum length must not exceed maximum length".to_owned(),
            });
        }

        Ok(())
    }
}

impl Default for MutationPolicy {
    fn default() -> Self {
        Self::standard()
    }
}

/// One explainable mutation operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutationChange {
    /// Applied mutation kind.
    pub kind: MorphMutationKind,

    /// Byte position affected in the original or resulting root.
    pub position: usize,

    /// Removed or replaced source text.
    pub before: String,

    /// Inserted or replacement text.
    pub after: String,

    /// Stable human-readable explanation.
    pub explanation: String,
}

/// Complete result of one deterministic mutation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutationReport {
    /// Original canonical root.
    pub original: String,

    /// Mutated canonical root.
    pub mutated: String,

    /// Seed used for deterministic selection.
    pub seed: u64,

    /// Applied mutation operation.
    pub change: MutationChange,
}

impl MutationReport {
    /// Returns whether the mutation changed the input.
    #[must_use]
    pub fn changed(&self) -> bool {
        self.original != self.mutated
    }

    /// Returns a stable provenance description.
    #[must_use]
    pub fn provenance_step(&self) -> String {
        format!(
            "mutate:{}:{}:{}>{}",
            self.change.kind, self.change.position, self.change.before, self.change.after
        )
    }
}

/// Error produced by morphology mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MutationError {
    /// Input root is empty.
    EmptyInput,

    /// Input root is not canonical lowercase ASCII.
    InvalidInput {
        /// Invalid source value.
        value: String,
    },

    /// Input root is shorter than the policy minimum.
    TooShort {
        /// Observed input length.
        length: usize,

        /// Required minimum length.
        minimum: usize,
    },

    /// Input root is longer than the policy maximum.
    TooLong {
        /// Observed input length.
        length: usize,

        /// Allowed maximum length.
        maximum: usize,
    },

    /// Mutation policy is internally inconsistent.
    InvalidPolicy {
        /// Human-readable policy error.
        reason: String,
    },

    /// The selected mutation cannot be applied to this root.
    MutationUnavailable {
        /// Selected mutation kind.
        kind: MorphMutationKind,
    },

    /// None of the enabled mutation kinds can transform the root.
    NoApplicableMutation,

    /// Mutation produced an invalid or unchanged result.
    InvalidOutput {
        /// Produced value.
        value: String,
    },
}

impl std::fmt::Display for MutationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyInput => formatter.write_str("mutation input must not be empty"),
            Self::InvalidInput { value } => write!(
                formatter,
                "mutation input must contain lowercase ASCII letters only: {value}"
            ),
            Self::TooShort { length, minimum } => write!(
                formatter,
                "mutation input length {length} is below minimum {minimum}"
            ),
            Self::TooLong { length, maximum } => write!(
                formatter,
                "mutation input length {length} exceeds maximum {maximum}"
            ),
            Self::InvalidPolicy { reason } => {
                write!(formatter, "invalid mutation policy: {reason}")
            }
            Self::MutationUnavailable { kind } => {
                write!(formatter, "mutation {kind} is unavailable for this root")
            }
            Self::NoApplicableMutation => {
                formatter.write_str("none of the enabled mutation kinds can transform this root")
            }
            Self::InvalidOutput { value } => write!(
                formatter,
                "mutation produced an invalid or unchanged root: {value}"
            ),
        }
    }
}

impl std::error::Error for MutationError {}

/// Applies one seed-selected mutation using [`MutationPolicy::standard`].
///
/// # Errors
///
/// Returns [`MutationError`] when the root or policy is invalid, or when none
/// of the enabled mutation kinds can transform the root.
pub fn mutate_root(root: &str, seed: u64) -> Result<MutationReport, MutationError> {
    mutate_root_with_policy(root, seed, &MutationPolicy::standard())
}

/// Applies one deterministic seed-selected mutation.
///
/// Selection begins at a seed-derived position in the allowed mutation list.
/// When that rule is unavailable for the supplied root, the engine checks the
/// remaining enabled rules in deterministic circular order.
///
/// # Errors
///
/// Returns [`MutationError`] when:
///
/// - the policy is invalid,
/// - the root is empty or non-canonical,
/// - the root violates configured length limits,
/// - or no enabled mutation can produce a valid changed result.
pub fn mutate_root_with_policy(
    root: &str,
    seed: u64,
    policy: &MutationPolicy,
) -> Result<MutationReport, MutationError> {
    validate_root(root, policy)?;

    let start = deterministic_index(seed, policy.allowed_kinds.len());

    for offset in 0..policy.allowed_kinds.len() {
        let index = (start + offset) % policy.allowed_kinds.len();
        let kind = policy.allowed_kinds[index];

        if let Ok(report) = mutate_root_with_kind(root, seed, kind, policy) {
            return Ok(report);
        }
    }

    Err(MutationError::NoApplicableMutation)
}

/// Applies one explicitly selected mutation.
///
/// # Errors
///
/// Returns [`MutationError`] when the root or policy is invalid, when the
/// selected mutation cannot be applied, or when it produces an invalid output.
pub fn mutate_root_with_kind(
    root: &str,
    seed: u64,
    kind: MorphMutationKind,
    policy: &MutationPolicy,
) -> Result<MutationReport, MutationError> {
    validate_root(root, policy)?;

    if !policy.allowed_kinds.contains(&kind) {
        return Err(MutationError::MutationUnavailable { kind });
    }

    let change = match kind {
        MorphMutationKind::VowelShift => mutate_vowel(root, seed)?,
        MorphMutationKind::ConsonantShift => {
            mutate_consonant(root, seed, ConsonantDirection::Neutral)?
        }
        MorphMutationKind::PrefixInsertion => {
            insert_affix(root, seed, AffixPosition::Prefix, policy)?
        }
        MorphMutationKind::SuffixInsertion => {
            insert_affix(root, seed, AffixPosition::Suffix, policy)?
        }
        MorphMutationKind::InternalVowelDeletion => delete_internal_vowel(root, seed)?,
        MorphMutationKind::SoftenConsonant => {
            mutate_consonant(root, seed, ConsonantDirection::Soft)?
        }
        MorphMutationKind::HardenConsonant => {
            mutate_consonant(root, seed, ConsonantDirection::Hard)?
        }
    };

    let mutated = apply_change(root, &change);

    if mutated == root
        || !is_canonical_root(&mutated)
        || mutated.len() < policy.minimum_length
        || mutated.len() > policy.maximum_length
    {
        return Err(MutationError::InvalidOutput { value: mutated });
    }

    Ok(MutationReport {
        original: root.to_owned(),
        mutated,
        seed,
        change,
    })
}

fn validate_root(root: &str, policy: &MutationPolicy) -> Result<(), MutationError> {
    policy.validate()?;

    if root.is_empty() {
        return Err(MutationError::EmptyInput);
    }

    if !is_canonical_root(root) {
        return Err(MutationError::InvalidInput {
            value: root.to_owned(),
        });
    }

    if root.len() < policy.minimum_length {
        return Err(MutationError::TooShort {
            length: root.len(),
            minimum: policy.minimum_length,
        });
    }

    if root.len() > policy.maximum_length {
        return Err(MutationError::TooLong {
            length: root.len(),
            maximum: policy.maximum_length,
        });
    }

    Ok(())
}

fn mutate_vowel(root: &str, seed: u64) -> Result<MutationChange, MutationError> {
    let positions = root
        .bytes()
        .enumerate()
        .filter_map(|(position, byte)| is_vowel(byte).then_some((position, byte)))
        .collect::<Vec<_>>();

    let &(position, before) = positions
        .get(deterministic_index(seed, positions.len()))
        .ok_or(MutationError::MutationUnavailable {
            kind: MorphMutationKind::VowelShift,
        })?;

    let alternatives = vowel_alternatives(before);
    let after =
        alternatives[deterministic_index(mix_seed(seed, position as u64), alternatives.len())];

    Ok(MutationChange {
        kind: MorphMutationKind::VowelShift,
        position,
        before: char::from(before).to_string(),
        after: char::from(after).to_string(),
        explanation: format!(
            "shifted vowel {:?} to {:?}",
            char::from(before),
            char::from(after)
        ),
    })
}

fn mutate_consonant(
    root: &str,
    seed: u64,
    direction: ConsonantDirection,
) -> Result<MutationChange, MutationError> {
    let positions = root
        .bytes()
        .enumerate()
        .filter_map(|(position, byte)| {
            consonant_replacement(byte, direction).map(|replacement| (position, byte, replacement))
        })
        .collect::<Vec<_>>();

    let &(position, before, after) = positions
        .get(deterministic_index(seed, positions.len()))
        .ok_or(MutationError::MutationUnavailable {
            kind: direction.kind(),
        })?;

    Ok(MutationChange {
        kind: direction.kind(),
        position,
        before: char::from(before).to_string(),
        after: char::from(after).to_string(),
        explanation: format!(
            "{} consonant {:?} to {:?}",
            direction.verb(),
            char::from(before),
            char::from(after)
        ),
    })
}

fn insert_affix(
    root: &str,
    seed: u64,
    position: AffixPosition,
    policy: &MutationPolicy,
) -> Result<MutationChange, MutationError> {
    let affixes = match position {
        AffixPosition::Prefix => ["a", "e", "i", "o", "va", "ny"],
        AffixPosition::Suffix => ["a", "e", "is", "on", "or", "ex"],
    };

    for offset in 0..affixes.len() {
        let index = (deterministic_index(seed, affixes.len()) + offset) % affixes.len();
        let affix = affixes[index];

        if root.len() + affix.len() > policy.maximum_length {
            continue;
        }

        if !policy.allow_adjacent_duplicates && creates_boundary_duplicate(root, affix, position) {
            continue;
        }

        let (kind, byte_position, explanation) = match position {
            AffixPosition::Prefix => (
                MorphMutationKind::PrefixInsertion,
                0,
                format!("inserted synthetic prefix {affix:?}"),
            ),
            AffixPosition::Suffix => (
                MorphMutationKind::SuffixInsertion,
                root.len(),
                format!("inserted synthetic suffix {affix:?}"),
            ),
        };

        return Ok(MutationChange {
            kind,
            position: byte_position,
            before: String::new(),
            after: affix.to_owned(),
            explanation,
        });
    }

    Err(MutationError::MutationUnavailable {
        kind: position.kind(),
    })
}

fn delete_internal_vowel(root: &str, seed: u64) -> Result<MutationChange, MutationError> {
    let positions = root
        .bytes()
        .enumerate()
        .filter_map(|(position, byte)| {
            (position > 0 && position + 1 < root.len() && is_vowel(byte))
                .then_some((position, byte))
        })
        .collect::<Vec<_>>();

    let &(position, before) = positions
        .get(deterministic_index(seed, positions.len()))
        .ok_or(MutationError::MutationUnavailable {
            kind: MorphMutationKind::InternalVowelDeletion,
        })?;

    Ok(MutationChange {
        kind: MorphMutationKind::InternalVowelDeletion,
        position,
        before: char::from(before).to_string(),
        after: String::new(),
        explanation: format!("removed internal vowel {:?}", char::from(before)),
    })
}

fn apply_change(root: &str, change: &MutationChange) -> String {
    let mut result = String::with_capacity(root.len() - change.before.len() + change.after.len());

    result.push_str(&root[..change.position]);
    result.push_str(&change.after);
    result.push_str(&root[change.position + change.before.len()..]);

    result
}

fn creates_boundary_duplicate(root: &str, affix: &str, position: AffixPosition) -> bool {
    match position {
        AffixPosition::Prefix => affix.as_bytes().last() == root.as_bytes().first(),
        AffixPosition::Suffix => root.as_bytes().last() == affix.as_bytes().first(),
    }
}

const fn is_vowel(byte: u8) -> bool {
    matches!(byte, b'a' | b'e' | b'i' | b'o' | b'u')
}

const fn vowel_alternatives(vowel: u8) -> &'static [u8] {
    match vowel {
        b'a' | b'i' => b"eo",
        b'e' => b"ai",
        b'o' => b"au",
        b'u' => b"io",
        _ => b"",
    }
}

#[derive(Debug, Clone, Copy)]
enum ConsonantDirection {
    Neutral,
    Soft,
    Hard,
}

impl ConsonantDirection {
    const fn kind(self) -> MorphMutationKind {
        match self {
            Self::Neutral => MorphMutationKind::ConsonantShift,
            Self::Soft => MorphMutationKind::SoftenConsonant,
            Self::Hard => MorphMutationKind::HardenConsonant,
        }
    }

    const fn verb(self) -> &'static str {
        match self {
            Self::Neutral => "shifted",
            Self::Soft => "softened",
            Self::Hard => "hardened",
        }
    }
}

const fn consonant_replacement(consonant: u8, direction: ConsonantDirection) -> Option<u8> {
    match direction {
        ConsonantDirection::Neutral => match consonant {
            b'b' | b'f' => Some(b'v'),
            b'v' => Some(b'b'),
            b'c' | b'g' => Some(b'k'),
            b'k' => Some(b'c'),
            b'd' => Some(b't'),
            b't' => Some(b'd'),
            b'j' => Some(b'y'),
            b'l' => Some(b'r'),
            b'r' => Some(b'l'),
            b'm' => Some(b'n'),
            b'n' => Some(b'm'),
            b's' => Some(b'z'),
            b'z' => Some(b's'),
            _ => None,
        },
        ConsonantDirection::Soft => match consonant {
            b'b' => Some(b'v'),
            b'c' => Some(b's'),
            b'd' => Some(b'z'),
            b'g' => Some(b'y'),
            b'k' => Some(b'g'),
            b'p' => Some(b'b'),
            b't' => Some(b'd'),
            _ => None,
        },
        ConsonantDirection::Hard => match consonant {
            b'b' => Some(b'p'),
            b'd' => Some(b't'),
            b'g' => Some(b'k'),
            b'v' => Some(b'f'),
            b'z' => Some(b's'),
            b'y' => Some(b'g'),
            _ => None,
        },
    }
}

#[derive(Debug, Clone, Copy)]
enum AffixPosition {
    Prefix,
    Suffix,
}

impl AffixPosition {
    const fn kind(self) -> MorphMutationKind {
        match self {
            Self::Prefix => MorphMutationKind::PrefixInsertion,
            Self::Suffix => MorphMutationKind::SuffixInsertion,
        }
    }
}

fn deterministic_index(seed: u64, length: usize) -> usize {
    if length == 0 {
        return 0;
    }

    let length_u64 = u64::try_from(length).expect("usize must fit into u64");

    usize::try_from(splitmix64(seed) % length_u64).expect("modulo result must fit into usize")
}

const fn mix_seed(seed: u64, salt: u64) -> u64 {
    seed ^ salt.wrapping_mul(0x9E37_79B9_7F4A_7C15)
}

const fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_MAXIMUM_MUTATION_LENGTH, MorphMutationKind, MutationError, MutationPolicy,
        mutate_root, mutate_root_with_kind, mutate_root_with_policy,
    };

    fn policy_for(kind: MorphMutationKind) -> MutationPolicy {
        MutationPolicy::standard().with_allowed_kinds([kind])
    }

    #[test]
    fn mutation_is_deterministic() {
        let first = mutate_root("velora", 42).expect("mutation must succeed");
        let second = mutate_root("velora", 42).expect("mutation must succeed");

        assert_eq!(first, second);
        assert!(first.changed());
    }

    #[test]
    fn different_seeds_can_change_selection() {
        let policy = policy_for(MorphMutationKind::VowelShift);

        let first =
            mutate_root_with_policy("velora", 1, &policy).expect("first mutation must succeed");

        let second =
            mutate_root_with_policy("velora", 2, &policy).expect("second mutation must succeed");

        assert!(first.mutated != second.mutated || first.change.position != second.change.position);
    }

    #[test]
    fn explicit_vowel_shift_changes_one_vowel() {
        let report = mutate_root_with_kind(
            "velora",
            5,
            MorphMutationKind::VowelShift,
            &policy_for(MorphMutationKind::VowelShift),
        )
        .expect("vowel shift must succeed");

        assert_eq!(report.original.len(), report.mutated.len());
        assert_eq!(report.change.before.len(), 1);
        assert_eq!(report.change.after.len(), 1);
        assert_ne!(report.change.before, report.change.after);
    }

    #[test]
    fn explicit_consonant_shift_changes_one_consonant() {
        let report = mutate_root_with_kind(
            "velora",
            7,
            MorphMutationKind::ConsonantShift,
            &policy_for(MorphMutationKind::ConsonantShift),
        )
        .expect("consonant shift must succeed");

        assert_eq!(report.original.len(), report.mutated.len());
        assert_ne!(report.change.before, report.change.after);
    }

    #[test]
    fn prefix_insertion_increases_length() {
        let report = mutate_root_with_kind(
            "velor",
            3,
            MorphMutationKind::PrefixInsertion,
            &policy_for(MorphMutationKind::PrefixInsertion),
        )
        .expect("prefix insertion must succeed");

        assert!(report.mutated.len() > report.original.len());
        assert_eq!(report.change.position, 0);
        assert!(report.change.before.is_empty());
    }

    #[test]
    fn suffix_insertion_increases_length() {
        let report = mutate_root_with_kind(
            "velor",
            3,
            MorphMutationKind::SuffixInsertion,
            &policy_for(MorphMutationKind::SuffixInsertion),
        )
        .expect("suffix insertion must succeed");

        assert!(report.mutated.len() > report.original.len());
        assert_eq!(report.change.position, report.original.len());
    }

    #[test]
    fn internal_vowel_deletion_reduces_length() {
        let report = mutate_root_with_kind(
            "velora",
            9,
            MorphMutationKind::InternalVowelDeletion,
            &policy_for(MorphMutationKind::InternalVowelDeletion),
        )
        .expect("internal vowel deletion must succeed");

        assert_eq!(report.mutated.len() + 1, report.original.len());
        assert!(report.change.after.is_empty());
    }

    #[test]
    fn softening_uses_supported_consonant_mapping() {
        let report = mutate_root_with_kind(
            "takor",
            2,
            MorphMutationKind::SoftenConsonant,
            &policy_for(MorphMutationKind::SoftenConsonant),
        )
        .expect("softening must succeed");

        assert_ne!(report.original, report.mutated);
        assert_eq!(report.change.kind, MorphMutationKind::SoftenConsonant);
    }

    #[test]
    fn hardening_uses_supported_consonant_mapping() {
        let report = mutate_root_with_kind(
            "vador",
            4,
            MorphMutationKind::HardenConsonant,
            &policy_for(MorphMutationKind::HardenConsonant),
        )
        .expect("hardening must succeed");

        assert_ne!(report.original, report.mutated);
        assert_eq!(report.change.kind, MorphMutationKind::HardenConsonant);
    }

    #[test]
    fn unavailable_explicit_mutation_returns_error() {
        let result = mutate_root_with_kind(
            "aeiou",
            1,
            MorphMutationKind::ConsonantShift,
            &policy_for(MorphMutationKind::ConsonantShift),
        );

        assert_eq!(
            result,
            Err(MutationError::MutationUnavailable {
                kind: MorphMutationKind::ConsonantShift,
            })
        );
    }

    #[test]
    fn seed_selected_mutation_falls_back_to_applicable_rule() {
        let policy = MutationPolicy::standard().with_allowed_kinds([
            MorphMutationKind::ConsonantShift,
            MorphMutationKind::VowelShift,
        ]);

        let report =
            mutate_root_with_policy("aeiou", 0, &policy).expect("fallback mutation must succeed");

        assert_eq!(report.change.kind, MorphMutationKind::VowelShift);
    }

    #[test]
    fn empty_input_is_rejected() {
        assert_eq!(mutate_root("", 1), Err(MutationError::EmptyInput));
    }

    #[test]
    fn noncanonical_input_is_rejected() {
        assert_eq!(
            mutate_root("Vel-ora", 1),
            Err(MutationError::InvalidInput {
                value: "Vel-ora".to_owned(),
            })
        );
    }

    #[test]
    fn short_input_is_rejected() {
        assert_eq!(
            mutate_root("ab", 1),
            Err(MutationError::TooShort {
                length: 2,
                minimum: 3,
            })
        );
    }

    #[test]
    fn long_input_is_rejected() {
        let value = "a".repeat(DEFAULT_MAXIMUM_MUTATION_LENGTH + 1);

        assert_eq!(
            mutate_root(&value, 1),
            Err(MutationError::TooLong {
                length: DEFAULT_MAXIMUM_MUTATION_LENGTH + 1,
                maximum: DEFAULT_MAXIMUM_MUTATION_LENGTH,
            })
        );
    }

    #[test]
    fn empty_allowed_kind_list_is_invalid() {
        let policy = MutationPolicy::standard().with_allowed_kinds([]);

        assert!(matches!(
            policy.validate(),
            Err(MutationError::InvalidPolicy { .. })
        ));
    }

    #[test]
    fn invalid_length_range_is_rejected() {
        let policy = MutationPolicy::standard()
            .with_minimum_length(9)
            .with_maximum_length(4);

        assert!(matches!(
            policy.validate(),
            Err(MutationError::InvalidPolicy { .. })
        ));
    }

    #[test]
    fn disabled_kind_cannot_be_called_explicitly() {
        let policy = policy_for(MorphMutationKind::VowelShift);

        assert_eq!(
            mutate_root_with_kind("velora", 1, MorphMutationKind::SuffixInsertion, &policy,),
            Err(MutationError::MutationUnavailable {
                kind: MorphMutationKind::SuffixInsertion,
            })
        );
    }

    #[test]
    fn provenance_is_stable() {
        let report = mutate_root_with_kind(
            "velora",
            12,
            MorphMutationKind::VowelShift,
            &policy_for(MorphMutationKind::VowelShift),
        )
        .expect("mutation must succeed");

        assert_eq!(
            report.provenance_step(),
            format!(
                "mutate:vowel-shift:{}:{}>{}",
                report.change.position, report.change.before, report.change.after
            )
        );
    }

    #[test]
    fn standard_policy_equals_default_policy() {
        assert_eq!(MutationPolicy::standard(), MutationPolicy::default());
    }
}
