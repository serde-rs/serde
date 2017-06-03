#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_test;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

use serde::de::{Deserialize, Deserializer, DeserializeSeed, Error, OptionSeed};

use serde_test::{Token, assert_de_seed_tokens};

#[derive(Clone)]
struct Seed(Rc<Cell<i32>>);

impl AsMut<Rc<Cell<i32>>> for Seed {
    fn as_mut(&mut self) -> &mut Rc<Cell<i32>> {
        &mut self.0
    }
}

#[derive(Deserialize, Debug, PartialEq)]
struct Inner;

#[derive(Clone)]
struct InnerSeed(Rc<Cell<i32>>);

fn deserialize_inner<'de, S, D>(seed: &mut S, deserializer: D) -> Result<Inner, D::Error>
where
    S: AsMut<Rc<Cell<i32>>>,
    D: Deserializer<'de>,
{
    InnerSeed(seed.as_mut().clone()).deserialize(deserializer)
}

impl<'de> DeserializeSeed<'de> for InnerSeed {
    type Value = Inner;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        self.0.set(self.0.get() + 1);
        Inner::deserialize(deserializer)
    }
}

#[derive(DeserializeSeed, Debug, PartialEq)]
#[serde(deserialize_seed = "Seed")]
struct SeedStruct {
    #[serde(deserialize_seed_with = "deserialize_inner")]
    value: Inner,
    #[serde(deserialize_seed_with = "deserialize_inner")]
    value2: Inner,
    value3: Inner,
}

#[test]
fn test_deserialize_seed() {
    let value = SeedStruct {
        value: Inner,
        value2: Inner,
        value3: Inner,
    };
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

#[derive(Clone)]
struct NewtypeSeed(Rc<Cell<i32>>);

impl AsMut<Rc<Cell<i32>>> for NewtypeSeed {
    fn as_mut(&mut self) -> &mut Rc<Cell<i32>> {
        &mut self.0
    }
}

#[derive(DeserializeSeed, Debug, PartialEq)]
#[serde(deserialize_seed = "NewtypeSeed")]
struct Newtype(
    #[serde(deserialize_seed_with = "deserialize_inner")]
    Inner
);

#[test]
fn test_newtype_deserialize_seed() {
    let value = Newtype(Inner);
    let seed = NewtypeSeed(Rc::new(Cell::new(0)));
    assert_de_seed_tokens(
        seed.clone(),
        &value,
        &[
            Token::NewtypeStruct { name: "Newtype" },

            Token::UnitStruct { name: "Inner" },
        ],
    );

    assert_eq!(seed.0.get(), 1);
}

#[derive(Clone)]
struct ExtraParameterNewtypeSeed<T>(Rc<Cell<i32>>, PhantomData<T>);

impl<T> AsMut<Rc<Cell<i32>>> for ExtraParameterNewtypeSeed<T> {
    fn as_mut(&mut self) -> &mut Rc<Cell<i32>> {
        &mut self.0
    }
}


#[derive(DeserializeSeed, Debug, PartialEq)]
#[serde(deserialize_seed = "ExtraParameterNewtypeSeed<T>")]
#[serde(de_parameter = "T")]
struct ExtraParameterNewtype(
    #[serde(deserialize_seed_with = "deserialize_inner")]
    Inner
);


#[test]
fn extra_parameter_test_newtype_deserialize_seed() {
    let value = ExtraParameterNewtype(Inner);
    let seed = ExtraParameterNewtypeSeed(Rc::new(Cell::new(0)), PhantomData::<i32>);
    assert_de_seed_tokens(
        seed.clone(),
        &value,
        &[
            Token::NewtypeStruct { name: "ExtraParameterNewtype" },

            Token::UnitStruct { name: "Inner" },
        ],
    );

    assert_eq!(seed.0.get(), 1);
}

#[derive(Clone)]
struct VecSeed<T>(T);

fn deserialize_vec<'de, T, D>(
    seed: &mut VecSeed<T>,
    deserializer: D,
) -> Result<Vec<T::Value>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeSeed<'de> + Clone,
{
    use serde::de::SeqSeed;
    SeqSeed::new(seed.0.clone(), Vec::with_capacity).deserialize(deserializer)
}

