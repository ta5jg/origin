/* =============================================================================
 * File:           crates/origin-core/src/morphology/normalize.rs
 * Project:        Origin
 * Author:         USDTG GROUP TECHNOLOGY LLC
 * Developer:      Irfan Gedik
 * Created Date:   2026-08-06
 * Version:        0.1.0
 *
 * Description:
 *   Normalizes historical-language roots and transliterations into the
 *   canonical lowercase ASCII representation consumed by Origin.
 *
 * License:
 *   Origin License v1.0 — see LICENSE in the repository root.
 * ============================================================================= */

//! Deterministic Unicode and transliteration normalization for Origin roots.
//!
//! The language datasets may contain diacritics, specialist transliteration
//! marks, ligatures, apostrophes, separators, and non-ASCII characters.
//! Origin's generation and comparison engines operate on a canonical
//! lowercase ASCII representation.
//!
//! This module converts supported forms without external dependencies and
//! records every transformation or removed character. It does not perform
//! aesthetic mutation, root merging, or candidate beautification.

use serde::{Deserialize, Serialize};

/// Canonical normalization mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NormalizationMode {
    /// Unsupported characters cause normalization to fail.
    Strict,

    /// Unsupported characters are removed and reported.
    #[default]
    Lenient,
}

/// Configuration controlling root normalization behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizationPolicy {
    /// Behavior when an unsupported character is encountered.
    pub mode: NormalizationMode,

    /// Whether whitespace, punctuation, and word separators are removed.
    pub remove_separators: bool,

    /// Whether Unicode combining marks are removed.
    pub remove_combining_marks: bool,

    /// Whether ASCII digits are removed.
    pub remove_digits: bool,
}

impl NormalizationPolicy {
    /// Returns the default policy used by Origin language datasets.
    #[must_use]
    pub const fn standard() -> Self {
        Self {
            mode: NormalizationMode::Lenient,
            remove_separators: true,
            remove_combining_marks: true,
            remove_digits: true,
        }
    }

    /// Returns a strict policy that rejects unsupported characters.
    #[must_use]
    pub const fn strict() -> Self {
        Self {
            mode: NormalizationMode::Strict,
            ..Self::standard()
        }
    }

    /// Sets the unsupported-character handling mode.
    #[must_use]
    pub const fn with_mode(mut self, mode: NormalizationMode) -> Self {
        self.mode = mode;
        self
    }

    /// Sets whether separators are removed.
    #[must_use]
    pub const fn with_remove_separators(mut self, remove_separators: bool) -> Self {
        self.remove_separators = remove_separators;
        self
    }

    /// Sets whether combining marks are removed.
    #[must_use]
    pub const fn with_remove_combining_marks(mut self, remove_combining_marks: bool) -> Self {
        self.remove_combining_marks = remove_combining_marks;
        self
    }

    /// Sets whether digits are removed.
    #[must_use]
    pub const fn with_remove_digits(mut self, remove_digits: bool) -> Self {
        self.remove_digits = remove_digits;
        self
    }
}

impl Default for NormalizationPolicy {
    fn default() -> Self {
        Self::standard()
    }
}

/// Classification of a normalization transformation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NormalizationChangeKind {
    /// Uppercase ASCII was converted to lowercase.
    CaseFold,

    /// A Unicode character was converted to an ASCII letter sequence.
    Transliteration,

    /// A Unicode combining mark was removed.
    CombiningMarkRemoved,

    /// Whitespace or punctuation was removed.
    SeparatorRemoved,

    /// A digit was removed.
    DigitRemoved,

    /// An unsupported character was removed in lenient mode.
    UnsupportedRemoved,
}

/// One attributable normalization operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizationChange {
    /// Byte offset of the original character in the source string.
    pub byte_offset: usize,

    /// Original source character.
    pub original: char,

    /// ASCII replacement. Empty when the character was removed.
    pub replacement: String,

    /// Classification of the transformation.
    pub kind: NormalizationChangeKind,
}

