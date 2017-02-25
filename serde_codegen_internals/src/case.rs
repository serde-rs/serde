use std::str::FromStr;

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
    pub fn apply_to_variant(&self, variant_name: String) -> Result<String, String> {
        if *self == RenameRule::None {
            return Ok(variant_name);
        }
        let mut chars = variant_name.chars();
        let start = chars.next().unwrap();
        if start.is_lowercase() {
            return Err(format!("#[serde(rename_all = \"...\")] expects enum variants to be \
                                named in `PascalCase`."));
        }
        Ok(self.apply_to_words(chars.fold(vec![start.to_lowercase().collect()], |mut words, c| {
            if c.is_uppercase() {
                words.push(c.to_lowercase().collect());
            } else {
                words.last_mut().unwrap().push(c);
            }
            words
        })))
    }

    pub fn apply_to_field(&self, field_name: String) -> Result<String, String> {
        if *self == RenameRule::None {
            return Ok(field_name);
        }
        if field_name != field_name.to_lowercase() {
            return Err(format!("#[serde(rename_all = \"...\")] expects fields to be named in \
                                `snake_case`."));
        }
        Ok(self.apply_to_words(field_name.split('_').map(|s| s.to_string()).collect()))
    }

    fn apply_to_words(&self, lowercased_words: Vec<String>) -> String {
        match *self {
                RenameRule::PascalCase => Self::capitalising_join(lowercased_words),
                RenameRule::CamelCase => {
                    let mut iter = lowercased_words.into_iter();
                    let mut first = iter.next().unwrap();
                    first.push_str(Self::capitalising_join(iter.collect()).as_str());
                    first
                }
                RenameRule::SnakeCase => Self::delimiting_join(lowercased_words, "_"),
                RenameRule::ScreamingSnakeCase => Self::delimiting_join(lowercased_words, "_").to_uppercase(),
                RenameRule::KebabCase => Self::delimiting_join(lowercased_words, "-"),
                _ => unreachable!(),
            }
            .clone()
    }

    fn delimiting_join(lowercased_words: Vec<String>, delimiter: &str) -> String {
        lowercased_words.join(delimiter)
    }

    fn capitalising_join(lowercased_words: Vec<String>) -> String {
        lowercased_words.into_iter().map(Self::capitalise).collect()
    }

    fn capitalise(lowercased_word: String) -> String {
        let mut iter = lowercased_word.chars();
        let mut first: String = iter.next().unwrap().to_uppercase().collect();
        first.push_str(iter.collect::<String>().as_str());
        first
    }
}

impl FromStr for RenameRule {
    type Err = String;

    fn from_str(rename_all_str: &str) -> Result<Self, Self::Err> {
        match rename_all_str {
            "PascalCase" => Ok(RenameRule::PascalCase),
            "camelCase" => Ok(RenameRule::CamelCase),
            "snake_case" => Ok(RenameRule::SnakeCase),
            "SCREAMING_SNAKE_CASE" => Ok(RenameRule::ScreamingSnakeCase),
            "kebab-case" => Ok(RenameRule::KebabCase),
            other => Err(other.to_string()),
        }
    }
}
