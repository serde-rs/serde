use internals::Ctxt;
use proc_macro2::{Group, Span, TokenStream, TokenTree};
use quote::ToTokens;
use std::collections::BTreeSet;
use std::str::FromStr;
use syn;
use syn::parse::{self, Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::Ident;
use syn::Meta::{List, NameValue, Word};
use syn::NestedMeta::{Literal, Meta};

// This module handles parsing of `#[serde(...)]` attributes. The entrypoints
// are `attr::Container::from_ast`, `attr::Variant::from_ast`, and
// `attr::Field::from_ast`. Each returns an instance of the corresponding
// struct. Note that none of them return a Result. Unrecognized, malformed, or
// duplicated attributes result in a span_err but otherwise are ignored. The
// user will see errors simultaneously for all bad attributes in the crate
// rather than just the first.

pub use internals::case::RenameRule;

struct Attr<'c, T> {
    cx: &'c Ctxt,
    name: &'static str,
    tokens: TokenStream,
    value: Option<T>,
}

impl<'c, T> Attr<'c, T> {
    fn none(cx: &'c Ctxt, name: &'static str) -> Self {
        Attr {
            cx: cx,
            name: name,
            tokens: TokenStream::new(),
            value: None,
        }
    }

    fn set<A: ToTokens>(&mut self, obj: A, value: T) {
        let tokens = obj.into_token_stream();

        if self.value.is_some() {
            self.cx
                .error_spanned_by(tokens, format!("duplicate serde attribute `{}`", self.name));
        } else {
            self.tokens = tokens;
            self.value = Some(value);
        }
    }

    fn set_opt<A: ToTokens>(&mut self, obj: A, value: Option<T>) {
        if let Some(value) = value {
            self.set(obj, value);
        }
    }

    fn set_if_none(&mut self, value: T) {
        if self.value.is_none() {
            self.value = Some(value);
        }
    }

    fn get(self) -> Option<T> {
        self.value
    }

    fn get_with_tokens(self) -> Option<(TokenStream, T)> {
        match self.value {
            Some(v) => Some((self.tokens, v)),
            None => None,
        }
    }
}

struct BoolAttr<'c>(Attr<'c, ()>);

impl<'c> BoolAttr<'c> {
    fn none(cx: &'c Ctxt, name: &'static str) -> Self {
        BoolAttr(Attr::none(cx, name))
    }

    fn set_true<A: ToTokens>(&mut self, obj: A) {
        self.0.set(obj, ());
    }

    fn get(&self) -> bool {
        self.0.value.is_some()
    }
}

struct VecAttr<'c, T> {
    cx: &'c Ctxt,
    name: &'static str,
    first_dup_tokens: TokenStream,
    values: Vec<T>,
}

impl<'c, T> VecAttr<'c, T> {
    fn none(cx: &'c Ctxt, name: &'static str) -> Self {
        VecAttr {
            cx: cx,
            name: name,
            first_dup_tokens: TokenStream::new(),
            values: Vec::new(),
        }
    }

    fn insert<A: ToTokens>(&mut self, obj: A, value: T) {
        if self.values.len() == 1 {
            self.first_dup_tokens = obj.into_token_stream();
        }
        self.values.push(value);
    }

    fn at_most_one(mut self) -> Result<Option<T>, ()> {
        if self.values.len() > 1 {
            let dup_token = self.first_dup_tokens;
            self.cx.error_spanned_by(
                dup_token,
                format!("duplicate serde attribute `{}`", self.name),
            );
            Err(())
        } else {
            Ok(self.values.pop())
        }
    }

    fn get(self) -> Vec<T> {
        self.values
    }
}

pub struct Name {
    serialize: String,
    serialize_renamed: bool,
    deserialize: String,
    deserialize_renamed: bool,
    deserialize_aliases: Vec<String>,
}

#[allow(deprecated)]
fn unraw(ident: &Ident) -> String {
    // str::trim_start_matches was added in 1.30, trim_left_matches deprecated
    // in 1.33. We currently support rustc back to 1.15 so we need to continue
    // to use the deprecated one.
    ident.to_string().trim_left_matches("r#").to_owned()
}

impl Name {
    fn from_attrs(
        source_name: String,
        ser_name: Attr<String>,
        de_name: Attr<String>,
        de_aliases: Option<VecAttr<String>>,
    ) -> Name {
        let deserialize_aliases = match de_aliases {
            Some(de_aliases) => {
                let mut alias_list = BTreeSet::new();
                for alias_name in de_aliases.get() {
                    alias_list.insert(alias_name);
                }
                alias_list.into_iter().collect()
            }
            None => Vec::new(),
        };

        let ser_name = ser_name.get();
        let ser_renamed = ser_name.is_some();
        let de_name = de_name.get();
        let de_renamed = de_name.is_some();
        Name {
            serialize: ser_name.unwrap_or_else(|| source_name.clone()),
            serialize_renamed: ser_renamed,
            deserialize: de_name.unwrap_or(source_name),
            deserialize_renamed: de_renamed,
            deserialize_aliases: deserialize_aliases,
        }
    }

    /// Return the container name for the container when serializing.
    pub fn serialize_name(&self) -> String {
        self.serialize.clone()
    }

    /// Return the container name for the container when deserializing.
    pub fn deserialize_name(&self) -> String {
        self.deserialize.clone()
    }

    fn deserialize_aliases(&self) -> Vec<String> {
        let mut aliases = self.deserialize_aliases.clone();
        let main_name = self.deserialize_name();
        if !aliases.contains(&main_name) {
            aliases.push(main_name);
        }
        aliases
    }
}

pub struct RenameAllRules {
    serialize: RenameRule,
    deserialize: RenameRule,
}

/// Represents struct or enum attribute information.
pub struct Container {
    name: Name,
    transparent: bool,
    deny_unknown_fields: bool,
    default: Default,
    rename_all_rules: RenameAllRules,
    ser_bound: Option<Vec<syn::WherePredicate>>,
    de_bound: Option<Vec<syn::WherePredicate>>,
    tag: TagType,
    type_from: Option<syn::Type>,
    type_into: Option<syn::Type>,
    remote: Option<syn::Path>,
    identifier: Identifier,
    has_flatten: bool,
}

/// Styles of representing an enum.
pub enum TagType {
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

/// Whether this enum represents the fields of a struct or the variants of an
/// enum.
#[derive(Copy, Clone)]
pub enum Identifier {
    /// It does not.
    No,

    /// This enum represents the fields of a struct. All of the variants must be
    /// unit variants, except possibly one which is annotated with
    /// `#[serde(other)]` and is a newtype variant.
    Field,