impl NormalizationChange {
    fn new(
        byte_offset: usize,
        original: char,
        replacement: impl Into<String>,
        kind: NormalizationChangeKind,
    ) -> Self {
        Self {
            byte_offset,
            original,
            replacement: replacement.into(),
            kind,
        }
    }
}

/// Complete result of one normalization operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizationReport {
    /// Original input.
    pub original: String,

    /// Canonical lowercase ASCII root.
    pub normalized: String,

    /// Ordered transformations applied to the input.
    pub changes: Vec<NormalizationChange>,
}

impl NormalizationReport {
    /// Returns whether normalization changed or removed any input character.
    #[must_use]
    pub fn is_lossy(&self) -> bool {
        !self.changes.is_empty()
    }

    /// Returns the number of removed characters.
    #[must_use]
    pub fn removed_count(&self) -> usize {
        self.changes
            .iter()
            .filter(|change| change.replacement.is_empty())
            .count()
    }

    /// Returns the number of transliterated characters.
    #[must_use]
    pub fn transliterated_count(&self) -> usize {
        self.changes
            .iter()
            .filter(|change| change.kind == NormalizationChangeKind::Transliteration)
            .count()
    }

    /// Returns whether the output satisfies Origin's canonical root format.
    #[must_use]
    pub fn is_canonical(&self) -> bool {
        is_canonical_root(&self.normalized)
    }

    /// Converts transformations into stable provenance descriptions.
    #[must_use]
    pub fn provenance_steps(&self) -> Vec<String> {
        self.changes
            .iter()
            .map(|change| {
                let replacement = if change.replacement.is_empty() {
                    "<removed>"
                } else {
                    change.replacement.as_str()
                };

                format!(
                    "normalize:{}:{}>{replacement}",
                    change.byte_offset, change.original
                )
            })
            .collect()
    }
}

/// Error produced while normalizing a root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NormalizationError {
    /// Input contains no characters after trimming.
    EmptyInput,

    /// Normalization produced no canonical letters.
    EmptyOutput,

    /// A separator was encountered while separator removal was disabled.
    SeparatorNotAllowed {
        /// Unsupported source character.
        character: char,

        /// Byte offset in the original input.
        byte_offset: usize,
    },

    /// A combining mark was encountered while removal was disabled.
    CombiningMarkNotAllowed {
        /// Combining mark.
        character: char,

        /// Byte offset in the original input.
        byte_offset: usize,
    },

    /// A digit was encountered while digit removal was disabled.
    DigitNotAllowed {
        /// Unsupported digit.
        character: char,

        /// Byte offset in the original input.
        byte_offset: usize,
    },

    /// Strict normalization encountered an unsupported character.
    UnsupportedCharacter {
        /// Unsupported source character.
        character: char,

        /// Byte offset in the original input.
        byte_offset: usize,
    },
}

impl std::fmt::Display for NormalizationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyInput => formatter.write_str("normalization input must not be empty"),
            Self::EmptyOutput => {
                formatter.write_str("normalization produced no canonical ASCII letters")
            }
            Self::SeparatorNotAllowed {
                character,
                byte_offset,
            } => write!(
                formatter,
                "separator {character:?} is not allowed at byte offset {byte_offset}"
            ),
            Self::CombiningMarkNotAllowed {
                character,
                byte_offset,
            } => write!(
                formatter,
                "combining mark {character:?} is not allowed at byte offset {byte_offset}"
            ),
            Self::DigitNotAllowed {
                character,
                byte_offset,
            } => write!(
                formatter,
                "digit {character:?} is not allowed at byte offset {byte_offset}"
            ),
            Self::UnsupportedCharacter {
                character,
                byte_offset,
            } => write!(
                formatter,
                "unsupported normalization character {character:?} at byte offset {byte_offset}"
            ),
        }
    }
}

impl std::error::Error for NormalizationError {}

/// Normalizes a root using [`NormalizationPolicy::standard`].
///
/// # Errors
///
/// Returns [`NormalizationError`] when the input is empty or normalization
/// produces no canonical ASCII letters.
pub fn normalize_root(input: &str) -> Result<String, NormalizationError> {
    normalize_root_with_policy(input, NormalizationPolicy::standard())
        .map(|report| report.normalized)
}

