//! Deterministic identity primitives shared by ORIGIN generators.

use std::{collections::BTreeMap, fmt, str::FromStr};

use serde::{Deserialize, Serialize};

/// Stable category of an entity whose identity is derived by ORIGIN.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum EntityKind {
    /// A complete universe.
    Universe,
    /// A galaxy.
    Galaxy,
    /// A star.
    Star,
    /// A planet.
    Planet,
    /// A natural satellite.
    Moon,
    /// A civilization.
    Civilization,
    /// A biological or synthetic species.
    Species,
    /// An individual person or agent.
    Individual,
    /// A settlement or city.
    Settlement,
    /// An organization.
    Organization,
    /// A company or commercial institution.
    Company,
    /// A product.
    Product,
    /// A technology.
    Technology,
    /// A language.
    Language,
    /// A vehicle.
    Vehicle,
    /// A currency.
    Currency,
}

impl EntityKind {
    /// Canonical stable tag used by hashing and persistence.
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Universe => "universe",
            Self::Galaxy => "galaxy",
            Self::Star => "star",
            Self::Planet => "planet",
            Self::Moon => "moon",
            Self::Civilization => "civilization",
            Self::Species => "species",
            Self::Individual => "individual",
            Self::Settlement => "settlement",
            Self::Organization => "organization",
            Self::Company => "company",
            Self::Product => "product",
            Self::Technology => "technology",
            Self::Language => "language",
            Self::Vehicle => "vehicle",
            Self::Currency => "currency",
        }
    }
}

impl fmt::Display for EntityKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.tag())
    }
}

/// Error returned when parsing an unsupported entity kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseEntityKindError {
    value: String,
}

impl fmt::Display for ParseEntityKindError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "unsupported entity kind: {}", self.value)
    }
}

impl std::error::Error for ParseEntityKindError {}

impl FromStr for EntityKind {
    type Err = ParseEntityKindError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "universe" => Ok(Self::Universe),
            "galaxy" => Ok(Self::Galaxy),
            "star" => Ok(Self::Star),
            "planet" => Ok(Self::Planet),
            "moon" => Ok(Self::Moon),
            "civilization" => Ok(Self::Civilization),
            "species" => Ok(Self::Species),
            "individual" => Ok(Self::Individual),
            "settlement" => Ok(Self::Settlement),
            "organization" => Ok(Self::Organization),
            "company" => Ok(Self::Company),
            "product" => Ok(Self::Product),
            "technology" => Ok(Self::Technology),
            "language" => Ok(Self::Language),
            "vehicle" => Ok(Self::Vehicle),
            "currency" => Ok(Self::Currency),
            _ => Err(ParseEntityKindError {
                value: value.to_owned(),
            }),
        }
    }
}

/// Stable 128-bit identifier derived from identity inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EntityId(u128);

impl EntityId {
    /// Derives an identifier from an entity kind, seed and namespace.
    #[must_use]
    pub fn derive(kind: EntityKind, seed: IdentitySeed, namespace: &str) -> Self {
        let high = hash_bytes(seed.value ^ 0x9E37_79B9_7F4A_7C15, kind.tag().as_bytes());
        let high = hash_bytes(high, namespace.as_bytes());
        let low = hash_bytes(seed.value ^ 0xD1B5_4A32_D192_ED03, namespace.as_bytes());
        let low = hash_bytes(low, kind.tag().as_bytes());
        Self((u128::from(high) << 64) | u128::from(low))
    }

    /// Returns the raw identifier value.
    #[must_use]
    pub const fn as_u128(self) -> u128 {
        self.0
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:032x}", self.0)
    }
}

/// Error returned when an entity identifier cannot be parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseEntityIdError;

impl fmt::Display for ParseEntityIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("entity id must contain exactly 32 hexadecimal characters")
    }
}

impl std::error::Error for ParseEntityIdError {}

impl FromStr for EntityId {
    type Err = ParseEntityIdError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() != 32 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(ParseEntityIdError);
        }
        u128::from_str_radix(value, 16)
            .map(Self)
            .map_err(|_| ParseEntityIdError)
    }
}

/// Reproducible seed used by identity derivation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct IdentitySeed {
    value: u64,
}

impl IdentitySeed {
    /// Creates a seed from a raw value.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self { value }
    }

    /// Returns the raw seed value.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.value
    }

    /// Derives a deterministic child seed without mutating the parent.
    #[must_use]
    pub fn derive(self, domain: &str) -> Self {
        Self::new(hash_bytes(self.value, domain.as_bytes()))
    }
}

