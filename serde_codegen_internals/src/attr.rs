use std::str::FromStr;
use Ctxt;
use syn;

// This module handles parsing of `#[serde(...)]` attributes. The entrypoints
// are `attr::Item::from_ast`, `attr::Variant::from_ast`, and
// `attr::Field::from_ast`. Each returns an instance of the corresponding
// struct. Note that none of them return a Result. Unrecognized, malformed, or
// duplicated attributes result in a span_err but otherwise are ignored. The
// user will see errors simultaneously for all bad attributes in the crate
// rather than just the first.

pub use case::RenameRule;

#[derive(Debug)]
pub struct Name {
    serialize: String,
    deserialize: String,
}

impl Name {
    /// Return the container name for the container when serializing.
    pub fn serialize_name(&self) -> String {
        self.serialize.clone()
    }

    /// Return the container name for the container when deserializing.
    pub fn deserialize_name(&self) -> String {
        self.deserialize.clone()
    }
}

/// Represents container (e.g. struct) attribute information
#[derive(Debug)]
pub struct Item {
    name: Name,
    deny_unknown_fields: bool,
    default: Default,
    rename_all: RenameRule,
    ser_bound: Option<Vec<syn::WherePredicate>>,
    de_bound: Option<Vec<syn::WherePredicate>>,
    tag: EnumTag,
}

/// Styles of representing an enum.
#[derive(Debug)]
pub enum EnumTag {
    /// The default.
    ///
    /// ```json
    /// {"variant1": {"key1": "value1", "key2": "value2"}}
    /// ```
    External,

    /// `#[serde(tag = "type")]`
    ///
    /// ```json
    /// {"type": "variant1", "key1": "value1", "key2": "value2"}
    /// ```
    Internal { tag: String },

    /// `#[serde(tag = "t", content = "c")]`
    ///
    /// ```json
    /// {"t": "variant1", "c": {"key1": "value1", "key2": "value2"}}
    /// ```
    Adjacent { tag: String, content: String },

    /// `#[serde(untagged)]`
    ///
    /// ```json
    /// {"key1": "value1", "key2": "value2"}
    /// ```
    None,
}

