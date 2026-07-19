//! Deterministic fuzzy inference for human-perception brand qualities.

use serde::Serialize;

/// Linguistic quality labels used by the Origin Fuzzy Engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LinguisticQuality {
    VeryLow,
    Low,
    Medium,
    High,
    Excellent,
}

/// One linguistic membership degree on a zero-to-one-hundred scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Membership {
    pub quality: LinguisticQuality,
    pub degree: u8,
}

/// Deterministic component scores consumed by the fuzzy engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FuzzyInputs {
    pub pronounceability: u8,
    pub rhythm: u8,
    pub vowel_balance: u8,
    pub repetition: u8,
    pub transition_quality: u8,
}

/// Explainable fuzzy assessment of perceived brand quality.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FuzzyReport {
    /// Defuzzified quality from zero to one hundred.
    pub score: u8,
    /// Confidence in the fuzzy conclusion from zero to one hundred.
    pub confidence: u8,
    /// Strongest linguistic conclusion.
    pub quality: LinguisticQuality,
    /// Output memberships retained for explanation and diagnostics.
    pub memberships: Vec<Membership>,
    /// Human-readable rules that contributed to the result.
    pub activated_rules: Vec<String>,
}

/// Evaluates deterministic scores through ORIGIN's integer fuzzy inference.
///
/// The implementation intentionally avoids floating-point arithmetic so the
/// same inputs produce byte-identical results across supported platforms.
#[must_use]
pub fn evaluate_fuzzy(inputs: FuzzyInputs) -> FuzzyReport {
    let naturalness = weighted_average(&[
        (inputs.pronounceability, 35),
        (inputs.rhythm, 25),
        (inputs.vowel_balance, 15),
        (inputs.transition_quality, 25),
    ]);
    let memorability = weighted_average(&[
        (inputs.repetition, 45),
        (inputs.rhythm, 25),
        (inputs.transition_quality, 30),
    ]);
    let distinctiveness = weighted_average(&[
        (inputs.repetition, 60),
        (inputs.transition_quality, 40),
    ]);

    let mut activated_rules = Vec::new();
    let excellent_flow = minimum(&[
        rising_membership(naturalness, 82, 96),
        rising_membership(inputs.rhythm, 82, 96),
    ]);
    if excellent_flow > 0 {
        activated_rules.push(String::from(
            "excellent naturalness and rhythm imply excellent brand quality",
        ));
    }

    let strong_balance = minimum(&[
        rising_membership(naturalness, 68, 88),
        rising_membership(memorability, 68, 88),
        rising_membership(distinctiveness, 65, 86),
    ]);
    if strong_balance > 0 {
        activated_rules.push(String::from(
            "balanced naturalness, memorability and distinctiveness imply high brand quality",
        ));
    }

    let weak_pronunciation = falling_membership(inputs.pronounceability, 45, 72);
    if weak_pronunciation > 0 {
        activated_rules.push(String::from(
            "weak pronunciation limits perceived brand quality",
        ));
    }

    let mechanical_pattern = maximum(&[
        falling_membership(inputs.repetition, 45, 72),
        falling_membership(inputs.transition_quality, 45, 72),
    ]);
    if mechanical_pattern > 0 {
        activated_rules.push(String::from(
            "mechanical repetition or transitions reduce perceived quality",
        ));
    }

    let baseline = weighted_average(&[
        (naturalness, 40),
        (memorability, 30),
        (distinctiveness, 30),
    ]);
    let positive_adjustment = (u16::from(excellent_flow) * 8
        + u16::from(strong_balance) * 5)
        / 100;
    let negative_adjustment = (u16::from(weak_pronunciation) * 12
        + u16::from(mechanical_pattern) * 10)
        / 100;
    let adjusted = u16::from(baseline)
        .saturating_add(positive_adjustment)
        .saturating_sub(negative_adjustment)
        .min(100);
    let score = u8::try_from(adjusted).unwrap_or(100);

    let memberships = memberships(score);
    let quality = memberships
        .iter()
        .max_by_key(|membership| membership.degree)
        .map_or(LinguisticQuality::VeryLow, |membership| membership.quality);
    let confidence = confidence(&memberships);

    FuzzyReport {
        score,
        confidence,
        quality,
        memberships,
        activated_rules,
    }
}

