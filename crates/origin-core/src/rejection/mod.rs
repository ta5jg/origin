//! Deterministic rules that can veto every downstream brand evaluation.

use serde::Serialize;

/// A reason that forces a candidate to receive a final score of zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RejectReason {
    /// The normalized candidate is empty, contains unsupported characters, or
    /// falls outside the supported length range.
    InvalidInput,
    /// The candidate contains an excessive run of the same character.
    ExcessiveRepetition,
    /// The candidate is present in the caller-provided used-name registry.
    AlreadyInUse,
}

/// Caller-provided data used by the hard reject engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RejectionPolicy<'a> {
    /// Names that must never be scored or suggested.
    pub blocked_names: &'a [&'a str],
}

/// Result of evaluating all hard reject rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct RejectionResult {
    /// Whether all later evaluation layers must be bypassed.
    pub hard_reject: bool,
    /// The first deterministic reason that caused rejection.
    pub reason: Option<RejectReason>,
}

impl RejectionResult {
    const fn accepted() -> Self {
        Self {
            hard_reject: false,
            reason: None,
        }
    }

    const fn rejected(reason: RejectReason) -> Self {
        Self {
            hard_reject: true,
            reason: Some(reason),
        }
    }
}

/// Evaluates rules whose result overrides deterministic and fuzzy scoring.
#[must_use]
pub fn evaluate_rejection(input: &str, policy: RejectionPolicy<'_>) -> RejectionResult {
    let normalized = input.trim().to_ascii_lowercase();
    let bytes = normalized.as_bytes();

    if bytes.is_empty()
        || !normalized.is_ascii()
        || !bytes.iter().all(u8::is_ascii_lowercase)
        || !(4..=12).contains(&bytes.len())
    {
        return RejectionResult::rejected(RejectReason::InvalidInput);
    }

    if bytes
        .windows(3)
        .any(|window| window[0] == window[1] && window[1] == window[2])
    {
        return RejectionResult::rejected(RejectReason::ExcessiveRepetition);
    }

    if policy
        .blocked_names
        .iter()
        .any(|blocked| blocked.trim().eq_ignore_ascii_case(&normalized))
    {
        return RejectionResult::rejected(RejectReason::AlreadyInUse);
    }

    RejectionResult::accepted()
}

#[cfg(test)]
mod tests {
    use super::{RejectReason, RejectionPolicy, evaluate_rejection};

    #[test]
    fn used_name_bypasses_all_other_evaluation() {
        let policy = RejectionPolicy {
            blocked_names: &["Foleri", "danoti"],
        };
        let result = evaluate_rejection("foleri", policy);

        assert!(result.hard_reject);
        assert_eq!(result.reason, Some(RejectReason::AlreadyInUse));
    }

    #[test]
    fn invalid_input_is_a_hard_reject() {
        let result = evaluate_rejection("nova-1", RejectionPolicy::default());

        assert!(result.hard_reject);
        assert_eq!(result.reason, Some(RejectReason::InvalidInput));
    }

    #[test]
    fn excessive_character_runs_are_a_hard_reject() {
        let result = evaluate_rejection("nooova", RejectionPolicy::default());

        assert!(result.hard_reject);
        assert_eq!(result.reason, Some(RejectReason::ExcessiveRepetition));
    }

    #[test]
    fn clean_unused_name_passes_the_veto_layer() {
        let result = evaluate_rejection("foleri", RejectionPolicy::default());

        assert!(!result.hard_reject);
        assert_eq!(result.reason, None);
    }
}
