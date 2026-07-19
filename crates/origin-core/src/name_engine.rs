//! Deterministic identity-aware name generation.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    BlueprintTrait, EntityId, IdentityBlueprint, IdentitySeed, LanguageSystem, SegmentClass,
    SyllablePattern,
};

/// Current stable name engine schema version.
pub const NAME_ENGINE_SCHEMA_VERSION: u16 = 1;

/// Default number of ranked candidates produced by [`NameGenerator`].
pub const DEFAULT_NAME_CANDIDATES: usize = 12;

/// One ranked deterministic name candidate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NameCandidate {
    /// Display form with canonical capitalization.
    pub text: String,
    /// Lowercase pronunciation approximation.
    pub pronunciation: String,
    /// Number of generated syllables.
    pub syllables: u8,
    /// Deterministic quality score in the inclusive range `0..=1000`.
    pub score: u16,
    /// Stable candidate derivation index.
    pub derivation_index: u16,
}

/// Explainable source record for a generated name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NameOrigin {
    /// Identity receiving the generated name.
    pub entity_id: EntityId,
    /// Language identity used during realization.
    pub language_id: EntityId,
    /// Original caller-provided seed.
    pub identity_seed: IdentitySeed,
    /// Domain-separated seed used by the name engine.
    pub generation_seed: IdentitySeed,
    /// Blueprint schema consumed by the engine.
    pub blueprint_schema_version: u16,
    /// Language schema consumed by the engine.
    pub language_schema_version: u16,
    /// Stable generation rule identifier.
    pub generation_rule: String,
}

/// Final deterministic naming result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedName {
    /// Name engine schema version.
    pub schema_version: u16,
    /// Highest-ranked canonical name.
    pub canonical: String,
    /// Pronunciation of the canonical name.
    pub pronunciation: String,
    /// Ranked unique candidates, best first.
    pub candidates: Vec<NameCandidate>,
    /// Explainable derivation origin.
    pub origin: NameOrigin,
}

/// Contract implemented by deterministic identity name generators.
pub trait GenerateName {
    /// Generates a ranked name set from a blueprint, language and seed.
    #[must_use]
    fn generate(
        &self,
        blueprint: &IdentityBlueprint,
        language: &LanguageSystem,
        seed: IdentitySeed,
    ) -> GeneratedName;
}

/// Deterministic phonotactic name generator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NameGenerator {
    candidate_count: usize,
}

impl Default for NameGenerator {
    fn default() -> Self {
        Self::new(DEFAULT_NAME_CANDIDATES)
    }
}

impl NameGenerator {
    /// Creates a generator with a bounded candidate count.
    #[must_use]
    pub const fn new(candidate_count: usize) -> Self {
        Self {
            candidate_count: if candidate_count == 0 { 1 } else { candidate_count },
        }
    }

    /// Returns the configured candidate count.
    #[must_use]
    pub const fn candidate_count(self) -> usize {
        self.candidate_count
    }
}

