//! Deterministic multi-strategy brand-name design.

use std::collections::BTreeSet;

use serde::Serialize;

use crate::{BrandReport, analyze_brand, built_in_catalog, merge_roots};

/// Maximum candidate count for one name-design run.
pub const MAX_DESIGN_CANDIDATES: usize = 10_000;

/// Candidate-origin strategy used by the brand designer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DesignStrategy {
    /// Invented but pronounceable phonetic construction.
    Invented,
    /// Sound-inspired construction based on a curated historical root.
    AncientInspired,
    /// Curated-root merge with a modern phonetic ending.
    Hybrid,
    /// Meaning-free, deliberately brandable phonetic construction.
    GoogleStyle,
}

/// Controls one deterministic brand-design run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignOptions {
    /// Maximum number of candidates returned, capped at [`MAX_DESIGN_CANDIDATES`].
    pub count: usize,
    /// Seed controlling deterministic traversal and tie-breaking.
    pub seed: u64,
    /// Desired themes or meanings, used to select root inspiration.
    pub meanings: Vec<String>,
    /// Optional industry signal included in root selection.
    pub industry: Option<String>,
    /// Explicit curated root identifiers, if the caller wants to override discovery.
    pub roots: Vec<String>,
}

impl Default for DesignOptions {
    fn default() -> Self {
        Self {
            count: 100,
            seed: 1,
            meanings: Vec::new(),
            industry: None,
            roots: Vec::new(),
        }
    }
}

/// One explainable candidate from the brand-design engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DesignedCandidate {
    /// Candidate spelling in canonical lowercase ASCII.
    pub name: String,
    /// Strategy that produced this candidate.
    pub strategy: DesignStrategy,
    /// Semantic roots or phonetic design cues that inspired this candidate.
    pub inspiration: Vec<String>,
    /// Intrinsic phonetic and brand-quality analysis.
    pub analysis: BrandReport,
    /// Deterministic visual wordmark balance score.
    pub typography_score: u8,
    /// Combined ranking score before external clearance research.
    pub design_score: u8,
}

/// Generates ranked, deterministic candidates from four complementary strategies.
///
/// The pool splits approximate capacity 50/15/20/15 across invented,
/// ancient-inspired, hybrid, and meaning-free Google-style constructions. The
/// result is deduplicated and never asserts external availability.
#[must_use]
pub fn design_brands(options: &DesignOptions) -> Vec<DesignedCandidate> {
    let requested = options.count.clamp(1, MAX_DESIGN_CANDIDATES);
    let root_ids = selected_root_ids(options);
    let mut unique = BTreeSet::new();
    let mut candidates = Vec::with_capacity(requested);
    let attempts = requested.saturating_mul(12);

    for index in 0..attempts {
        let (name, strategy, inspiration) = match index % 20 {
            0..=9 => (
                invented_name(options.seed, index, false),
                DesignStrategy::Invented,
                vec!["invented phonetic structure".into()],
            ),
            10..=12 => ancient_inspired_name(options.seed, index, &root_ids),
            13..=16 => hybrid_name(options.seed, index, &root_ids),
            _ => (
                invented_name(options.seed, index, true),
                DesignStrategy::GoogleStyle,
                vec!["meaning-free brand construction".into()],
            ),
        };
        if !(5..=8).contains(&name.len()) || !unique.insert(name.clone()) {
            continue;
        }
        let analysis = analyze_brand(&name);
        let typography_score = typography_score(&name);
        let design_score = weighted_score(&analysis, typography_score, strategy);
        candidates.push(DesignedCandidate {
            name,
            strategy,
            inspiration,
            analysis,
            typography_score,
            design_score,
        });
        if candidates.len() == requested {
            break;
        }
    }

    candidates.sort_unstable_by(|left, right| {
        right
            .analysis
            .accepted
            .cmp(&left.analysis.accepted)
            .then_with(|| right.design_score.cmp(&left.design_score))
            .then_with(|| left.name.cmp(&right.name))
    });
    candidates
}