#[derive(DeserializeSeed, Debug, PartialEq)]
#[serde(deserialize_seed = "VecSeed<T>")]
#[serde(bound = "T: DeserializeSeed<'de> + Clone")]
struct VecNewtype<T>(
    #[serde(deserialize_seed_with = "deserialize_vec")]
    Vec<T>
);

#[test]
fn test_vec_newtype_deserialize_seed() {
    let value = VecNewtype(vec![Inner, Inner]);
    let seed = VecSeed(InnerSeed(Rc::new(Cell::new(0))));
    assert_de_seed_tokens(
        seed.clone(),
        &value,
        &[
            Token::NewtypeStruct { name: "VecNewtype" },

            Token::Seq { len: Some(2) },
            Token::UnitStruct { name: "Inner" },
            Token::UnitStruct { name: "Inner" },
            Token::SeqEnd,
        ],
    );

    assert_eq!((seed.0).0.get(), 2);
}

#[derive(Clone)]
struct GenericTypeSeed<T>(Rc<Cell<i32>>, T);

impl<T> AsMut<Rc<Cell<i32>>> for GenericTypeSeed<T> {
    fn as_mut(&mut self) -> &mut Rc<Cell<i32>> {
        &mut self.0
    }
}

fn deserialize_nested_seed<'de, T, D>(
    seed: &mut GenericTypeSeed<T>,
    deserializer: D,
) -> Result<T::Value, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeSeed<'de> + Clone,
{
    seed.1.clone().deserialize(deserializer)
}

#[derive(DeserializeSeed, Debug, PartialEq)]
#[serde(deserialize_seed = "GenericTypeSeed<PhantomData<T>>")]
struct GenericType<T> {
    #[serde(deserialize_seed_with = "deserialize_inner")]
    inner: Inner,
    #[serde(deserialize_seed_with = "deserialize_nested_seed")]
    t: T
}

#[test]
fn test_generic_deserialize_seed() {
    let value = GenericType { inner: Inner, t: 3 };
    let seed = GenericTypeSeed(Rc::new(Cell::new(0)), PhantomData);
    assert_de_seed_tokens(
        seed.clone(),
        &value,
        &[
            Token::Struct {
                name: "GenericType",
                len: 2,
            },

            Token::String("inner"),
            Token::UnitStruct { name: "Inner" },

            Token::String("t"),
            Token::I32(3),

            Token::StructEnd,
        ],
    );

    assert_eq!(seed.0.get(), 1);
}


#[derive(Clone)]
struct EnumSeed(Rc<Cell<i32>>);

impl AsMut<Rc<Cell<i32>>> for EnumSeed {
    fn as_mut(&mut self) -> &mut Rc<Cell<i32>> {
        &mut self.0
    }
}

#[derive(DeserializeSeed, Debug, PartialEq)]
#[serde(deserialize_seed = "EnumSeed")]
enum Enum {
    Inner(
        #[serde(deserialize_seed_with = "deserialize_inner")]
        Inner
    ),
    Inner2(
        u32,
        #[serde(deserialize_seed_with = "deserialize_inner")]
        Inner
    ),
}

#[test]
fn test_enum_deserialize_seed() {
    let value = Enum::Inner(Inner);
    let seed = EnumSeed(Rc::new(Cell::new(0)));
    assert_de_seed_tokens(
        seed.clone(),
        &value,
        &[
            Token::NewtypeVariant {
                name: "Enum",
                variant: "Inner",
            },

            Token::UnitStruct { name: "Inner" },
        ],
    );

    assert_eq!(seed.0.get(), 1);
}


#[test]
fn test_enum_deserialize_seed_2() {
    let value = Enum::Inner2(3, Inner);
    let seed = EnumSeed(Rc::new(Cell::new(0)));
    assert_de_seed_tokens(
        seed.clone(),
        &value,
        &[
            Token::TupleVariant {
                name: "Enum",
                variant: "Inner2",
                len: 2,
            },

            Token::U32(3),

            Token::UnitStruct { name: "Inner" },

            Token::TupleVariantEnd,
        ],
    );

    assert_eq!(seed.0.get(), 1);
}

