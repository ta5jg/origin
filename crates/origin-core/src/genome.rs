//! Deterministic identity genome and derivation provenance.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{EntityId, EntityKind, IdentityContext, IdentitySeed};

/// Current stable genome schema version.
pub const IDENTITY_GENOME_SCHEMA_VERSION: u16 = 1;

/// A normalized deterministic gene value in the inclusive range `0..=1000`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GeneValue(u16);

impl GeneValue {
    /// Lowest valid gene value.
    pub const MIN: u16 = 0;
    /// Highest valid gene value.
    pub const MAX: u16 = 1000;

    /// Creates a normalized value, clamping values above [`Self::MAX`].
    #[must_use]
    pub const fn new(value: u16) -> Self {
        Self(value.min(Self::MAX))
    }

    /// Returns the normalized raw value.
    #[must_use]
    pub const fn value(self) -> u16 {
        self.0
    }
}

/// Stable dimensions that influence an identity's expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum GenomeAxis {
    /// Preference for open vowel sounds.
    OpenVowels,
    /// Preference for consonant density and hard clusters.
    Harshness,
    /// Preference for smooth transitions between sounds.
    Flow,
    /// Preference for compact or elaborate forms.
    Complexity,
    /// Preference for balanced, refined forms.
    Elegance,
    /// Preference for ancient or archaic expression.
    Antiquity,
    /// Preference for novelty and unusual structures.
    Novelty,
    /// Preference for regular and systematic construction.
    Regularity,
}

impl GenomeAxis {
    /// Canonical tag used for deterministic domain separation.
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::OpenVowels => "open_vowels",
            Self::Harshness => "harshness",
            Self::Flow => "flow",
            Self::Complexity => "complexity",
            Self::Elegance => "elegance",
            Self::Antiquity => "antiquity",
            Self::Novelty => "novelty",
            Self::Regularity => "regularity",
        }
    }

    const ALL: [Self; 8] = [
        Self::OpenVowels,
        Self::Harshness,
        Self::Flow,
        Self::Complexity,
        Self::Elegance,
        Self::Antiquity,
        Self::Novelty,
        Self::Regularity,
    ];
}

/// Origin of a provenance step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceSource {
    /// Root seed supplied to ORIGIN.
    Seed,
    /// Entity context supplied by Brain, Omnis or an external caller.
    Context,
    /// Stable ORIGIN derivation rule.
    Rule,
}

/// One explainable step in deterministic genome derivation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceStep {
    /// Ordered step index.
    pub index: u16,
    /// Source category.
    pub source: ProvenanceSource,
    /// Stable machine-readable rule identifier.
    pub rule: String,
    /// Human-readable deterministic result.
    pub result: String,
}

/// Complete ordered derivation record for an identity genome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityProvenance {
    /// Identity whose genome was derived.
    pub entity_id: EntityId,
    /// Ordered derivation steps.
    pub steps: Vec<ProvenanceStep>,
}

/// Stable parameters from which identity expression is derived.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityGenome {
    /// Genome schema version.
    pub schema_version: u16,
    /// Entity category used during derivation.
    pub kind: EntityKind,
    /// Root genome seed, domain-separated from the entity seed.
    pub seed: IdentitySeed,
    /// Canonically ordered normalized axes.
    pub axes: BTreeMap<GenomeAxis, GeneValue>,
    /// Explainable derivation record.
    pub provenance: IdentityProvenance,
}

impl IdentityGenome {
    /// Derives a complete genome from stable identity inputs.
    #[must_use]
    pub fn derive(
        entity_id: EntityId,
        identity_seed: IdentitySeed,
        context: &IdentityContext,
    ) -> Self {
        let genome_seed = identity_seed
            .derive("origin.identity_genome.v1")
            .derive(context.kind.tag());
        let context_seed = derive_context_seed(genome_seed, context);

        let axes = GenomeAxis::ALL
            .into_iter()
            .map(|axis| {
                let axis_seed = context_seed.derive(axis.tag());
                let value = u16::try_from(axis_seed.value() % 1001).unwrap_or_default();
                (axis, GeneValue::new(value))
            })
            .collect();

        let provenance = IdentityProvenance {
            entity_id,
            steps: build_provenance(identity_seed, genome_seed, context_seed, context),
        };

        Self {
            schema_version: IDENTITY_GENOME_SCHEMA_VERSION,
            kind: context.kind,
            seed: genome_seed,
            axes,
            provenance,
        }
    }

    /// Returns one axis value.
    #[must_use]
    pub fn axis(&self, axis: GenomeAxis) -> GeneValue {
        self.axes.get(&axis).copied().unwrap_or_else(|| GeneValue::new(0))
    }
}

