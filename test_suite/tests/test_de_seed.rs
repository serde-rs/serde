#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_test;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use serde::de::{Deserialize, Deserializer, DeserializeSeed, EnumAccess, Error, MapAccess, SeqAccess, VariantAccess, Visitor, Unexpected};

use serde_test::{Token, assert_de_seed_tokens};

#[derive(Clone)]
struct Seed(Rc<Cell<i32>>);

#[derive(Deserialize, Debug, PartialEq)]
struct Inner;

struct InnerSeed(Rc<Cell<i32>>);

impl From<Seed> for InnerSeed {
    fn from(value: Seed) -> Self {
        InnerSeed(value.0.clone())
    }
}

impl<'de> DeserializeSeed<'de> for InnerSeed {
    type Value = Inner;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer<'de>
    {
        self.0.set(self.0.get() + 1);
        Inner::deserialize(deserializer)
    }
}

#[derive(DeserializeSeed, Debug, PartialEq)]
#[serde(deserialize_seed = "Seed")]
struct SeedStruct {
    #[serde(deserialize_seed_with = "InnerSeed::from")]
    value: Inner,
    #[serde(deserialize_seed_with = "InnerSeed::from")]
    value2: Inner,
    value3: Inner,
}

#[test]
fn test_deserialize_seed() {
    let value = SeedStruct { value: Inner, value2: Inner, value3: Inner };
    let seed = Seed(Rc::new(Cell::new(0)));
    assert_de_seed_tokens(
        seed.clone(),
        &value,
        &[
            Token::Struct {
                name: "SeedStruct",
                len: 1,
            },

            Token::Str("value"),
            Token::UnitStruct { name: "Inner" },

            Token::Str("value2"),
            Token::UnitStruct { name: "Inner" },

            Token::Str("value3"),
            Token::UnitStruct { name: "Inner" },

            Token::StructEnd,
        ],
    );

    assert_eq!(seed.0.get(), 2);
}


#[derive(Debug, PartialEq)]
struct Node {
    data: char,
    left: Option<Rc<Node>>,
    right: Option<Rc<Node>>,
}

type Id = u32;
type SharedToId<T> = HashMap<*const T, Id>;
type IdToShared<T> = HashMap<Id, T>;

type NodeToId = SharedToId<Node>;
type IdToNode = IdToShared<Rc<Node>>;

enum Lookup {
    Unique,
    Found(Id),
    Inserted(Id),
}

pub trait Shared: ::std::ops::Deref
    where Self::Target: Sized
{
    fn unique(&self) -> bool;
}

impl<T> Shared for Rc<T>
    where T: Sized
{
    fn unique(&self) -> bool {
        Rc::strong_count(self) == 1
    }
}

impl<'de> Deserialize<'de> for Node {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let mut map = IdToNode::default();
        deserializer.deserialize_struct("Node", FIELDS, PlainNodeVisitor { map: &mut map })
    }
}

//////////////////////////////////////////////////////////////////////////////

struct NodeSeed<'a, T: 'a> {
    map: &'a mut IdToShared<T>,
}