fn memberships(value: u8) -> Vec<Membership> {
    vec![
        Membership {
            quality: LinguisticQuality::VeryLow,
            degree: falling_membership(value, 15, 35),
        },
        Membership {
            quality: LinguisticQuality::Low,
            degree: triangular_membership(value, 20, 40, 58),
        },
        Membership {
            quality: LinguisticQuality::Medium,
            degree: triangular_membership(value, 45, 62, 78),
        },
        Membership {
            quality: LinguisticQuality::High,
            degree: triangular_membership(value, 65, 80, 94),
        },
        Membership {
            quality: LinguisticQuality::Excellent,
            degree: rising_membership(value, 84, 97),
        },
    ]
}

fn confidence(memberships: &[Membership]) -> u8 {
    let mut degrees = memberships
        .iter()
        .map(|membership| membership.degree)
        .collect::<Vec<_>>();
    degrees.sort_unstable_by(|left, right| right.cmp(left));
    match degrees.as_slice() {
        [first, second, ..] => 50_u8.saturating_add(first.saturating_sub(*second) / 2),
        [first] => *first,
        [] => 0,
    }
}

fn weighted_average(values: &[(u8, u16)]) -> u8 {
    let weighted_sum = values
        .iter()
        .map(|(value, weight)| u32::from(*value) * u32::from(*weight))
        .sum::<u32>();
    let total_weight = values
        .iter()
        .map(|(_, weight)| u32::from(*weight))
        .sum::<u32>();
    if total_weight == 0 {
        return 0;
    }
    u8::try_from(weighted_sum / total_weight).unwrap_or(100)
}

fn triangular_membership(value: u8, start: u8, peak: u8, end: u8) -> u8 {
    if value <= start || value >= end {
        0
    } else if value == peak {
        100
    } else if value < peak {
        scale(value - start, peak - start)
    } else {
        scale(end - value, end - peak)
    }
}

fn rising_membership(value: u8, start: u8, end: u8) -> u8 {
    if value <= start {
        0
    } else if value >= end {
        100
    } else {
        scale(value - start, end - start)
    }
}

fn falling_membership(value: u8, start: u8, end: u8) -> u8 {
    100_u8.saturating_sub(rising_membership(value, start, end))
}

fn scale(numerator: u8, denominator: u8) -> u8 {
    if denominator == 0 {
        return 100;
    }
    u8::try_from(u16::from(numerator) * 100 / u16::from(denominator)).unwrap_or(100)
}

fn minimum(values: &[u8]) -> u8 {
    values.iter().copied().min().unwrap_or_default()
}

fn maximum(values: &[u8]) -> u8 {
    values.iter().copied().max().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{FuzzyInputs, LinguisticQuality, evaluate_fuzzy};

    #[test]
    fn fuzzy_evaluation_is_deterministic() {
        let inputs = FuzzyInputs {
            pronounceability: 91,
            rhythm: 88,
            vowel_balance: 84,
            repetition: 90,
            transition_quality: 89,
        };

        assert_eq!(evaluate_fuzzy(inputs), evaluate_fuzzy(inputs));
    }

    #[test]
    fn excellent_inputs_produce_excellent_quality() {
        let report = evaluate_fuzzy(FuzzyInputs {
            pronounceability: 100,
            rhythm: 100,
            vowel_balance: 100,
            repetition: 100,
            transition_quality: 100,
        });

        assert_eq!(report.score, 100);
        assert_eq!(report.quality, LinguisticQuality::Excellent);
        assert!(report.confidence >= 90);
        assert!(!report.activated_rules.is_empty());
    }

    #[test]
    fn weak_inputs_are_ranked_below_strong_inputs() {
        let weak = evaluate_fuzzy(FuzzyInputs {
            pronounceability: 35,
            rhythm: 42,
            vowel_balance: 55,
            repetition: 30,
            transition_quality: 38,
        });
        let strong = evaluate_fuzzy(FuzzyInputs {
            pronounceability: 90,
            rhythm: 88,
            vowel_balance: 85,
            repetition: 92,
            transition_quality: 90,
        });

        assert!(weak.score < strong.score);
        assert!(weak.activated_rules.iter().any(|rule| rule.contains("weak pronunciation")));
    }

    #[test]
    fn all_public_outputs_stay_in_range() {
        for value in 0..=100 {
            let report = evaluate_fuzzy(FuzzyInputs {
                pronounceability: value,
                rhythm: value,
                vowel_balance: value,
                repetition: value,
                transition_quality: value,
            });

            assert!(report.score <= 100);
            assert!(report.confidence <= 100);
            assert!(report.memberships.iter().all(|membership| membership.degree <= 100));
        }
    }
}
