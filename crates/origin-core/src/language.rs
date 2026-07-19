//! Deterministic language systems derived from identity blueprints.

use serde::{Deserialize, Serialize};

use crate::{
    BlueprintTrait, EntityId, ExpressionFamily, IdentityBlueprint, IdentitySeed,
};

/// Current stable language system schema version.
pub const LANGUAGE_SYSTEM_SCHEMA_VERSION: u16 = 1;

const VOWEL_POOL: [&str; 12] = ["a", "e", "i", "o", "u", "ae", "ai", "ia", "io", "oa", "ui", "y"];
const SOFT_CONSONANT_POOL: [&str; 14] = [
    "l", "m", "n", "r", "s", "v", "f", "h", "w", "y", "th", "sh", "z", "j",
];
const HARD_CONSONANT_POOL: [&str; 14] = [
    "k", "g", "t", "d", "p", "b", "q", "x", "kh", "kr", "gr", "vr", "zh", "ch",
];
const CODA_POOL: [&str; 12] = ["n", "r", "s", "l", "m", "k", "th", "x", "nd", "rn", "sh", "v"];

/// Canonical segment category used by syllable templates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SegmentClass {
    /// Optional or required consonantal onset.
    Onset,
    /// Vocalic nucleus.
    Nucleus,
    /// Optional consonantal ending.
    Coda,
}

/// One deterministic syllable construction template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyllablePattern {
    /// Ordered segment classes.
    pub segments: Vec<SegmentClass>,
    /// Relative selection weight.
    pub weight: u16,
}

/// Deterministic morphological strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum MorphologyStrategy {
    /// Mostly indivisible roots.
    Isolating,
    /// Roots extended by clear reusable affixes.
    Agglutinative,
    /// Roots and affixes blend into compact forms.
    Fusional,
    /// Multiple semantic units combine into long forms.
    Synthetic,
}

/// Deterministic word-order preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum WordOrder {
    /// Subject–verb–object.
    Svo,
    /// Subject–object–verb.
    Sov,
    /// Verb–subject–object.
    Vso,
}

/// Complete language rules used by later naming and text generators.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageSystem {
    /// Language schema version.
    pub schema_version: u16,
    /// Identity of the language entity.
    pub language_id: EntityId,
    /// Domain-separated seed for language realization.
    pub seed: IdentitySeed,
    /// Canonical vowel inventory.
    pub vowels: Vec<String>,
    /// Canonical consonant inventory.
    pub consonants: Vec<String>,
    /// Permitted word-final consonants.
    pub codas: Vec<String>,
    /// Weighted syllable templates.
    pub syllable_patterns: Vec<SyllablePattern>,
    /// Morphological construction strategy.
    pub morphology: MorphologyStrategy,
    /// Preferred clause order.
    pub word_order: WordOrder,
    /// Minimum preferred word syllables.
    pub min_word_syllables: u8,
    /// Maximum preferred word syllables.
    pub max_word_syllables: u8,
    /// Whether adjacent vowels may form diphthongs.
    pub allows_diphthongs: bool,
    /// Whether consonant clusters may begin a syllable.
    pub allows_onset_clusters: bool,
    /// Stable compiler rule identifier.
    pub compiler_rule: String,
}