impl Item {
    /// Extract out the `#[serde(...)]` attributes from an item.
    pub fn from_ast(cx: &Ctxt, item: &syn::MacroInput) -> Self {
        #[derive(Debug, PromAttire, Default)]
        #[attire(scope = "serde")]
        pub struct ItemAttrs {
            #[attire(split_attribute_of = "rename", attribute = "serialize")]
            rename_serialize: Option<String>,
            #[attire(split_attribute_of = "rename", attribute = "deserialize")]
            rename_deserialize: Option<String>,
            deny_unknown_fields: bool,
            #[attire(default, flag_value = "Default::Default")]
            default: Default,
            rename_all: Option<RenameRuleInternal>,
            #[attire(split_attribute_of = "bound", attribute = "serialize")]
            bound_serialize: Option<WherePredicates>,
            #[attire(split_attribute_of = "bound", attribute = "deserialize")]
            bound_deserialize: Option<WherePredicates>,
            untagged: bool,
            tag: Option<String>,
            content: Option<String>,
        }

        let attrs = match ItemAttrs::try_from(item.attrs.as_slice()) {
            Ok(attrs) => attrs,
            Err(errs) => {
                for err in errs {
                    cx.error(err.to_string());
                }
                ItemAttrs::default()
            }
        };

        if let syn::Body::Enum(_) = item.body {
        } else {
            if attrs.untagged {
                cx.error("#[serde(untagged)] can only be used on enums");
            }
            if attrs.tag.is_some() {
                cx.error("#[serde(tag = \"...\")] can only be used on enums");
            }
            if attrs.content.is_some() {
                cx.error("#[serde(content = \"...\")] can only be used on enums");
            }
        }

        if let syn::Body::Struct(syn::VariantData::Struct(_)) = item.body {
        } else {
            if attrs.default != Default::None  {
                cx.error("#[serde(default)] can only be used on structs with named fields");
            }
        }

        let tag = match (attrs.untagged, attrs.tag, attrs.content) {
            (false, None, None) => EnumTag::External,
            (true, None, None) => EnumTag::None,
            (false, Some(tag), None) => {
                // Check that there are no tuple variants.
                if let syn::Body::Enum(ref variants) = item.body {
                    for variant in variants {
                        match variant.data {
                            syn::VariantData::Struct(_) |
                            syn::VariantData::Unit => {}
                            syn::VariantData::Tuple(ref fields) => {
                                if fields.len() != 1 {
                                    cx.error("#[serde(tag = \"...\")] cannot be used with tuple \
                                              variants");
                                    break;
                                }
                            }
                        }
                    }
                }
                EnumTag::Internal { tag: tag }
            }
            (true, Some(_), None) => {
                cx.error("enum cannot be both untagged and internally tagged");
                EnumTag::External // doesn't matter, will error
            }
            (false, None, Some(_)) => {
                cx.error("#[serde(tag = \"...\", content = \"...\")] must be used together");
                EnumTag::External
            }
            (true, None, Some(_)) => {
                cx.error("untagged enum cannot have #[serde(content = \"...\")]");
                EnumTag::External
            }
            (false, Some(tag), Some(content)) => {
                EnumTag::Adjacent {
                    tag: tag,
                    content: content,
                }
            }
            (true, Some(_), Some(_)) => {
                cx.error("untagged enum cannot have #[serde(tag = \"...\", content = \"...\")]");
                EnumTag::External
            }
        };

        Item {
            name: Name {
                serialize: attrs.rename_serialize.unwrap_or_else(|| item.ident.to_string()),
                deserialize: attrs.rename_deserialize.unwrap_or_else(|| item.ident.to_string()),
            },
            deny_unknown_fields: attrs.deny_unknown_fields,
            default: attrs.default,
            rename_all: attrs.rename_all.map(|a| a.0).unwrap_or(RenameRule::None),
            ser_bound: attrs.bound_serialize.map(|a| a.0),
            de_bound: attrs.bound_deserialize.map(|a| a.0),
            tag: tag,
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn rename_all(&self) -> &RenameRule {
        &self.rename_all
    }

    pub fn deny_unknown_fields(&self) -> bool {
        self.deny_unknown_fields
    }

    pub fn default(&self) -> &Default {
        &self.default
    }

    pub fn ser_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.ser_bound.as_ref().map(|vec| &vec[..])
    }

    pub fn de_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.de_bound.as_ref().map(|vec| &vec[..])
    }

    pub fn tag(&self) -> &EnumTag {
        &self.tag
    }
}

/// Represents variant attribute information
#[derive(Debug)]
pub struct Variant {
    name: Name,
    ser_renamed: bool,
    de_renamed: bool,
    rename_all: RenameRule,
    skip_deserializing: bool,
    skip_serializing: bool,
}

