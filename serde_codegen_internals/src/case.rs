use std::ascii::AsciiExt;
use std::str::FromStr;

use self::RenameRule::*;

#[derive(Debug, PartialEq)]
pub enum RenameRule {
    /// Don't apply a default rename rule.
    None,
    /// Rename direct children to "PascalCase" style, as typically used for enum variants.
    PascalCase,
    /// Rename direct children to "camelCase" style.
    CamelCase,
    /// Rename direct children to "snake_case" style, as commonly used for fields.
    SnakeCase,
    /// Rename direct children to "SCREAMING_SNAKE_CASE" style, as commonly used for constants.
    ScreamingSnakeCase,
    /// Rename direct children to "kebab-case" style.
    KebabCase,
}

impl RenameRule {
    pub fn apply_to_variant(&self, variant: &str) -> String {
        match *self {
            None | PascalCase => variant.to_owned(),
            CamelCase => variant[..1].to_ascii_lowercase() + &variant[1..],
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
        }
    }

    pub fn apply_to_field(&self, field: &str) -> String {
        match *self {
            None | SnakeCase => field.to_owned(),
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
                pascal[..1].to_ascii_lowercase() + &pascal[1..]
            }
            ScreamingSnakeCase => field.to_ascii_uppercase(),
            KebabCase => field.replace('_', "-"),
        }
    }
}

impl FromStr for RenameRule {
    type Err = ();

    fn from_str(rename_all_str: &str) -> Result<Self, Self::Err> {
        match rename_all_str {
            "PascalCase" => Ok(PascalCase),
            "camelCase" => Ok(CamelCase),
            "snake_case" => Ok(SnakeCase),
            "SCREAMING_SNAKE_CASE" => Ok(ScreamingSnakeCase),
            "kebab-case" => Ok(KebabCase),
            _ => Err(()),
        }
    }
}

#[test]
fn rename_variants() {
    for &(original, camel, snake, screaming, kebab) in
        &[("Outcome", "outcome", "outcome", "OUTCOME", "outcome"),
          ("VeryTasty", "veryTasty", "very_tasty", "VERY_TASTY", "very-tasty"),
          ("A", "a", "a", "A", "a"),
          ("Z42", "z42", "z42", "Z42", "z42")] {
        assert_eq!(None.apply_to_variant(original), original);
        assert_eq!(PascalCase.apply_to_variant(original), original);
        assert_eq!(CamelCase.apply_to_variant(original), camel);
        assert_eq!(SnakeCase.apply_to_variant(original), snake);
        assert_eq!(ScreamingSnakeCase.apply_to_variant(original), screaming);
        assert_eq!(KebabCase.apply_to_variant(original), kebab);
    }
}

#[test]
fn rename_fields() {
    for &(original, pascal, camel, screaming, kebab) in
        &[("outcome", "Outcome", "outcome", "OUTCOME", "outcome"),
          ("very_tasty", "VeryTasty", "veryTasty", "VERY_TASTY", "very-tasty"),
          ("a", "A", "a", "A", "a"),
          ("z42", "Z42", "z42", "Z42", "z42")] {
        assert_eq!(None.apply_to_field(original), original);
        assert_eq!(PascalCase.apply_to_field(original), pascal);
        assert_eq!(CamelCase.apply_to_field(original), camel);
        assert_eq!(SnakeCase.apply_to_field(original), original);
        assert_eq!(ScreamingSnakeCase.apply_to_field(original), screaming);
        assert_eq!(KebabCase.apply_to_field(original), kebab);
    }
}
