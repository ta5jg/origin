//! Explainable multi-component scoring for candidate brand names.

use serde::Serialize;

use crate::phonotactics::analyze_name;

/// Configuration describing one linguistic scoring profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LanguageProfile {
    /// Stable identifier included in reports.
    pub id: &'static str,
    /// Minimum overall score required for acceptance.
    pub acceptance_threshold: u8,
}

/// Default profile for internationally oriented technology brands.
pub const INTERNATIONAL_TECH_V1: LanguageProfile = LanguageProfile {
    id: "international-tech-v1",
    acceptance_threshold: 75,
};

/// Explainable component scores, each ranging from zero to one hundred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ScoreBreakdown {
    /// Ease of pronunciation according to the phonotactic engine.
    pub pronounceability: u8,
    /// Regularity of vowel and consonant alternation.
    pub rhythm: u8,
    /// Balance between vowels and consonants.
    pub vowel_balance: u8,
    /// Resistance to mechanical letter and bigram repetition.
    pub repetition: u8,
    /// Smoothness and diversity of adjacent phoneme transitions.
    pub transition_quality: u8,
}

/// Complete explainable analysis of one candidate name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BrandReport {
    /// Lowercase, trimmed representation used by the analyzer.
    pub normalized: String,
    /// Identifier of the language profile used for scoring.
    pub profile: String,
    /// Weighted overall score from zero to one hundred.
    pub overall_score: u8,
    /// Whether the name passes both phonotactic and overall thresholds.
    pub accepted: bool,
    /// Individual scoring components.
    pub scores: ScoreBreakdown,
    /// Human-readable explanations for detected weaknesses.
    pub warnings: Vec<String>,
}

/// Analyzes a name with the default international technology-brand profile.
#[must_use]
pub fn analyze_brand(input: &str) -> BrandReport {
    analyze_brand_with_profile(input, INTERNATIONAL_TECH_V1)
}

/// Analyzes a name with an explicitly selected language profile.
#[must_use]
pub fn analyze_brand_with_profile(input: &str, profile: LanguageProfile) -> BrandReport {
    let phonotactic = analyze_name(input);
    let bytes = phonotactic.normalized.as_bytes();
    let valid = !bytes.is_empty()
        && phonotactic.normalized.is_ascii()
        && bytes.iter().all(u8::is_ascii_lowercase)
        && (4..=12).contains(&bytes.len());

    if !valid {
        return BrandReport {
            normalized: phonotactic.normalized,
            profile: String::from(profile.id),
            overall_score: 0,
            accepted: false,
            scores: ScoreBreakdown {
                pronounceability: 0,
                rhythm: 0,
                vowel_balance: 0,
                repetition: 0,
                transition_quality: 0,
            },
            warnings: phonotactic.warnings,
        };
    }

    let scores = ScoreBreakdown {
        pronounceability: phonotactic.score,
        rhythm: rhythm_score(bytes),
        vowel_balance: vowel_balance_score(bytes),
        repetition: repetition_score(bytes),
        transition_quality: transition_score(bytes),
    };
    let overall_score = weighted_overall(scores);
    let mut warnings = phonotactic.warnings;

    if scores.repetition < 75 {
        warnings.push(String::from(
            "letter or bigram repetition reduces distinctiveness",
        ));
    }
    if scores.rhythm < 70 {
        warnings.push(String::from(
            "vowel and consonant rhythm is less regular than preferred",
        ));
    }
    if scores.transition_quality < 80 {
        warnings.push(String::from(
            "repeated or difficult phoneme transitions reduce pronunciation flow",
        ));
    }

    BrandReport {
        normalized: phonotactic.normalized,
        profile: String::from(profile.id),
        overall_score,
        accepted: phonotactic.accepted && overall_score >= profile.acceptance_threshold,
        scores,
        warnings,
    }
}

fn weighted_overall(scores: ScoreBreakdown) -> u8 {
    let weighted = u16::from(scores.pronounceability) * 30
        + u16::from(scores.rhythm) * 15
        + u16::from(scores.vowel_balance) * 20
        + u16::from(scores.repetition) * 20
        + u16::from(scores.transition_quality) * 15;
    u8::try_from(weighted / 100).unwrap_or(100)
}

fn rhythm_score(bytes: &[u8]) -> u8 {
    let transition_count = bytes.len().saturating_sub(1);
    if transition_count == 0 {
        return 0;
    }

    let alternating = bytes
        .windows(2)
        .filter(|pair| is_vowel(pair[0]) != is_vowel(pair[1]))
        .count();
    let bonus = alternating * 40 / transition_count;
    u8::try_from(60 + bonus).unwrap_or(100)
}

fn vowel_balance_score(bytes: &[u8]) -> u8 {
    let vowel_count = bytes.iter().copied().filter(|byte| is_vowel(*byte)).count();
    let imbalance = (vowel_count * 2).abs_diff(bytes.len());
    let penalty = imbalance * 60 / bytes.len();
    u8::try_from(100_usize.saturating_sub(penalty)).unwrap_or_default()
}