pub trait DagVisitable<'de, 'a>: Shared + Sized
    where <Self as std::ops::Deref>::Target: Sized
{
    type PlainVisitor: Visitor<'de, Value = Self::Target> + 'a;
    type MarkedVisitor: Visitor<'de, Value = Self> + 'a;
    fn plain(map: &'a mut IdToShared<Self>) -> Self::PlainVisitor;
    fn marked(map: &'a mut IdToShared<Self>) -> Self::MarkedVisitor;
}

impl<'de, 'a> DagVisitable<'de, 'a> for Rc<Node> {
    type PlainVisitor = PlainNodeVisitor<'a>;
    type MarkedVisitor = MarkedNodeVisitor<'a>;
    fn plain(map: &'a mut IdToShared<Self>) -> Self::PlainVisitor {
        PlainNodeVisitor { map: map }
    }
    fn marked(map: &'a mut IdToShared<Self>) -> Self::MarkedVisitor {
        MarkedNodeVisitor { map: map }
    }
}

impl<'de, 'a, T> Visitor<'de> for NodeSeed<'a, Rc<T>>
    where Rc<T>: DagVisitable<'de, 'a> + std::ops::Deref<Target = T>
{
    type Value = Rc<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct Node")
    }

    fn visit_enum<V>(self, visitor: V) -> Result<Self::Value, V::Error>
        where V: EnumAccess<'de>
    {
        match visitor.variant()? {
            (Variant::Plain, variant) => {
                variant
                    .struct_variant(FIELDS, <Rc<T>>::plain(self.map))
                    .map(Rc::new)
            }
            (Variant::Marked, variant) => {
                variant.struct_variant(MARKED_FIELDS, <Rc<T>>::marked(self.map))
            }
            (Variant::Reference, variant) => {
                let id = variant.newtype_variant()?;
                match self.map.get(&id) {
                    Some(rc) => Ok(rc.clone()),
                    None => Err(Error::custom(format_args!("missing id {}", id))),
                }
            }
        }
    }
}

impl<'de, 'a> DeserializeSeed<'de> for NodeSeed<'a, Rc<Node>> {
    type Value = Rc<Node>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_enum("Node", VARIANTS, self)
    }
}

struct PlainNodeVisitor<'a> {
    map: &'a mut IdToNode,
}

impl<'de, 'a> Visitor<'de> for PlainNodeVisitor<'a> {
    type Value = Node;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct Node")
    }

    fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
        where V: SeqAccess<'de>
    {
        let data = visitor
            .next_element()?
            .ok_or_else(|| Error::invalid_length(0, &self))?;
        let left = visitor
            .next_element_seed(OptionSeed(NodeSeed { map: self.map }))?
            .ok_or_else(|| Error::invalid_length(1, &self))?;
        let right = visitor
            .next_element_seed(OptionSeed(NodeSeed { map: self.map }))?
            .ok_or_else(|| Error::invalid_length(2, &self))?;
        Ok(Node {
               data: data,
               left: left,
               right: right,
           })
    }

    fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
        where V: MapAccess<'de>
    {
        let mut data = None;
        let mut left = None;
        let mut right = None;
        while let Some(key) = visitor.next_key()? {
            match key {
                Field::Data => {
                    if data.is_some() {
                        return Err(Error::duplicate_field("data"));
                    }
                    data = Some(visitor.next_value()?);
                }
                Field::Left => {
                    if left.is_some() {
                        return Err(Error::duplicate_field("left"));
                    }
                    left = visitor
                        .next_value_seed(OptionSeed(NodeSeed { map: self.map }))?;
                }
                Field::Right => {
                    if right.is_some() {
                        return Err(Error::duplicate_field("right"));
                    }
                    right = visitor
                        .next_value_seed(OptionSeed(NodeSeed { map: self.map }))?;
                }
            }
        }
        let data = data.ok_or_else(|| Error::missing_field("data"))?;
        Ok(Node {
               data: data,
               left: left,
               right: right,
           })
    }
}

struct MarkedNodeVisitor<'a> {
    map: &'a mut IdToNode,
}

