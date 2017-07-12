#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_derive_seed;
extern crate serde;
extern crate serde_test;

use std::cell::Cell;

use serde::Serialize;
use serde::ser::{Seeded, SerializeSeed};

use serde_test::{Token, assert_ser_tokens};

#[derive(Serialize)]
struct Inner;

impl SerializeSeed for Inner {
    type Seed = Cell<i32>;

    fn serialize_seed<S>(&self, serializer: S, seed: &Self::Seed) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        seed.set(seed.get() + 1);
        self.serialize(serializer)
    }
}

#[derive(SerializeSeed)]
#[serde(serialize_seed = "Cell<i32>")]
struct SeedStruct {
    #[serde(serialize_seed)]
    value: Inner,
}

#[test]
fn test_serialize_seed() {
    let value = SeedStruct { value: Inner };
    let seed = Cell::new(0);
    assert_ser_tokens(
        &Seeded::new(&seed, &value),
        &[
            Token::Struct {
                name: "SeedStruct",
                len: 1,
            },

            Token::Str("value"),
            Token::UnitStruct { name: "Inner" },

            Token::StructEnd,
        ],
    );

    assert_eq!(seed.get(), 1);
}

#[test]
fn test_serialize_vec_seed() {
    let value = [SeedStruct { value: Inner }, SeedStruct { value: Inner }];
    let seed = Cell::new(0);
    assert_ser_tokens(
        &Seeded::new(&seed, &value[..]),
        &[
            Token::Seq { len: Some(2) },

            Token::Struct {
                name: "SeedStruct",
                len: 1,
            },

            Token::Str("value"),
            Token::UnitStruct { name: "Inner" },

            Token::StructEnd,

            Token::Struct {
                name: "SeedStruct",
                len: 1,
            },

            Token::Str("value"),
            Token::UnitStruct { name: "Inner" },

            Token::StructEnd,

            Token::SeqEnd,
        ],
    );

    assert_eq!(seed.get(), 2);
}

#[test]
fn test_serialize_option_some_seed() {
    let value = Some(SeedStruct { value: Inner });
    let seed = Cell::new(0);
    assert_ser_tokens(
        &Seeded::new(&seed, &value),
        &[
            Token::Some,

            Token::Struct {
                name: "SeedStruct",
                len: 1,
            },

            Token::Str("value"),
            Token::UnitStruct { name: "Inner" },

            Token::StructEnd,
        ],
    );

    assert_eq!(seed.get(), 1);
}

#[test]
fn test_serialize_option_none_seed() {
    let value: Option<SeedStruct> = None;
    let seed = Cell::new(0);
    assert_ser_tokens(&Seeded::new(&seed, &value), &[Token::None]);

    assert_eq!(seed.get(), 0);
}

#[derive(SerializeSeed)]
#[serde(serialize_seed = "Cell<i32>")]
enum SeedEnum {
    A(
        #[serde(serialize_seed)]
        Inner
    ),
    B {
        #[serde(serialize_seed)]
        inner: Inner,
    },
}

#[test]
fn test_serialize_seed_newtype_variant() {
    let value = SeedEnum::A(Inner);
    let seed = Cell::new(0);
    assert_ser_tokens(
        &Seeded::new(&seed, &value),
        &[
            Token::NewtypeVariant {
                name: "SeedEnum",
                variant: "A",
            },

            Token::UnitStruct { name: "Inner" },
        ],
    );

    assert_eq!(seed.get(), 1);
}

#[test]
fn test_serialize_seed_newtype_variant2() {
    let value = SeedEnum::B { inner: Inner };
    let seed = Cell::new(0);
    assert_ser_tokens(
        &Seeded::new(&seed, &value),
        &[
            Token::StructVariant {
                name: "SeedEnum",
                variant: "B",
                len: 1,
            },

            Token::Str("inner"),
            Token::UnitStruct { name: "Inner" },

            Token::StructVariantEnd,
        ],
    );

    assert_eq!(seed.get(), 1);
}