    /// This enum represents the variants of an enum. All of the variants must
    /// be unit variants.
    Variant,
}

impl Identifier {
    #[cfg(feature = "deserialize_in_place")]
    pub fn is_some(self) -> bool {
        match self {
            Identifier::No => false,
            Identifier::Field | Identifier::Variant => true,
        }
    }
}

impl Container {
    /// Extract out the `#[serde(...)]` attributes from an item.
    pub fn from_ast(cx: &Ctxt, item: &syn::DeriveInput) -> Self {
        let mut ser_name = Attr::none(cx, "rename");
        let mut de_name = Attr::none(cx, "rename");
        let mut transparent = BoolAttr::none(cx, "transparent");
        let mut deny_unknown_fields = BoolAttr::none(cx, "deny_unknown_fields");
        let mut default = Attr::none(cx, "default");
        let mut rename_all_ser_rule = Attr::none(cx, "rename_all");
        let mut rename_all_de_rule = Attr::none(cx, "rename_all");
        let mut ser_bound = Attr::none(cx, "bound");
        let mut de_bound = Attr::none(cx, "bound");
        let mut untagged = BoolAttr::none(cx, "untagged");
        let mut internal_tag = Attr::none(cx, "tag");
        let mut content = Attr::none(cx, "content");
        let mut type_from = Attr::none(cx, "from");
        let mut type_into = Attr::none(cx, "into");
        let mut remote = Attr::none(cx, "remote");
        let mut field_identifier = BoolAttr::none(cx, "field_identifier");
        let mut variant_identifier = BoolAttr::none(cx, "variant_identifier");

        for meta_items in item.attrs.iter().filter_map(get_serde_meta_items) {
            for meta_item in meta_items {
                match meta_item {
                    // Parse `#[serde(rename = "foo")]`
                    Meta(NameValue(ref m)) if m.ident == "rename" => {
                        if let Ok(s) = get_lit_str(cx, &m.ident, &m.ident, &m.lit) {
                            ser_name.set(&m.ident, s.value());
                            de_name.set(&m.ident, s.value());
                        }
                    }

                    // Parse `#[serde(rename(serialize = "foo", deserialize = "bar"))]`
                    Meta(List(ref m)) if m.ident == "rename" => {
                        if let Ok((ser, de)) = get_renames(cx, &m.nested) {
                            ser_name.set_opt(&m.ident, ser.map(syn::LitStr::value));
                            de_name.set_opt(&m.ident, de.map(syn::LitStr::value));
                        }
                    }

                    // Parse `#[serde(rename_all = "foo")]`
                    Meta(NameValue(ref m)) if m.ident == "rename_all" => {
                        if let Ok(s) = get_lit_str(cx, &m.ident, &m.ident, &m.lit) {
                            match RenameRule::from_str(&s.value()) {
                                Ok(rename_rule) => {
                                    rename_all_ser_rule.set(&m.ident, rename_rule);
                                    rename_all_de_rule.set(&m.ident, rename_rule);
                                }
                                Err(()) => cx.error_spanned_by(
                                    s,
                                    format!(
                                        "unknown rename rule for #[serde(rename_all \
                                         = {:?})]",
                                        s.value(),
                                    ),
                                ),
                            }
                        }
                    }

                    // Parse `#[serde(rename_all(serialize = "foo", deserialize = "bar"))]`
                    Meta(List(ref m)) if m.ident == "rename_all" => {
                        if let Ok((ser, de)) = get_renames(cx, &m.nested) {
                            if let Some(ser) = ser {
                                match RenameRule::from_str(&ser.value()) {
                                    Ok(rename_rule) => {
                                        rename_all_ser_rule.set(&m.ident, rename_rule)
                                    }
                                    Err(()) => cx.error_spanned_by(
                                        ser,
                                        format!(
                                            "unknown rename rule for #[serde(rename_all \
                                             = {:?})]",
                                            ser.value(),
                                        ),
                                    ),
                                }
                            }
                            if let Some(de) = de {
                                match RenameRule::from_str(&de.value()) {
                                    Ok(rename_rule) => {
                                        rename_all_de_rule.set(&m.ident, rename_rule)
                                    }
                                    Err(()) => cx.error_spanned_by(
                                        de,
                                        format!(
                                            "unknown rename rule for #[serde(rename_all \
                                             = {:?})]",
                                            de.value(),
                                        ),
                                    ),
                                }
                            }
                        }
                    }

                    // Parse `#[serde(transparent)]`
                    Meta(Word(ref word)) if word == "transparent" => {
                        transparent.set_true(word);
                    }

                    // Parse `#[serde(deny_unknown_fields)]`
                    Meta(Word(ref word)) if word == "deny_unknown_fields" => {
                        deny_unknown_fields.set_true(word);
                    }

                    // Parse `#[serde(default)]`
                    Meta(Word(ref word)) if word == "default" => match item.data {
                        syn::Data::Struct(syn::DataStruct { ref fields, .. }) => match *fields {
                            syn::Fields::Named(_) => {
                                default.set(word, Default::Default);
                            }
                            syn::Fields::Unnamed(_) | syn::Fields::Unit => cx.error_spanned_by(
                                fields,
                                "#[serde(default)] can only be used on structs \
                                 with named fields",
                            ),
                        },
                        syn::Data::Enum(syn::DataEnum { ref enum_token, .. }) => cx
                            .error_spanned_by(
                                enum_token,
                                "#[serde(default)] can only be used on structs \
                                 with named fields",
                            ),
                        syn::Data::Union(syn::DataUnion {
                            ref union_token, ..
                        }) => cx.error_spanned_by(
                            union_token,
                            "#[serde(default)] can only be used on structs \
                             with named fields",
                        ),
                    },

                    // Parse `#[serde(default = "...")]`
                    Meta(NameValue(ref m)) if m.ident == "default" => {
                        if let Ok(path) = parse_lit_into_expr_path(cx, &m.ident, &m.lit) {
                            match item.data {
                                syn::Data::Struct(syn::DataStruct { ref fields, .. }) => {
                                    match *fields {
                                        syn::Fields::Named(_) => {
                                            default.set(&m.ident, Default::Path(path));
                                        }
                                        syn::Fields::Unnamed(_) | syn::Fields::Unit => cx
                                            .error_spanned_by(
                                                fields,
                                                "#[serde(default = \"...\")] can only be used \
                                                 on structs with named fields",
                                            ),
                                    }
                                }
                                syn::Data::Enum(syn::DataEnum { ref enum_token, .. }) => cx
                                    .error_spanned_by(
                                        enum_token,
                                        "#[serde(default = \"...\")] can only be used \
                                         on structs with named fields",
                                    ),
                                syn::Data::Union(syn::DataUnion {
                                    ref union_token, ..
                                }) => cx.error_spanned_by(
                                    union_token,
                                    "#[serde(default = \"...\")] can only be used \
                                     on structs with named fields",
                                ),
                            }
                        }
                    }

                    // Parse `#[serde(bound = "T: SomeBound")]`
                    Meta(NameValue(ref m)) if m.ident == "bound" => {
                        if let Ok(where_predicates) =
                            parse_lit_into_where(cx, &m.ident, &m.ident, &m.lit)
                        {
                            ser_bound.set(&m.ident, where_predicates.clone());
                            de_bound.set(&m.ident, where_predicates);
                        }
                    }

                    // Parse `#[serde(bound(serialize = "...", deserialize = "..."))]`
                    Meta(List(ref m)) if m.ident == "bound" => {
                        if let Ok((ser, de)) = get_where_predicates(cx, &m.nested) {
                            ser_bound.set_opt(&m.ident, ser);
                            de_bound.set_opt(&m.ident, de);
                        }
                    }

                    // Parse `#[serde(untagged)]`
                    Meta(Word(ref word)) if word == "untagged" => match item.data {
                        syn::Data::Enum(_) => {
                            untagged.set_true(word);
                        }
                        syn::Data::Struct(syn::DataStruct {
                            ref struct_token, ..
                        }) => {
                            cx.error_spanned_by(
                                struct_token,
                                "#[serde(untagged)] can only be used on enums",
                            );
                        }
                        syn::Data::Union(syn::DataUnion {
                            ref union_token, ..
                        }) => {
                            cx.error_spanned_by(
                                union_token,
                                "#[serde(untagged)] can only be used on enums",
                            );
                        }
                    },

                    // Parse `#[serde(tag = "type")]`
                    Meta(NameValue(ref m)) if m.ident == "tag" => {
                        if let Ok(s) = get_lit_str(cx, &m.ident, &m.ident, &m.lit) {
                            match item.data {
                                syn::Data::Enum(_) => {
                                    internal_tag.set(&m.ident, s.value());
                                }
                                syn::Data::Struct(syn::DataStruct { ref fields, .. }) => {
                                    match *fields {
                                        syn::Fields::Named(_) => {
                                            internal_tag.set(&m.ident, s.value());
                                        }
                                        syn::Fields::Unnamed(_) | syn::Fields::Unit => {
                                            cx.error_spanned_by(
                                                fields,
                                                "#[serde(tag = \"...\")] can only be used on enums \
                                                and structs with named fields",
                                            );
                                        }
                                    }
                                }
                                syn::Data::Union(syn::DataUnion {
                                    ref union_token, ..
                                }) => {
                                    cx.error_spanned_by(
                                        union_token,
                                        "#[serde(tag = \"...\")] can only be used on enums \
                                         and structs with named fields",
                                    );
                                }
                            }
                        }
                    }

                    // Parse `#[serde(content = "c")]`
                    Meta(NameValue(ref m)) if m.ident == "content" => {
                        if let Ok(s) = get_lit_str(cx, &m.ident, &m.ident, &m.lit) {
                            match item.data {
                                syn::Data::Enum(_) => {
                                    content.set(&m.ident, s.value());
                                }
                                syn::Data::Struct(syn::DataStruct {
                                    ref struct_token, ..
                                }) => {
                                    cx.error_spanned_by(
                                        struct_token,
                                        "#[serde(content = \"...\")] can only be used on enums",
                                    );
                                }
                                syn::Data::Union(syn::DataUnion {
                                    ref union_token, ..
                                }) => {
                                    cx.error_spanned_by(
                                        union_token,
                                        "#[serde(content = \"...\")] can only be used on enums",
                                    );
                                }
                            }
                        }
                    }

                    // Parse `#[serde(from = "Type")]
                    Meta(NameValue(ref m)) if m.ident == "from" => {
                        if let Ok(from_ty) = parse_lit_into_ty(cx, &m.ident, &m.lit) {
                            type_from.set_opt(&m.ident, Some(from_ty));
                        }
                    }

                    // Parse `#[serde(into = "Type")]
                    Meta(NameValue(ref m)) if m.ident == "into" => {
                        if let Ok(into_ty) = parse_lit_into_ty(cx, &m.ident, &m.lit) {
                            type_into.set_opt(&m.ident, Some(into_ty));
                        }
                    }

                    // Parse `#[serde(remote = "...")]`
                    Meta(NameValue(ref m)) if m.ident == "remote" => {
                        if let Ok(path) = parse_lit_into_path(cx, &m.ident, &m.lit) {
                            if is_primitive_path(&path, "Self") {
                                remote.set(&m.ident, item.ident.clone().into());
                            } else {
                                remote.set(&m.ident, path);
                            }
                        }
                    }

                    // Parse `#[serde(field_identifier)]`
                    Meta(Word(ref word)) if word == "field_identifier" => {
                        field_identifier.set_true(word);
                    }

                    // Parse `#[serde(variant_identifier)]`
                    Meta(Word(ref word)) if word == "variant_identifier" => {
                        variant_identifier.set_true(word);
                    }

                    Meta(ref meta_item) => {
                        cx.error_spanned_by(
                            meta_item.name(),
                            format!("unknown serde container attribute `{}`", meta_item.name()),
                        );
                    }

                    Literal(ref lit) => {
                        cx.error_spanned_by(lit, "unexpected literal in serde container attribute");
                    }
                }
            }
        }

        Container {
            name: Name::from_attrs(unraw(&item.ident), ser_name, de_name, None),
            transparent: transparent.get(),
            deny_unknown_fields: deny_unknown_fields.get(),
            default: default.get().unwrap_or(Default::None),
            rename_all_rules: RenameAllRules {
                serialize: rename_all_ser_rule.get().unwrap_or(RenameRule::None),
                deserialize: rename_all_de_rule.get().unwrap_or(RenameRule::None),
            },
            ser_bound: ser_bound.get(),
            de_bound: de_bound.get(),
            tag: decide_tag(cx, item, untagged, internal_tag, content),
            type_from: type_from.get(),
            type_into: type_into.get(),
            remote: remote.get(),
            identifier: decide_identifier(cx, item, field_identifier, variant_identifier),
            has_flatten: false,
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn rename_all_rules(&self) -> &RenameAllRules {
        &self.rename_all_rules
    }

    pub fn transparent(&self) -> bool {
        self.transparent
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

    pub fn tag(&self) -> &TagType {
        &self.tag
    }

    pub fn type_from(&self) -> Option<&syn::Type> {
        self.type_from.as_ref()
    }

    pub fn type_into(&self) -> Option<&syn::Type> {
        self.type_into.as_ref()
    }

    pub fn remote(&self) -> Option<&syn::Path> {
        self.remote.as_ref()
    }

    pub fn identifier(&self) -> Identifier {
        self.identifier
    }

    pub fn has_flatten(&self) -> bool {
        self.has_flatten
    }

    pub fn mark_has_flatten(&mut self) {
        self.has_flatten = true;
    }
}

fn decide_tag(
    cx: &Ctxt,
    item: &syn::DeriveInput,
    untagged: BoolAttr,
    internal_tag: Attr<String>,
    content: Attr<String>,
) -> TagType {
    match (
        untagged.0.get_with_tokens(),
        internal_tag.get_with_tokens(),
        content.get_with_tokens(),
    ) {
        (None, None, None) => TagType::External,
        (Some(_), None, None) => TagType::None,
        (None, Some((_, tag)), None) => {
            // Check that there are no tuple variants.
            if let syn::Data::Enum(ref data) = item.data {
                for variant in &data.variants {
                    match variant.fields {
                        syn::Fields::Named(_) | syn::Fields::Unit => {}
                        syn::Fields::Unnamed(ref fields) => {
                            if fields.unnamed.len() != 1 {
                                cx.error_spanned_by(
                                    variant,
                                    "#[serde(tag = \"...\")] cannot be used with tuple \
                                     variants",
                                );
                                break;
                            }
                        }
                    }
                }
            }
            TagType::Internal { tag: tag }
        }
        (Some((untagged_tokens, _)), Some((tag_tokens, _)), None) => {
            cx.error_spanned_by(
                untagged_tokens,
                "enum cannot be both untagged and internally tagged",
            );
            cx.error_spanned_by(
                tag_tokens,
                "enum cannot be both untagged and internally tagged",
            );
            TagType::External // doesn't matter, will error
        }
        (None, None, Some((content_tokens, _))) => {
            cx.error_spanned_by(
                content_tokens,
                "#[serde(tag = \"...\", content = \"...\")] must be used together",
            );
            TagType::External
        }
        (Some((untagged_tokens, _)), None, Some((content_tokens, _))) => {
            cx.error_spanned_by(
                untagged_tokens,
                "untagged enum cannot have #[serde(content = \"...\")]",
            );
            cx.error_spanned_by(
                content_tokens,
                "untagged enum cannot have #[serde(content = \"...\")]",
            );
            TagType::External
        }
        (None, Some((_, tag)), Some((_, content))) => TagType::Adjacent {
            tag: tag,
            content: content,
        },
        (Some((untagged_tokens, _)), Some((tag_tokens, _)), Some((content_tokens, _))) => {
            cx.error_spanned_by(
                untagged_tokens,
                "untagged enum cannot have #[serde(tag = \"...\", content = \"...\")]",
            );
            cx.error_spanned_by(
                tag_tokens,
                "untagged enum cannot have #[serde(tag = \"...\", content = \"...\")]",
            );
            cx.error_spanned_by(
                content_tokens,
                "untagged enum cannot have #[serde(tag = \"...\", content = \"...\")]",
            );
            TagType::External
        }
    }
}

fn decide_identifier(
    cx: &Ctxt,
    item: &syn::DeriveInput,
    field_identifier: BoolAttr,
    variant_identifier: BoolAttr,
) -> Identifier {
    match (
        &item.data,
        field_identifier.0.get_with_tokens(),
        variant_identifier.0.get_with_tokens(),
    ) {
        (_, None, None) => Identifier::No,
        (_, Some((field_identifier_tokens, _)), Some((variant_identifier_tokens, _))) => {
            cx.error_spanned_by(
                field_identifier_tokens,
                "#[serde(field_identifier)] and #[serde(variant_identifier)] cannot both be set",
            );
            cx.error_spanned_by(
                variant_identifier_tokens,
                "#[serde(field_identifier)] and #[serde(variant_identifier)] cannot both be set",
            );
            Identifier::No
        }
        (&syn::Data::Enum(_), Some(_), None) => Identifier::Field,
        (&syn::Data::Enum(_), None, Some(_)) => Identifier::Variant,
        (
            &syn::Data::Struct(syn::DataStruct {
                ref struct_token, ..
            }),
            Some(_),
            None,
        ) => {
            cx.error_spanned_by(
                struct_token,
                "#[serde(field_identifier)] can only be used on an enum",
            );
            Identifier::No
        }
        (
            &syn::Data::Union(syn::DataUnion {
                ref union_token, ..
            }),
            Some(_),
            None,
        ) => {
            cx.error_spanned_by(
                union_token,
                "#[serde(field_identifier)] can only be used on an enum",
            );
            Identifier::No
        }
        (
            &syn::Data::Struct(syn::DataStruct {
                ref struct_token, ..
            }),
            None,
            Some(_),
        ) => {
            cx.error_spanned_by(
                struct_token,
                "#[serde(variant_identifier)] can only be used on an enum",
            );
            Identifier::No
        }
        (
            &syn::Data::Union(syn::DataUnion {
                ref union_token, ..
            }),
            None,
            Some(_),
        ) => {
            cx.error_spanned_by(
                union_token,
                "#[serde(variant_identifier)] can only be used on an enum",
            );
            Identifier::No
        }
    }
}

/// Represents variant attribute information
pub struct Variant {
    name: Name,
    rename_all_rules: RenameAllRules,
    ser_bound: Option<Vec<syn::WherePredicate>>,
    de_bound: Option<Vec<syn::WherePredicate>>,
    skip_deserializing: bool,
    skip_serializing: bool,
    other: bool,
    serialize_with: Option<syn::ExprPath>,
    deserialize_with: Option<syn::ExprPath>,
    borrow: Option<syn::Meta>,
}

impl Variant {
    pub fn from_ast(cx: &Ctxt, variant: &syn::Variant) -> Self {
        let mut ser_name = Attr::none(cx, "rename");
        let mut de_name = Attr::none(cx, "rename");
        let mut de_aliases = VecAttr::none(cx, "rename");
        let mut skip_deserializing = BoolAttr::none(cx, "skip_deserializing");
        let mut skip_serializing = BoolAttr::none(cx, "skip_serializing");
        let mut rename_all_ser_rule = Attr::none(cx, "rename_all");
        let mut rename_all_de_rule = Attr::none(cx, "rename_all");
        let mut ser_bound = Attr::none(cx, "bound");
        let mut de_bound = Attr::none(cx, "bound");
        let mut other = BoolAttr::none(cx, "other");
        let mut serialize_with = Attr::none(cx, "serialize_with");
        let mut deserialize_with = Attr::none(cx, "deserialize_with");
        let mut borrow = Attr::none(cx, "borrow");

        for meta_items in variant.attrs.iter().filter_map(get_serde_meta_items) {
            for meta_item in meta_items {
                match meta_item {
                    // Parse `#[serde(rename = "foo")]`
                    Meta(NameValue(ref m)) if m.ident == "rename" => {
                        if let Ok(s) = get_lit_str(cx, &m.ident, &m.ident, &m.lit) {
                            ser_name.set(&m.ident, s.value());
                            de_name.set_if_none(s.value());
                            de_aliases.insert(&m.ident, s.value());
                        }
                    }

                    // Parse `#[serde(rename(serialize = "foo", deserialize = "bar"))]`
                    Meta(List(ref m)) if m.ident == "rename" => {
                        if let Ok((ser, de)) = get_multiple_renames(cx, &m.nested) {
                            ser_name.set_opt(&m.ident, ser.map(syn::LitStr::value));
                            for de_value in de {
                                de_name.set_if_none(de_value.value());
                                de_aliases.insert(&m.ident, de_value.value());
                            }
                        }
                    }

                    // Parse `#[serde(alias = "foo")]`
                    Meta(NameValue(ref m)) if m.ident == "alias" => {
                        if let Ok(s) = get_lit_str(cx, &m.ident, &m.ident, &m.lit) {
                            de_aliases.insert(&m.ident, s.value());
                        }
                    }

                    // Parse `#[serde(rename_all = "foo")]`
                    Meta(NameValue(ref m)) if m.ident == "rename_all" => {
                        if let Ok(s) = get_lit_str(cx, &m.ident, &m.ident, &m.lit) {
                            match RenameRule::from_str(&s.value()) {
                                Ok(rename_rule) => {
                                    rename_all_ser_rule.set(&m.ident, rename_rule);
                                    rename_all_de_rule.set(&m.ident, rename_rule);
                                }
                                Err(()) => cx.error_spanned_by(
                                    s,
                                    format!(
                                        "unknown rename rule for #[serde(rename_all \
                                         = {:?})]",
                                        s.value()
                                    ),
                                ),
                            }
                        }
                    }

                    // Parse `#[serde(rename_all(serialize = "foo", deserialize = "bar"))]`
                    Meta(List(ref m)) if m.ident == "rename_all" => {
                        if let Ok((ser, de)) = get_renames(cx, &m.nested) {
                            if let Some(ser) = ser {
                                match RenameRule::from_str(&ser.value()) {
                                    Ok(rename_rule) => {
                                        rename_all_ser_rule.set(&m.ident, rename_rule)
                                    }
                                    Err(()) => cx.error_spanned_by(
                                        ser,
                                        format!(
                                            "unknown rename rule for #[serde(rename_all \
                                             = {:?})]",
                                            ser.value(),
                                        ),
                                    ),
                                }
                            }
                            if let Some(de) = de {
                                match RenameRule::from_str(&de.value()) {
                                    Ok(rename_rule) => {
                                        rename_all_de_rule.set(&m.ident, rename_rule)
                                    }
                                    Err(()) => cx.error_spanned_by(
                                        de,
                                        format!(
                                            "unknown rename rule for #[serde(rename_all \
                                             = {:?})]",
                                            de.value(),
                                        ),
                                    ),
                                }
                            }
                        }
                    }

                    // Parse `#[serde(skip)]`
                    Meta(Word(ref word)) if word == "skip" => {
                        skip_serializing.set_true(word);
                        skip_deserializing.set_true(word);
                    }

                    // Parse `#[serde(skip_deserializing)]`
                    Meta(Word(ref word)) if word == "skip_deserializing" => {
                        skip_deserializing.set_true(word);
                    }

                    // Parse `#[serde(skip_serializing)]`
                    Meta(Word(ref word)) if word == "skip_serializing" => {
                        skip_serializing.set_true(word);
                    }

                    // Parse `#[serde(other)]`
                    Meta(Word(ref word)) if word == "other" => {
                        other.set_true(word);
                    }

                    // Parse `#[serde(bound = "T: SomeBound")]`
                    Meta(NameValue(ref m)) if m.ident == "bound" => {
                        if let Ok(where_predicates) =
                            parse_lit_into_where(cx, &m.ident, &m.ident, &m.lit)
                        {
                            ser_bound.set(&m.ident, where_predicates.clone());
                            de_bound.set(&m.ident, where_predicates);
                        }
                    }

                    // Parse `#[serde(bound(serialize = "...", deserialize = "..."))]`
                    Meta(List(ref m)) if m.ident == "bound" => {
                        if let Ok((ser, de)) = get_where_predicates(cx, &m.nested) {
                            ser_bound.set_opt(&m.ident, ser);
                            de_bound.set_opt(&m.ident, de);
                        }
                    }

                    // Parse `#[serde(with = "...")]`
                    Meta(NameValue(ref m)) if m.ident == "with" => {
                        if let Ok(path) = parse_lit_into_expr_path(cx, &m.ident, &m.lit) {
                            let mut ser_path = path.clone();
                            ser_path
                                .path
                                .segments
                                .push(Ident::new("serialize", Span::call_site()).into());
                            serialize_with.set(&m.ident, ser_path);
                            let mut de_path = path;
                            de_path
                                .path
                                .segments
                                .push(Ident::new("deserialize", Span::call_site()).into());
                            deserialize_with.set(&m.ident, de_path);
                        }
                    }

                    // Parse `#[serde(serialize_with = "...")]`
                    Meta(NameValue(ref m)) if m.ident == "serialize_with" => {
                        if let Ok(path) = parse_lit_into_expr_path(cx, &m.ident, &m.lit) {
                            serialize_with.set(&m.ident, path);
                        }
                    }

                    // Parse `#[serde(deserialize_with = "...")]`
                    Meta(NameValue(ref m)) if m.ident == "deserialize_with" => {
                        if let Ok(path) = parse_lit_into_expr_path(cx, &m.ident, &m.lit) {
                            deserialize_with.set(&m.ident, path);
                        }
                    }

                    // Defer `#[serde(borrow)]` and `#[serde(borrow = "'a + 'b")]`
                    Meta(ref m) if m.name() == "borrow" => match variant.fields {
                        syn::Fields::Unnamed(ref fields) if fields.unnamed.len() == 1 => {
                            borrow.set(m.name(), m.clone());
                        }
                        _ => {
                            cx.error_spanned_by(
                                variant,
                                "#[serde(borrow)] may only be used on newtype variants",
                            );
                        }
                    },

                    Meta(ref meta_item) => {
                        cx.error_spanned_by(
                            meta_item.name(),
                            format!("unknown serde variant attribute `{}`", meta_item.name()),
                        );
                    }

                    Literal(ref lit) => {
                        cx.error_spanned_by(lit, "unexpected literal in serde variant attribute");
                    }
                }
            }
        }

        Variant {
            name: Name::from_attrs(unraw(&variant.ident), ser_name, de_name, Some(de_aliases)),
            rename_all_rules: RenameAllRules {
                serialize: rename_all_ser_rule.get().unwrap_or(RenameRule::None),
                deserialize: rename_all_de_rule.get().unwrap_or(RenameRule::None),
            },
            ser_bound: ser_bound.get(),
            de_bound: de_bound.get(),
            skip_deserializing: skip_deserializing.get(),
            skip_serializing: skip_serializing.get(),
            other: other.get(),
            serialize_with: serialize_with.get(),
            deserialize_with: deserialize_with.get(),
            borrow: borrow.get(),
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn aliases(&self) -> Vec<String> {
        self.name.deserialize_aliases()
    }

    pub fn rename_by_rules(&mut self, rules: &RenameAllRules) {
        if !self.name.serialize_renamed {
            self.name.serialize = rules.serialize.apply_to_variant(&self.name.serialize);
        }
        if !self.name.deserialize_renamed {
            self.name.deserialize = rules.deserialize.apply_to_variant(&self.name.deserialize);
        }
    }

    pub fn rename_all_rules(&self) -> &RenameAllRules {
        &self.rename_all_rules
    }

    pub fn ser_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.ser_bound.as_ref().map(|vec| &vec[..])
    }

    pub fn de_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.de_bound.as_ref().map(|vec| &vec[..])
    }

    pub fn skip_deserializing(&self) -> bool {
        self.skip_deserializing
    }

    pub fn skip_serializing(&self) -> bool {
        self.skip_serializing
    }

    pub fn other(&self) -> bool {
        self.other
    }

    pub fn serialize_with(&self) -> Option<&syn::ExprPath> {
        self.serialize_with.as_ref()
    }

    pub fn deserialize_with(&self) -> Option<&syn::ExprPath> {
        self.deserialize_with.as_ref()
    }
}

/// Represents field attribute information
pub struct Field {
    name: Name,
    skip_serializing: bool,
    skip_deserializing: bool,
    skip_serializing_if: Option<syn::ExprPath>,
    default: Default,
    serialize_with: Option<syn::ExprPath>,
    deserialize_with: Option<syn::ExprPath>,
    ser_bound: Option<Vec<syn::WherePredicate>>,
    de_bound: Option<Vec<syn::WherePredicate>>,
    borrowed_lifetimes: BTreeSet<syn::Lifetime>,
    getter: Option<syn::ExprPath>,
    flatten: bool,
    transparent: bool,
}

/// Represents the default to use for a field when deserializing.
pub enum Default {
    /// Field must always be specified because it does not have a default.
    None,
    /// The default is given by `std::default::Default::default()`.
    Default,
    /// The default is given by this function.
    Path(syn::ExprPath),
}

impl Default {
    pub fn is_none(&self) -> bool {
        match *self {
            Default::None => true,
            Default::Default | Default::Path(_) => false,
        }
    }
}

impl Field {
    /// Extract out the `#[serde(...)]` attributes from a struct field.
    pub fn from_ast(
        cx: &Ctxt,
        index: usize,
        field: &syn::Field,
        attrs: Option<&Variant>,
        container_default: &Default,
    ) -> Self {
        let mut ser_name = Attr::none(cx, "rename");
        let mut de_name = Attr::none(cx, "rename");
        let mut de_aliases = VecAttr::none(cx, "rename");
        let mut skip_serializing = BoolAttr::none(cx, "skip_serializing");
        let mut skip_deserializing = BoolAttr::none(cx, "skip_deserializing");
        let mut skip_serializing_if = Attr::none(cx, "skip_serializing_if");
        let mut default = Attr::none(cx, "default");
        let mut serialize_with = Attr::none(cx, "serialize_with");
        let mut deserialize_with = Attr::none(cx, "deserialize_with");
        let mut ser_bound = Attr::none(cx, "bound");
        let mut de_bound = Attr::none(cx, "bound");
        let mut borrowed_lifetimes = Attr::none(cx, "borrow");
        let mut getter = Attr::none(cx, "getter");
        let mut flatten = BoolAttr::none(cx, "flatten");

        let ident = match field.ident {
            Some(ref ident) => unraw(ident),
            None => index.to_string(),
        };

        let variant_borrow = attrs
            .and_then(|variant| variant.borrow.as_ref())
            .map(|borrow| vec![Meta(borrow.clone())]);

        for meta_items in field
            .attrs
            .iter()
            .filter_map(get_serde_meta_items)
            .chain(variant_borrow)
        {
            for meta_item in meta_items {
                match meta_item {
                    // Parse `#[serde(rename = "foo")]`
                    Meta(NameValue(ref m)) if m.ident == "rename" => {
                        if let Ok(s) = get_lit_str(cx, &m.ident, &m.ident, &m.lit) {
                            ser_name.set(&m.ident, s.value());
                            de_name.set_if_none(s.value());
                            de_aliases.insert(&m.ident, s.value());
                        }
                    }

                    // Parse `#[serde(rename(serialize = "foo", deserialize = "bar"))]`
                    Meta(List(ref m)) if m.ident == "rename" => {
                        if let Ok((ser, de)) = get_multiple_renames(cx, &m.nested) {
                            ser_name.set_opt(&m.ident, ser.map(syn::LitStr::value));
                            for de_value in de {
                                de_name.set_if_none(de_value.value());
                                de_aliases.insert(&m.ident, de_value.value());
                            }
                        }
                    }

                    // Parse `#[serde(alias = "foo")]`
                    Meta(NameValue(ref m)) if m.ident == "alias" => {
                        if let Ok(s) = get_lit_str(cx, &m.ident, &m.ident, &m.lit) {
                            de_aliases.insert(&m.ident, s.value());
                        }
                    }

                    // Parse `#[serde(default)]`
                    Meta(Word(ref word)) if word == "default" => {
                        default.set(word, Default::Default);
                    }

                    // Parse `#[serde(default = "...")]`
                    Meta(NameValue(ref m)) if m.ident == "default" => {
                        if let Ok(path) = parse_lit_into_expr_path(cx, &m.ident, &m.lit) {
                            default.set(&m.ident, Default::Path(path));
                        }
                    }

                    // Parse `#[serde(skip_serializing)]`
                    Meta(Word(ref word)) if word == "skip_serializing" => {
                        skip_serializing.set_true(word);
                    }

                    // Parse `#[serde(skip_deserializing)]`
                    Meta(Word(ref word)) if word == "skip_deserializing" => {
                        skip_deserializing.set_true(word);
                    }

                    // Parse `#[serde(skip)]`
                    Meta(Word(ref word)) if word == "skip" => {
                        skip_serializing.set_true(word);
                        skip_deserializing.set_true(word);
                    }

                    // Parse `#[serde(skip_serializing_if = "...")]`
                    Meta(NameValue(ref m)) if m.ident == "skip_serializing_if" => {
                        if let Ok(path) = parse_lit_into_expr_path(cx, &m.ident, &m.lit) {
                            skip_serializing_if.set(&m.ident, path);
                        }
                    }

                    // Parse `#[serde(serialize_with = "...")]`
                    Meta(NameValue(ref m)) if m.ident == "serialize_with" => {
                        if let Ok(path) = parse_lit_into_expr_path(cx, &m.ident, &m.lit) {
                            serialize_with.set(&m.ident, path);
                        }
                    }

                    // Parse `#[serde(deserialize_with = "...")]`
                    Meta(NameValue(ref m)) if m.ident == "deserialize_with" => {
                        if let Ok(path) = parse_lit_into_expr_path(cx, &m.ident, &m.lit) {
                            deserialize_with.set(&m.ident, path);
                        }
                    }

                    // Parse `#[serde(with = "...")]`
                    Meta(NameValue(ref m)) if m.ident == "with" => {
                        if let Ok(path) = parse_lit_into_expr_path(cx, &m.ident, &m.lit) {
                            let mut ser_path = path.clone();
                            ser_path
                                .path
                                .segments
                                .push(Ident::new("serialize", Span::call_site()).into());
                            serialize_with.set(&m.ident, ser_path);
                            let mut de_path = path;
                            de_path
                                .path
                                .segments
                                .push(Ident::new("deserialize", Span::call_site()).into());
                            deserialize_with.set(&m.ident, de_path);
                        }
                    }

                    // Parse `#[serde(bound = "T: SomeBound")]`
                    Meta(NameValue(ref m)) if m.ident == "bound" => {
                        if let Ok(where_predicates) =
                            parse_lit_into_where(cx, &m.ident, &m.ident, &m.lit)
                        {
                            ser_bound.set(&m.ident, where_predicates.clone());
                            de_bound.set(&m.ident, where_predicates);
                        }
                    }

                    // Parse `#[serde(bound(serialize = "...", deserialize = "..."))]`
                    Meta(List(ref m)) if m.ident == "bound" => {
                        if let Ok((ser, de)) = get_where_predicates(cx, &m.nested) {
                            ser_bound.set_opt(&m.ident, ser);
                            de_bound.set_opt(&m.ident, de);
                        }
                    }

                    // Parse `#[serde(borrow)]`
                    Meta(Word(ref word)) if word == "borrow" => {
                        if let Ok(borrowable) = borrowable_lifetimes(cx, &ident, field) {
                            borrowed_lifetimes.set(word, borrowable);
                        }
                    }

                    // Parse `#[serde(borrow = "'a + 'b")]`
                    Meta(NameValue(ref m)) if m.ident == "borrow" => {
                        if let Ok(lifetimes) = parse_lit_into_lifetimes(cx, &m.ident, &m.lit) {
                            if let Ok(borrowable) = borrowable_lifetimes(cx, &ident, field) {
                                for lifetime in &lifetimes {
                                    if !borrowable.contains(lifetime) {
                                        cx.error_spanned_by(
                                            field,
                                            format!(
                                                "field `{}` does not have lifetime {}",
                                                ident, lifetime
                                            ),
                                        );
                                    }
                                }
                                borrowed_lifetimes.set(&m.ident, lifetimes);
                            }
                        }
                    }

                    // Parse `#[serde(getter = "...")]`
                    Meta(NameValue(ref m)) if m.ident == "getter" => {
                        if let Ok(path) = parse_lit_into_expr_path(cx, &m.ident, &m.lit) {
                            getter.set(&m.ident, path);
                        }
                    }

                    // Parse `#[serde(flatten)]`
                    Meta(Word(ref word)) if word == "flatten" => {
                        flatten.set_true(word);
                    }

                    Meta(ref meta_item) => {
                        cx.error_spanned_by(
                            meta_item.name(),
                            format!("unknown serde field attribute `{}`", meta_item.name()),
                        );
                    }

                    Literal(ref lit) => {
                        cx.error_spanned_by(lit, "unexpected literal in serde field attribute");
                    }
                }
            }
        }

        // Is skip_deserializing, initialize the field to Default::default() unless a
        // different default is specified by `#[serde(default = "...")]` on
        // ourselves or our container (e.g. the struct we are in).
        if let Default::None = *container_default {
            if skip_deserializing.0.value.is_some() {
                default.set_if_none(Default::Default);
            }
        }

        let mut borrowed_lifetimes = borrowed_lifetimes.get().unwrap_or_default();
        if !borrowed_lifetimes.is_empty() {
            // Cow<str> and Cow<[u8]> never borrow by default:
            //
            //     impl<'de, 'a, T: ?Sized> Deserialize<'de> for Cow<'a, T>
            //
            // A #[serde(borrow)] attribute enables borrowing that corresponds
            // roughly to these impls:
            //
            //     impl<'de: 'a, 'a> Deserialize<'de> for Cow<'a, str>
            //     impl<'de: 'a, 'a> Deserialize<'de> for Cow<'a, [u8]>
            if is_cow(&field.ty, is_str) {
                let mut path = syn::Path {
                    leading_colon: None,
                    segments: Punctuated::new(),
                };
                path.segments
                    .push(Ident::new("_serde", Span::call_site()).into());
                path.segments
                    .push(Ident::new("private", Span::call_site()).into());
                path.segments
                    .push(Ident::new("de", Span::call_site()).into());
                path.segments
                    .push(Ident::new("borrow_cow_str", Span::call_site()).into());
                let expr = syn::ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path: path,
                };
                deserialize_with.set_if_none(expr);
            } else if is_cow(&field.ty, is_slice_u8) {
                let mut path = syn::Path {
                    leading_colon: None,
                    segments: Punctuated::new(),
                };
                path.segments
                    .push(Ident::new("_serde", Span::call_site()).into());
                path.segments
                    .push(Ident::new("private", Span::call_site()).into());
                path.segments
                    .push(Ident::new("de", Span::call_site()).into());
                path.segments
                    .push(Ident::new("borrow_cow_bytes", Span::call_site()).into());
                let expr = syn::ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path: path,
                };
                deserialize_with.set_if_none(expr);
            }
        } else if is_implicitly_borrowed(&field.ty) {
            // Types &str and &[u8] are always implicitly borrowed. No need for
            // a #[serde(borrow)].
            collect_lifetimes(&field.ty, &mut borrowed_lifetimes);
        }

        Field {
            name: Name::from_attrs(ident, ser_name, de_name, Some(de_aliases)),
            skip_serializing: skip_serializing.get(),
            skip_deserializing: skip_deserializing.get(),
            skip_serializing_if: skip_serializing_if.get(),
            default: default.get().unwrap_or(Default::None),
            serialize_with: serialize_with.get(),
            deserialize_with: deserialize_with.get(),
            ser_bound: ser_bound.get(),
            de_bound: de_bound.get(),
            borrowed_lifetimes: borrowed_lifetimes,
            getter: getter.get(),
            flatten: flatten.get(),
            transparent: false,
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn aliases(&self) -> Vec<String> {
        self.name.deserialize_aliases()
    }

    pub fn rename_by_rules(&mut self, rules: &RenameAllRules) {
        if !self.name.serialize_renamed {
            self.name.serialize = rules.serialize.apply_to_field(&self.name.serialize);
        }
        if !self.name.deserialize_renamed {
            self.name.deserialize = rules.deserialize.apply_to_field(&self.name.deserialize);
        }
    }

    pub fn skip_serializing(&self) -> bool {
        self.skip_serializing
    }

    pub fn skip_deserializing(&self) -> bool {
        self.skip_deserializing
    }

    pub fn skip_serializing_if(&self) -> Option<&syn::ExprPath> {
        self.skip_serializing_if.as_ref()
    }

    pub fn default(&self) -> &Default {
        &self.default
    }

    pub fn serialize_with(&self) -> Option<&syn::ExprPath> {
        self.serialize_with.as_ref()
    }

    pub fn deserialize_with(&self) -> Option<&syn::ExprPath> {
        self.deserialize_with.as_ref()
    }

    pub fn ser_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.ser_bound.as_ref().map(|vec| &vec[..])
    }

    pub fn de_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.de_bound.as_ref().map(|vec| &vec[..])
    }

    pub fn borrowed_lifetimes(&self) -> &BTreeSet<syn::Lifetime> {
        &self.borrowed_lifetimes
    }

    pub fn getter(&self) -> Option<&syn::ExprPath> {
        self.getter.as_ref()
    }

    pub fn flatten(&self) -> bool {
        self.flatten
    }

    pub fn transparent(&self) -> bool {
        self.transparent
    }

    pub fn mark_transparent(&mut self) {
        self.transparent = true;
    }
}

type SerAndDe<T> = (Option<T>, Option<T>);

fn get_ser_and_de<'a, 'b, T, F>(
    cx: &'b Ctxt,
    attr_name: &'static str,
    metas: &'a Punctuated<syn::NestedMeta, Token![,]>,
    f: F,
) -> Result<(VecAttr<'b, T>, VecAttr<'b, T>), ()>
where
    T: 'a,
    F: Fn(&Ctxt, &Ident, &Ident, &'a syn::Lit) -> Result<T, ()>,
{
    let mut ser_meta = VecAttr::none(cx, attr_name);
    let mut de_meta = VecAttr::none(cx, attr_name);
    let attr_name = Ident::new(attr_name, Span::call_site());

    for meta in metas {
        match *meta {
            Meta(NameValue(ref meta)) if meta.ident == "serialize" => {
                if let Ok(v) = f(cx, &attr_name, &meta.ident, &meta.lit) {
                    ser_meta.insert(&meta.ident, v);
                }
            }

            Meta(NameValue(ref meta)) if meta.ident == "deserialize" => {
                if let Ok(v) = f(cx, &attr_name, &meta.ident, &meta.lit) {
                    de_meta.insert(&meta.ident, v);
                }
            }

            _ => {
                cx.error_spanned_by(
                    meta,
                    format!(
                        "malformed {0} attribute, expected `{0}(serialize = ..., \
                         deserialize = ...)`",
                        attr_name
                    ),
                );
                return Err(());
            }
        }
    }

    Ok((ser_meta, de_meta))
}

fn get_renames<'a>(
    cx: &Ctxt,
    items: &'a Punctuated<syn::NestedMeta, Token![,]>,
) -> Result<SerAndDe<&'a syn::LitStr>, ()> {
    let (ser, de) = try!(get_ser_and_de(cx, "rename", items, get_lit_str));
    Ok((try!(ser.at_most_one()), try!(de.at_most_one())))
}

fn get_multiple_renames<'a>(
    cx: &Ctxt,
    items: &'a Punctuated<syn::NestedMeta, Token![,]>,
) -> Result<(Option<&'a syn::LitStr>, Vec<&'a syn::LitStr>), ()> {
    let (ser, de) = try!(get_ser_and_de(cx, "rename", items, get_lit_str));
    Ok((try!(ser.at_most_one()), de.get()))
}