impl GenerateName for NameGenerator {
    fn generate(
        &self,
        blueprint: &IdentityBlueprint,
        language: &LanguageSystem,
        seed: IdentitySeed,
    ) -> GeneratedName {
        let generation_seed = seed
            .derive("origin.name_engine.v1")
            .derive(&blueprint.entity_id.to_string())
            .derive(&language.language_id.to_string());

        let mut seen = BTreeSet::new();
        let mut candidates = Vec::with_capacity(self.candidate_count);
        let search_limit = self.candidate_count.saturating_mul(16).max(32);

        for index in 0..search_limit {
            if candidates.len() >= self.candidate_count {
                break;
            }

            let candidate_seed = generation_seed.derive(&format!("candidate:{index}"));
            let syllable_count = select_syllable_count(blueprint, language, candidate_seed);
            let raw = build_word(language, candidate_seed, syllable_count);
            let normalized = normalize_name(&raw);

            if normalized.is_empty() || !seen.insert(normalized.clone()) {
                continue;
            }

            let pronunciation = normalized.clone();
            let text = capitalize(&normalized);
            let score = score_candidate(blueprint, language, &normalized, syllable_count);

            candidates.push(NameCandidate {
                text,
                pronunciation,
                syllables: syllable_count,
                score,
                derivation_index: u16::try_from(index).unwrap_or(u16::MAX),
            });
        }

        if candidates.is_empty() {
            candidates.push(fallback_candidate(language, generation_seed));
        }

        candidates.sort_unstable_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.derivation_index.cmp(&right.derivation_index))
                .then_with(|| left.text.cmp(&right.text))
        });

        let canonical = candidates[0].text.clone();
        let pronunciation = candidates[0].pronunciation.clone();

        GeneratedName {
            schema_version: NAME_ENGINE_SCHEMA_VERSION,
            canonical,
            pronunciation,
            candidates,
            origin: NameOrigin {
                entity_id: blueprint.entity_id,
                language_id: language.language_id,
                identity_seed: seed,
                generation_seed,
                blueprint_schema_version: blueprint.schema_version,
                language_schema_version: language.schema_version,
                generation_rule: "origin.name_engine.v1".to_owned(),
            },
        }
    }
}

fn select_syllable_count(
    blueprint: &IdentityBlueprint,
    language: &LanguageSystem,
    seed: IdentitySeed,
) -> u8 {
    let minimum = language
        .min_word_syllables
        .max(blueprint.limits.min_syllables)
        .max(1);
    let maximum = language
        .max_word_syllables
        .min(blueprint.limits.max_syllables)
        .max(minimum);
    let span = u64::from(maximum - minimum + 1);
    minimum + u8::try_from(seed.derive("syllable_count").value() % span).unwrap_or_default()
}

fn build_word(language: &LanguageSystem, seed: IdentitySeed, syllables: u8) -> String {
    let mut word = String::new();

    for syllable_index in 0..syllables {
        let syllable_seed = seed.derive(&format!("syllable:{syllable_index}"));
        let pattern = select_pattern(&language.syllable_patterns, syllable_seed);
        word.push_str(&realize_pattern(language, pattern, syllable_seed));
    }

    word
}

fn select_pattern(patterns: &[SyllablePattern], seed: IdentitySeed) -> &SyllablePattern {
    let total_weight = patterns
        .iter()
        .map(|pattern| u64::from(pattern.weight))
        .sum::<u64>();

    let mut cursor = if total_weight == 0 {
        0
    } else {
        seed.derive("pattern").value() % total_weight
    };

    for pattern in patterns {
        let weight = u64::from(pattern.weight);
        if cursor < weight {
            return pattern;
        }
        cursor = cursor.saturating_sub(weight);
    }

    patterns
        .first()
        .expect("language systems always contain at least one syllable pattern")
}

fn realize_pattern(
    language: &LanguageSystem,
    pattern: &SyllablePattern,
    seed: IdentitySeed,
) -> String {
    let mut output = String::new();
    let mut onset_count = 0_u8;
    let mut nucleus_count = 0_u8;
    let mut coda_count = 0_u8;

    for (position, class) in pattern.segments.iter().enumerate() {
        let segment_seed = seed.derive(&format!("segment:{position}"));
        let segment = match class {
            SegmentClass::Onset => {
                onset_count = onset_count.saturating_add(1);
                select_segment(&language.consonants, segment_seed, onset_count)
            }
            SegmentClass::Nucleus => {
                nucleus_count = nucleus_count.saturating_add(1);
                select_segment(&language.vowels, segment_seed, nucleus_count)
            }
            SegmentClass::Coda => {
                coda_count = coda_count.saturating_add(1);
                select_segment(&language.codas, segment_seed, coda_count)
            }
        };
        output.push_str(segment);
    }

    output
}