#[derive(SerializeSeed)]
#[serde(serialize_seed = "Cell<i32>")]
struct GenericNewtype<T>(
    #[serde(serialize_seed)]
    T
);

#[test]
fn test_serialize_seed_generic_newtype() {
    let value = GenericNewtype(Inner);
    let seed = Cell::new(0);
    assert_ser_tokens(
        &Seeded::new(&seed, &value),
        &[
            Token::NewtypeStruct {
                name: "GenericNewtype",
            },

            Token::UnitStruct { name: "Inner" },
        ],
    );

    assert_eq!(seed.get(), 1);
}


#[derive(SerializeSeed)]
#[serde(serialize_seed = "Cell<i32>")]
enum GenericSeedEnum<T> {
    A(
        #[serde(serialize_seed)]
        T
    ),
    B {
        #[serde(serialize_seed)]
        inner: T,
    },
}

#[test]
fn test_serialize_seed_generic_newtype_variant() {
    let value = GenericSeedEnum::A(Inner);
    let seed = Cell::new(0);
    assert_ser_tokens(
        &Seeded::new(&seed, &value),
        &[
            Token::NewtypeVariant {
                name: "GenericSeedEnum",
                variant: "A",
            },

            Token::UnitStruct { name: "Inner" },
        ],
    );

    assert_eq!(seed.get(), 1);
}

#[test]
fn test_serialize_seed_generic_struct_variant() {
    let value = GenericSeedEnum::B { inner: Inner };
    let seed = Cell::new(0);
    assert_ser_tokens(
        &Seeded::new(&seed, &value),
        &[
            Token::StructVariant {
                name: "GenericSeedEnum",
                variant: "B",
                len: 1
            },

            Token::Str("inner"),
            Token::UnitStruct { name: "Inner" },

            Token::StructVariantEnd
        ],
    );

    assert_eq!(seed.get(), 1);
}


use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use serde::Serializer;

#[derive(Debug, SerializeSeed)]
#[serde(serialize_seed = "RefCell<NodeToId>")]
struct Node {
    data: char,
    #[serde(serialize_seed_with = "serialize_option_rc_seed")]
    left: Option<Rc<Node>>,
    #[serde(serialize_seed_with = "serialize_option_rc_seed")]
    right: Option<Rc<Node>>,
}

/// ```
///   A
///  / \
/// (   B
///  \ / \
///   C   )
///  / \ /
/// D   E
/// ```
fn example() -> Node {
    let e = Rc::new(
        Node {
            data: 'E',
            left: None,
            right: None,
        },
    );
    let d = Rc::new(
        Node {
            data: 'D',
            left: None,
            right: None,
        },
    );
    let c = Rc::new(
        Node {
            data: 'C',
            left: Some(d),
            right: Some(e.clone()),
        },
    );
    let b = Rc::new(
        Node {
            data: 'B',
            left: Some(c.clone()),
            right: Some(e),
        },
    );
    Node {
        data: 'A',
        left: Some(c),
        right: Some(b),
    }
}

#[test]
fn serialize_node_graph() {
    let e = Rc::new(
        Node {
            data: 'E',
            left: None,
            right: None,
        },
    );
    let d = Rc::new(
        Node {
            data: 'D',
            left: Some(e.clone()),
            right: None,
        },
    );
    let c = Rc::new(
        Node {
            data: 'C',
            left: Some(d.clone()),
            right: Some(e.clone()),
        },
    );

    let seed = RefCell::default();
    assert_ser_tokens(
        &Tracked {
             node: &c,
             map: &seed,
         },
        &[
            Token::StructVariant {
                name: "Node",
                len: 3,
                variant: "Plain",
            },

            Token::Str("data"),
            Token::Char('C'),

            Token::Str("left"),
            Token::Some,

            // D {
            Token::StructVariant {
                name: "Node",
                len: 4,
                variant: "Marked",
            },

            Token::Str("id"),
            Token::U32(0),

            Token::Str("data"),
            Token::Char('D'),

            Token::Str("left"),
            Token::Some,
            // E {
            Token::StructVariant {
                name: "Node",
                len: 4,
                variant: "Marked",
            },

            Token::Str("id"),
            Token::U32(1),

            Token::Str("data"),
            Token::Char('E'),

            Token::Str("left"),
            Token::None,

            Token::Str("right"),
            Token::None,

            Token::StructVariantEnd,
            // E }
            Token::Str("right"),
            Token::None,

            Token::StructVariantEnd,
            // D }
            Token::Str("right"),
            Token::Some,

            Token::NewtypeVariant {
                name: "Node",
                variant: "Reference",
            },
            Token::U32(1),

            Token::StructVariantEnd,
        ],
    );
}

