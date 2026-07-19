//! Deterministic, explainable similarity analysis for brand names.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

/// Shared contract for pairwise analyzers.
pub trait Analyzer {
    /// Report produced by the analyzer.
    type Report;

    /// Analyzes `candidate` against `reference`.
    fn analyze(&self, candidate: &str, reference: &str) -> Self::Report;
}

/// Risk classification derived from the overall similarity score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SimilarityRisk {
    /// Little or no meaningful resemblance.
    Minimal,
    /// Weak resemblance.
    Low,
    /// Material resemblance that merits review.
    Moderate,
    /// Strong resemblance.
    High,
    /// Exact or near-exact resemblance.
    Critical,
}

/// Weight configuration for the deterministic aggregate score.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimilarityWeights {
    /// Normalized Levenshtein contribution.
    pub levenshtein: u8,
    /// Normalized Damerau-Levenshtein contribution.
    pub damerau: u8,
    /// Bigram overlap contribution.
    pub bigram: u8,
    /// Trigram overlap contribution.
    pub trigram: u8,
    /// Prefix overlap contribution.
    pub prefix: u8,
    /// Suffix overlap contribution.
    pub suffix: u8,
    /// Shared-character contribution.
    pub shared_characters: u8,
    /// Phonetic approximation contribution.
    pub phonetic: u8,
    /// Visual-confusability contribution.
    pub visual: u8,
    /// Keyboard-neighbour contribution.
    pub keyboard: u8,
}

impl Default for SimilarityWeights {
    fn default() -> Self {
        Self {
            levenshtein: 18,
            damerau: 17,
            bigram: 10,
            trigram: 10,
            prefix: 7,
            suffix: 7,
            shared_characters: 6,
            phonetic: 15,
            visual: 6,
            keyboard: 4,
        }
    }
}

impl SimilarityWeights {
    fn total(self) -> u16 {
        u16::from(self.levenshtein)
            + u16::from(self.damerau)
            + u16::from(self.bigram)
            + u16::from(self.trigram)
            + u16::from(self.prefix)
            + u16::from(self.suffix)
            + u16::from(self.shared_characters)
            + u16::from(self.phonetic)
            + u16::from(self.visual)
            + u16::from(self.keyboard)
    }
}

/// Complete pairwise brand-name similarity report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SimilarityReport {
    /// Normalized candidate used by the engine.
    pub candidate: String,
    /// Normalized reference used by the engine.
    pub reference: String,
    /// Weighted aggregate score from zero to one hundred.
    pub overall_similarity: u8,
    /// Risk classification derived from the aggregate and exact-match rules.
    pub risk: SimilarityRisk,
    /// Levenshtein similarity from zero to one hundred.
    pub levenshtein_score: u8,
    /// Damerau-Levenshtein similarity from zero to one hundred.
    pub damerau_score: u8,
    /// Multiset Sørensen-Dice bigram similarity.
    pub bigram_similarity: u8,
    /// Multiset Sørensen-Dice trigram similarity.
    pub trigram_similarity: u8,
    /// Common-prefix similarity relative to the shorter name.
    pub prefix_similarity: u8,
    /// Common-suffix similarity relative to the shorter name.
    pub suffix_similarity: u8,
    /// Multiset shared-character ratio.
    pub shared_character_ratio: u8,
    /// Similarity after deterministic phonetic folding.
    pub phonetic_similarity: u8,
    /// Similarity after visual-confusable folding.
    pub visual_similarity: u8,
    /// Character-position similarity with QWERTY-neighbour tolerance.
    pub keyboard_similarity: u8,
    /// Human-readable reasons supporting the classification.
    pub warnings: Vec<String>,
}

/// Stateless deterministic similarity analyzer.
#[derive(Debug, Clone, Copy, Default)]
pub struct SimilarityAnalyzer {
    weights: SimilarityWeights,
}

impl SimilarityAnalyzer {
    /// Creates an analyzer using caller-provided aggregate weights.
    #[must_use]
    pub const fn with_weights(weights: SimilarityWeights) -> Self {
        Self { weights }
    }
}

impl Analyzer for SimilarityAnalyzer {
    type Report = SimilarityReport;

    fn analyze(&self, candidate: &str, reference: &str) -> Self::Report {
        analyze_similarity_with_weights(candidate, reference, self.weights)
    }
}