impl<'de, 'a> Visitor<'de> for MarkedNodeVisitor<'a> {
    type Value = Rc<Node>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct Node")
    }

    fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
        where V: SeqAccess<'de>
    {
        let id = visitor
            .next_element()?
            .ok_or_else(|| Error::invalid_length(0, &self))?;
        let data = visitor
            .next_element()?
            .ok_or_else(|| Error::invalid_length(1, &self))?;
        let left = visitor
            .next_element_seed(OptionSeed(NodeSeed { map: self.map }))?
            .ok_or_else(|| Error::invalid_length(2, &self))?;
        let right = visitor
            .next_element_seed(OptionSeed(NodeSeed { map: self.map }))?
            .ok_or_else(|| Error::invalid_length(3, &self))?;
        let node = Rc::new(Node {
                               data: data,
                               left: left,
                               right: right,
                           });
        self.map.insert(id, node.clone());
        Ok(node)
    }

    fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
        where V: MapAccess<'de>
    {
        let mut id = None;
        let mut data = None;
        let mut left = None;
        let mut right = None;
        while let Some(key) = visitor.next_key()? {
            match key {
                MarkedField::Id => {
                    if id.is_some() {
                        return Err(Error::duplicate_field("id"));
                    }
                    id = Some(visitor.next_value()?);
                }
                MarkedField::Data => {
                    if data.is_some() {
                        return Err(Error::duplicate_field("data"));
                    }
                    data = Some(visitor.next_value()?);
                }
                MarkedField::Left => {
                    if left.is_some() {
                        return Err(Error::duplicate_field("left"));
                    }
                    left = visitor
                        .next_value_seed(OptionSeed(NodeSeed { map: self.map }))?;
                }
                MarkedField::Right => {
                    if right.is_some() {
                        return Err(Error::duplicate_field("right"));
                    }
                    right = visitor
                        .next_value_seed(OptionSeed(NodeSeed { map: self.map }))?;
                }
            }
        }
        let id = id.ok_or_else(|| Error::missing_field("id"))?;
        let data = data.ok_or_else(|| Error::missing_field("data"))?;
        let node = Rc::new(Node {
                               data: data,
                               left: left,
                               right: right,
                           });
        self.map.insert(id, node.clone());
        Ok(node)
    }
}

//////////////////////////////////////////////////////////////////////////////

/// Maybe this should be provided by Serde. Just turns any seed into an
/// optional one.
struct OptionSeed<S>(S);

impl<'de, S> Visitor<'de> for OptionSeed<S>
    where S: DeserializeSeed<'de>
{
    type Value = Option<S::Value>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("option")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
        where E: Error
    {
        Ok(None)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer<'de>
    {
        self.0.deserialize(deserializer).map(Some)
    }
}

impl<'de, S> DeserializeSeed<'de> for OptionSeed<S>
    where S: DeserializeSeed<'de>
{
    type Value = Option<S::Value>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_option(self)
    }
}

//////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize)]
#[serde(variant_identifier)]
enum Variant {
    Plain,
    Marked,
    Reference,
}

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum Field {
    Data,
    Left,
    Right,
}

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum MarkedField {
    Id,
    Data,
    Left,
    Right,
}

const VARIANTS: &'static [&'static str] = &["Plain", "Marked", "Reference"];
const FIELDS: &'static [&'static str] = &["data", "left", "right"];
const MARKED_FIELDS: &'static [&'static str] = &["id", "data", "left", "right"];

#[test]
fn test_node_deserialize() {
    let b = Rc::new(Node {
        data: 'b',
        left: None,
        right: None,
    });
    let a = Rc::new(Node {
        data: 'a',
        left: Some(b.clone()),
        right: Some(b.clone()),
    });
    let mut map = HashMap::new();
    let seed = NodeSeed {
        map: &mut map,
    };
    assert_de_seed_tokens(
        seed,
        &a,
        &[
            Token::StructVariant { name: "Node", variant: "Plain", len: 3 },

            Token::Str("data"),
            Token::Char('a'),

            Token::Str("left"),
            Token::Some,

            Token::StructVariant { name: "Node", variant: "Marked", len: 4 },

            Token::Str("id"),
            Token::I32(0),

            Token::Str("data"),
            Token::Char('b'),

            Token::Str("left"),
            Token::None,

            Token::Str("right"),
            Token::None,

            Token::StructVariantEnd,

            Token::Str("right"),
            Token::Some,
            
            Token::NewtypeVariant { name: "Node", variant: "Reference" },
            Token::U32(0),

            Token::StructVariantEnd,
        ],
    );
}