fn get_where_predicates(
    cx: &Ctxt,
    items: &Punctuated<syn::NestedMeta, Token![,]>,
) -> Result<SerAndDe<Vec<syn::WherePredicate>>, ()> {
    let (ser, de) = try!(get_ser_and_de(cx, "bound", items, parse_lit_into_where));
    Ok((try!(ser.at_most_one()), try!(de.at_most_one())))
}

pub fn get_serde_meta_items(attr: &syn::Attribute) -> Option<Vec<syn::NestedMeta>> {
    if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "serde" {
        match attr.interpret_meta() {
            Some(List(ref meta)) => Some(meta.nested.iter().cloned().collect()),
            _ => {
                // TODO: produce an error
                None
            }
        }
    } else {
        None
    }
}

fn get_lit_str<'a>(
    cx: &Ctxt,
    attr_name: &Ident,
    meta_item_name: &Ident,
    lit: &'a syn::Lit,
) -> Result<&'a syn::LitStr, ()> {
    if let syn::Lit::Str(ref lit) = *lit {
        Ok(lit)
    } else {
        cx.error_spanned_by(
            lit,
            format!(
                "expected serde {} attribute to be a string: `{} = \"...\"`",
                attr_name, meta_item_name
            ),
        );
        Err(())
    }
}