/// Analyzes two names using the default deterministic weight profile.
#[must_use]
pub fn analyze_similarity(candidate: &str, reference: &str) -> SimilarityReport {
    SimilarityAnalyzer::default().analyze(candidate, reference)
}

/// Analyzes two names using an explicit deterministic weight profile.
#[must_use]
pub fn analyze_similarity_with_weights(
    candidate: &str,
    reference: &str,
    weights: SimilarityWeights,
) -> SimilarityReport {
    let candidate = normalize(candidate);
    let reference = normalize(reference);
    let left = candidate.chars().collect::<Vec<_>>();
    let right = reference.chars().collect::<Vec<_>>();

    let levenshtein_score = distance_similarity(levenshtein_distance(&left, &right), max_len(&left, &right));
    let damerau_score = distance_similarity(damerau_levenshtein_distance(&left, &right), max_len(&left, &right));
    let bigram_similarity = ngram_similarity(&left, &right, 2);
    let trigram_similarity = ngram_similarity(&left, &right, 3);
    let prefix_similarity = edge_similarity(&left, &right, false);
    let suffix_similarity = edge_similarity(&left, &right, true);
    let shared_character_ratio = shared_character_similarity(&left, &right);

    let phonetic_left = phonetic_fold(&candidate).chars().collect::<Vec<_>>();
    let phonetic_right = phonetic_fold(&reference).chars().collect::<Vec<_>>();
    let phonetic_similarity = distance_similarity(
        damerau_levenshtein_distance(&phonetic_left, &phonetic_right),
        max_len(&phonetic_left, &phonetic_right),
    );

    let visual_left = visual_fold(&candidate).chars().collect::<Vec<_>>();
    let visual_right = visual_fold(&reference).chars().collect::<Vec<_>>();
    let visual_similarity = distance_similarity(
        damerau_levenshtein_distance(&visual_left, &visual_right),
        max_len(&visual_left, &visual_right),
    );
    let keyboard_similarity = keyboard_similarity(&left, &right);

    let metrics = [
        (levenshtein_score, weights.levenshtein),
        (damerau_score, weights.damerau),
        (bigram_similarity, weights.bigram),
        (trigram_similarity, weights.trigram),
        (prefix_similarity, weights.prefix),
        (suffix_similarity, weights.suffix),
        (shared_character_ratio, weights.shared_characters),
        (phonetic_similarity, weights.phonetic),
        (visual_similarity, weights.visual),
        (keyboard_similarity, weights.keyboard),
    ];
    let overall_similarity = weighted_score(&metrics, weights.total());
    let risk = classify_risk(&candidate, &reference, overall_similarity, damerau_score);
    let warnings = build_warnings(
        &candidate,
        &reference,
        overall_similarity,
        levenshtein_score,
        damerau_score,
        bigram_similarity,
        trigram_similarity,
        prefix_similarity,
        suffix_similarity,
        phonetic_similarity,
        visual_similarity,
        keyboard_similarity,
    );

    SimilarityReport {
        candidate,
        reference,
        overall_similarity,
        risk,
        levenshtein_score,
        damerau_score,
        bigram_similarity,
        trigram_similarity,
        prefix_similarity,
        suffix_similarity,
        shared_character_ratio,
        phonetic_similarity,
        visual_similarity,
        keyboard_similarity,
        warnings,
    }
}

