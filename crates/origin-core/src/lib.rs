//! Core generation and scoring primitives for ORIGIN.

use serde::Serialize;
use std::collections::HashSet;

const ONSETS: &[&str] = &[
    "b", "d", "f", "g", "k", "l", "m", "n", "p", "r", "s", "t", "v", "z", "br", "dr",
    "kr", "ly", "ny", "vr",
];
const NUCLEI: &[&str] = &["a", "e", "i", "o", "u", "ae", "ai", "eo", "ia", "io"];
const MEDIALS: &[&str] = &["l", "m", "n", "r", "s", "v", "x", "th"];
const CODAS: &[&str] = &["a", "e", "i", "o", "on", "or", "en", "is", "um", "yn"];

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
    /// Seed used by the deterministic pseudo-random sequence.
    pub seed: u64,
}

impl Default for GenerateOptions {
    fn default() -> Self {
        Self { count: 100, seed: 1 }
    }
}

/// Generates unique candidates using deterministic phoneme composition.
///
/// The current implementation is intentionally compact. It validates the
/// workspace architecture and CLI contract before the evolutionary generator
/// and language-specific models are introduced.
#[must_use]
pub fn generate(options: GenerateOptions) -> Vec<Candidate> {
    let mut rng = SplitMix64::new(options.seed);
    let mut seen = HashSet::with_capacity(options.count);
    let mut candidates = Vec::with_capacity(options.count);
    let attempt_limit = options.count.saturating_mul(40).max(100);

    for _ in 0..attempt_limit {
        if candidates.len() >= options.count {
            break;
        }

        let name = compose(&mut rng);
        if !is_structurally_valid(&name) || !seen.insert(name.clone()) {
            continue;
        }

        candidates.push(Candidate {
            score: structural_score(&name),
            name,
        });
    }

    candidates.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.name.cmp(&right.name))
    });
    candidates
}

fn compose(rng: &mut SplitMix64) -> String {
    let onset = choose(ONSETS, rng);
    let nucleus = choose(NUCLEI, rng);
    let medial = choose(MEDIALS, rng);
    let coda = choose(CODAS, rng);
    format!("{onset}{nucleus}{medial}{coda}")
}

fn choose<'a>(values: &'a [&str], rng: &mut SplitMix64) -> &'a str {
    let index = (rng.next() as usize) % values.len();
    values[index]
}

fn is_structurally_valid(name: &str) -> bool {
    let length = name.len();
    (5..=9).contains(&length)
        && !name.contains("xxx")
        && !name.contains("vvv")
        && !name.contains("tech")
        && !name.contains("verse")
}

fn structural_score(name: &str) -> u8 {
    let length_score = match name.len() {
        6 | 7 => 45,
        5 | 8 => 38,
        9 => 30,
        _ => 15,
    };
    let vowel_count = name
        .bytes()
        .filter(|byte| matches!(byte, b'a' | b'e' | b'i' | b'o' | b'u'))
        .count();
    let vowel_score = match vowel_count {
        2 | 3 => 35,
        1 | 4 => 25,
        _ => 10,
    };
    let ending_score = if name.ends_with(['a', 'e', 'i', 'o', 'n', 'r', 's']) {
        20
    } else {
        12
    };

    (length_score + vowel_score + ending_score).min(100)
}

#[derive(Debug, Clone, Copy)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        value ^ (value >> 31)
    }
}

#[cfg(test)]
mod tests {
    use super::{GenerateOptions, generate};

    #[test]
    fn generation_is_deterministic() {
        let options = GenerateOptions { count: 25, seed: 42 };
        assert_eq!(generate(options), generate(options));
    }

    #[test]
    fn generation_returns_unique_structurally_valid_names() {
        let candidates = generate(GenerateOptions { count: 500, seed: 7 });
        let unique = candidates
            .iter()
            .map(|candidate| candidate.name.as_str())
            .collect::<std::collections::HashSet<_>>();

        assert_eq!(candidates.len(), unique.len());
        assert!(candidates.iter().all(|candidate| {
            (5..=9).contains(&candidate.name.len()) && candidate.score <= 100
        }));
    }
}