fn select_segment(inventory: &[String], seed: IdentitySeed, ordinal: u8) -> &str {
    let len = inventory.len();
    let index = usize::try_from(seed.derive(&format!("ordinal:{ordinal}")).value())
        .unwrap_or_default()
        % len;
    inventory[index].as_str()
}

fn normalize_name(raw: &str) -> String {
    let mut normalized = String::with_capacity(raw.len());
    let mut previous = '\0';

    for character in raw.chars().flat_map(char::to_lowercase) {
        if !character.is_ascii_alphabetic() {
            continue;
        }
        if character == previous && !matches!(character, 'l' | 'm' | 'n' | 'r' | 's') {
            continue;
        }
        normalized.push(character);
        previous = character;
    }

    normalized
}

fn capitalize(value: &str) -> String {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return String::new();
    };

    let mut capitalized = first.to_uppercase().collect::<String>();
    capitalized.push_str(characters.as_str());
    capitalized
}

fn score_candidate(
    blueprint: &IdentityBlueprint,
    language: &LanguageSystem,
    name: &str,
    syllables: u8,
) -> u16 {
    let target = u16::from(language.min_word_syllables + language.max_word_syllables) / 2;
    let syllable_distance = u16::from(syllables).abs_diff(target);
    let length = u16::try_from(name.len()).unwrap_or(u16::MAX);
    let compactness = blueprint
        .trait_value(BlueprintTrait::Compactness)
        .value();
    let fluidity = blueprint.trait_value(BlueprintTrait::Fluidity).value();
    let distinctiveness = blueprint
        .trait_value(BlueprintTrait::Distinctiveness)
        .value();

    let mut score = 1000_u16;
    score = score.saturating_sub(syllable_distance.saturating_mul(80));

    if length < 3 {
        score = score.saturating_sub(250);
    } else if length > 14 {
        score = score.saturating_sub((length - 14).saturating_mul(35));
    }

    let repeated = repeated_adjacent_count(name);
    score = score.saturating_sub(repeated.saturating_mul(45));

    let vowel_ratio = vowel_ratio_per_mille(name, language);
    let desired_ratio = 350_u16.saturating_add(fluidity / 4).min(700);
    score = score.saturating_sub(vowel_ratio.abs_diff(desired_ratio) / 3);

    if compactness >= 700 && length > 9 {
        score = score.saturating_sub((length - 9).saturating_mul(25));
    }
    if distinctiveness >= 700 && repeated == 0 {
        score = score.saturating_add(25).min(1000);
    }

    score
}

fn repeated_adjacent_count(value: &str) -> u16 {
    let bytes = value.as_bytes();
    u16::try_from(bytes.windows(2).filter(|pair| pair[0] == pair[1]).count())
        .unwrap_or(u16::MAX)
}

fn vowel_ratio_per_mille(value: &str, language: &LanguageSystem) -> u16 {
    if value.is_empty() {
        return 0;
    }

    let vowel_characters = value
        .chars()
        .filter(|character| "aeiouy".contains(*character))
        .count();
    let ratio = vowel_characters.saturating_mul(1000) / value.chars().count();
    let inventory_bonus = language
        .vowels
        .iter()
        .filter(|segment| value.contains(segment.as_str()))
        .count()
        .min(3)
        * 10;

    u16::try_from(ratio.saturating_add(inventory_bonus).min(1000)).unwrap_or(1000)
}