impl LanguageSystem {
    /// Derives a complete deterministic language system from an identity blueprint.
    #[must_use]
    pub fn derive(blueprint: &IdentityBlueprint, identity_seed: IdentitySeed) -> Self {
        let seed = identity_seed
            .derive("origin.language_system.v1")
            .derive(&blueprint.entity_id.to_string());

        let sonority = blueprint.trait_value(BlueprintTrait::Sonority).value();
        let force = blueprint.trait_value(BlueprintTrait::Force).value();
        let fluidity = blueprint.trait_value(BlueprintTrait::Fluidity).value();
        let distinctiveness = blueprint
            .trait_value(BlueprintTrait::Distinctiveness)
            .value();
        let systematicity = blueprint
            .trait_value(BlueprintTrait::Systematicity)
            .value();

        let vowel_count = usize::from(match sonority {
            750..=1000 => 9,
            400..=749 => 7,
            _ => 5,
        });
        let consonant_count = usize::from(match force {
            750..=1000 => 14,
            400..=749 => 11,
            _ => 8,
        });
        let coda_count = usize::from(match force.saturating_add(systematicity) / 2 {
            750..=1000 => 8,
            400..=749 => 5,
            _ => 3,
        });

        let vowels = select_inventory(&VOWEL_POOL, vowel_count, seed.derive("vowels"));
        let consonants = select_consonants(consonant_count, force, seed.derive("consonants"));
        let codas = select_inventory(&CODA_POOL, coda_count, seed.derive("codas"));
        let allows_diphthongs = sonority >= 550 || distinctiveness >= 700;
        let allows_onset_clusters = force >= 600 && blueprint.limits.max_cluster_width >= 2;
        let syllable_patterns = compile_patterns(
            fluidity,
            force,
            allows_diphthongs,
            allows_onset_clusters,
        );

        Self {
            schema_version: LANGUAGE_SYSTEM_SCHEMA_VERSION,
            language_id: blueprint.entity_id,
            seed,
            vowels,
            consonants,
            codas,
            syllable_patterns,
            morphology: select_morphology(blueprint),
            word_order: select_word_order(seed, systematicity),
            min_word_syllables: blueprint.limits.min_syllables,
            max_word_syllables: blueprint.limits.max_syllables,
            allows_diphthongs,
            allows_onset_clusters,
            compiler_rule: "origin.language_system.v1".to_owned(),
        }
    }

    /// Returns whether a segment belongs to the vowel inventory.
    #[must_use]
    pub fn contains_vowel(&self, segment: &str) -> bool {
        self.vowels.iter().any(|candidate| candidate == segment)
    }

    /// Returns whether a segment belongs to the consonant inventory.
    #[must_use]
    pub fn contains_consonant(&self, segment: &str) -> bool {
        self.consonants.iter().any(|candidate| candidate == segment)
    }
}

fn select_inventory(pool: &[&str], count: usize, seed: IdentitySeed) -> Vec<String> {
    let mut ranked = pool
        .iter()
        .enumerate()
        .map(|(index, segment)| {
            let rank = seed.derive(segment).value();
            (rank, index, *segment)
        })
        .collect::<Vec<_>>();
    ranked.sort_unstable_by_key(|(rank, index, _)| (*rank, *index));

    let mut selected = ranked
        .into_iter()
        .take(count.min(pool.len()))
        .map(|(_, index, segment)| (index, segment.to_owned()))
        .collect::<Vec<_>>();
    selected.sort_unstable_by_key(|(index, _)| *index);
    selected.into_iter().map(|(_, segment)| segment).collect()
}

fn select_consonants(count: usize, force: u16, seed: IdentitySeed) -> Vec<String> {
    let hard_target = usize::from(match force {
        750..=1000 => 9_u8,
        400..=749 => 6_u8,
        _ => 3_u8,
    })
    .min(count);
    let soft_target = count.saturating_sub(hard_target);

    let mut selected = select_inventory(&SOFT_CONSONANT_POOL, soft_target, seed.derive("soft"));
    selected.extend(select_inventory(
        &HARD_CONSONANT_POOL,
        hard_target,
        seed.derive("hard"),
    ));
    selected.sort();
    selected.dedup();
    selected
}

fn compile_patterns(
    fluidity: u16,
    force: u16,
    allows_diphthongs: bool,
    allows_onset_clusters: bool,
) -> Vec<SyllablePattern> {
    let mut patterns = vec![
        SyllablePattern {
            segments: vec![SegmentClass::Onset, SegmentClass::Nucleus],
            weight: 100,
        },
        SyllablePattern {
            segments: vec![SegmentClass::Onset, SegmentClass::Nucleus, SegmentClass::Coda],
            weight: 45 + force / 20,
        },
        SyllablePattern {
            segments: vec![SegmentClass::Nucleus, SegmentClass::Coda],
            weight: 25 + fluidity / 25,
        },
    ];

    if allows_diphthongs {
        patterns.push(SyllablePattern {
            segments: vec![
                SegmentClass::Onset,
                SegmentClass::Nucleus,
                SegmentClass::Nucleus,
            ],
            weight: 20 + fluidity / 20,
        });
    }
    if allows_onset_clusters {
        patterns.push(SyllablePattern {
            segments: vec![
                SegmentClass::Onset,
                SegmentClass::Onset,
                SegmentClass::Nucleus,
            ],
            weight: 20 + force / 18,
        });
    }

    patterns
}