fn parse_lit_into_path(cx: &Ctxt, attr_name: &Ident, lit: &syn::Lit) -> Result<syn::Path, ()> {
    let string = try!(get_lit_str(cx, attr_name, attr_name, lit));
    parse_lit_str(string).map_err(|_| {
        cx.error_spanned_by(lit, format!("failed to parse path: {:?}", string.value()))
    })
}

fn parse_lit_into_expr_path(
    cx: &Ctxt,
    attr_name: &Ident,
    lit: &syn::Lit,
) -> Result<syn::ExprPath, ()> {
    let string = try!(get_lit_str(cx, attr_name, attr_name, lit));
    parse_lit_str(string).map_err(|_| {
        cx.error_spanned_by(lit, format!("failed to parse path: {:?}", string.value()))
    })
}

fn parse_lit_into_where(
    cx: &Ctxt,
    attr_name: &Ident,
    meta_item_name: &Ident,
    lit: &syn::Lit,
) -> Result<Vec<syn::WherePredicate>, ()> {
    let string = try!(get_lit_str(cx, attr_name, meta_item_name, lit));
    if string.value().is_empty() {
        return Ok(Vec::new());
    }

    let where_string = syn::LitStr::new(&format!("where {}", string.value()), string.span());

    parse_lit_str::<syn::WhereClause>(&where_string)
        .map(|wh| wh.predicates.into_iter().collect())
        .map_err(|err| cx.error_spanned_by(lit, err))
}