fn derive_context_seed(mut seed: IdentitySeed, context: &IdentityContext) -> IdentitySeed {
    seed = seed.derive(&format!("epoch:{}", context.epoch));

    if let Some(parent) = context.parent {
        seed = seed.derive(&format!("parent:{parent}"));
    }
    if let Some(language) = context.language {
        seed = seed.derive(&format!("language:{language}"));
    }
    for (key, value) in &context.attributes {
        seed = seed.derive(&format!("attribute:{key}={value}"));
    }

    seed
}

fn build_provenance(
    identity_seed: IdentitySeed,
    genome_seed: IdentitySeed,
    context_seed: IdentitySeed,
    context: &IdentityContext,
) -> Vec<ProvenanceStep> {
    let mut steps = vec![
        ProvenanceStep {
            index: 0,
            source: ProvenanceSource::Seed,
            rule: "identity_seed".to_owned(),
            result: identity_seed.value().to_string(),
        },
        ProvenanceStep {
            index: 1,
            source: ProvenanceSource::Rule,
            rule: "origin.identity_genome.v1".to_owned(),
            result: genome_seed.value().to_string(),
        },
        ProvenanceStep {
            index: 2,
            source: ProvenanceSource::Context,
            rule: "entity_kind".to_owned(),
            result: context.kind.tag().to_owned(),
        },
        ProvenanceStep {
            index: 3,
            source: ProvenanceSource::Context,
            rule: "epoch".to_owned(),
            result: context.epoch.to_string(),
        },
    ];

    for (key, value) in &context.attributes {
        let index = u16::try_from(steps.len()).unwrap_or(u16::MAX);
        steps.push(ProvenanceStep {
            index,
            source: ProvenanceSource::Context,
            rule: format!("attribute.{key}"),
            result: value.clone(),
        });
    }

    let index = u16::try_from(steps.len()).unwrap_or(u16::MAX);
    steps.push(ProvenanceStep {
        index,
        source: ProvenanceSource::Rule,
        rule: "context_fold".to_owned(),
        result: context_seed.value().to_string(),
    });

    steps
}

#[cfg(test)]
mod tests {
    use super::{GeneValue, GenomeAxis, IdentityGenome, IDENTITY_GENOME_SCHEMA_VERSION};
    use crate::{EntityId, EntityKind, IdentityContext, IdentitySeed};

    fn derive(seed: u64, context: &IdentityContext) -> IdentityGenome {
        let seed = IdentitySeed::new(seed);
        let id = EntityId::derive(context.kind, seed, "qverse");
        IdentityGenome::derive(id, seed, context)
    }

    #[test]
    fn gene_value_is_clamped_to_the_normalized_range() {
        assert_eq!(GeneValue::new(999).value(), 999);
        assert_eq!(GeneValue::new(1001).value(), GeneValue::MAX);
    }

    #[test]
    fn genome_derivation_is_deterministic() {
        let context = IdentityContext::new(EntityKind::Civilization)
            .with_attribute("culture", "scholarly")
            .with_attribute("region", "outer-rim");
        assert_eq!(derive(42, &context), derive(42, &context));
    }

    #[test]
    fn context_changes_genome_expression() {
        let peaceful = IdentityContext::new(EntityKind::Civilization)
            .with_attribute("culture", "peaceful");
        let martial = IdentityContext::new(EntityKind::Civilization)
            .with_attribute("culture", "martial");
        assert_ne!(derive(42, &peaceful).axes, derive(42, &martial).axes);
    }

    #[test]
    fn every_axis_is_present_and_normalized() {
        let genome = derive(7, &IdentityContext::new(EntityKind::Language));
        assert_eq!(genome.axes.len(), 8);
        assert!(genome.axes.values().all(|value| value.value() <= GeneValue::MAX));
        assert!(genome.axis(GenomeAxis::Flow).value() <= GeneValue::MAX);
        assert_eq!(genome.schema_version, IDENTITY_GENOME_SCHEMA_VERSION);
    }

    #[test]
    fn provenance_is_ordered_and_bound_to_the_entity() {
        let context = IdentityContext::new(EntityKind::Planet)
            .with_attribute("climate", "temperate");
        let genome = derive(99, &context);
        assert_eq!(genome.provenance.entity_id, EntityId::derive(context.kind, IdentitySeed::new(99), "qverse"));
        assert!(genome.provenance.steps.windows(2).all(|pair| pair[0].index < pair[1].index));
        assert!(genome.provenance.steps.iter().any(|step| step.rule == "attribute.climate"));
    }
}
