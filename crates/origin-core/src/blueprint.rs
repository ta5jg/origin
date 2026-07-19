//! Deterministic identity blueprints compiled from identity genomes.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{EntityId, EntityKind, GeneValue, GenomeAxis, IdentityGenome};

/// Current stable blueprint schema version.
pub const IDENTITY_BLUEPRINT_SCHEMA_VERSION: u16 = 1;

/// Stable dimensions consumed by downstream identity generators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum BlueprintTrait {
    /// Desired compactness of generated forms.
    Compactness,
    /// Desired phonetic force.
    Force,
    /// Desired continuity and smoothness.
    Fluidity,
    /// Desired structural refinement.
    Refinement,
    /// Desired historical depth.
    Heritage,
    /// Desired distinctiveness.
    Distinctiveness,
    /// Desired construction regularity.
    Systematicity,
    /// Desired vowel openness.
    Sonority,
}

impl BlueprintTrait {
    /// Canonical stable tag.
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Compactness => "compactness",
            Self::Force => "force",
            Self::Fluidity => "fluidity",
            Self::Refinement => "refinement",
            Self::Heritage => "heritage",
            Self::Distinctiveness => "distinctiveness",
            Self::Systematicity => "systematicity",
            Self::Sonority => "sonority",
        }
    }

    const ALL: [Self; 8] = [
        Self::Compactness,
        Self::Force,
        Self::Fluidity,
        Self::Refinement,
        Self::Heritage,
        Self::Distinctiveness,
        Self::Systematicity,
        Self::Sonority,
    ];
}

/// Broad deterministic expression family selected for an identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ExpressionFamily {
    /// Short, regular and functional forms.
    Utilitarian,
    /// Smooth, balanced and refined forms.
    Harmonic,
    /// Dense, forceful and compact forms.
    Monumental,
    /// Elaborate, historical and ceremonial forms.
    Archaic,
    /// Unusual, distinctive and experimental forms.
    Experimental,
}

/// Deterministic generation limits derived from the genome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlueprintLimits {
    /// Minimum preferred syllable count.
    pub min_syllables: u8,
    /// Maximum preferred syllable count.
    pub max_syllables: u8,
    /// Maximum preferred consonant cluster width.
    pub max_cluster_width: u8,
    /// Maximum preferred morpheme count.
    pub max_morphemes: u8,
}

/// Complete deterministic bridge between genome and downstream generators.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityBlueprint {
    /// Blueprint schema version.
    pub schema_version: u16,
    /// Identity whose blueprint was compiled.
    pub entity_id: EntityId,
    /// Entity category.
    pub kind: EntityKind,
    /// Selected broad expression family.
    pub family: ExpressionFamily,
    /// Canonically ordered normalized blueprint traits.
    pub traits: BTreeMap<BlueprintTrait, GeneValue>,
    /// Deterministic generation limits.
    pub limits: BlueprintLimits,
    /// Stable compiler rule identifier.
    pub compiler_rule: String,
}

impl IdentityBlueprint {
    /// Compiles an identity blueprint from a deterministic genome.
    #[must_use]
    pub fn compile(genome: &IdentityGenome) -> Self {
        let traits = BlueprintTrait::ALL
            .into_iter()
            .map(|blueprint_trait| {
                let value = trait_value(genome, blueprint_trait);
                (blueprint_trait, GeneValue::new(value))
            })
            .collect::<BTreeMap<_, _>>();

        let family = select_family(&traits);
        let limits = compile_limits(&traits, family);

        Self {
            schema_version: IDENTITY_BLUEPRINT_SCHEMA_VERSION,
            entity_id: genome.provenance.entity_id,
            kind: genome.kind,
            family,
            traits,
            limits,
            compiler_rule: "origin.identity_blueprint.v1".to_owned(),
        }
    }

    /// Returns one compiled blueprint trait.
    #[must_use]
    pub fn trait_value(&self, blueprint_trait: BlueprintTrait) -> GeneValue {
        self.traits
            .get(&blueprint_trait)
            .copied()
            .unwrap_or_else(|| GeneValue::new(0))
    }
}

fn trait_value(genome: &IdentityGenome, blueprint_trait: BlueprintTrait) -> u16 {
    let axis = |axis| u32::from(genome.axis(axis).value());
    let value = match blueprint_trait {
        BlueprintTrait::Compactness => {
            weighted_inverse(axis(GenomeAxis::Complexity), axis(GenomeAxis::Antiquity), 3, 1)
        }
        BlueprintTrait::Force => {
            weighted_average(axis(GenomeAxis::Harshness), axis(GenomeAxis::Complexity), 3, 1)
        }
        BlueprintTrait::Fluidity => {
            weighted_average(axis(GenomeAxis::Flow), axis(GenomeAxis::OpenVowels), 3, 2)
        }
        BlueprintTrait::Refinement => {
            weighted_average(axis(GenomeAxis::Elegance), axis(GenomeAxis::Regularity), 3, 2)
        }
        BlueprintTrait::Heritage => {
            weighted_average(axis(GenomeAxis::Antiquity), axis(GenomeAxis::Regularity), 4, 1)
        }
        BlueprintTrait::Distinctiveness => {
            weighted_average(axis(GenomeAxis::Novelty), axis(GenomeAxis::Complexity), 4, 1)
        }
        BlueprintTrait::Systematicity => {
            weighted_average(axis(GenomeAxis::Regularity), axis(GenomeAxis::Elegance), 4, 1)
        }
        BlueprintTrait::Sonority => {
            weighted_average(axis(GenomeAxis::OpenVowels), axis(GenomeAxis::Flow), 4, 1)
        }
    };

    u16::try_from(value.min(u32::from(GeneValue::MAX))).unwrap_or(GeneValue::MAX)
}