fn parse_lit_into_ty(cx: &Ctxt, attr_name: &Ident, lit: &syn::Lit) -> Result<syn::Type, ()> {
    let string = try!(get_lit_str(cx, attr_name, attr_name, lit));

    parse_lit_str(string).map_err(|_| {
        cx.error_spanned_by(
            lit,
            format!("failed to parse type: {} = {:?}", attr_name, string.value()),
        )
    })
}

// Parses a string literal like "'a + 'b + 'c" containing a nonempty list of
// lifetimes separated by `+`.
fn parse_lit_into_lifetimes(
    cx: &Ctxt,
    attr_name: &Ident,
    lit: &syn::Lit,
) -> Result<BTreeSet<syn::Lifetime>, ()> {
    let string = try!(get_lit_str(cx, attr_name, attr_name, lit));
    if string.value().is_empty() {
        cx.error_spanned_by(lit, "at least one lifetime must be borrowed");
        return Err(());
    }

    struct BorrowedLifetimes(Punctuated<syn::Lifetime, Token![+]>);

    impl Parse for BorrowedLifetimes {
        fn parse(input: ParseStream) -> parse::Result<Self> {
            Punctuated::parse_separated_nonempty(input).map(BorrowedLifetimes)
        }
    }

    if let Ok(BorrowedLifetimes(lifetimes)) = parse_lit_str(string) {
        let mut set = BTreeSet::new();
        for lifetime in lifetimes {
            if !set.insert(lifetime.clone()) {
                cx.error_spanned_by(lit, format!("duplicate borrowed lifetime `{}`", lifetime));
            }
        }
        return Ok(set);
    }

    cx.error_spanned_by(
        lit,
        format!("failed to parse borrowed lifetimes: {:?}", string.value()),
    );
    Err(())
}