impl Variant {
    pub fn from_ast(cx: &Ctxt, variant: &syn::Variant) -> Self {
        #[derive(Debug, PromAttire, Default)]
        #[attire(scope = "serde")]
        pub struct VariantAttrs {
            #[attire(split_attribute_of = "rename", attribute = "serialize")]
            rename_serialize: Option<String>,
            #[attire(split_attribute_of = "rename", attribute = "deserialize")]
            rename_deserialize: Option<String>,
            rename_all: Option<RenameRuleInternal>,
            skip_deserializing: bool,
            skip_serializing: bool,
        }

        let attrs = match VariantAttrs::try_from(variant.attrs.as_slice()) {
            Ok(attrs) => attrs,
            Err(errs) => {
                for err in errs {
                    cx.error(err.to_string());
                }
                VariantAttrs::default()
            }
        };

        Variant {
            ser_renamed: attrs.rename_serialize.is_some(),
            de_renamed: attrs.rename_deserialize.is_some(),
            name: Name {
                serialize: attrs.rename_serialize.unwrap_or_else(|| variant.ident.to_string()),
                deserialize: attrs.rename_deserialize.unwrap_or_else(|| variant.ident.to_string()),
            },
            rename_all: attrs.rename_all.map(|a| a.0).unwrap_or(RenameRule::None),
            skip_deserializing: attrs.skip_deserializing,
            skip_serializing: attrs.skip_serializing,
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn rename_by_rule(&mut self, rule: &RenameRule) {
        if !self.ser_renamed {
            self.name.serialize = rule.apply_to_variant(&self.name.serialize);
        }
        if !self.de_renamed {
            self.name.deserialize = rule.apply_to_variant(&self.name.deserialize);
        }
    }

    pub fn rename_all(&self) -> &RenameRule {
        &self.rename_all
    }

    pub fn skip_deserializing(&self) -> bool {
        self.skip_deserializing
    }

    pub fn skip_serializing(&self) -> bool {
        self.skip_serializing
    }
}

/// Represents field attribute information
#[derive(Debug)]
pub struct Field {
    name: Name,
    ser_renamed: bool,
    de_renamed: bool,
    skip_serializing: bool,
    skip_deserializing: bool,
    skip_serializing_if: Option<syn::Path>,
    default: Default,
    serialize_with: Option<syn::Path>,
    deserialize_with: Option<syn::Path>,
    ser_bound: Option<Vec<syn::WherePredicate>>,
    de_bound: Option<Vec<syn::WherePredicate>>,
}

/// Represents the default to use for a field when deserializing.
#[derive(Debug, PartialEq)]
pub enum Default {
    /// Field must always be specified because it does not have a default.
    None,
    /// The default is given by `std::default::Default::default()`.
    Default,
    /// The default is given by this function.
    Path(syn::Path),
}

impl Field {
    /// Extract out the `#[serde(...)]` attributes from a struct field.
    pub fn from_ast(cx: &Ctxt,
                    index: usize,
                    field: &syn::Field) -> Self {

        #[derive(Debug, PromAttire, Default)]
        #[attire(scope = "serde")]
        pub struct FieldAttrs {
            #[attire(split_attribute_of = "rename", attribute = "serialize")]
            rename_serialize: Option<String>,
            #[attire(split_attribute_of = "rename", attribute = "deserialize")]
            rename_deserialize: Option<String>,
            skip_deserializing: bool,
            skip_serializing: bool,
            skip_serializing_if: Option<Path>,
            #[attire(default, flag_value = "Default::Default")]
            default: Default,
            with: Option<Path>,
            serialize_with: Option<Path>,
            deserialize_with: Option<Path>,
            #[attire(split_attribute_of = "bound", attribute = "serialize")]
            bound_serialize: Option<WherePredicates>,
            #[attire(split_attribute_of = "bound", attribute = "deserialize")]
            bound_deserialize: Option<WherePredicates>,
        }

        let mut attrs = match FieldAttrs::try_from(field.attrs.as_slice()) {
            Ok(attrs) => attrs,
            Err(errs) => {
                for err in errs {
                    cx.error(err.to_string());
                }
                FieldAttrs::default()
            }
        };

        let ident = match field.ident {
            Some(ref ident) => ident.to_string(),
            None => index.to_string(),
        };

        // If skip_deserializing, initialize the field to Default::default()
        // unless a different default is specified by `#[serde(default="...")]`
        if attrs.skip_deserializing {
            if attrs.default == Default::None {
                attrs.default = Default::Default;
            }
        }

        let with = attrs.with;
        Field {
            ser_renamed: attrs.rename_serialize.is_some(),
            de_renamed: attrs.rename_deserialize.is_some(),
            name: Name {
                serialize: attrs.rename_serialize.unwrap_or_else(|| ident.clone()),
                deserialize: attrs.rename_deserialize.unwrap_or(ident),
            },
            skip_serializing: attrs.skip_serializing,
            skip_deserializing: attrs.skip_deserializing,
            skip_serializing_if: attrs.skip_serializing_if.map(|a| a.0),
            default: attrs.default,
            serialize_with: attrs.serialize_with
                .map(|a| a.0)
                .or_else(|| with.as_ref().map(|path| {
                    let mut path = path.0.clone();
                    path.segments.push("serialize".into());
                    path
                })),
            deserialize_with: attrs.deserialize_with
                .map(|a| a.0)
                .or_else(|| with.map(|path| {
                    let mut path = path.0;
                    path.segments.push("deserialize".into());
                    path
                })),
            ser_bound: attrs.bound_serialize.map(|a| a.0),
            de_bound: attrs.bound_deserialize.map(|a| a.0),
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn rename_by_rule(&mut self, rule: &RenameRule) {
        if !self.ser_renamed {
            self.name.serialize = rule.apply_to_field(&self.name.serialize);
        }
        if !self.de_renamed {
            self.name.deserialize = rule.apply_to_field(&self.name.deserialize);
        }
    }

    pub fn skip_serializing(&self) -> bool {
        self.skip_serializing
    }

    pub fn skip_deserializing(&self) -> bool {
        self.skip_deserializing
    }

    pub fn skip_serializing_if(&self) -> Option<&syn::Path> {
        self.skip_serializing_if.as_ref()
    }

    pub fn default(&self) -> &Default {
        &self.default
    }

    pub fn serialize_with(&self) -> Option<&syn::Path> {
        self.serialize_with.as_ref()
    }

    pub fn deserialize_with(&self) -> Option<&syn::Path> {
        self.deserialize_with.as_ref()
    }

    pub fn ser_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.ser_bound.as_ref().map(|vec| &vec[..])
    }

    pub fn de_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.de_bound.as_ref().map(|vec| &vec[..])
    }
}

#[derive(Debug, Default, Clone)]
struct WherePredicates(Vec<syn::WherePredicate>);

impl FromStr for WherePredicates {
    type Err = SynError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if string.is_empty() {
            return Ok(WherePredicates(Vec::new()));
        }

        syn::parse_where_clause(&format!("where {}", string))
            .map(|wh| WherePredicates(wh.predicates))
            .map_err(SynError)
    }
}

#[derive(Debug)]
struct Path(syn::Path);

impl FromStr for Path {
    type Err = SynError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        syn::parse_path(string).map(Path).map_err(SynError)
    }
}

#[derive(Debug)]
pub struct SynError(String);

impl ::std::fmt::Display for SynError {
    fn fmt(&self, mut w: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(w, "{}", self.0)
    }
}

impl ::std::error::Error for SynError {
    fn description(&self) -> &str {
        &self.0
    }
}

#[derive(Debug)]
struct RenameRuleInternal(RenameRule);

#[derive(Debug)]
struct RenameRuleError(String);

impl FromStr for RenameRuleInternal {
    type Err = RenameRuleError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        RenameRule::from_str(string)
            .map(RenameRuleInternal)
            .map_err(|_| RenameRuleError(string.to_owned()))
    }
}

impl ::std::fmt::Display for RenameRuleError {
    fn fmt(&self, mut w: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(w, "unknown rename rule '{}'", self.0)
    }
}

impl ::std::error::Error for RenameRuleError {
    fn description(&self) -> &str {
        "unknown rename rule"
    }
}

impl ::std::default::Default for Default {
    fn default() -> Default {
        Default::None
    }
}

impl FromStr for Default {
    type Err = SynError;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        Ok(match string {
            "Default::Default" => Default::Default,
            _ => Default::Path(Path::from_str(string)?.0),
        })
    }
}