#[derive(DeserializeSeed, Debug, PartialEq)]
#[serde(deserialize_seed = "NodeSeed<Rc<Node>>")]
struct Node {
    data: char,
    #[serde(deserialize_seed_with = "deserialize_option_node")]
    left: Option<Rc<Node>>,
    #[serde(deserialize_seed_with = "deserialize_option_node")]
    right: Option<Rc<Node>>,
}

fn deserialize_option_node<'de, S, D>(
    seed: &mut S,
    deserializer: D,
) -> Result<Option<Rc<Node>>, D::Error>
where
    S: AsMut<NodeMap>,
    D: Deserializer<'de>,
{
    let variant = OptionSeed(VariantSeed { map: seed.as_mut().clone() })
        .deserialize(deserializer)?;
    match variant {
        None => Ok(None),
        Some(variant) => {
            match variant {
                Variant::Marked {
                    id,
                    data,
                    left,
                    right,
                } => {
                    let node = Rc::new(Node { data, left, right });
                    seed.as_mut().borrow_mut().insert(id, node.clone());
                    Ok(Some(node))
                }
                Variant::Plain { data, left, right } => Ok(Some(Rc::new(Node { data, left, right }),),),
                Variant::Reference(id) => {
                    match seed.as_mut().borrow_mut().get(&id) {
                        Some(rc) => Ok(Some(rc.clone())),
                        None => Err(Error::custom(format_args!("missing id {}", id))),
                    }
                }
            }
        }
    }
}


type Id = u32;
type IdToShared<T> = HashMap<Id, T>;

type IdToNode = IdToShared<Rc<Node>>;

type NodeMap = Rc<RefCell<IdToShared<Rc<Node>>>>;

impl<'de> Deserialize<'de> for Node {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map = NodeSeed { map: Rc::new(RefCell::new(IdToNode::default())) };
        map.deserialize(deserializer)
    }
}

//////////////////////////////////////////////////////////////////////////////

struct NodeSeed<T> {
    map: Rc<RefCell<IdToShared<T>>>,
}

impl AsMut<NodeMap> for NodeSeed<Rc<Node>> {
    fn as_mut(&mut self) -> &mut NodeMap {
        &mut self.map
    }
}

//////////////////////////////////////////////////////////////////////////////
struct VariantSeed {
    map: NodeMap,
}

impl AsMut<NodeMap> for VariantSeed {
    fn as_mut(&mut self) -> &mut NodeMap {
        &mut self.map
    }
}

#[derive(DeserializeSeed)]
#[serde(deserialize_seed = "VariantSeed", rename = "Node")]
enum Variant {
    Plain {
        data: char,
        #[serde(deserialize_seed_with = "deserialize_option_node")]
        left: Option<Rc<Node>>,
        #[serde(deserialize_seed_with = "deserialize_option_node")]
        right: Option<Rc<Node>>,
    },
    Marked {
        id: u32,
        data: char,
        #[serde(deserialize_seed_with = "deserialize_option_node")]
        left: Option<Rc<Node>>,
        #[serde(deserialize_seed_with = "deserialize_option_node")]
        right: Option<Rc<Node>>,
    },
    Reference(u32),
}

#[test]
fn test_node_deserialize() {
    let b = Rc::new(
        Node {
            data: 'b',
            left: None,
            right: None,
        },
    );
    let a = Rc::new(
        Node {
            data: 'a',
            left: Some(b.clone()),
            right: Some(b.clone()),
        },
    );
    let map = Rc::new(RefCell::new(HashMap::new()));
    let seed = NodeSeed { map: map };
    assert_de_seed_tokens(
        seed,
        &a,
        &[
            Token::Struct {
                name: "Node",
                len: 3,
            },

            Token::Str("data"),
            Token::Char('a'),

            Token::Str("left"),
            Token::Some,

            Token::StructVariant {
                name: "Node",
                variant: "Marked",
                len: 4,
            },

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

            Token::NewtypeVariant {
                name: "Node",
                variant: "Reference",
            },
            Token::U32(0),

            Token::StructEnd,
        ],
    );
}