fn select_morphology(blueprint: &IdentityBlueprint) -> MorphologyStrategy {
    let heritage = blueprint.trait_value(BlueprintTrait::Heritage).value();
    let systematicity = blueprint
        .trait_value(BlueprintTrait::Systematicity)
        .value();
    let distinctiveness = blueprint
        .trait_value(BlueprintTrait::Distinctiveness)
        .value();

    match blueprint.family {
        ExpressionFamily::Utilitarian if systematicity >= 600 => MorphologyStrategy::Agglutinative,
        ExpressionFamily::Archaic => MorphologyStrategy::Fusional,
        ExpressionFamily::Experimental if distinctiveness >= 650 => MorphologyStrategy::Synthetic,
        ExpressionFamily::Monumental if heritage >= 550 => MorphologyStrategy::Fusional,
        _ => MorphologyStrategy::Isolating,
    }
}

fn select_word_order(seed: IdentitySeed, systematicity: u16) -> WordOrder {
    if systematicity >= 800 {
        return WordOrder::Sov;
    }

    match seed.derive("word_order").value() % 3 {
        0 => WordOrder::Svo,
        1 => WordOrder::Sov,
        _ => WordOrder::Vso,
    }
}

#[cfg(test)]
mod tests {
    use super::{LANGUAGE_SYSTEM_SCHEMA_VERSION, LanguageSystem, SegmentClass};
    use crate::{
        EntityId, EntityKind, IdentityBlueprint, IdentityContext, IdentityGenome, IdentitySeed,
    };

    fn language(seed_value: u64) -> LanguageSystem {
        let context = IdentityContext::new(EntityKind::Language)
            .with_attribute("culture", "scholarly")
            .with_attribute("region", "outer-rim");
        let seed = IdentitySeed::new(seed_value);
        let id = EntityId::derive(context.kind, seed, "qverse");
        let genome = IdentityGenome::derive(id, seed, &context);
        let blueprint = IdentityBlueprint::compile(&genome);
        LanguageSystem::derive(&blueprint, seed)
    }

    #[test]
    fn language_derivation_is_deterministic() {
        assert_eq!(language(42), language(42));
    }

    #[test]
    fn different_seeds_produce_different_language_rules() {
        assert_ne!(language(42), language(43));
    }

    #[test]
    fn inventories_are_non_empty_and_unique() {
        let language = language(7);
        assert!(!language.vowels.is_empty());
        assert!(!language.consonants.is_empty());
        assert!(!language.codas.is_empty());

        let mut vowels = language.vowels.clone();
        vowels.sort();
        vowels.dedup();
        assert_eq!(vowels.len(), language.vowels.len());

        let mut consonants = language.consonants.clone();
        consonants.sort();
        consonants.dedup();
        assert_eq!(consonants.len(), language.consonants.len());
    }

    #[test]
    fn every_language_has_a_nucleus_bearing_pattern() {
        let language = language(99);
        assert!(language.syllable_patterns.iter().all(|pattern| {
            pattern.segments.contains(&SegmentClass::Nucleus) && pattern.weight > 0
        }));
    }

    #[test]
    fn language_limits_follow_the_blueprint_contract() {
        let language = language(123);
        assert!(language.min_word_syllables >= 1);
        assert!(language.max_word_syllables >= language.min_word_syllables);
        assert_eq!(language.schema_version, LANGUAGE_SYSTEM_SCHEMA_VERSION);
        assert!(language.contains_vowel(&language.vowels[0]));
        assert!(language.contains_consonant(&language.consonants[0]));
    }
}
