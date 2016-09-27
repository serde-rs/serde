use Ctxt;
use syn;

// This module handles parsing of `#[serde(...)]` attributes. The entrypoints
// are `attr::Item::from_ast`, `attr::Variant::from_ast`, and
// `attr::Field::from_ast`. Each returns an instance of the corresponding
// struct. Note that none of them return a Result. Unrecognized, malformed, or
// duplicated attributes result in a span_err but otherwise are ignored. The
// user will see errors simultaneously for all bad attributes in the crate
// rather than just the first.

struct Attr<'c, T> {
    cx: &'c Ctxt,
    name: &'static str,
    value: Option<T>,
}

impl<'c, T> Attr<'c, T> {
    fn none(cx: &'c Ctxt, name: &'static str) -> Self {
        Attr {
            cx: cx,
            name: name,
            value: None,
        }
    }

    fn set(&mut self, value: T) {
        if self.value.is_some() {
            self.cx.error(format!("duplicate serde attribute `{}`", self.name));
        } else {
            self.value = Some(value);
        }
    }

    fn set_opt(&mut self, value: Option<T>) {
        if let Some(value) = value {
            self.set(value);
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
}

struct BoolAttr<'c>(Attr<'c, ()>);

impl<'c> BoolAttr<'c> {
    fn none(cx: &'c Ctxt, name: &'static str) -> Self {
        BoolAttr(Attr::none(cx, name))
    }

    fn set_true(&mut self) {
        self.0.set(());
    }

    fn get(&self) -> bool {
        self.0.value.is_some()
    }
}

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
    ser_bound: Option<Vec<syn::WherePredicate>>,
    de_bound: Option<Vec<syn::WherePredicate>>,
}

impl Item {
    /// Extract out the `#[serde(...)]` attributes from an item.
    pub fn from_ast(cx: &Ctxt, item: &syn::MacroInput) -> Self {
        let mut ser_name = Attr::none(cx, "rename");
        let mut de_name = Attr::none(cx, "rename");
        let mut deny_unknown_fields = BoolAttr::none(cx, "deny_unknown_fields");
        let mut ser_bound = Attr::none(cx, "bound");
        let mut de_bound = Attr::none(cx, "bound");

        for meta_items in item.attrs.iter().filter_map(get_serde_meta_items) {
            for meta_item in meta_items {
                match meta_item {
                    // Parse `#[serde(rename="foo")]`
                    syn::MetaItem::NameValue(ref name, ref lit) if name == "rename" => {
                        if let Ok(s) = get_string_from_lit(cx, name.as_ref(), name.as_ref(), lit) {
                            ser_name.set(s.clone());
                            de_name.set(s);
                        }
                    }

                    // Parse `#[serde(rename(serialize="foo", deserialize="bar"))]`
                    syn::MetaItem::List(ref name, ref meta_items) if name == "rename" => {
                        if let Ok((ser, de)) = get_renames(cx, meta_items) {
                            ser_name.set_opt(ser);
                            de_name.set_opt(de);
                        }
                    }

                    // Parse `#[serde(deny_unknown_fields)]`
                    syn::MetaItem::Word(ref name) if name == "deny_unknown_fields" => {
                        deny_unknown_fields.set_true();
                    }

                    // Parse `#[serde(bound="D: Serialize")]`
                    syn::MetaItem::NameValue(ref name, ref lit) if name == "bound" => {
                        if let Ok(where_predicates) = parse_lit_into_where(cx, name.as_ref(), name.as_ref(), lit) {
                            ser_bound.set(where_predicates.clone());
                            de_bound.set(where_predicates);
                        }
                    }

                    // Parse `#[serde(bound(serialize="D: Serialize", deserialize="D: Deserialize"))]`
                    syn::MetaItem::List(ref name, ref meta_items) if name == "bound" => {
                        if let Ok((ser, de)) = get_where_predicates(cx, meta_items) {
                            ser_bound.set_opt(ser);
                            de_bound.set_opt(de);
                        }
                    }

                    _ => {
                        cx.error(format!("unknown serde container attribute `{}`",
                                         meta_item.name()));
                    }
                }
            }
        }

        Item {
            name: Name {
                serialize: ser_name.get().unwrap_or_else(|| item.ident.to_string()),
                deserialize: de_name.get().unwrap_or_else(|| item.ident.to_string()),
            },
            deny_unknown_fields: deny_unknown_fields.get(),
            ser_bound: ser_bound.get(),
            de_bound: de_bound.get(),
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn deny_unknown_fields(&self) -> bool {
        self.deny_unknown_fields
    }

    pub fn ser_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.ser_bound.as_ref().map(|vec| &vec[..])
    }

    pub fn de_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.de_bound.as_ref().map(|vec| &vec[..])
    }
}

/// Represents variant attribute information
#[derive(Debug)]
pub struct Variant {
    name: Name,
}

