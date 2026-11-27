//! Code to convert the Rust-styled field/variant (e.g. `my_field`, `MyType`) to the
//! case of the source (e.g. `my-field`, `MY_FIELD`).

use self::RenameRule::*;
use std::fmt::{self, Debug, Display};

/// The different possible ways to change case of fields in a struct, or variants in an enum.
#[derive(Copy, Clone, PartialEq)]
pub enum RenameRule {
    /// Don't apply a default rename rule.
    None,
    /// Rename direct children to "lowercase" style.
    LowerCase,
    /// Rename direct children to "UPPERCASE" style.
    UpperCase,
    /// Rename direct children to "PascalCase" style, as typically used for
    /// enum variants.
    PascalCase,
    /// Rename direct children to "camelCase" style.
    CamelCase,
    /// Rename direct children to "snake_case" style, as commonly used for
    /// fields.
    SnakeCase,
    /// Rename direct children to "SCREAMING_SNAKE_CASE" style, as commonly
    /// used for constants.
    ScreamingSnakeCase,
    /// Rename direct children to "kebab-case" style.
    KebabCase,
    /// Rename direct children to "SCREAMING-KEBAB-CASE" style.
    ScreamingKebabCase,
}

static RENAME_RULES: &[(&str, RenameRule)] = &[
    ("lowercase", LowerCase),
    ("UPPERCASE", UpperCase),
    ("PascalCase", PascalCase),
    ("camelCase", CamelCase),
    ("snake_case", SnakeCase),
    ("SCREAMING_SNAKE_CASE", ScreamingSnakeCase),
    ("kebab-case", KebabCase),
    ("SCREAMING-KEBAB-CASE", ScreamingKebabCase),
];

impl RenameRule {
    pub fn from_str(rename_all_str: &str) -> Result<Self, ParseError> {
        for (name, rule) in RENAME_RULES {
            if rename_all_str == *name {
                return Ok(*rule);
            }
        }
        Err(ParseError {
            unknown: rename_all_str,
        })
    }

    /// Apply a renaming rule to an enum variant, returning the version expected in the source.
    pub fn apply_to_variant(self, variant: &str) -> String {
        match self {
            None | PascalCase => variant.to_owned(),
            LowerCase => variant.to_ascii_lowercase(),
            UpperCase => variant.to_ascii_uppercase(),
            CamelCase => Self::lowercase_first_char_unicode(variant),
            SnakeCase => {
                let mut snake = String::new();
                for (i, ch) in variant.char_indices() {
                    if i > 0 && ch.is_uppercase() {
                        snake.push('_');
                    }
                    snake.push(ch.to_ascii_lowercase());
                }
                snake
            }
            ScreamingSnakeCase => SnakeCase.apply_to_variant(variant).to_ascii_uppercase(),
            KebabCase => SnakeCase.apply_to_variant(variant).replace('_', "-"),
            ScreamingKebabCase => ScreamingSnakeCase
                .apply_to_variant(variant)
                .replace('_', "-"),
        }
    }

    /// Apply a renaming rule to a struct field, returning the version expected in the source.
    pub fn apply_to_field(self, field: &str) -> String {
        match self {
            None | LowerCase | SnakeCase => field.to_owned(),
            UpperCase => field.to_ascii_uppercase(),
            PascalCase => {
                let mut pascal = String::new();
                let mut capitalize = true;
                for ch in field.chars() {
                    if ch == '_' {
                        capitalize = true;
                    } else if capitalize {
                        pascal.push(ch.to_ascii_uppercase());
                        capitalize = false;
                    } else {
                        pascal.push(ch);
                    }
                }
                pascal
            }
            CamelCase => {
                let pascal = PascalCase.apply_to_field(field);
                Self::lowercase_first_char_unicode(&pascal)
            }
            ScreamingSnakeCase => field.to_ascii_uppercase(),
            KebabCase => field.replace('_', "-"),
            ScreamingKebabCase => ScreamingSnakeCase.apply_to_field(field).replace('_', "-"),
        }
    }

    /// Lowercase the first Unicode scalar using full Unicode case mapping,
    /// then append the remainder unchanged. Avoids UTF-8 slicing panics.
    fn lowercase_first_char_unicode(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            ::std::option::Option::None => String::new(),
            ::std::option::Option::Some(first) => {
                // `to_lowercase()` may expand (e.g., 'İ' -> 'i' + U+0307)
                let mut out = String::with_capacity(s.len());
                out.extend(first.to_lowercase());
                out.extend(chars);
                out
            }
        }
    }

    /// Returns the `RenameRule` if it is not `None`, `rule_b` otherwise.
    pub fn or(self, rule_b: Self) -> Self {
        match self {
            None => rule_b,
            _ => self,
        }
    }
}

pub struct ParseError<'a> {
    unknown: &'a str,
}

impl<'a> Display for ParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("unknown rename rule `rename_all = ")?;
        Debug::fmt(self.unknown, f)?;
        f.write_str("`, expected one of ")?;
        for (i, (name, _rule)) in RENAME_RULES.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            Debug::fmt(name, f)?;
        }
        Ok(())
    }
}