fn selected_root_ids(options: &DesignOptions) -> Vec<String> {
    let catalog = built_in_catalog();
    let mut selected = options
        .roots
        .iter()
        .filter(|id| catalog.contains_id(id))
        .cloned()
        .collect::<Vec<_>>();
    let signals = options
        .meanings
        .iter()
        .chain(options.industry.iter())
        .flat_map(|value| value.split(|character: char| !character.is_ascii_alphabetic()))
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();
    for signal in signals {
        for id in roots_for_signal(&signal) {
            if catalog.contains_id(id) && !selected.iter().any(|existing| existing == id) {
                selected.push((*id).into());
            }
        }
    }
    if selected.is_empty() {
        selected = catalog.iter().map(|root| root.id.clone()).collect();
    }
    selected
}

fn roots_for_signal(signal: &str) -> &'static [&'static str] {
    match signal {
        "future" | "new" | "next" => &["latin-nov", "latin-lux", "sanskrit-veda"],
        "light" | "vision" | "clarity" => &["latin-lux", "latin-ver", "sumerian-eme"],
        "trust" | "truth" | "secure" | "security" => &["latin-ver", "latin-fort", "old-turkic-kut"],
        "space" | "world" | "civilization" | "planet" => {
            &["latin-terra", "latin-loc", "sumerian-uru", "sumerian-edin"]
        }
        "mind" | "intelligence" | "ai" => &["latin-nex", "latin-lux", "sanskrit-veda"],
        "network" | "connection" | "distributed" => {
            &["latin-nex", "latin-via", "sanskrit-sutra", "akkadian-babu"]
        }
        "life" | "health" | "growth" => &["latin-vita", "latin-nov", "sanskrit-dhara"],
        "movement" | "travel" | "logistics" | "flow" => &[
            "latin-via",
            "latin-temp",
            "old-turkic-yol",
            "sanskrit-yatra",
        ],
        _ => &[],
    }
}

fn ancient_inspired_name(
    seed: u64,
    index: usize,
    roots: &[String],
) -> (String, DesignStrategy, Vec<String>) {
    let root = &roots[choose(seed, index, roots.len())];
    let catalog = built_in_catalog();
    let form = &catalog
        .get(root)
        .expect("selected roots come from catalog")
        .normalized;
    let suffix = ending(seed, index + 17);
    (
        format!("{form}{suffix}"),
        DesignStrategy::AncientInspired,
        vec![root.clone()],
    )
}

fn hybrid_name(seed: u64, index: usize, roots: &[String]) -> (String, DesignStrategy, Vec<String>) {
    let catalog = built_in_catalog();
    let left_id = &roots[choose(seed, index + 31, roots.len())];
    let right_id = &roots[choose(seed, index + 73, roots.len())];
    let left = &catalog
        .get(left_id)
        .expect("selected roots come from catalog")
        .normalized;
    let right = &catalog
        .get(right_id)
        .expect("selected roots come from catalog")
        .normalized;
    let merged = merge_roots(left, right).unwrap_or_else(|_| format!("{left}{right}"));
    let name = if merged.len() < 5 {
        format!("{merged}{}", ending(seed, index + 107))
    } else if merged.len() > 8 {
        format!(
            "{}{}",
            &left[..left.len().min(5)],
            ending(seed, index + 107)
        )
    } else {
        merged
    };
    (
        name,
        DesignStrategy::Hybrid,
        vec![left_id.clone(), right_id.clone()],
    )
}