fn repetition_score(bytes: &[u8]) -> u8 {
    let repeated_letters = bytes.len().saturating_sub(distinct_letter_count(bytes));
    let repeated_bigrams = repeated_bigram_count(bytes);
    let adjacent_duplicates = bytes.windows(2).filter(|pair| pair[0] == pair[1]).count();
    let penalty = repeated_letters * 8 + repeated_bigrams * 20 + adjacent_duplicates * 25;
    u8::try_from(100_usize.saturating_sub(penalty)).unwrap_or_default()
}

fn transition_score(bytes: &[u8]) -> u8 {
    let mut penalty = 0_usize;

    for pair in bytes.windows(2) {
        penalty += transition_penalty(pair[0], pair[1]);
    }

    let syllables = bytes.chunks_exact(2).collect::<Vec<_>>();
    for pair in syllables.windows(2) {
        if pair[0] == pair[1] {
            penalty += 24;
        }
    }

    for onsets in bytes.iter().step_by(2).collect::<Vec<_>>().windows(2) {
        if onsets[0] == onsets[1] {
            penalty += 7;
        }
    }

    for vowels in bytes
        .iter()
        .skip(1)
        .step_by(2)
        .collect::<Vec<_>>()
        .windows(2)
    {
        if vowels[0] == vowels[1] {
            penalty += 5;
        }
    }

    u8::try_from(100_usize.saturating_sub(penalty)).unwrap_or_default()
}

fn transition_penalty(left: u8, right: u8) -> usize {
    if left == right {
        return 25;
    }

    match (sound_class(left), sound_class(right)) {
        (SoundClass::Stop, SoundClass::Stop) => 12,
        (SoundClass::Fricative, SoundClass::Fricative) => 10,
        (SoundClass::Vowel, SoundClass::Vowel) | (SoundClass::Nasal, SoundClass::Nasal) => 8,
        (SoundClass::Liquid, SoundClass::Liquid) => 7,
        (SoundClass::Stop, SoundClass::Fricative) | (SoundClass::Fricative, SoundClass::Stop) => 5,
        _ => 0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SoundClass {
    Vowel,
    Stop,
    Fricative,
    Nasal,
    Liquid,
}

fn sound_class(byte: u8) -> SoundClass {
    match byte {
        b'a' | b'e' | b'i' | b'o' | b'u' => SoundClass::Vowel,
        b'b' | b'd' | b'g' | b'k' | b'p' | b't' => SoundClass::Stop,
        b'f' | b'h' | b'j' | b's' | b'v' | b'w' | b'x' | b'y' | b'z' | b'c' => {
            SoundClass::Fricative
        }
        b'm' | b'n' => SoundClass::Nasal,
        b'l' | b'r' => SoundClass::Liquid,
        _ => SoundClass::Fricative,
    }
}

fn repeated_bigram_count(bytes: &[u8]) -> usize {
    let windows = bytes.windows(2).collect::<Vec<_>>();
    windows
        .iter()
        .enumerate()
        .filter(|(index, bigram)| windows[..*index].contains(bigram))
        .count()
}

fn distinct_letter_count(bytes: &[u8]) -> usize {
    let mut present = [false; 26];
    for &byte in bytes {
        present[usize::from(byte - b'a')] = true;
    }
    present.into_iter().filter(|value| *value).count()
}

fn is_vowel(byte: u8) -> bool {
    matches!(byte, b'a' | b'e' | b'i' | b'o' | b'u')
}

#[cfg(test)]
mod tests {
    use super::analyze_brand;

    #[test]
    fn clean_candidate_exposes_full_score_breakdown() {
        let report = analyze_brand("Danoti");

        assert_eq!(report.normalized, "danoti");
        assert_eq!(report.profile, "international-tech-v1");
        assert_eq!(report.overall_score, 100);
        assert!(report.accepted);
        assert_eq!(report.scores.pronounceability, 100);
        assert_eq!(report.scores.rhythm, 100);
        assert_eq!(report.scores.vowel_balance, 100);
        assert_eq!(report.scores.repetition, 100);
        assert_eq!(report.scores.transition_quality, 100);
    }

    #[test]
    fn mechanical_repetition_reduces_brand_score() {
        let clean = analyze_brand("danoti");
        let repetitive = analyze_brand("pogoga");

        assert!(repetitive.scores.repetition < clean.scores.repetition);
        assert!(repetitive.scores.transition_quality < clean.scores.transition_quality);
        assert!(repetitive.overall_score < clean.overall_score);
        assert!(!repetitive.warnings.is_empty());
    }

    #[test]
    fn repeated_aligned_syllable_is_worse_than_partial_repetition() {
        let partial_repetition = analyze_brand("pogoga");
        let repeated_syllable = analyze_brand("folele");

        assert!(
            repeated_syllable.scores.transition_quality
                < partial_repetition.scores.transition_quality
        );
        assert!(!repeated_syllable.accepted);
    }

    #[test]
    fn invalid_input_is_rejected_with_zero_components() {
        let report = analyze_brand("nova-1");

        assert!(!report.accepted);
        assert_eq!(report.overall_score, 0);
        assert_eq!(report.scores.pronounceability, 0);
    }
}