fn is_implicitly_borrowed(ty: &syn::Type) -> bool {
    is_implicitly_borrowed_reference(ty) || is_option(ty, is_implicitly_borrowed_reference)
}

fn is_implicitly_borrowed_reference(ty: &syn::Type) -> bool {
    is_reference(ty, is_str) || is_reference(ty, is_slice_u8)
}

// Whether the type looks like it might be `std::borrow::Cow<T>` where elem="T".
// This can have false negatives and false positives.
//
// False negative:
//
//     use std::borrow::Cow as Pig;
//
//     #[derive(Deserialize)]
//     struct S<'a> {
//         #[serde(borrow)]
//         pig: Pig<'a, str>,
//     }
//
// False positive:
//
//     type str = [i16];
//
//     #[derive(Deserialize)]
//     struct S<'a> {
//         #[serde(borrow)]
//         cow: Cow<'a, str>,
//     }
fn is_cow(ty: &syn::Type, elem: fn(&syn::Type) -> bool) -> bool {
    let path = match *ty {
        syn::Type::Path(ref ty) => &ty.path,
        _ => {
            return false;
        }
    };
    let seg = match path.segments.last() {
        Some(seg) => seg.into_value(),
        None => {
            return false;
        }
    };
    let args = match seg.arguments {
        syn::PathArguments::AngleBracketed(ref bracketed) => &bracketed.args,
        _ => {
            return false;
        }
    };
    seg.ident == "Cow"
        && args.len() == 2
        && match (&args[0], &args[1]) {
            (&syn::GenericArgument::Lifetime(_), &syn::GenericArgument::Type(ref arg)) => elem(arg),
            _ => false,
        }
}