const fn weighted_average(left: u32, right: u32, left_weight: u32, right_weight: u32) -> u32 {
    (left * left_weight + right * right_weight) / (left_weight + right_weight)
}

const fn weighted_inverse(left: u32, right: u32, left_weight: u32, right_weight: u32) -> u32 {
    let pressure = weighted_average(left, right, left_weight, right_weight);
    1000_u32.saturating_sub(pressure)
}

fn select_family(traits: &BTreeMap<BlueprintTrait, GeneValue>) -> ExpressionFamily {
    let score = |key| u32::from(traits.get(&key).copied().unwrap_or_else(|| GeneValue::new(0)).value());

    let candidates = [
        (
            ExpressionFamily::Utilitarian,
            score(BlueprintTrait::Compactness) + score(BlueprintTrait::Systematicity),
        ),
        (
            ExpressionFamily::Harmonic,
            score(BlueprintTrait::Fluidity) + score(BlueprintTrait::Refinement) + score(BlueprintTrait::Sonority),
        ),
        (
            ExpressionFamily::Monumental,
            score(BlueprintTrait::Force) + score(BlueprintTrait::Compactness),
        ),
        (
            ExpressionFamily::Archaic,
            score(BlueprintTrait::Heritage) + score(BlueprintTrait::Refinement),
        ),
        (
            ExpressionFamily::Experimental,
            score(BlueprintTrait::Distinctiveness) + score(BlueprintTrait::Force),
        ),
    ];

    candidates
        .into_iter()
        .max_by_key(|(_, candidate_score)| *candidate_score)
        .map(|(family, _)| family)
        .unwrap_or(ExpressionFamily::Utilitarian)
}

fn compile_limits(
    traits: &BTreeMap<BlueprintTrait, GeneValue>,
    family: ExpressionFamily,
) -> BlueprintLimits {
    let value = |key| traits.get(&key).copied().unwrap_or_else(|| GeneValue::new(0)).value();
    let compactness = value(BlueprintTrait::Compactness);
    let force = value(BlueprintTrait::Force);
    let heritage = value(BlueprintTrait::Heritage);
    let distinctiveness = value(BlueprintTrait::Distinctiveness);

    let min_syllables = if heritage >= 700 { 2 } else { 1 };
    let mut max_syllables = match compactness {
        750..=1000 => 2,
        400..=749 => 3,
        _ => 4,
    };
    if matches!(family, ExpressionFamily::Archaic) {
        max_syllables = max_syllables.max(4);
    }

    let max_cluster_width = match force {
        750..=1000 => 3,
        350..=749 => 2,
        _ => 1,
    };
    let max_morphemes = match heritage.saturating_add(distinctiveness) / 2 {
        750..=1000 => 4,
        400..=749 => 3,
        _ => 2,
    };

    BlueprintLimits {
        min_syllables,
        max_syllables,
        max_cluster_width,
        max_morphemes,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BlueprintTrait, ExpressionFamily, IDENTITY_BLUEPRINT_SCHEMA_VERSION, IdentityBlueprint,
    };
    use crate::{EntityId, EntityKind, IdentityContext, IdentityGenome, IdentitySeed};

    fn blueprint(seed: u64, kind: EntityKind) -> IdentityBlueprint {
        let context = IdentityContext::new(kind).with_attribute("culture", "scholarly");
        let seed = IdentitySeed::new(seed);
        let id = EntityId::derive(kind, seed, "qverse");
        IdentityBlueprint::compile(&IdentityGenome::derive(id, seed, &context))
    }

    #[test]
    fn blueprint_compilation_is_deterministic() {
        assert_eq!(
            blueprint(42, EntityKind::Civilization),
            blueprint(42, EntityKind::Civilization)
        );
    }

    #[test]
    fn every_blueprint_trait_is_present_and_normalized() {
        let blueprint = blueprint(7, EntityKind::Language);
        assert_eq!(blueprint.traits.len(), 8);
        assert!(blueprint.traits.values().all(|value| value.value() <= 1000));
        assert!(blueprint.trait_value(BlueprintTrait::Fluidity).value() <= 1000);
    }

    #[test]
    fn blueprint_is_bound_to_genome_entity() {
        let context = IdentityContext::new(EntityKind::Planet);
        let seed = IdentitySeed::new(99);
        let id = EntityId::derive(context.kind, seed, "qverse");
        let genome = IdentityGenome::derive(id, seed, &context);
        let blueprint = IdentityBlueprint::compile(&genome);

        assert_eq!(blueprint.entity_id, id);
        assert_eq!(blueprint.kind, EntityKind::Planet);
        assert_eq!(blueprint.schema_version, IDENTITY_BLUEPRINT_SCHEMA_VERSION);
    }

    #[test]
    fn limits_are_always_structurally_valid() {
        for seed in 0..256 {
            let blueprint = blueprint(seed, EntityKind::Civilization);
            assert!(blueprint.limits.min_syllables >= 1);
            assert!(blueprint.limits.max_syllables >= blueprint.limits.min_syllables);
            assert!((1..=3).contains(&blueprint.limits.max_cluster_width));
            assert!((2..=4).contains(&blueprint.limits.max_morphemes));
        }
    }

    #[test]
    fn compiler_selects_a_known_expression_family() {
        let blueprint = blueprint(123, EntityKind::Organization);
        assert!(matches!(
            blueprint.family,
            ExpressionFamily::Utilitarian
                | ExpressionFamily::Harmonic
                | ExpressionFamily::Monumental
                | ExpressionFamily::Archaic
                | ExpressionFamily::Experimental
        ));
    }
}
