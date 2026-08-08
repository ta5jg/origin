use std::collections::BTreeMap;

use super::{AvailabilityResult, AvailabilityTarget};

/// Stable lookup key used by the in-memory availability cache.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CacheKey {
    target: AvailabilityTarget,
    name: String,
}

impl CacheKey {
    /// Creates a normalized cache key.
    #[must_use]
    pub fn new(target: AvailabilityTarget, name: &str) -> Self {
        Self {
            target,
            name: name.trim().to_ascii_lowercase(),
        }
    }
}

/// Small deterministic in-memory cache for repeated checks in one run.
#[derive(Debug, Default)]
pub struct AvailabilityCache {
    entries: BTreeMap<CacheKey, AvailabilityResult>,
}

impl AvailabilityCache {
    /// Returns a cached result, if present.
    #[must_use]
    pub fn get(&self, target: &AvailabilityTarget, name: &str) -> Option<AvailabilityResult> {
        self.entries
            .get(&CacheKey::new(target.clone(), name))
            .cloned()
    }

    /// Stores a provider result for future lookups.
    pub fn insert(&mut self, result: AvailabilityResult) {
        self.entries
            .insert(CacheKey::new(result.target.clone(), &result.name), result);
    }

    /// Number of results retained by this cache.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the cache has no retained results.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
