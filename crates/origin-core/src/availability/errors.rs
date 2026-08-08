use std::fmt;

/// Errors raised before or while a provider is queried.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AvailabilityError {
    /// Candidate names must contain at least one non-whitespace character.
    EmptyName,
    /// A provider could not complete a request.
    Provider {
        /// Stable identifier of the target whose provider failed.
        target: String,
        /// Provider-supplied failure context.
        message: String,
    },
}

impl fmt::Display for AvailabilityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyName => formatter.write_str("availability name cannot be empty"),
            Self::Provider { target, message } => {
                write!(
                    formatter,
                    "availability provider for {target} failed: {message}"
                )
            }
        }
    }
}

impl std::error::Error for AvailabilityError {}