/// Ordered, serializable context used while deriving an identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityContext {
    /// Entity category.
    pub kind: EntityKind,
    /// Optional parent identity.
    pub parent: Option<EntityId>,
    /// Optional language identity used for naming.
    pub language: Option<EntityId>,
    /// Timeline coordinate supplied by Brain or Omnis.
    pub epoch: i64,
    /// Stable context attributes. `BTreeMap` preserves canonical ordering.
    pub attributes: BTreeMap<String, String>,
}

impl IdentityContext {
    /// Creates an empty context for the requested entity kind.
    #[must_use]
    pub const fn new(kind: EntityKind) -> Self {
        Self {
            kind,
            parent: None,
            language: None,
            epoch: 0,
            attributes: BTreeMap::new(),
        }
    }

    /// Adds or replaces a stable context attribute.
    #[must_use]
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
}

/// Minimum complete identity record produced by the kernel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity {
    /// Stable identifier.
    pub id: EntityId,
    /// Entity category.
    pub kind: EntityKind,
    /// Seed from which this identity was derived.
    pub seed: IdentitySeed,
    /// Namespace separating independent identity domains.
    pub namespace: String,
    /// Schema version for deterministic migrations.
    pub schema_version: u16,
}

impl Identity {
    /// Constructs the minimum identity record from deterministic inputs.
    #[must_use]
    pub fn derive(seed: IdentitySeed, context: &IdentityContext, namespace: impl Into<String>) -> Self {
        let namespace = namespace.into();
        Self {
            id: EntityId::derive(context.kind, seed, &namespace),
            kind: context.kind,
            seed,
            namespace,
            schema_version: 1,
        }
    }
}

const fn mix(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

fn hash_bytes(seed: u64, bytes: &[u8]) -> u64 {
    let mut state = mix(seed ^ 0xCBF2_9CE4_8422_2325);
    for &byte in bytes {
        state ^= u64::from(byte);
        state = state.wrapping_mul(0x0000_0100_0000_01B3);
        state = mix(state);
    }
    state
}

#[cfg(test)]
mod tests {
    use super::{EntityId, EntityKind, Identity, IdentityContext, IdentitySeed};
    use std::str::FromStr;

    #[test]
    fn entity_kind_roundtrips_through_its_canonical_tag() {
        for kind in [
            EntityKind::Universe,
            EntityKind::Galaxy,
            EntityKind::Star,
            EntityKind::Planet,
            EntityKind::Moon,
            EntityKind::Civilization,
            EntityKind::Species,
            EntityKind::Individual,
            EntityKind::Settlement,
            EntityKind::Organization,
            EntityKind::Company,
            EntityKind::Product,
            EntityKind::Technology,
            EntityKind::Language,
            EntityKind::Vehicle,
            EntityKind::Currency,
        ] {
            assert_eq!(EntityKind::from_str(kind.tag()), Ok(kind));
        }
    }

    #[test]
    fn entity_id_roundtrips_through_hexadecimal_text() {
        let id = EntityId::derive(EntityKind::Civilization, IdentitySeed::new(42), "qverse");
        assert_eq!(EntityId::from_str(&id.to_string()), Ok(id));
    }

    #[test]
    fn seed_derivation_is_deterministic_and_domain_separated() {
        let seed = IdentitySeed::new(7);
        assert_eq!(seed.derive("language"), seed.derive("language"));
        assert_ne!(seed.derive("language"), seed.derive("civilization"));
    }

    #[test]
    fn context_attributes_have_canonical_order() {
        let context = IdentityContext::new(EntityKind::Planet)
            .with_attribute("region", "outer-rim")
            .with_attribute("climate", "temperate");
        let keys = context.attributes.keys().map(String::as_str).collect::<Vec<_>>();
        assert_eq!(keys, ["climate", "region"]);
    }

    #[test]
    fn identity_derivation_is_stable_and_namespace_separated() {
        let context = IdentityContext::new(EntityKind::Civilization);
        let seed = IdentitySeed::new(99);
        let first = Identity::derive(seed, &context, "qverse");
        let second = Identity::derive(seed, &context, "qverse");
        let external = Identity::derive(seed, &context, "brand");

        assert_eq!(first, second);
        assert_ne!(first.id, external.id);
        assert_eq!(first.schema_version, 1);
    }
}