fn invented_name(seed: u64, index: usize, google_style: bool) -> String {
    const STARTS: &[&str] = &[
        "av", "vel", "nor", "or", "zen", "tal", "kor", "vir", "lor", "qer", "xen", "rav", "syl",
        "nuv", "mer",
    ];
    const MIDDLES: &[&str] = &["a", "e", "i", "o", "ar", "er", "or", "en", "iv", "av"];
    const BRIDGES: &[&str] = &["", "a", "e", "i", "o", "n", "r", "v"];
    const ENDS: &[&str] = &["n", "r", "x", "on", "or", "ia", "iq", "ex", "en", "is"];
    const GOOGLE_ENDS: &[&str] = &["on", "or", "ex", "iq", "io", "ar", "en", "um"];
    let start = STARTS[choose(seed, index, STARTS.len())];
    let middle = MIDDLES[choose(seed, index + 11, MIDDLES.len())];
    let bridge = BRIDGES[choose(seed, index + 19, BRIDGES.len())];
    let ends = if google_style { GOOGLE_ENDS } else { ENDS };
    let end = ends[choose(seed, index + 29, ends.len())];
    format!("{start}{middle}{bridge}{end}")
}

fn ending(seed: u64, index: usize) -> &'static str {
    const ENDINGS: &[&str] = &["on", "or", "ex", "ion", "ara", "en", "is", "um"];
    ENDINGS[choose(seed, index, ENDINGS.len())]
}

fn choose(seed: u64, index: usize, length: usize) -> usize {
    let mut value = seed
        ^ u64::try_from(index)
            .unwrap_or_default()
            .wrapping_mul(0x9e37_79b9_7f4a_7c15);
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^= value >> 31;
    usize::try_from(value % u64::try_from(length).unwrap_or(1)).unwrap_or_default()
}

fn typography_score(name: &str) -> u8 {
    let vowels = name.bytes().filter(|byte| b"aeiou".contains(byte)).count();
    let length = name.len();
    let vowel_balance = (i16::try_from(vowels * 2).unwrap_or_default()
        - i16::try_from(length).unwrap_or_default() / 2)
        .unsigned_abs();
    let distinct = name.bytes().collect::<BTreeSet<_>>().len();
    let balance =
        100_u8.saturating_sub(u8::try_from(vowel_balance.saturating_mul(18)).unwrap_or(u8::MAX));
    let diversity = u8::try_from((distinct * 100) / length).unwrap_or_default();
    u8::try_from(u16::from(balance) * 55 / 100 + u16::from(diversity) * 45 / 100).unwrap_or(u8::MAX)
}

fn weighted_score(analysis: &BrandReport, typography: u8, strategy: DesignStrategy) -> u8 {
    let strategy_bonus = match strategy {
        DesignStrategy::Invented => 1,
        DesignStrategy::AncientInspired => 3,
        DesignStrategy::Hybrid => 5,
        DesignStrategy::GoogleStyle => 2,
    };
    let score = (u16::from(analysis.overall_score) * 80 + u16::from(typography) * 20) / 100;
    u8::try_from(score)
        .unwrap_or_default()
        .saturating_add(strategy_bonus)
        .min(100)
}

#[cfg(test)]
mod tests {
    use super::{DesignOptions, DesignStrategy, MAX_DESIGN_CANDIDATES, design_brands};

    #[test]
    fn design_is_deterministic_and_respects_length_contract() {
        let options = DesignOptions {
            count: 200,
            seed: 42,
            meanings: vec!["future civilization".into()],
            ..Default::default()
        };
        let first = design_brands(&options);
        let second = design_brands(&options);
        assert_eq!(first, second);
        assert!(
            first
                .iter()
                .all(|candidate| (5..=8).contains(&candidate.name.len()))
        );
    }

    #[test]
    fn semantic_input_selects_all_design_strategies() {
        let options = DesignOptions {
            count: 100,
            meanings: vec!["distributed trust".into()],
            ..Default::default()
        };
        let candidates = design_brands(&options);
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.strategy == DesignStrategy::Invented)
        );
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.strategy == DesignStrategy::AncientInspired)
        );
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.strategy == DesignStrategy::Hybrid)
        );
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.strategy == DesignStrategy::GoogleStyle)
        );
    }

    #[test]
    fn design_caps_to_the_product_candidate_limit() {
        let options = DesignOptions {
            count: MAX_DESIGN_CANDIDATES + 1,
            ..Default::default()
        };
        let candidates = design_brands(&options);
        assert_eq!(candidates.len(), MAX_DESIGN_CANDIDATES);
    }
}
