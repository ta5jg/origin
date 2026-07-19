//! Deterministic phonotactic analysis for candidate brand names.

use serde::Serialize;

/// Result of the phonotactic analysis of a single name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PhonotacticReport {
    /// Lowercase, trimmed representation used by the analyzer.
    pub normalized: String,
    /// Pronounceability-oriented score from zero to one hundred.
    pub score: u8,
    /// Whether the name passes the current phonotactic quality threshold.
    pub accepted: bool,
    /// Human-readable explanations for detected weaknesses.
    pub warnings: Vec<String>,
}

/// Evaluates a candidate name using deterministic, language-neutral rules.
///
/// The first model deliberately avoids claiming linguistic universality. It
/// catches structural defects that are broadly useful for technology-brand
/// discovery: invalid characters, extreme length, difficult consonant or vowel
/// runs, adjacent duplicates, repeated syllables and low vowel diversity.
#[must_use]
pub fn analyze_name(input: &str) -> PhonotacticReport {
    let normalized = input.trim().to_ascii_lowercase();
    let bytes = normalized.as_bytes();
    let mut warnings = Vec::new();

    if normalized.is_empty() {
        warnings.push(String::from("name is empty"));
        return rejected_report(normalized, warnings);
    }

    if !normalized.is_ascii() || !bytes.iter().all(u8::is_ascii_lowercase) {
        warnings.push(String::from(
            "name must contain lowercase ASCII letters after normalization",
        ));
        return rejected_report(normalized, warnings);
    }

    if !(4..=12).contains(&bytes.len()) {
        warnings.push(String::from("name length should be between 4 and 12 letters"));
        return rejected_report(normalized, warnings);
    }

    let mut score = 100_u8;

    if bytes.windows(2).any(|pair| pair[0] == pair[1]) {
        apply_penalty(
            &mut score,
            25,
            &mut warnings,
            "adjacent duplicate letters reduce clarity",
        );
    }

    let longest_vowel_run = longest_run(bytes, is_vowel);
    if longest_vowel_run > 2 {
        apply_penalty(
            &mut score,
            25,
            &mut warnings,
            "more than two consecutive vowels may be difficult to pronounce",
        );
    }

    let longest_consonant_run = longest_run(bytes, |byte| !is_vowel(byte));
    if longest_consonant_run > 3 {
        apply_penalty(
            &mut score,
            30,
            &mut warnings,
            "more than three consecutive consonants may be difficult to pronounce",
        );
    } else if longest_consonant_run == 3 {
        apply_penalty(
            &mut score,
            10,
            &mut warnings,
            "three consecutive consonants increase pronunciation difficulty",
        );
    }

    let vowel_count = bytes.iter().copied().filter(|byte| is_vowel(*byte)).count();
    if vowel_count == 0 {
        apply_penalty(
            &mut score,
            50,
            &mut warnings,
            "name contains no conventional vowel",
        );
    } else if vowel_count * 4 < bytes.len() || vowel_count * 3 > bytes.len() * 2 {
        apply_penalty(
            &mut score,
            15,
            &mut warnings,
            "vowel balance is outside the preferred range",
        );
    }

    if vowel_count >= 3 && distinct_vowel_count(bytes) == 1 {
        apply_penalty(
            &mut score,
            12,
            &mut warnings,
            "reusing a single vowel weakens phonetic variety",
        );
    }

    if has_repeated_two_letter_syllable(bytes) {
        apply_penalty(
            &mut score,
            30,
            &mut warnings,
            "a repeated two-letter syllable makes the name feel mechanical",
        );
    }

    let accepted = score >= 75;
    PhonotacticReport {
        normalized,
        score,
        accepted,
        warnings,
    }
}

fn rejected_report(normalized: String, warnings: Vec<String>) -> PhonotacticReport {
    PhonotacticReport {
        normalized,
        score: 0,
        accepted: false,
        warnings,
    }
}

fn apply_penalty(score: &mut u8, penalty: u8, warnings: &mut Vec<String>, warning: &str) {
    *score = score.saturating_sub(penalty);
    warnings.push(String::from(warning));
}

fn is_vowel(byte: u8) -> bool {
    matches!(byte, b'a' | b'e' | b'i' | b'o' | b'u')
}

fn longest_run(bytes: &[u8], predicate: impl Fn(u8) -> bool) -> usize {
    let mut longest = 0;
    let mut current = 0;

    for &byte in bytes {
        if predicate(byte) {
            current += 1;
            longest = longest.max(current);
        } else {
            current = 0;
        }
    }

    longest
}

fn distinct_vowel_count(bytes: &[u8]) -> usize {
    let mut present = [false; 5];
    for &byte in bytes {
        let index = match byte {
            b'a' => Some(0),
            b'e' => Some(1),
            b'i' => Some(2),
            b'o' => Some(3),
            b'u' => Some(4),
            _ => None,
        };
        if let Some(index) = index {
            present[index] = true;
        }
    }
    present.into_iter().filter(|value| *value).count()
}

fn has_repeated_two_letter_syllable(bytes: &[u8]) -> bool {
    let syllables = bytes.chunks_exact(2).collect::<Vec<_>>();
    syllables
        .iter()
        .enumerate()
        .any(|(index, syllable)| syllables[(index + 1)..].contains(syllable))
}

#[cfg(test)]
mod tests {
    use super::analyze_name;

    #[test]
    fn clean_name_is_accepted() {
        let report = analyze_name("Danoti");
        assert_eq!(report.normalized, "danoti");
        assert!(report.accepted);
        assert_eq!(report.score, 100);
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn repeated_syllable_is_penalized_and_rejected() {
        let report = analyze_name("folele");
        assert!(!report.accepted);
        assert_eq!(report.score, 70);
        assert_eq!(report.warnings.len(), 1);
    }

    #[test]
    fn invalid_characters_are_hard_rejected() {
        let report = analyze_name("nova-1");
        assert!(!report.accepted);
        assert_eq!(report.score, 0);
    }
}