fn fallback_candidate(language: &LanguageSystem, seed: IdentitySeed) -> NameCandidate {
    let consonant = select_segment(&language.consonants, seed.derive("fallback.c"), 0);
    let vowel = select_segment(&language.vowels, seed.derive("fallback.v"), 0);
    let pronunciation = normalize_name(&format!("{consonant}{vowel}"));

    NameCandidate {
        text: capitalize(&pronunciation),
        pronunciation,
        syllables: 1,
        score: 1,
        derivation_index: u16::MAX,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{GenerateName, NAME_ENGINE_SCHEMA_VERSION, NameGenerator};
    use crate::{
        EntityId, EntityKind, IdentityBlueprint, IdentityContext, IdentityGenome, IdentitySeed,
        LanguageSystem,
    };

    fn fixture(seed_value: u64) -> (IdentityBlueprint, LanguageSystem, IdentitySeed) {
        let context = IdentityContext::new(EntityKind::Civilization)
            .with_attribute("culture", "scholarly")
            .with_attribute("region", "outer-rim");
        let seed = IdentitySeed::new(seed_value);
        let id = EntityId::derive(context.kind, seed, "qverse");
        let genome = IdentityGenome::derive(id, seed, &context);
        let blueprint = IdentityBlueprint::compile(&genome);
        let language = LanguageSystem::derive(&blueprint, seed);
        (blueprint, language, seed)
    }

    #[test]
    fn same_seed_produces_the_same_name_set() {
        let (blueprint, language, seed) = fixture(42);
        let generator = NameGenerator::default();
        assert_eq!(
            generator.generate(&blueprint, &language, seed),
            generator.generate(&blueprint, &language, seed)
        );
    }

    #[test]
    fn different_seeds_change_the_generated_name() {
        let (blueprint_a, language_a, seed_a) = fixture(42);
        let (blueprint_b, language_b, seed_b) = fixture(43);
        let generator = NameGenerator::default();
        assert_ne!(
            generator.generate(&blueprint_a, &language_a, seed_a).canonical,
            generator.generate(&blueprint_b, &language_b, seed_b).canonical
        );
    }

    #[test]
    fn candidates_are_unique_and_ranked() {
        let (blueprint, language, seed) = fixture(7);
        let generated = NameGenerator::new(24).generate(&blueprint, &language, seed);
        let unique = generated
            .candidates
            .iter()
            .map(|candidate| candidate.text.as_str())
            .collect::<BTreeSet<_>>();

        assert_eq!(unique.len(), generated.candidates.len());
        assert!(generated.candidates.windows(2).all(|pair| {
            pair[0].score > pair[1].score
                || (pair[0].score == pair[1].score
                    && pair[0].derivation_index <= pair[1].derivation_index)
        }));
    }

    #[test]
    fn canonical_name_matches_the_highest_ranked_candidate() {
        let (blueprint, language, seed) = fixture(99);
        let generated = NameGenerator::default().generate(&blueprint, &language, seed);
        assert_eq!(generated.canonical, generated.candidates[0].text);
        assert_eq!(generated.pronunciation, generated.candidates[0].pronunciation);
        assert_eq!(generated.schema_version, NAME_ENGINE_SCHEMA_VERSION);
    }

    #[test]
    fn generated_names_are_ascii_alphabetic_and_capitalized() {
        let generator = NameGenerator::new(32);
        for seed_value in 0..64 {
            let (blueprint, language, seed) = fixture(seed_value);
            let generated = generator.generate(&blueprint, &language, seed);
            assert!(generated.candidates.iter().all(|candidate| {
                candidate.text.is_ascii()
                    && candidate.text.chars().all(|character| character.is_ascii_alphabetic())
                    && candidate.text.chars().next().is_some_and(char::is_uppercase)
                    && candidate.score <= 1000
                    && candidate.syllables >= language.min_word_syllables
                    && candidate.syllables <= language.max_word_syllables
            }));
        }
    }

    #[test]
    fn origin_is_bound_to_blueprint_and_language() {
        let (blueprint, language, seed) = fixture(123);
        let generated = NameGenerator::default().generate(&blueprint, &language, seed);
        assert_eq!(generated.origin.entity_id, blueprint.entity_id);
        assert_eq!(generated.origin.language_id, language.language_id);
        assert_eq!(generated.origin.identity_seed, seed);
        assert_eq!(generated.origin.generation_rule, "origin.name_engine.v1");
    }
}