/// Normalizes a root and returns a complete transformation report.
///
/// # Errors
///
/// Returns [`NormalizationError`] when:
///
/// - the input is empty,
/// - the active policy rejects a separator, combining mark, or digit,
/// - strict mode encounters an unsupported character,
/// - or normalization produces no canonical ASCII letters.
pub fn normalize_root_with_policy(
    input: &str,
    policy: NormalizationPolicy,
) -> Result<NormalizationReport, NormalizationError> {
    if input.trim().is_empty() {
        return Err(NormalizationError::EmptyInput);
    }

    let mut normalized = String::with_capacity(input.len());
    let mut changes = Vec::new();

    for (byte_offset, character) in input.char_indices() {
        normalize_character(
            character,
            byte_offset,
            policy,
            &mut normalized,
            &mut changes,
        )?;
    }

    if normalized.is_empty() {
        return Err(NormalizationError::EmptyOutput);
    }

    debug_assert!(is_canonical_root(&normalized));

    Ok(NormalizationReport {
        original: input.to_owned(),
        normalized,
        changes,
    })
}

/// Returns whether a value is a canonical Origin root.
///
/// Canonical roots contain one or more lowercase ASCII letters.
#[must_use]
pub fn is_canonical_root(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_lowercase())
}

fn normalize_character(
    character: char,
    byte_offset: usize,
    policy: NormalizationPolicy,
    normalized: &mut String,
    changes: &mut Vec<NormalizationChange>,
) -> Result<(), NormalizationError> {
    if character.is_ascii_lowercase() {
        normalized.push(character);
        return Ok(());
    }

    if character.is_ascii_uppercase() {
        let replacement = character.to_ascii_lowercase();
        normalized.push(replacement);
        changes.push(NormalizationChange::new(
            byte_offset,
            character,
            replacement.to_string(),
            NormalizationChangeKind::CaseFold,
        ));
        return Ok(());
    }

    if character.is_ascii_digit() {
        return handle_digit(character, byte_offset, policy, changes);
    }

    if is_combining_mark(character) {
        return handle_combining_mark(character, byte_offset, policy, changes);
    }

    if is_separator(character) {
        return handle_separator(character, byte_offset, policy, changes);
    }

    let lowercase = character.to_lowercase().collect::<String>();

    if lowercase != character.to_string() {
        let mut fully_supported = true;
        let mut replacement = String::new();

        for lowered in lowercase.chars() {
            if lowered.is_ascii_lowercase() {
                replacement.push(lowered);
            } else if is_combining_mark(lowered) {
                if !policy.remove_combining_marks {
                    fully_supported = false;
                    break;
                }
            } else if let Some(mapped) = transliterate(lowered) {
                replacement.push_str(mapped);
            } else {
                fully_supported = false;
                break;
            }
        }

        if fully_supported && !replacement.is_empty() {
            normalized.push_str(&replacement);
            changes.push(NormalizationChange::new(
                byte_offset,
                character,
                replacement,
                NormalizationChangeKind::Transliteration,
            ));
            return Ok(());
        }
    }

    if let Some(replacement) = transliterate(character) {
        normalized.push_str(replacement);
        changes.push(NormalizationChange::new(
            byte_offset,
            character,
            replacement,
            NormalizationChangeKind::Transliteration,
        ));
        return Ok(());
    }

    handle_unsupported(character, byte_offset, policy, changes)
}

fn handle_digit(
    character: char,
    byte_offset: usize,
    policy: NormalizationPolicy,
    changes: &mut Vec<NormalizationChange>,
) -> Result<(), NormalizationError> {
    if policy.remove_digits {
        changes.push(NormalizationChange::new(
            byte_offset,
            character,
            "",
            NormalizationChangeKind::DigitRemoved,
        ));
        Ok(())
    } else {
        Err(NormalizationError::DigitNotAllowed {
            character,
            byte_offset,
        })
    }
}

