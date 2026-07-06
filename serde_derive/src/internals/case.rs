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
    pub fn apply_to_variant(self, variant: &str, independent_number: bool) -> String {
        match self {
            None | PascalCase => variant.to_owned(),
            LowerCase => variant.to_ascii_lowercase(),
            UpperCase => variant.to_ascii_uppercase(),
            CamelCase => variant[..1].to_ascii_lowercase() + &variant[1..],
            SnakeCase => {
                let mut snake = String::new();
                let mut prev_ch: Option<char> = Option::None;
                for ch in variant.chars() {
                    if let Some(prev) = prev_ch {
                        let mut needs_underscore = ch.is_uppercase();
                        if independent_number && !needs_underscore {
                            needs_underscore = (prev.is_ascii_digit() && ch.is_ascii_alphabetic())
                                || (prev.is_ascii_alphabetic() && ch.is_ascii_digit());
                        }
                        if needs_underscore {
                            snake.push('_');
                        }
                    }
                    snake.push(ch.to_ascii_lowercase());
                    prev_ch = Some(ch);
                }
                snake
            }
            ScreamingSnakeCase => SnakeCase
                .apply_to_variant(variant, independent_number)
                .to_ascii_uppercase(),
            KebabCase => SnakeCase
                .apply_to_variant(variant, independent_number)
                .replace('_', "-"),
            ScreamingKebabCase => ScreamingSnakeCase
                .apply_to_variant(variant, independent_number)
                .replace('_', "-"),
        }
    }

    /// Apply a renaming rule to a struct field, returning the version expected in the source.
    pub fn apply_to_field(self, field: &str, independent_number: bool) -> String {
        match self {
            None | LowerCase => field.to_owned(),
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
                let pascal = PascalCase.apply_to_field(field, independent_number);
                pascal[..1].to_ascii_lowercase() + &pascal[1..]
            }
            SnakeCase => {
                if independent_number {
                    let mut snake = String::new();
                    let mut prev_ch: Option<char> = Option::None;
                    for ch in field.chars() {
                        if let Some(prev) = prev_ch {
                            let mut needs_underscore = ch.is_uppercase();
                            if !needs_underscore {
                                needs_underscore = (prev.is_ascii_digit()
                                    && ch.is_ascii_alphabetic())
                                    || (prev.is_ascii_alphabetic() && ch.is_ascii_digit());
                            }
                            if needs_underscore {
                                snake.push('_');
                            }
                        }
                        snake.push(ch.to_ascii_lowercase());
                        prev_ch = Some(ch);
                    }
                    snake
                } else {
                    field.to_owned()
                }
            }
            ScreamingSnakeCase => {
                if independent_number {
                    SnakeCase.apply_to_field(field, true).to_ascii_uppercase()
                } else {
                    field.to_ascii_uppercase()
                }
            }
            KebabCase => {
                if independent_number {
                    SnakeCase.apply_to_field(field, true).replace('_', "-")
                } else {
                    field.replace('_', "-")
                }
            }
            ScreamingKebabCase => ScreamingSnakeCase
                .apply_to_field(field, independent_number)
                .replace('_', "-"),
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
        assert_eq!(None.apply_to_variant(original, false), original);
        assert_eq!(LowerCase.apply_to_variant(original, false), lower);
        assert_eq!(UpperCase.apply_to_variant(original, false), upper);
        assert_eq!(PascalCase.apply_to_variant(original, false), original);
        assert_eq!(CamelCase.apply_to_variant(original, false), camel);
        assert_eq!(SnakeCase.apply_to_variant(original, false), snake);
        assert_eq!(
            ScreamingSnakeCase.apply_to_variant(original, false),
            screaming
        );
        assert_eq!(KebabCase.apply_to_variant(original, false), kebab);
        assert_eq!(
            ScreamingKebabCase.apply_to_variant(original, false),
            screaming_kebab
        );
    }

    // Test independent_number = true
    assert_eq!(
        SnakeCase.apply_to_variant("Have1Apple", true),
        "have_1_apple"
    );
    assert_eq!(
        KebabCase.apply_to_variant("Have1Apple", true),
        "have-1-apple"
    );
    assert_eq!(
        ScreamingSnakeCase.apply_to_variant("Have1Apple", true),
        "HAVE_1_APPLE"
    );
    assert_eq!(
        ScreamingKebabCase.apply_to_variant("Have1Apple", true),
        "HAVE-1-APPLE"
    );
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
        assert_eq!(None.apply_to_field(original, false), original);
        assert_eq!(UpperCase.apply_to_field(original, false), upper);
        assert_eq!(PascalCase.apply_to_field(original, false), pascal);
        assert_eq!(CamelCase.apply_to_field(original, false), camel);
        assert_eq!(SnakeCase.apply_to_field(original, false), original);
        assert_eq!(
            ScreamingSnakeCase.apply_to_field(original, false),
            screaming
        );
        assert_eq!(KebabCase.apply_to_field(original, false), kebab);
        assert_eq!(
            ScreamingKebabCase.apply_to_field(original, false),
            screaming_kebab
        );
    }

    // Test independent_number = true on fields
    assert_eq!(SnakeCase.apply_to_field("have1Apple", true), "have_1_apple");
    assert_eq!(SnakeCase.apply_to_field("have1apple", true), "have_1_apple");
    assert_eq!(
        KebabCase.apply_to_field("have1_apple", true),
        "have-1-apple"
    );
    assert_eq!(
        ScreamingSnakeCase.apply_to_field("have1apple", true),
        "HAVE_1_APPLE"
    );
}