#[test]
fn check_graph() {
    let ref a = example();
    let b = a.right.as_ref().unwrap();
    let c = a.left.as_ref().unwrap();
    let d = c.left.as_ref().unwrap();
    let e = c.right.as_ref().unwrap();
    assert_eq!('A', a.data);
    assert_eq!('B', b.data);
    assert_eq!('C', c.data);
    assert_eq!('D', d.data);
    assert_eq!('E', e.data);
    assert_eq!(
        &**c as *const Node,
        &**b.left.as_ref().unwrap() as *const Node
    );
    assert_eq!(
        &**e as *const Node,
        &**b.right.as_ref().unwrap() as *const Node
    );
}

type Id = u32;
type NodeToId = HashMap<*const Node, Id>;

enum Lookup {
    Unique,
    Found(Id),
    Inserted(Id),
}

fn node_to_id(map: &RefCell<NodeToId>, node: &Rc<Node>) -> Lookup {
    if Rc::strong_count(node) == 1 {
        return Lookup::Unique;
    }
    let mut map = map.borrow_mut();
    if let Some(id) = map.get(&(&**node as *const Node)) {
        return Lookup::Found(*id);
    }
    let id = map.len() as Id;
    map.insert(&**node, id);
    Lookup::Inserted(id)
}

struct Tracked<'a> {
    node: &'a Rc<Node>,
    map: &'a RefCell<NodeToId>,
}

impl<'a> Serialize for Tracked<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_rc_seed(self.node, serializer, self.map)
    }
}

impl Serialize for Node {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.serialize_seed(serializer, &RefCell::default())
    }
}

fn serialize_option_rc_seed<S>(
    self_: &Option<Rc<Node>>,
    serializer: S,
    map: &RefCell<NodeToId>,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    self_
        .as_ref()
        .map(|n| Tracked { node: n, map: map })
        .serialize(serializer)
}

fn serialize_rc_seed<S>(
    self_: &Rc<Node>,
    serializer: S,
    map: &RefCell<NodeToId>,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    #[derive(Debug, SerializeSeed)]
    #[serde(serialize_seed = "RefCell<NodeToId>")]
    #[serde(rename = "Node")]
    enum NodeVariant<'a> {
        Plain {
            data: char,
            #[serde(serialize_seed_with = "serialize_option_rc_seed")]
            left: &'a Option<Rc<Node>>,
            #[serde(serialize_seed_with = "serialize_option_rc_seed")]
            right: &'a Option<Rc<Node>>,
        },
        Reference(Id),
        Marked {
            id: Id,
            data: char,
            #[serde(serialize_seed_with = "serialize_option_rc_seed")]
            left: &'a Option<Rc<Node>>,
            #[serde(serialize_seed_with = "serialize_option_rc_seed")]
            right: &'a Option<Rc<Node>>,
        },
    }

    let node = match node_to_id(map, self_) {
        Lookup::Unique => {
            NodeVariant::Plain {
                data: self_.data,
                left: &self_.left,
                right: &self_.right,
            }
        }
        Lookup::Found(id) => NodeVariant::Reference(id),
        Lookup::Inserted(id) => {
            NodeVariant::Marked {
                id: id,
                data: self_.data,
                left: &self_.left,
                right: &self_.right,
            }
        }
    };
    node.serialize_seed(serializer, map)
}