impl Variant {
    pub fn from_ast(cx: &Ctxt, variant: &syn::Variant) -> Self {
        let mut ser_name = Attr::none(cx, "rename");
        let mut de_name = Attr::none(cx, "rename");

        for meta_items in variant.attrs.iter().filter_map(get_serde_meta_items) {
            for meta_item in meta_items {
                match meta_item {
                    // Parse `#[serde(rename="foo")]`
                    syn::MetaItem::NameValue(ref name, ref lit) if name == "rename" => {
                        if let Ok(s) = get_string_from_lit(cx, name.as_ref(), name.as_ref(), lit) {
                            ser_name.set(s.clone());
                            de_name.set(s);
                        }
                    }

                    // Parse `#[serde(rename(serialize="foo", deserialize="bar"))]`
                    syn::MetaItem::List(ref name, ref meta_items) if name == "rename" => {
                        if let Ok((ser, de)) = get_renames(cx, meta_items) {
                            ser_name.set_opt(ser);
                            de_name.set_opt(de);
                        }
                    }

                    _ => {
                        cx.error(format!("unknown serde variant attribute `{}`",
                                         meta_item.name()));
                    }
                }
            }
        }

        Variant {
            name: Name {
                serialize: ser_name.get().unwrap_or_else(|| variant.ident.to_string()),
                deserialize: de_name.get().unwrap_or_else(|| variant.ident.to_string()),
            },
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }
}

/// Represents field attribute information
#[derive(Debug)]
pub struct Field {
    name: Name,
    skip_serializing: bool,
    skip_deserializing: bool,
    skip_serializing_if: Option<syn::Path>,
    default: FieldDefault,
    serialize_with: Option<syn::Path>,
    deserialize_with: Option<syn::Path>,
    ser_bound: Option<Vec<syn::WherePredicate>>,
    de_bound: Option<Vec<syn::WherePredicate>>,
}

/// Represents the default to use for a field when deserializing.
#[derive(Debug, PartialEq)]
pub enum FieldDefault {
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
        let mut ser_name = Attr::none(cx, "rename");
        let mut de_name = Attr::none(cx, "rename");
        let mut skip_serializing = BoolAttr::none(cx, "skip_serializing");
        let mut skip_deserializing = BoolAttr::none(cx, "skip_deserializing");
        let mut skip_serializing_if = Attr::none(cx, "skip_serializing_if");
        let mut default = Attr::none(cx, "default");
        let mut serialize_with = Attr::none(cx, "serialize_with");
        let mut deserialize_with = Attr::none(cx, "deserialize_with");
        let mut ser_bound = Attr::none(cx, "bound");
        let mut de_bound = Attr::none(cx, "bound");

        let ident = match field.ident {
            Some(ref ident) => ident.to_string(),
            None => index.to_string(),
        };

        for meta_items in field.attrs.iter().filter_map(get_serde_meta_items) {
            for meta_item in meta_items {
                match meta_item {
                    // Parse `#[serde(rename="foo")]`
                    syn::MetaItem::NameValue(ref name, ref lit) if name == "rename" => {
                        if let Ok(s) = get_string_from_lit(cx, name.as_ref(), name.as_ref(), lit) {
                            ser_name.set(s.clone());
                            de_name.set(s);
                        }
                    }

                    // Parse `#[serde(rename(serialize="foo", deserialize="bar"))]`
                    syn::MetaItem::List(ref name, ref meta_items) if name == "rename" => {
                        if let Ok((ser, de)) = get_renames(cx, meta_items) {
                            ser_name.set_opt(ser);
                            de_name.set_opt(de);
                        }
                    }

                    // Parse `#[serde(default)]`
                    syn::MetaItem::Word(ref name) if name == "default" => {
                        default.set(FieldDefault::Default);
                    }

                    // Parse `#[serde(default="...")]`
                    syn::MetaItem::NameValue(ref name, ref lit) if name == "default" => {
                        if let Ok(path) = parse_lit_into_path(cx, name.as_ref(), lit) {
                            default.set(FieldDefault::Path(path));
                        }
                    }

                    // Parse `#[serde(skip_serializing)]`
                    syn::MetaItem::Word(ref name) if name == "skip_serializing" => {
                        skip_serializing.set_true();
                    }

                    // Parse `#[serde(skip_deserializing)]`
                    syn::MetaItem::Word(ref name) if name == "skip_deserializing" => {
                        skip_deserializing.set_true();
                    }

                    // Parse `#[serde(skip_serializing_if="...")]`
                    syn::MetaItem::NameValue(ref name, ref lit) if name == "skip_serializing_if" => {
                        if let Ok(path) = parse_lit_into_path(cx, name.as_ref(), lit) {
                            skip_serializing_if.set(path);
                        }
                    }

                    // Parse `#[serde(serialize_with="...")]`
                    syn::MetaItem::NameValue(ref name, ref lit) if name == "serialize_with" => {
                        if let Ok(path) = parse_lit_into_path(cx, name.as_ref(), lit) {
                            serialize_with.set(path);
                        }
                    }

                    // Parse `#[serde(deserialize_with="...")]`
                    syn::MetaItem::NameValue(ref name, ref lit) if name == "deserialize_with" => {
                        if let Ok(path) = parse_lit_into_path(cx, name.as_ref(), lit) {
                            deserialize_with.set(path);
                        }
                    }

                    // Parse `#[serde(bound="D: Serialize")]`
                    syn::MetaItem::NameValue(ref name, ref lit) if name == "bound" => {
                        if let Ok(where_predicates) = parse_lit_into_where(cx, name.as_ref(), name.as_ref(), lit) {
                            ser_bound.set(where_predicates.clone());
                            de_bound.set(where_predicates);
                        }
                    }

                    // Parse `#[serde(bound(serialize="D: Serialize", deserialize="D: Deserialize"))]`
                    syn::MetaItem::List(ref name, ref meta_items) if name == "bound" => {
                        if let Ok((ser, de)) = get_where_predicates(cx, meta_items) {
                            ser_bound.set_opt(ser);
                            de_bound.set_opt(de);
                        }
                    }

                    _ => {
                        cx.error(format!("unknown serde field attribute `{}`",
                                         meta_item.name()));
                    }
                }
            }
        }

