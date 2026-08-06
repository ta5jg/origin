//! Deterministic semantic profiles for ORIGIN identities.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    BlueprintTrait,
    ExpressionFamily,
    IdentityBlueprint,
};

pub const SEMANTIC_SCHEMA_VERSION: u16 = 1;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum SemanticConcept {
    Power,
    Light,
    Darkness,
    Water,
    Fire,
    Earth,
    Sky,
    Wind,
    Ice,
    Nature,
    Life,
    Death,
    Spirit,
    Machine,
    Knowledge,
    Wisdom,
    Order,
    Chaos,
    Harmony,
    Conflict,
    Time,
    Memory,
    Destiny,
    Hope,
    Void,
    Crystal,
    Ocean,
    Forest,
    Mountain,
    Civilization,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticWeight(u16);

impl SemanticWeight {
    pub const MAX: u16 = 1000;

    pub fn new(value: u16) -> Self {
        Self(value.min(Self::MAX))
    }

    pub fn value(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticProfile {
    pub schema_version: u16,
    pub concepts: BTreeMap<SemanticConcept, SemanticWeight>,
    pub compiler_rule: String,
}

impl SemanticProfile {
    #[must_use]
    pub fn compile(blueprint: &IdentityBlueprint) -> Self {
        let mut concepts = BTreeMap::new();

        for concept in SemanticConcept::all() {
            let score = concept.score(blueprint);
            concepts.insert(concept, SemanticWeight::new(score));
        }

        Self {
            schema_version: SEMANTIC_SCHEMA_VERSION,
            concepts,
            compiler_rule: "origin.semantic_profile.v1".to_owned(),
        }
    }

    #[must_use]
    pub fn weight(&self, concept: SemanticConcept) -> SemanticWeight {
        self.concepts
            .get(&concept)
            .copied()
            .unwrap_or(SemanticWeight::new(0))
    }
}

impl SemanticConcept {
    pub const fn all() -> [SemanticConcept; 30] {
        [
            Self::Power,
            Self::Light,
            Self::Darkness,
            Self::Water,
            Self::Fire,
            Self::Earth,
            Self::Sky,
            Self::Wind,
            Self::Ice,
            Self::Nature,
            Self::Life,
            Self::Death,
            Self::Spirit,
            Self::Machine,
            Self::Knowledge,
            Self::Wisdom,
            Self::Order,
            Self::Chaos,
            Self::Harmony,
            Self::Conflict,
            Self::Time,
            Self::Memory,
            Self::Destiny,
            Self::Hope,
            Self::Void,
            Self::Crystal,
            Self::Ocean,
            Self::Forest,
            Self::Mountain,
            Self::Civilization,
        ]
    }

    fn score(self, blueprint: &IdentityBlueprint) -> u16 {
        let force = blueprint.trait_value(BlueprintTrait::Force).value();
        let fluidity = blueprint.trait_value(BlueprintTrait::Fluidity).value();
        let refinement = blueprint.trait_value(BlueprintTrait::Refinement).value();
        let heritage = blueprint.trait_value(BlueprintTrait::Heritage).value();
        let distinct = blueprint.trait_value(BlueprintTrait::Distinctiveness).value();

        match self {
            SemanticConcept::Power => force,
            SemanticConcept::Light => refinement,
            SemanticConcept::Darkness => 1000 - refinement,
            SemanticConcept::Water => fluidity,
            SemanticConcept::Fire => force,
            SemanticConcept::Earth => heritage,
            SemanticConcept::Sky => fluidity,
            SemanticConcept::Wind => fluidity,
            SemanticConcept::Ice => refinement,
            SemanticConcept::Nature => heritage,
            SemanticConcept::Life => refinement,
            SemanticConcept::Death => 1000 - refinement,
            SemanticConcept::Spirit => refinement,
            SemanticConcept::Machine => distinct,
            SemanticConcept::Knowledge => refinement,
            SemanticConcept::Wisdom => heritage,
            SemanticConcept::Order => match blueprint.family {
                ExpressionFamily::Utilitarian => 900,
                ExpressionFamily::Harmonic => 800,
                _ => 500,
            },
            SemanticConcept::Chaos => match blueprint.family {
                ExpressionFamily::Experimental => 900,
                _ => 250,
            },
            SemanticConcept::Harmony => fluidity,
            SemanticConcept::Conflict => force,
            SemanticConcept::Time => heritage,
            SemanticConcept::Memory => heritage,
            SemanticConcept::Destiny => refinement,
            SemanticConcept::Hope => refinement,
            SemanticConcept::Void => distinct,
            SemanticConcept::Crystal => refinement,
            SemanticConcept::Ocean => fluidity,
            SemanticConcept::Forest => heritage,
            SemanticConcept::Mountain => force,
            SemanticConcept::Civilization => (heritage + refinement) / 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_weight_is_clamped() {
        assert_eq!(SemanticWeight::new(5000).value(), 1000);
    }
}