fn normalize(input: &str) -> String {
    input
        .trim()
        .chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn max_len(left: &[char], right: &[char]) -> usize {
    left.len().max(right.len())
}

fn distance_similarity(distance: usize, longest: usize) -> u8 {
    if longest == 0 {
        return 100;
    }
    let retained = longest.saturating_sub(distance.min(longest));
    percentage(retained, longest)
}

fn percentage(numerator: usize, denominator: usize) -> u8 {
    if denominator == 0 {
        return 100;
    }
    let rounded = (numerator.saturating_mul(100) + denominator / 2) / denominator;
    u8::try_from(rounded.min(100)).unwrap_or(100)
}

fn levenshtein_distance(left: &[char], right: &[char]) -> usize {
    if left.is_empty() {
        return right.len();
    }
    if right.is_empty() {
        return left.len();
    }

    let (rows, columns) = if left.len() >= right.len() {
        (left, right)
    } else {
        (right, left)
    };
    let mut previous = (0..=columns.len()).collect::<Vec<_>>();
    let mut current = vec![0; columns.len() + 1];

    for (row_index, row_character) in rows.iter().enumerate() {
        current[0] = row_index + 1;
        for (column_index, column_character) in columns.iter().enumerate() {
            let substitution = usize::from(row_character != column_character);
            current[column_index + 1] = (previous[column_index + 1] + 1)
                .min(current[column_index] + 1)
                .min(previous[column_index] + substitution);
        }
        std::mem::swap(&mut previous, &mut current);
    }

    previous[columns.len()]
}

fn damerau_levenshtein_distance(left: &[char], right: &[char]) -> usize {
    let rows = left.len() + 1;
    let columns = right.len() + 1;
    let mut matrix = vec![vec![0_usize; columns]; rows];

    for (index, row) in matrix.iter_mut().enumerate() {
        row[0] = index;
    }
    for (index, cell) in matrix[0].iter_mut().enumerate() {
        *cell = index;
    }

    for left_index in 1..rows {
        for right_index in 1..columns {
            let substitution = usize::from(left[left_index - 1] != right[right_index - 1]);
            let mut value = (matrix[left_index - 1][right_index] + 1)
                .min(matrix[left_index][right_index - 1] + 1)
                .min(matrix[left_index - 1][right_index - 1] + substitution);

            if left_index > 1
                && right_index > 1
                && left[left_index - 1] == right[right_index - 2]
                && left[left_index - 2] == right[right_index - 1]
            {
                value = value.min(matrix[left_index - 2][right_index - 2] + 1);
            }
            matrix[left_index][right_index] = value;
        }
    }

    matrix[left.len()][right.len()]
}

fn ngram_similarity(left: &[char], right: &[char], size: usize) -> u8 {
    if left == right {
        return 100;
    }
    let left_ngrams = ngram_counts(left, size);
    let right_ngrams = ngram_counts(right, size);
    let left_total = left_ngrams.values().sum::<usize>();
    let right_total = right_ngrams.values().sum::<usize>();
    if left_total + right_total == 0 {
        return 0;
    }

    let overlap = left_ngrams
        .iter()
        .map(|(ngram, count)| count.min(right_ngrams.get(ngram).unwrap_or(&0)))
        .sum::<usize>();
    percentage(overlap.saturating_mul(2), left_total + right_total)
}

fn ngram_counts(input: &[char], size: usize) -> BTreeMap<Vec<char>, usize> {
    if size == 0 || input.len() < size {
        return BTreeMap::new();
    }
    let mut counts = BTreeMap::new();
    for window in input.windows(size) {
        *counts.entry(window.to_vec()).or_insert(0) += 1;
    }
    counts
}

fn edge_similarity(left: &[char], right: &[char], reverse: bool) -> u8 {
    let denominator = left.len().min(right.len());
    if denominator == 0 {
        return u8::from(left.is_empty() && right.is_empty()) * 100;
    }
    let matches = if reverse {
        left.iter()
            .rev()
            .zip(right.iter().rev())
            .take_while(|(a, b)| a == b)
            .count()
    } else {
        left.iter()
            .zip(right.iter())
            .take_while(|(a, b)| a == b)
            .count()
    };
    percentage(matches, denominator)
}

fn shared_character_similarity(left: &[char], right: &[char]) -> u8 {
    if left.is_empty() && right.is_empty() {
        return 100;
    }
    let mut left_counts = BTreeMap::new();
    let mut right_counts = BTreeMap::new();
    for character in left {
        *left_counts.entry(*character).or_insert(0_usize) += 1;
    }
    for character in right {
        *right_counts.entry(*character).or_insert(0_usize) += 1;
    }
    let shared = left_counts
        .iter()
        .map(|(character, count)| count.min(right_counts.get(character).unwrap_or(&0)))
        .sum::<usize>();
    percentage(shared.saturating_mul(2), left.len() + right.len())
}

fn phonetic_fold(input: &str) -> String {
    let mut folded = String::with_capacity(input.len());
    for character in input.chars() {
        let replacement = match character {
            'c' | 'k' | 'q' => 'k',
            'f' | 'v' | 'w' => 'f',
            'g' | 'j' => 'j',
            's' | 'z' | 'x' => 's',
            'y' | 'i' => 'i',
            'o' | 'u' => 'u',
            other => other,
        };
        if folded.ends_with(replacement) {
            continue;
        }
        folded.push(replacement);
    }
    folded
        .replace("ph", "f")
        .replace("ch", "c")
        .replace("sh", "s")
}

fn visual_fold(input: &str) -> String {
    input.chars().map(visual_character).collect()
}

fn visual_character(character: char) -> char {
    match character {
        '0' | 'ο' | 'о' => 'o',
        '1' | 'ı' | 'і' | 'ӏ' | 'l' => 'i',
        '3' => 'e',
        '5' | 'ѕ' => 's',
        '8' => 'b',
        'а' | 'α' => 'a',
        'с' | 'ϲ' => 'c',
        'е' | 'ε' => 'e',
        'к' | 'κ' => 'k',
        'м' | 'μ' => 'm',
        'н' | 'η' => 'h',
        'р' | 'ρ' => 'p',
        'т' | 'τ' => 't',
        'х' | 'χ' => 'x',
        'у' | 'γ' => 'y',
        other => other,
    }
}

fn keyboard_similarity(left: &[char], right: &[char]) -> u8 {
    if left.is_empty() && right.is_empty() {
        return 100;
    }
    let denominator = left.len().max(right.len());
    let points = left
        .iter()
        .zip(right.iter())
        .map(|(a, b)| {
            if a == b {
                2_usize
            } else if keyboard_neighbours(*a, *b) {
                1
            } else {
                0
            }
        })
        .sum::<usize>();
    percentage(points, denominator.saturating_mul(2))
}

fn keyboard_neighbours(left: char, right: char) -> bool {
    const ROWS: [&str; 3] = ["qwertyuiop", "asdfghjkl", "zxcvbnm"];
    let positions = ROWS
        .iter()
        .enumerate()
        .flat_map(|(row, letters)| {
            letters
                .chars()
                .enumerate()
                .map(move |(column, character)| (character, row, column))
        })
        .collect::<BTreeSet<_>>();
    let left_position = positions.iter().find(|(character, _, _)| *character == left);
    let right_position = positions.iter().find(|(character, _, _)| *character == right);
    match (left_position, right_position) {
        (Some((_, left_row, left_column)), Some((_, right_row, right_column))) => {
            left_row.abs_diff(*right_row) <= 1 && left_column.abs_diff(*right_column) <= 1
        }
        _ => false,
    }
}

fn weighted_score(metrics: &[(u8, u8)], total_weight: u16) -> u8 {
    if total_weight == 0 {
        return 0;
    }
    let weighted = metrics
        .iter()
        .map(|(score, weight)| u32::from(*score) * u32::from(*weight))
        .sum::<u32>();
    let rounded = (weighted + u32::from(total_weight) / 2) / u32::from(total_weight);
    u8::try_from(rounded.min(100)).unwrap_or(100)
}

fn classify_risk(candidate: &str, reference: &str, overall: u8, damerau: u8) -> SimilarityRisk {
    if !candidate.is_empty() && candidate == reference {
        return SimilarityRisk::Critical;
    }
    let effective = overall.max(damerau.saturating_sub(3));
    match effective {
        90..=100 => SimilarityRisk::Critical,
        78..=89 => SimilarityRisk::High,
        60..=77 => SimilarityRisk::Moderate,
        40..=59 => SimilarityRisk::Low,
        _ => SimilarityRisk::Minimal,
    }
}

#[allow(clippy::too_many_arguments)]
fn build_warnings(
    candidate: &str,
    reference: &str,
    overall: u8,
    levenshtein: u8,
    damerau: u8,
    bigram: u8,
    trigram: u8,
    prefix: u8,
    suffix: u8,
    phonetic: u8,
    visual: u8,
    keyboard: u8,
) -> Vec<String> {
    let mut warnings = Vec::new();
    if candidate.is_empty() || reference.is_empty() {
        warnings.push(String::from("one or both normalized names are empty"));
        return warnings;
    }
    if candidate == reference {
        warnings.push(String::from("the normalized names are identical"));
    }
    if damerau > levenshtein && damerau >= 75 {
        warnings.push(String::from("a character transposition strongly links the names"));
    }
    if bigram >= 70 {
        warnings.push(String::from("the names share a high proportion of adjacent letter pairs"));
    }
    if trigram >= 65 {
        warnings.push(String::from("the names share distinctive three-letter sequences"));
    }
    if prefix >= 70 {
        warnings.push(String::from("the names begin with the same or a very similar sequence"));
    }
    if suffix >= 70 {
        warnings.push(String::from("the names end with the same or a very similar sequence"));
    }
    if phonetic >= 80 {
        warnings.push(String::from("the names are likely to sound similar when spoken"));
    }
    if visual >= 85 && visual > damerau {
        warnings.push(String::from("visually confusable characters increase resemblance"));
    }
    if keyboard >= 85 && overall >= 60 {
        warnings.push(String::from("the difference is consistent with a nearby-key typing error"));
    }
    if overall >= 78 {
        warnings.push(String::from("the combined similarity is high enough to merit conflict review"));
    }
    warnings
}

#[cfg(test)]
mod tests {
    use super::{Analyzer, SimilarityAnalyzer, SimilarityRisk, analyze_similarity};

    #[test]
    fn exact_names_are_critical() {
        let report = analyze_similarity(" Nova ", "nova");
        assert_eq!(report.overall_similarity, 100);
        assert_eq!(report.risk, SimilarityRisk::Critical);
        assert!(report.warnings.iter().any(|warning| warning.contains("identical")));
    }

    #[test]
    fn transposition_is_detected_by_damerau_metric() {
        let report = analyze_similarity("google", "googel");
        assert!(report.damerau_score > report.levenshtein_score);
        assert!(report.damerau_score >= 80);
        assert!(report.risk >= SimilarityRisk::High);
    }

    #[test]
    fn deletion_remains_high_similarity() {
        let report = analyze_similarity("origin", "orign");
        assert!(report.levenshtein_score >= 80);
        assert!(report.overall_similarity >= 70);
    }

    #[test]
    fn unrelated_names_have_low_risk() {
        let report = analyze_similarity("danoti", "xelvar");
        assert!(report.overall_similarity < 50);
        assert!(report.risk <= SimilarityRisk::Low);
    }

    #[test]
    fn phonetic_folding_links_cargo_and_kargo() {
        let report = analyze_similarity("cargo", "kargo");
        assert_eq!(report.phonetic_similarity, 100);
        assert!(report.overall_similarity >= 60);
    }

    #[test]
    fn visual_confusables_are_folded() {
        let report = analyze_similarity("n0va", "nova");
        assert_eq!(report.visual_similarity, 100);
        assert!(report.overall_similarity >= 70);
    }

    #[test]
    fn unicode_confusables_are_folded() {
        let report = analyze_similarity("nоva", "nova");
        assert_eq!(report.visual_similarity, 100);
    }

    #[test]
    fn ngrams_capture_shared_structure() {
        let report = analyze_similarity("spotify", "spotifai");
        assert!(report.bigram_similarity >= 65);
        assert!(report.trigram_similarity >= 50);
    }

    #[test]
    fn prefix_and_suffix_are_reported_separately() {
        let prefix = analyze_similarity("openai", "openly");
        let suffix = analyze_similarity("datasync", "filesync");
        assert!(prefix.prefix_similarity > prefix.suffix_similarity);
        assert!(suffix.suffix_similarity > suffix.prefix_similarity);
    }

    #[test]
    fn analyzer_trait_matches_free_function() {
        let analyzer = SimilarityAnalyzer::default();
        assert_eq!(analyzer.analyze("pixel", "pixxel"), analyze_similarity("pixel", "pixxel"));
    }

    #[test]
    fn empty_inputs_are_deterministic() {
        let report = analyze_similarity("", "");
        assert_eq!(report.overall_similarity, 100);
        assert_eq!(report.risk, SimilarityRisk::Critical);
    }

    #[test]
    fn punctuation_and_case_do_not_create_false_distance() {
        let report = analyze_similarity("Open-AI", "open ai");
        assert_eq!(report.candidate, "openai");
        assert_eq!(report.reference, "openai");
        assert_eq!(report.overall_similarity, 100);
    }
}
