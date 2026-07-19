//! Deterministic multi-step search over one-phoneme improvements.

use std::collections::HashSet;

use serde::Serialize;

use super::{ImproveOptions, MutationKind, improve};
use crate::{BrandReport, analyze_brand};

/// Configuration for deterministic multi-step improvement search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BeamSearchOptions {
    /// Maximum number of final ranked results to return.
    pub count: usize,
    /// Maximum number of active candidates retained after each depth.
    pub beam_width: usize,
    /// Maximum number of sequential one-phoneme mutations.
    pub depth: usize,
    /// Seed used for deterministic tie-breaking at every depth.
    pub seed: u64,
}

impl Default for BeamSearchOptions {
    fn default() -> Self {
        Self {
            count: 10,
            beam_width: 12,
            depth: 2,
            seed: 1,
        }
    }
}

/// One explainable mutation inside a multi-step search path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MutationStep {
    /// Name before this mutation.
    pub from: String,
    /// Name after this mutation.
    pub to: String,
    /// Score change relative to the preceding path node.
    pub score_delta: i16,
    /// Position changed in the preceding name.
    pub changed_position: usize,
    /// Original character at the changed position.
    pub replaced: char,
    /// Replacement character at the changed position.
    pub replacement: char,
    /// Phonetic similarity of the replacement, from one to three.
    pub phonetic_affinity: u8,
    /// Semantic category of the replacement.
    pub mutation_kind: MutationKind,
    /// Stable human-readable explanation of the replacement.
    pub explanation: &'static str,
}

/// One ranked result reached through a deterministic mutation path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BeamCandidate {
    /// Final candidate name.
    pub name: String,
    /// Final weighted score.
    pub score: u8,
    /// Score change relative to the original input.
    pub total_delta: i16,
    /// Whether the final candidate passes the active profile threshold.
    pub accepted: bool,
    /// Ordered names from the original input to the final candidate.
    pub path: Vec<String>,
    /// Explainable mutations connecting adjacent path nodes.
    pub steps: Vec<MutationStep>,
    /// Complete report for the final candidate.
    pub report: BrandReport,
}

/// Complete deterministic beam-search result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BeamSearchReport {
    /// Analysis of the normalized original name.
    pub original: BrandReport,
    /// Ranked candidates discovered across all explored depths.
    pub results: Vec<BeamCandidate>,
}

/// Searches multiple sequential one-phoneme improvements deterministically.
///
/// The search retains only the strongest `beam_width` candidates after each
/// depth, while preserving the full explainable mutation path for every result.
#[must_use]
pub fn beam_search(input: &str, options: BeamSearchOptions) -> BeamSearchReport {
    let original = analyze_brand(input);
    if original.overall_score == 0 || options.depth == 0 || options.count == 0 {
        return BeamSearchReport {
            original,
            results: Vec::new(),
        };
    }

    let beam_width = options.beam_width.max(1);
    let original_score = original.overall_score;
    let original_name = original.normalized.clone();
    let mut seen = HashSet::from([original_name.clone()]);
    let mut frontier = vec![BeamCandidate {
        name: original_name.clone(),
        score: original_score,
        total_delta: 0,
        accepted: original.accepted,
        path: vec![original_name],
        steps: Vec::new(),
        report: original.clone(),
    }];
    let mut discovered = Vec::new();

    for depth_index in 0..options.depth {
        let mut expanded = Vec::new();
        let depth_seed = options.seed.wrapping_add(depth_index as u64);

        for parent in &frontier {
            let improvements = improve(
                &parent.name,
                ImproveOptions {
                    count: beam_width,
                    seed: depth_seed,
                },
            );

            for suggestion in improvements.suggestions {
                if !seen.insert(suggestion.name.clone()) {
                    continue;
                }

                let mut path = parent.path.clone();
                path.push(suggestion.name.clone());

                let mut steps = parent.steps.clone();
                steps.push(MutationStep {
                    from: parent.name.clone(),
                    to: suggestion.name.clone(),
                    score_delta: suggestion.score_delta,
                    changed_position: suggestion.changed_position,
                    replaced: suggestion.replaced,
                    replacement: suggestion.replacement,
                    phonetic_affinity: suggestion.phonetic_affinity,
                    mutation_kind: suggestion.mutation_kind,
                    explanation: suggestion.explanation,
                });

                expanded.push(BeamCandidate {
                    name: suggestion.name,
                    score: suggestion.score,
                    total_delta: i16::from(suggestion.score) - i16::from(original_score),
                    accepted: suggestion.accepted,
                    path,
                    steps,
                    report: suggestion.report,
                });
            }
        }

        rank_candidates(&mut expanded, options.seed);
        expanded.truncate(beam_width);
        discovered.extend(expanded.iter().cloned());
        frontier = expanded;

        if frontier.is_empty() {
            break;
        }
    }

    rank_candidates(&mut discovered, options.seed);
    discovered.truncate(options.count);

    BeamSearchReport {
        original,
        results: discovered,
    }
}

fn rank_candidates(candidates: &mut [BeamCandidate], seed: u64) {
    candidates.sort_unstable_by(|left, right| {
        right
            .accepted
            .cmp(&left.accepted)
            .then_with(|| right.score.cmp(&left.score))
            .then_with(|| right.total_delta.cmp(&left.total_delta))
            .then_with(|| left.steps.len().cmp(&right.steps.len()))
            .then_with(|| tie_key(&left.name, seed).cmp(&tie_key(&right.name, seed)))
            .then_with(|| left.name.cmp(&right.name))
    });
}

fn tie_key(name: &str, seed: u64) -> u64 {
    name.bytes().fold(seed ^ 0x517C_C1B7_2722_0A95, |value, byte| {
        value
            .wrapping_mul(0x100_0000_01B3)
            .wrapping_add(u64::from(byte))
    })
}

#[cfg(test)]
mod tests {
    use super::{BeamSearchOptions, beam_search};

    #[test]
    fn beam_search_is_deterministic() {
        let options = BeamSearchOptions {
            count: 8,
            beam_width: 10,
            depth: 2,
            seed: 42,
        };
        assert_eq!(beam_search("folele", options), beam_search("folele", options));
    }

    #[test]
    fn beam_search_preserves_explainable_paths() {
        let report = beam_search("folele", BeamSearchOptions::default());

        assert!(!report.results.is_empty());
        assert!(report.results.iter().all(|candidate| {
            candidate.path.len() == candidate.steps.len() + 1
                && candidate.path.first() == Some(&report.original.normalized)
                && candidate.path.last() == Some(&candidate.name)
        }));
    }

    #[test]
    fn invalid_input_returns_no_beam_results() {
        let report = beam_search("nova-1", BeamSearchOptions::default());
        assert!(report.results.is_empty());
    }
}
