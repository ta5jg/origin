//! Core generation and scoring primitives for ORIGIN.

use serde::Serialize;

const ONSETS: &[u8; 20] = b"bdfgklmnprstvwxyzchj";
const VOWELS: &[u8; 5] = b"aeiou";
const SYLLABLE_RADIX: usize = ONSETS.len() * VOWELS.len();
/// Maximum number of unique three-syllable candidates in the current model.
pub const MAX_CANDIDATES: usize = SYLLABLE_RADIX * SYLLABLE_RADIX * SYLLABLE_RADIX;

/// A generated brand-name candidate and its preliminary structural score.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Candidate {
    /// Candidate text in lowercase ASCII.
    pub name: String,
    /// Preliminary score from zero to one hundred.
    pub score: u8,
}

/// Configuration for deterministic candidate generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GenerateOptions {
    /// Maximum number of unique candidates to return.
    pub count: usize,
    /// Seed used to choose a deterministic traversal through the name space.
    pub seed: u64,
}

impl Default for GenerateOptions {
    fn default() -> Self {
        Self { count: 100, seed: 1 }
    }
}

/// Generates unique candidates using deterministic phoneme composition.
///
/// The current model uses three fixed-width consonant-vowel syllables. This
/// creates exactly one million unique six-letter candidates while preserving
/// reproducibility and constant-time candidate construction.
///
/// Requests larger than [`MAX_CANDIDATES`] are capped at that limit.
#[must_use]
pub fn generate(options: GenerateOptions) -> Vec<Candidate> {
    let count = options.count.min(MAX_CANDIDATES);
    let start = seed_start(options.seed);
    let step = seed_step(options.seed);

    let mut candidates = Vec::with_capacity(count);
    for offset in 0..count {
        let index = (start + offset * step) % MAX_CANDIDATES;
        let name = compose_from_index(index);
        candidates.push(Candidate {
            score: structural_score(&name),
            name,
        });
    }

    candidates.sort_unstable_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.name.cmp(&right.name))
    });
    candidates
}

fn compose_from_index(mut index: usize) -> String {
    let mut bytes = [0_u8; 6];

    for syllable_position in (0..3).rev() {
        let syllable = index % SYLLABLE_RADIX;
        index /= SYLLABLE_RADIX;

        let onset = ONSETS[syllable / VOWELS.len()];
        let vowel = VOWELS[syllable % VOWELS.len()];
        let byte_position = syllable_position * 2;
        bytes[byte_position] = onset;
        bytes[byte_position + 1] = vowel;
    }

    String::from_utf8(bytes.to_vec()).expect("the phoneme table contains ASCII only")
}

fn seed_start(seed: u64) -> usize {
    (mix(seed) as usize) % MAX_CANDIDATES
}

fn seed_step(seed: u64) -> usize {
    let mut step = ((mix(seed ^ 0xA5A5_A5A5_A5A5_A5A5) as usize) % MAX_CANDIDATES).max(1);

    while step % 2 == 0 || step % 5 == 0 {
        step += 1;
        if step >= MAX_CANDIDATES {
            step = 1;
        }
    }

    step
}

const fn mix(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

fn structural_score(name: &str) -> u8 {
    let bytes = name.as_bytes();
    let distinct_letters = {
        let mut seen = [false; 26];
        for &byte in bytes {
            seen[usize::from(byte - b'a')] = true;
        }
        seen.into_iter().filter(|present| *present).count()
    };

    let diversity_score = match distinct_letters {
        6 => 45,
        5 => 40,
        4 => 32,
        _ => 20,
    };
    let ending_score = if matches!(bytes.last(), Some(b'a' | b'e' | b'i' | b'o' | b'u')) {
        30
    } else {
        20
    };
    let repetition_score = if bytes.windows(2).any(|pair| pair[0] == pair[1]) {
        15
    } else {
        25
    };

    diversity_score + ending_score + repetition_score
}

#[cfg(test)]
mod tests {
    use super::{GenerateOptions, MAX_CANDIDATES, generate};
    use std::collections::HashSet;

    #[test]
    fn generation_is_deterministic() {
        let options = GenerateOptions { count: 25, seed: 42 };
        assert_eq!(generate(options), generate(options));
    }

    #[test]
    fn different_seeds_change_the_traversal() {
        assert_ne!(
            generate(GenerateOptions { count: 25, seed: 1 }),
            generate(GenerateOptions { count: 25, seed: 2 })
        );
    }

    #[test]
    fn generation_returns_requested_unique_names() {
        let candidates = generate(GenerateOptions {
            count: 10_000,
            seed: 7,
        });
        let unique = candidates
            .iter()
            .map(|candidate| candidate.name.as_str())
            .collect::<HashSet<_>>();

        assert_eq!(candidates.len(), 10_000);
        assert_eq!(candidates.len(), unique.len());
        assert!(candidates.iter().all(|candidate| {
            candidate.name.len() == 6
                && candidate.name.is_ascii()
                && candidate.score <= 100
        }));
    }

    #[test]
    fn current_model_exposes_one_million_candidates() {
        assert_eq!(MAX_CANDIDATES, 1_000_000);
    }
}
