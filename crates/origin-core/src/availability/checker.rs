use std::cell::RefCell;

use super::{
    AvailabilityCache, AvailabilityError, AvailabilityProvider, AvailabilityReport,
    AvailabilityResult, AvailabilityTarget,
};

/// Coordinates provider requests and prevents duplicate checks during one run.
#[derive(Debug)]
pub struct AvailabilityChecker<P> {
    provider: P,
    cache: RefCell<AvailabilityCache>,
}

impl<P> AvailabilityChecker<P>
where
    P: AvailabilityProvider,
{
    /// Creates a checker with an empty in-memory cache.
    #[must_use]
    pub fn new(provider: P) -> Self {
        Self {
            provider,
            cache: RefCell::new(AvailabilityCache::default()),
        }
    }

    /// Checks one candidate against one target.
    ///
    /// # Errors
    ///
    /// Returns [`AvailabilityError::EmptyName`] for blank names or propagates a
    /// provider failure.
    pub fn check(
        &self,
        name: &str,
        target: &AvailabilityTarget,
    ) -> Result<AvailabilityResult, AvailabilityError> {
        let name = normalize_name(name)?;
        if let Some(result) = self.cache.borrow().get(target, &name) {
            return Ok(result);
        }

        let result = self.provider.check(target, &name)?;
        self.cache.borrow_mut().insert(result.clone());
        Ok(result)
    }

    /// Checks one candidate across all requested targets.
    ///
    /// # Errors
    ///
    /// Returns [`AvailabilityError::EmptyName`] for blank names or propagates a
    /// provider failure for the first target that cannot be checked.
    pub fn check_all(
        &self,
        name: &str,
        targets: impl IntoIterator<Item = AvailabilityTarget>,
    ) -> Result<AvailabilityReport, AvailabilityError> {
        let name = normalize_name(name)?;
        let results = targets
            .into_iter()
            .map(|target| self.check(&name, &target))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(AvailabilityReport::new(name, results))
    }

    /// Returns the number of cached provider responses.
    #[must_use]
    pub fn cache_len(&self) -> usize {
        self.cache.borrow().len()
    }
}

fn normalize_name(name: &str) -> Result<String, AvailabilityError> {
    let normalized = name.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        Err(AvailabilityError::EmptyName)
    } else {
        Ok(normalized)
    }
}