fn handle_combining_mark(
    character: char,
    byte_offset: usize,
    policy: NormalizationPolicy,
    changes: &mut Vec<NormalizationChange>,
) -> Result<(), NormalizationError> {
    if policy.remove_combining_marks {
        changes.push(NormalizationChange::new(
            byte_offset,
            character,
            "",
            NormalizationChangeKind::CombiningMarkRemoved,
        ));
        Ok(())
    } else {
        Err(NormalizationError::CombiningMarkNotAllowed {
            character,
            byte_offset,
        })
    }
}

fn handle_separator(
    character: char,
    byte_offset: usize,
    policy: NormalizationPolicy,
    changes: &mut Vec<NormalizationChange>,
) -> Result<(), NormalizationError> {
    if policy.remove_separators {
        changes.push(NormalizationChange::new(
            byte_offset,
            character,
            "",
            NormalizationChangeKind::SeparatorRemoved,
        ));
        Ok(())
    } else {
        Err(NormalizationError::SeparatorNotAllowed {
            character,
            byte_offset,
        })
    }
}

fn handle_unsupported(
    character: char,
    byte_offset: usize,
    policy: NormalizationPolicy,
    changes: &mut Vec<NormalizationChange>,
) -> Result<(), NormalizationError> {
    match policy.mode {
        NormalizationMode::Strict => Err(NormalizationError::UnsupportedCharacter {
            character,
            byte_offset,
        }),
        NormalizationMode::Lenient => {
            changes.push(NormalizationChange::new(
                byte_offset,
                character,
                "",
                NormalizationChangeKind::UnsupportedRemoved,
            ));
            Ok(())
        }
    }
}

const fn is_combining_mark(character: char) -> bool {
    matches!(
        character as u32,
        0x0300..=0x036F
            | 0x1AB0..=0x1AFF
            | 0x1DC0..=0x1DFF
            | 0x20D0..=0x20FF
            | 0xFE20..=0xFE2F
    )
}

fn is_separator(character: char) -> bool {
    character.is_whitespace()
        || matches!(
            character,
            '-' | '_'
                | '\''
                | '’'
                | '‘'
                | 'ʼ'
                | 'ʹ'
                | 'ʺ'
                | 'ʾ'
                | 'ʿ'
                | '`'
                | '´'
                | '.'
                | ','
                | ':'
                | ';'
                | '/'
                | '\\'
                | '|'
                | '·'
                | '•'
                | '‐'
                | '‒'
                | '–'
                | '—'
        )
}

