//! Deterministic mutation and ranking for improving existing brand names.

use std::collections::HashSet;

use serde::Serialize;

use crate::{BrandReport, analyze_brand};

const ONSETS: &[u8] = b"bdfgklmnprstvwxyzchj";
const VOWELS: &[u8] = b"aeiou";

/// Configuration for deterministic brand-name improvement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImproveOptions {
    /// Maximum number of ranked suggestions to return.
    pub count: usize,
    /// Seed used to break otherwise equal ranking ties deterministically.
    pub seed: u64,
}

impl Default for ImproveOptions {
    fn default() -> Self {
        Self { count: 10, seed: 1 }
    }
}

/// One ranked mutation derived from an existing name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ImprovementCandidate {
    /// Mutated candidate text.
    pub name: String,
    /// Weighted overall score from zero to one hundred.
    pub score: u8,
    /// Signed score change relative to the original name.
    pub score_delta: i16,
    /// Whether the mutation passes the active profile threshold.
    pub accepted: bool,
    /// Position changed in the normalized original name.
    pub changed_position: usize,
    /// Original ASCII character at the changed position.
    pub replaced: char,
    /// Replacement ASCII character at the changed position.
    pub replacement: char,
    /// Complete explainable report for the mutation.
    pub report: BrandReport,
}

/// Complete deterministic improvement result for one source name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ImprovementReport {
    /// Analysis of the normalized original name.
    pub original: BrandReport,
    /// Ranked one-phoneme mutations.
    pub suggestions: Vec<ImprovementCandidate>,
}

/// Generates and ranks deterministic one-phoneme improvements.
///
/// Consonant positions are replaced only with supported onsets and vowel
/// positions only with supported vowels. This preserves the source name's
/// broad consonant-vowel shape while allowing weaknesses to be repaired.
#[must_use]
pub fn improve(input: &str, options: ImproveOptions) -> ImprovementReport {
    let original = analyze_brand(input);
    let bytes = original.normalized.as_bytes();

    if original.overall_score == 0 || bytes.is_empty() {
        return ImprovementReport {
            original,
            suggestions: Vec::new(),
        };
    }

    let mut seen = HashSet::new();
    let mut suggestions = Vec::new();

    for position in 0..bytes.len() {
        let replacements = if is_vowel(bytes[position]) {
            VOWELS
        } else {
            ONSETS
        };

        for &replacement in replacements {
            if replacement == bytes[position] {
                continue;
            }

            let mut mutated = bytes.to_vec();
            mutated[position] = replacement;
            let name = String::from_utf8(mutated).expect("mutation tables contain ASCII only");

            if !seen.insert(name.clone()) {
                continue;
            }

            let report = analyze_brand(&name);
            suggestions.push(ImprovementCandidate {
                score: report.overall_score,
                score_delta: i16::from(report.overall_score)
                    - i16::from(original.overall_score),
                accepted: report.accepted,
                changed_position: position,
                replaced: char::from(bytes[position]),
                replacement: char::from(replacement),
                name,
                report,
            });
        }
    }

    suggestions.sort_unstable_by(|left, right| {
        right
            .accepted
            .cmp(&left.accepted)
            .then_with(|| right.score.cmp(&left.score))
            .then_with(|| right.report.scores.repetition.cmp(&left.report.scores.repetition))
            .then_with(|| {
                mutation_tie_key(&left.name, options.seed)
                    .cmp(&mutation_tie_key(&right.name, options.seed))
            })
            .then_with(|| left.name.cmp(&right.name))
    });
    suggestions.truncate(options.count);

    ImprovementReport {
        original,
        suggestions,
    }
}

fn mutation_tie_key(name: &str, seed: u64) -> u64 {
    name.bytes().fold(seed ^ 0x9E37_79B9_7F4A_7C15, |value, byte| {
        value
            .wrapping_mul(0x100_0000_01B3)
            .wrapping_add(u64::from(byte))
    })
}

fn is_vowel(byte: u8) -> bool {
    matches!(byte, b'a' | b'e' | b'i' | b'o' | b'u')
}

#[cfg(test)]
mod tests {
    use super::{ImproveOptions, improve};

    #[test]
    fn improvement_is_deterministic() {
        let options = ImproveOptions { count: 10, seed: 42 };
        assert_eq!(improve("pogoga", options), improve("pogoga", options));
    }

    #[test]
    fn repetitive_name_receives_better_ranked_mutations() {
        let result = improve("folele", ImproveOptions { count: 10, seed: 7 });

        assert!(!result.suggestions.is_empty());
        assert!(result.suggestions[0].score > result.original.overall_score);
        assert!(result.suggestions[0].score_delta > 0);
        assert!(result.suggestions[0].accepted);
    }

    #[test]
    fn mutations_preserve_length_and_change_one_character() {
        let result = improve("pogoga", ImproveOptions { count: 25, seed: 1 });

        for suggestion in result.suggestions {
            assert_eq!(suggestion.name.len(), result.original.normalized.len());
            assert_eq!(
                suggestion
                    .name
                    .bytes()
                    .zip(result.original.normalized.bytes())
                    .filter(|(left, right)| left != right)
                    .count(),
                1
            );
        }
    }

    #[test]
    fn invalid_input_returns_no_suggestions() {
        let result = improve("nova-1", ImproveOptions::default());

        assert_eq!(result.original.overall_score, 0);
        assert!(result.suggestions.is_empty());
    }
}