#[test]
fn rename_variants() {
    for &(original, lower, upper, camel, snake, screaming, kebab, screaming_kebab) in &[
        (
            "Outcome", "outcome", "OUTCOME", "outcome", "outcome", "OUTCOME", "outcome", "OUTCOME",
        ),
        (
            "VeryTasty",
            "verytasty",
            "VERYTASTY",
            "veryTasty",
            "very_tasty",
            "VERY_TASTY",
            "very-tasty",
            "VERY-TASTY",
        ),
        ("A", "a", "A", "a", "a", "A", "a", "A"),
        ("Z42", "z42", "Z42", "z42", "z42", "Z42", "z42", "Z42"),
    ] {
        assert_eq!(None.apply_to_variant(original), original);
        assert_eq!(LowerCase.apply_to_variant(original), lower);
        assert_eq!(UpperCase.apply_to_variant(original), upper);
        assert_eq!(PascalCase.apply_to_variant(original), original);
        assert_eq!(CamelCase.apply_to_variant(original), camel);
        assert_eq!(SnakeCase.apply_to_variant(original), snake);
        assert_eq!(ScreamingSnakeCase.apply_to_variant(original), screaming);
        assert_eq!(KebabCase.apply_to_variant(original), kebab);
        assert_eq!(
            ScreamingKebabCase.apply_to_variant(original),
            screaming_kebab
        );
    }
}

#[test]
fn rename_fields() {
    for &(original, upper, pascal, camel, screaming, kebab, screaming_kebab) in &[
        (
            "outcome", "OUTCOME", "Outcome", "outcome", "OUTCOME", "outcome", "OUTCOME",
        ),
        (
            "very_tasty",
            "VERY_TASTY",
            "VeryTasty",
            "veryTasty",
            "VERY_TASTY",
            "very-tasty",
            "VERY-TASTY",
        ),
        ("a", "A", "A", "a", "A", "a", "A"),
        ("z42", "Z42", "Z42", "z42", "Z42", "z42", "Z42"),
    ] {
        assert_eq!(None.apply_to_field(original), original);
        assert_eq!(UpperCase.apply_to_field(original), upper);
        assert_eq!(PascalCase.apply_to_field(original), pascal);
        assert_eq!(CamelCase.apply_to_field(original), camel);
        assert_eq!(SnakeCase.apply_to_field(original), original);
        assert_eq!(ScreamingSnakeCase.apply_to_field(original), screaming);
        assert_eq!(KebabCase.apply_to_field(original), kebab);
        assert_eq!(ScreamingKebabCase.apply_to_field(original), screaming_kebab);
    }
}

#[cfg(test)]
mod unicode_camelcase_tests {
    use super::RenameRule;

    // --- ASCII regressions: behavior must remain identical ---

    #[test]
    fn camelcase_variant_ascii_regression() {
        assert_eq!(
            RenameRule::CamelCase.apply_to_variant("FieldName"),
            "fieldName"
        );
        assert_eq!(
            RenameRule::CamelCase.apply_to_variant("URLValue"),
            "uRLValue" // existing behavior: only first scalar is lowercased
        );
    }

    #[test]
    fn camelcase_field_ascii_regression() {
        assert_eq!(
            RenameRule::CamelCase.apply_to_field("field_name"),
            "fieldName"
        );
        assert_eq!(
            RenameRule::CamelCase.apply_to_field("long_field_name"),
            "longFieldName"
        );
    }

    // --- Unicode behavior: first scalar lowercased using full Unicode mapping ---

    #[test]
    fn camelcase_variant_non_ascii_basic() {
        // Greek capital sigma -> small sigma (single-scalar mapping)
        assert_eq!(
            RenameRule::CamelCase.apply_to_variant("Σomething"),
            "σomething"
        );
    }

    #[test]
    fn camelcase_variant_non_ascii_expanding() {
        // LATIN CAPITAL LETTER I WITH DOT ABOVE (U+0130) lowercases to
        // 'i' + COMBINING DOT ABOVE (U+0307) in Unicode
        assert_eq!(
            RenameRule::CamelCase.apply_to_variant("İstanbul"),
            "i\u{307}stanbul"
        );
    }

    #[test]
    fn camelcase_field_mixed_identifier_unicode() {
        // apply_to_field: first makes PascalCase from snake_case,
        // then CamelCase lowercases the first Unicode scalar only.
        // Non-ASCII first scalar stays semantically lowercased by Unicode rules;
        // ASCII segment after '_' still Pascalizes into 'Feature'.
        assert_eq!(
            RenameRule::CamelCase.apply_to_field("परिणाम_feature"),
            "परिणामFeature"
        );
    }

    #[test]
    fn camelcase_field_chinese_identifiers_stable() {
        // Fields with non-cased Han characters should remain unchanged.
        assert_eq!(RenameRule::CamelCase.apply_to_field("项目名称"), "项目名称");
        assert_eq!(RenameRule::CamelCase.apply_to_field("项目地址"), "项目地址");
    }

    #[test]
    fn camelcase_variant_chinese_identifiers_stable() {
        // Same expectation for variant names.
        assert_eq!(
            RenameRule::CamelCase.apply_to_variant("项目名称"),
            "项目名称"
        );
        assert_eq!(
            RenameRule::CamelCase.apply_to_variant("项目地址"),
            "项目地址"
        );
    }

    // Sanity: other rename rules remain unaffected by our change
    #[test]
    fn other_rules_unchanged() {
        assert_eq!(
            RenameRule::SnakeCase.apply_to_variant("FieldName"),
            "field_name"
        );
        assert_eq!(
            RenameRule::KebabCase.apply_to_field("field_name"),
            "field-name"
        );
        assert_eq!(
            RenameRule::ScreamingSnakeCase.apply_to_variant("FieldName"),
            "FIELD_NAME"
        );
    }
}