fn is_option(ty: &syn::Type, elem: fn(&syn::Type) -> bool) -> bool {
    let path = match *ty {
        syn::Type::Path(ref ty) => &ty.path,
        _ => {
            return false;
        }
    };
    let seg = match path.segments.last() {
        Some(seg) => seg.into_value(),
        None => {
            return false;
        }
    };
    let args = match seg.arguments {
        syn::PathArguments::AngleBracketed(ref bracketed) => &bracketed.args,
        _ => {
            return false;
        }
    };
    seg.ident == "Option"
        && args.len() == 1
        && match args[0] {
            syn::GenericArgument::Type(ref arg) => elem(arg),
            _ => false,
        }
}

// Whether the type looks like it might be `&T` where elem="T". This can have
// false negatives and false positives.
//
// False negative:
//
//     type Yarn = str;
//
//     #[derive(Deserialize)]
//     struct S<'a> {
//         r: &'a Yarn,
//     }
//
// False positive:
//
//     type str = [i16];
//
//     #[derive(Deserialize)]
//     struct S<'a> {
//         r: &'a str,
//     }
fn is_reference(ty: &syn::Type, elem: fn(&syn::Type) -> bool) -> bool {
    match *ty {
        syn::Type::Reference(ref ty) => ty.mutability.is_none() && elem(&ty.elem),
        _ => false,
    }
}