        // Is skip_deserializing, initialize the field to Default::default()
        // unless a different default is specified by `#[serde(default="...")]`
        if skip_deserializing.0.value.is_some() {
            default.set_if_none(FieldDefault::Default);
        }

        Field {
            name: Name {
                serialize: ser_name.get().unwrap_or(ident.clone()),
                deserialize: de_name.get().unwrap_or(ident),
            },
            skip_serializing: skip_serializing.get(),
            skip_deserializing: skip_deserializing.get(),
            skip_serializing_if: skip_serializing_if.get(),
            default: default.get().unwrap_or(FieldDefault::None),
            serialize_with: serialize_with.get(),
            deserialize_with: deserialize_with.get(),
            ser_bound: ser_bound.get(),
            de_bound: de_bound.get(),
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
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

    pub fn default(&self) -> &FieldDefault {
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

type SerAndDe<T> = (Option<T>, Option<T>);

fn get_ser_and_de<T, F>(
    cx: &Ctxt,
    attr_name: &'static str,
    items: &[syn::MetaItem],
    f: F
) -> Result<SerAndDe<T>, ()>
    where F: Fn(&Ctxt, &str, &str, &syn::Lit) -> Result<T, ()>,
{
    let mut ser_item = Attr::none(cx, attr_name);
    let mut de_item = Attr::none(cx, attr_name);

    for item in items {
        match *item {
            syn::MetaItem::NameValue(ref name, ref lit) if name == "serialize" => {
                if let Ok(v) = f(cx, attr_name, name.as_ref(), lit) {
                    ser_item.set(v);
                }
            }

            syn::MetaItem::NameValue(ref name, ref lit) if name == "deserialize" => {
                if let Ok(v) = f(cx, attr_name, name.as_ref(), lit) {
                    de_item.set(v);
                }
            }

            _ => {
                cx.error(format!("malformed {0} attribute, expected `{0}(serialize = ..., deserialize = ...)`",
                                 attr_name));
                return Err(());
            }
        }
    }

    Ok((ser_item.get(), de_item.get()))
}

fn get_renames(
    cx: &Ctxt,
    items: &[syn::MetaItem],
) -> Result<SerAndDe<String>, ()> {
    get_ser_and_de(cx, "rename", items, get_string_from_lit)
}

fn get_where_predicates(
    cx: &Ctxt,
    items: &[syn::MetaItem],
) -> Result<SerAndDe<Vec<syn::WherePredicate>>, ()> {
    get_ser_and_de(cx, "bound", items, parse_lit_into_where)
}

pub fn get_serde_meta_items(attr: &syn::Attribute) -> Option<Vec<syn::MetaItem>> {
    match attr.value {
        syn::MetaItem::List(ref name, ref items) if name == "serde" => {
            Some(items.iter().cloned().collect())
        }
        _ => None
    }
}

fn get_string_from_lit(cx: &Ctxt, attr_name: &str, meta_item_name: &str, lit: &syn::Lit) -> Result<String, ()> {
    if let syn::Lit::Str(ref s, _) = *lit {
        Ok(s.clone())
    } else {
        cx.error(format!("expected serde {} attribute to be a string: `{} = \"...\"`",
                         attr_name, meta_item_name));
        Err(())
    }
}

fn parse_lit_into_path(cx: &Ctxt, attr_name: &str, lit: &syn::Lit) -> Result<syn::Path, ()> {
    let string = try!(get_string_from_lit(cx, attr_name, attr_name, lit));
    syn::parse_path(&string).map_err(|err| cx.error(err))
}

fn parse_lit_into_where(cx: &Ctxt, attr_name: &str, meta_item_name: &str, lit: &syn::Lit) -> Result<Vec<syn::WherePredicate>, ()> {
    let string = try!(get_string_from_lit(cx, attr_name, meta_item_name, lit));
    if string.is_empty() {
        return Ok(Vec::new());
    }

    let where_string = format!("where {}", string);

    syn::parse_where_clause(&where_string).map(|wh| wh.predicates).map_err(|err| cx.error(err))
}