#[allow(clippy::too_many_lines)]
const fn transliterate(character: char) -> Option<&'static str> {
    match character {
        'á' | 'à' | 'â' | 'ä' | 'ã' | 'å' | 'ā' | 'ă' | 'ą' | 'ǎ' | 'ạ' | 'ả' | 'ấ' | 'ầ' | 'ẩ'
        | 'ẫ' | 'ậ' | 'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' => Some("a"),

        'æ' => Some("ae"),

        'ç' | 'ć' | 'ĉ' | 'ċ' | 'č' => Some("c"),

        'ď' | 'đ' | 'ð' | 'ḍ' | 'ḏ' => Some("d"),

        'é' | 'è' | 'ê' | 'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' | 'ẹ' | 'ẻ' | 'ẽ' | 'ế' | 'ề' | 'ể'
        | 'ễ' | 'ệ' | 'ə' => Some("e"),

        'ğ' | 'ĝ' | 'ġ' | 'ģ' | 'ǧ' | 'ḡ' => Some("g"),

        'ḫ' | 'ḥ' | 'ħ' | 'ĥ' => Some("h"),

        'í' | 'ì' | 'î' | 'ï' | 'ĩ' | 'ī' | 'ĭ' | 'į' | 'ı' | 'ǐ' | 'ị' | 'ỉ' => {
            Some("i")
        }

        'ĵ' => Some("j"),

        'ķ' | 'ḳ' => Some("k"),

        'ĺ' | 'ļ' | 'ľ' | 'ŀ' | 'ł' => Some("l"),

        'ñ' | 'ń' | 'ņ' | 'ň' | 'ŋ' | 'ṅ' | 'ṇ' => Some("n"),

        'ó' | 'ò' | 'ô' | 'ö' | 'õ' | 'ø' | 'ō' | 'ŏ' | 'ő' | 'ǒ' | 'ọ' | 'ỏ' | 'ố' | 'ồ' | 'ổ'
        | 'ỗ' | 'ộ' | 'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ' => Some("o"),

        'œ' => Some("oe"),

        'ŕ' | 'ŗ' | 'ř' | 'ṛ' => Some("r"),

        'ś' | 'ŝ' | 'ş' | 'š' | 'ṣ' | 'ș' => Some("s"),

        'ß' => Some("ss"),

        'ť' | 'ţ' | 'ṭ' | 'ț' | 'ŧ' => Some("t"),

        'þ' => Some("th"),

        'ú' | 'ù' | 'û' | 'ü' | 'ũ' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' | 'ǔ' | 'ụ' | 'ủ' | 'ứ' | 'ừ'
        | 'ử' | 'ữ' | 'ự' => Some("u"),

        'ŵ' => Some("w"),

        'ý' | 'ÿ' | 'ŷ' | 'ỳ' | 'ỵ' | 'ỷ' | 'ỹ' => Some("y"),

        'ź' | 'ż' | 'ž' | 'ẓ' => Some("z"),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        NormalizationChangeKind, NormalizationError, NormalizationMode, NormalizationPolicy,
        is_canonical_root, normalize_root, normalize_root_with_policy,
    };

    #[test]
    fn canonical_ascii_root_is_preserved() {
        let report = normalize_root_with_policy("qervon", NormalizationPolicy::standard())
            .expect("canonical root must normalize");

        assert_eq!(report.normalized, "qervon");
        assert!(report.changes.is_empty());
        assert!(!report.is_lossy());
        assert!(report.is_canonical());
    }

    #[test]
    fn uppercase_ascii_is_case_folded() {
        let report = normalize_root_with_policy("QERVON", NormalizationPolicy::standard())
            .expect("uppercase root must normalize");

        assert_eq!(report.normalized, "qervon");
        assert_eq!(report.changes.len(), 6);
        assert!(
            report
                .changes
                .iter()
                .all(|change| change.kind == NormalizationChangeKind::CaseFold)
        );
    }

    #[test]
    fn old_turkic_and_turkish_characters_are_normalized() {
        assert_eq!(
            normalize_root("YOLCUĞU").expect("root must normalize"),
            "yolcugu"
        );

        assert_eq!(normalize_root("IŞIK").expect("root must normalize"), "isik");
    }

    #[test]
    fn akkadian_transliteration_is_normalized() {
        let report = normalize_root_with_policy("šarrūtu", NormalizationPolicy::standard())
            .expect("Akkadian transliteration must normalize");

        assert_eq!(report.normalized, "sarrutu");
        assert_eq!(report.transliterated_count(), 2);
    }

    #[test]
    fn specialist_consonants_are_normalized() {
        assert_eq!(
            normalize_root("ḫaṭṭu").expect("root must normalize"),
            "hattu"
        );

        assert_eq!(normalize_root("ṣēru").expect("root must normalize"), "seru");
    }

    #[test]
    fn ligatures_expand_to_ascii_sequences() {
        assert_eq!(
            normalize_root("cœlum").expect("root must normalize"),
            "coelum"
        );

        assert_eq!(normalize_root("æra").expect("root must normalize"), "aera");
    }

    #[test]
    fn separators_are_removed_and_reported() {
        let report = normalize_root_with_policy("old-turkic_yol", NormalizationPolicy::standard())
            .expect("separated root must normalize");

        assert_eq!(report.normalized, "oldturkicyol");
        assert_eq!(report.removed_count(), 2);
        assert!(
            report
                .changes
                .iter()
                .all(|change| { change.kind == NormalizationChangeKind::SeparatorRemoved })
        );
    }

    #[test]
    fn apostrophic_transliteration_marks_are_removed() {
        assert_eq!(
            normalize_root("šaʿru").expect("root must normalize"),
            "saru"
        );

        assert_eq!(normalize_root("ilu’").expect("root must normalize"), "ilu");
    }

    #[test]
    fn decomposed_combining_marks_are_removed() {
        let decomposed = "a\u{0304}n";

        let report = normalize_root_with_policy(decomposed, NormalizationPolicy::standard())
            .expect("decomposed form must normalize");

        assert_eq!(report.normalized, "an");
        assert_eq!(report.removed_count(), 1);
        assert_eq!(
            report.changes[0].kind,
            NormalizationChangeKind::CombiningMarkRemoved
        );
    }

    #[test]
    fn digits_are_removed_by_standard_policy() {
        let report = normalize_root_with_policy("uru2", NormalizationPolicy::standard())
            .expect("indexed source form must normalize");

        assert_eq!(report.normalized, "uru");
        assert_eq!(report.removed_count(), 1);
        assert_eq!(
            report.changes[0].kind,
            NormalizationChangeKind::DigitRemoved
        );
    }

    #[test]
    fn strict_mode_rejects_unknown_characters() {
        let result = normalize_root_with_policy("via☃", NormalizationPolicy::strict());

        assert_eq!(
            result,
            Err(NormalizationError::UnsupportedCharacter {
                character: '☃',
                byte_offset: 3,
            })
        );
    }

    #[test]
    fn lenient_mode_removes_unknown_characters() {
        let report = normalize_root_with_policy("via☃", NormalizationPolicy::standard())
            .expect("lenient mode must remove unknown characters");

        assert_eq!(report.normalized, "via");
        assert_eq!(
            report.changes[0].kind,
            NormalizationChangeKind::UnsupportedRemoved
        );
    }

    #[test]
    fn disabled_separator_removal_returns_error() {
        let policy = NormalizationPolicy::standard().with_remove_separators(false);

        assert_eq!(
            normalize_root_with_policy("old-turkic", policy),
            Err(NormalizationError::SeparatorNotAllowed {
                character: '-',
                byte_offset: 3,
            })
        );
    }

    #[test]
    fn disabled_digit_removal_returns_error() {
        let policy = NormalizationPolicy::standard().with_remove_digits(false);

        assert_eq!(
            normalize_root_with_policy("uru2", policy),
            Err(NormalizationError::DigitNotAllowed {
                character: '2',
                byte_offset: 3,
            })
        );
    }

    #[test]
    fn disabled_combining_mark_removal_returns_error() {
        let policy = NormalizationPolicy::standard().with_remove_combining_marks(false);

        assert_eq!(
            normalize_root_with_policy("a\u{0304}n", policy),
            Err(NormalizationError::CombiningMarkNotAllowed {
                character: '\u{0304}',
                byte_offset: 1,
            })
        );
    }

    #[test]
    fn empty_input_is_rejected() {
        assert_eq!(normalize_root("   "), Err(NormalizationError::EmptyInput));
    }

    #[test]
    fn input_without_supported_letters_is_rejected() {
        assert_eq!(
            normalize_root("---123☃"),
            Err(NormalizationError::EmptyOutput)
        );
    }

    #[test]
    fn canonical_root_validation_is_explicit() {
        assert!(is_canonical_root("qervon"));
        assert!(!is_canonical_root(""));
        assert!(!is_canonical_root("Qervon"));
        assert!(!is_canonical_root("qervon1"));
        assert!(!is_canonical_root("qer-von"));
    }

    #[test]
    fn provenance_steps_are_stable() {
        let report = normalize_root_with_policy("Ša-ru", NormalizationPolicy::standard())
            .expect("root must normalize");

        assert_eq!(report.normalized, "saru");
        assert_eq!(
            report.provenance_steps(),
            ["normalize:0:Š>s", "normalize:3:-><removed>",]
        );
    }

    #[test]
    fn policy_mode_can_be_changed_explicitly() {
        let policy = NormalizationPolicy::standard().with_mode(NormalizationMode::Strict);

        assert_eq!(policy.mode, NormalizationMode::Strict);
    }
}