fn is_str(ty: &syn::Type) -> bool {
    is_primitive_type(ty, "str")
}

fn is_slice_u8(ty: &syn::Type) -> bool {
    match *ty {
        syn::Type::Slice(ref ty) => is_primitive_type(&ty.elem, "u8"),
        _ => false,
    }
}

fn is_primitive_type(ty: &syn::Type, primitive: &str) -> bool {
    match *ty {
        syn::Type::Path(ref ty) => ty.qself.is_none() && is_primitive_path(&ty.path, primitive),
        _ => false,
    }
}

fn is_primitive_path(path: &syn::Path, primitive: &str) -> bool {
    path.leading_colon.is_none()
        && path.segments.len() == 1
        && path.segments[0].ident == primitive
        && path.segments[0].arguments.is_empty()
}

// All lifetimes that this type could borrow from a Deserializer.
//
// For example a type `S<'a, 'b>` could borrow `'a` and `'b`. On the other hand
// a type `for<'a> fn(&'a str)` could not borrow `'a` from the Deserializer.
//
// This is used when there is an explicit or implicit `#[serde(borrow)]`
// attribute on the field so there must be at least one borrowable lifetime.
fn borrowable_lifetimes(
    cx: &Ctxt,
    name: &str,
    field: &syn::Field,
) -> Result<BTreeSet<syn::Lifetime>, ()> {
    let mut lifetimes = BTreeSet::new();
    collect_lifetimes(&field.ty, &mut lifetimes);
    if lifetimes.is_empty() {
        cx.error_spanned_by(
            field,
            format!("field `{}` has no lifetimes to borrow", name),
        );
        Err(())
    } else {
        Ok(lifetimes)
    }
}

fn collect_lifetimes(ty: &syn::Type, out: &mut BTreeSet<syn::Lifetime>) {
    match *ty {
        syn::Type::Slice(ref ty) => {
            collect_lifetimes(&ty.elem, out);
        }
        syn::Type::Array(ref ty) => {
            collect_lifetimes(&ty.elem, out);
        }
        syn::Type::Ptr(ref ty) => {
            collect_lifetimes(&ty.elem, out);
        }
        syn::Type::Reference(ref ty) => {
            out.extend(ty.lifetime.iter().cloned());
            collect_lifetimes(&ty.elem, out);
        }
        syn::Type::Tuple(ref ty) => {
            for elem in &ty.elems {
                collect_lifetimes(elem, out);
            }
        }
        syn::Type::Path(ref ty) => {
            if let Some(ref qself) = ty.qself {
                collect_lifetimes(&qself.ty, out);
            }
            for seg in &ty.path.segments {
                if let syn::PathArguments::AngleBracketed(ref bracketed) = seg.arguments {
                    for arg in &bracketed.args {
                        match *arg {
                            syn::GenericArgument::Lifetime(ref lifetime) => {
                                out.insert(lifetime.clone());
                            }
                            syn::GenericArgument::Type(ref ty) => {
                                collect_lifetimes(ty, out);
                            }
                            syn::GenericArgument::Binding(ref binding) => {
                                collect_lifetimes(&binding.ty, out);
                            }
                            syn::GenericArgument::Constraint(_)
                            | syn::GenericArgument::Const(_) => {}
                        }
                    }
                }
            }
        }
        syn::Type::Paren(ref ty) => {
            collect_lifetimes(&ty.elem, out);
        }
        syn::Type::Group(ref ty) => {
            collect_lifetimes(&ty.elem, out);
        }
        syn::Type::BareFn(_)
        | syn::Type::Never(_)
        | syn::Type::TraitObject(_)
        | syn::Type::ImplTrait(_)
        | syn::Type::Infer(_)
        | syn::Type::Macro(_)
        | syn::Type::Verbatim(_) => {}
    }
}

fn parse_lit_str<T>(s: &syn::LitStr) -> parse::Result<T>
where
    T: Parse,
{
    let tokens = try!(spanned_tokens(s));
    syn::parse2(tokens)
}

fn spanned_tokens(s: &syn::LitStr) -> parse::Result<TokenStream> {
    let stream = try!(syn::parse_str(&s.value()));
    Ok(respan_token_stream(stream, s.span()))
}

fn respan_token_stream(stream: TokenStream, span: Span) -> TokenStream {
    stream
        .into_iter()
        .map(|token| respan_token_tree(token, span))
        .collect()
}

fn respan_token_tree(mut token: TokenTree, span: Span) -> TokenTree {
    if let TokenTree::Group(ref mut g) = token {
        *g = Group::new(g.delimiter(), respan_token_stream(g.stream().clone(), span));
    }
    token.set_span(span);
    token
}
