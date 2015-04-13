Serde Rust Serialization Framework
==================================

[![Build Status](https://travis-ci.org/erickt/rust-serde.png?branch=master)](https://travis-ci.org/erickt/rust-serde)

Serde is a powerful framework that enables serialization libraries to
generically serialize Rust data structures without the overhead of runtime type
information. In many situations, the handshake protocol between serializers and
serializees can be completely optimized away, leaving Serde to perform roughly
the same speed as a hand written serializer for a specific type.

Documentation is available at http://erickt.github.io/rust-serde/serde

Making a Type Serializable
==========================

The simplest way to make a type serializable is to use the `serde_macros`
syntax extension, which comes with a `#[derive(Serialize, Deserialize)]`
annotation, which automatically generates implementations of 
[Serialize](http://erickt.github.io/rust-serde/serde/ser/trait.Serialize.html)
and
[Deserialize](http://erickt.github.io/rust-serde/serde/de/trait.Deserialize.html)
for the annotated type:

```rust
#[feature(custom_derive, plugin)]
#[plugin(serde_macros)]

extern crate serde;

...

#[derive(Serialize, Deserialize)]
struct Point {
    x: i32,
    y: i32,
}
```

Serde bundles a high performance JSON serializer and deserializer,
[serde::json](http://erickt.github.io/rust-serde/serde/json/index.html),
which comes with the helper functions
[to_string](http://erickt.github.io/rust-serde/serde/json/ser/fn.to_string.html)
and
[from_str](http://erickt.github.io/rust-serde/serde/json/de/fn.from_str.html)
that make it easy to go to and from JSON:

```rust
use serde::json;

...

let point = Point { x: 1, y: 2 };
let serialized_point = json::to_string(&point).unwrap();

println!("{}", serialized_point); // prints: {"x":1,"y":2}

let deserialize_point: Point = json::from_str(&serialized_point).unwrap();
```

[serde::json](http://erickt.github.io/rust-serde/serde/json/index.html) also
supports a generic
[Value](http://erickt.github.io/rust-serde/serde/json/value/enum.Value.html)
type, which can represent any JSON value. Also, any
[Serialize](http://erickt.github.io/rust-serde/serde/ser/trait.Serialize.html)
and
[Deserialize](http://erickt.github.io/rust-serde/serde/de/trait.Deserialize.html)
can be converted into a 
[Value](http://erickt.github.io/rust-serde/serde/json/value/enum.Value.html)
with the methods
[to_value](http://erickt.github.io/rust-serde/serde/json/value/fn.to_value.html)
and
[from_value](http://erickt.github.io/rust-serde/serde/json/value/fn.from_value.html):

```rust
let point = Point { x: 1, y: 2 };
let point_value = json::to_value(&point).unwrap();

println!("{}", point_value.find("x")); // prints: Some(1)

let deserialize_point: Point = json::from_value(point_value).unwrap();
```

Serialization without Macros
============================

Under the covers, Serde extensively uses the Visitor pattern to thread state
between the
[Serializer](http://erickt.github.io/rust-serde/serde/ser/trait.Serializer.html)
and
[Serialize](http://erickt.github.io/rust-serde/serde/ser/trait.Serialize.html)
without the two having specific information about each other's concrete type.
This has many of the same benefits as frameworks that use runtime type
information without the overhead.  In fact, when compiling with optimizations,
Rust is able to remove most or all the visitor state, and generate code that's
nearly as fast as a hand written serializer format for a specific type.

To see it in action, lets look at how a simple type like `i32` is serialized.
The
[Serializer](http://erickt.github.io/rust-serde/serde/ser/trait.Serializer.html)
is threaded through the type:

```rust
impl serde::Serialize for i32 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer,
    {
        serializer.visit_i32(*self)
    }
}
```

As you can see it's pretty simple. More complex types like `BTreeMap` need to
pass a
[MapVisitor](http://erickt.github.io/rust-serde/serde/ser/trait.MapVisitor.html)
to the 
[Serializer](http://erickt.github.io/rust-serde/serde/ser/trait.Serializer.html)
in order to walk through the type:

```rust
impl<K, V> Serialize for BTreeMap<K, V>
    where K: Serialize + Ord,
          V: Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        serializer.visit_map(MapIteratorVisitor::new(self.iter(), Some(self.len())))
    }
}

pub struct MapIteratorVisitor<Iter> {
    iter: Iter,
    len: Option<usize>,
}

impl<K, V, Iter> MapIteratorVisitor<Iter>
    where Iter: Iterator<Item=(K, V)>
{
    #[inline]
    pub fn new(iter: Iter, len: Option<usize>) -> MapIteratorVisitor<Iter> {
        MapIteratorVisitor {
            iter: iter,
            len: len,
        }
    }
}

impl<K, V, I> MapVisitor for MapIteratorVisitor<I>
    where K: Serialize,
          V: Serialize,
          I: Iterator<Item=(K, V)>,
{
    #[inline]
    fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
        where S: Serializer,
    {
        match self.iter.next() {
            Some((key, value)) => {
                let value = try!(serializer.visit_map_elt(key, value));
                Ok(Some(value))
            }
            None => Ok(None)
        }
    }

    #[inline]
    fn len(&self) -> Option<usize> {
        self.len
    }
}
```

Serializing structs follow this same pattern. In fact, structs are represented
as a named map. It's visitor uses a simple state machine to iterate through all
the fields:

```rust
struct Point {
    x: i32,
    y: i32,
}

impl serde::Serialize for Point {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        serializer.visit_named_map("Point", PointMapVisitor {
            value: self,
            state: 0,
        })
    }
}

struct PointMapVisitor<'a> {
    value: &'a Point,
    state: u8,
}

impl<'a> serde::ser::MapVisitor for PointMapVisitor<'a> {
    fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
        where S: serde::Serializer
    {
        match self.state {
            0 => {
                self.state += 1;
                Ok(Some(try!(serializer.visit_map_elt("x", &self.value.x))))
            }
            1 => {
                self.state += 1;
                Ok(Some(try!(serializer.visit_map_elt("y", &self.value.y))))
            }
            _ => {
                Ok(None)
            }
        }
    }
}
```

Deserialization without Macros
==============================

Deserialization is a little more complicated since there's a bit more error
handling that needs to occur. Let's start with the simple `i32`
[Deserialize](http://erickt.github.io/rust-serde/serde/de/trait.Deserialize.html)
implementation. It passes a
[Visitor](http://erickt.github.io/rust-serde/serde/de/trait.Visitor.html) to the
[Deserializer](http://erickt.github.io/rust-serde/serde/de/trait.Deserializer.html).
The [Visitor](http://erickt.github.io/rust-serde/serde/de/trait.Visitor.html)
can create the `i32` from a variety of different types:

```rust
impl Deserialize for i32 {
    fn deserialize<D>(deserializer: &mut D) -> Result<$ty, D::Error>
        where D: serde::Deserializer,
    {
        deserializer.visit(I32Visitor)
    }
}

struct I32Visitor;

impl serde::de::Visitor for I32Visitor {
    type Value = i32;

    fn visit_i16<E>(&mut self, value: i16) -> Result<i16, E>
        where E: Error,
    {
        self.visit_i32(value as i32)
    }

    fn visit_i32<E>(&mut self, value: i32) -> Result<i32, E>
        where E: Error,
    {
        Ok(value)
    }

    ...

```

Since it's possible for this type to get passed an unexpected type, we need a
way to error out. This is done by way of the
[Error](http://erickt.github.io/rust-serde/serde/de/trait.Error.html) trait,
which allows a 
[Deserialize](http://erickt.github.io/rust-serde/serde/de/trait.Deserialize.html)
to generate an error for a few common error conditions. Here's how it could be used:

```rust
    ...

    fn visit_string<E>(&mut self, _: String) -> Result<i32, E>
        where E: Error,
    {
        Err(serde::de::Error::syntax_error())
    }

    ...

```

Maps follow a similar pattern as before, and use a
[MapVisitor](http://erickt.github.io/rust-serde/serde/de/trait.MapVisitor.html)
to walk through the values generated by the 
[Deserializer](http://erickt.github.io/rust-serde/serde/de/trait.Deserializer.html).

```rust
impl<K, V> serde::Deserialize for BTreeMap<K, V>
    where K: serde::Deserialize + Eq + Ord,
          V: serde::Deserialize,
{
    fn deserialize<D>(deserializer: &mut D) -> Result<BTreeMap<K, V>, D::Error>
        where D: serde::Deserializer,
    {
        deserializer.visit(BTreeMapVisitor::new())
    }
}

pub struct BTreeMapVisitor<K, V> {
    marker: PhantomData<BTreeMap<K, V>>,
}

impl<K, V> BTreeMapVisitor<K, V> {
    pub fn new() -> Self {
        BTreeMapVisitor {
            marker: PhantomData,
        }
    }
}

impl<K, V> serde::de::Visitor for BTreeMapVisitor<K, V>
    where K: serde::de::Deserialize + Ord,
          V: serde::de::Deserialize
{
    type Value = BTreeMap<K, V>;

    fn visit_unit<E>(&mut self) -> Result<BTreeMap<K, V>, E>
        where E: Error,
    {
        Ok(BTreeMap::new())
    }

    fn visit_map<V_>(&mut self, mut visitor: V_) -> Result<BTreeMap<K, V>, V_::Error>
        where V_: MapVisitor,
    {
        let mut values = BTreeMap::new();

        while let Some((key, value)) = try!(visitor.visit()) {
            values.insert(key, value);
        }

        try!(visitor.end());

        Ok(values)
    }
}

```

Deserializing structs goes a step further in order to support not allocating a
`String` to hold the field names. This is done by custom field enum that
deserializes an enum variant from a string. So for our `Point` example from
before, we need to generate:

```rust
enum PointField {
    X,
    Y,
}

impl serde::Deserialize for PointField {
    fn deserialize<D>(deserializer: &mut D) -> Result<PointField, D::Error>
        where D: serde::de::Deserializer
    {
        struct FieldVisitor;

        impl serde::de::Visitor for FieldVisitor {
            type Value = Field;

            fn visit_str<E>(&mut self, value: &str) -> Result<PointField, E>
                where E: serde::de::Error
            {
                match value {
                    "x" => Ok(Field::X),
                    "y" => Ok(Field::Y),
                    _ => Err(serde::de::Error::syntax_error()),
                }
            }
        }

        deserializer.visit(FieldVisitor)
    }
}
```

This is then used in our actual deserializer:

```rust
impl serde::Deserialize for Point {
    fn deserialize<D>(deserializer: &mut D) -> Result<Point, D::Error>
        where D: serde::de::Deserializer
    {
        deserializer.visit_named_map("Point", PointVisitor)
    }
}

struct PointVisitor;

impl serde::de::Visitor for PointVisitor {
    type Value = Point;

    fn visit_map<V>(&mut self, mut visitor: V) -> Result<Point, V::Error>
        where V: serde::de::MapVisitor
    {
        let mut x = None;
        let mut y = None;

        loop {
            match try!(visitor.visit_key()) {
                Some(Field::X) => { x = Some(try!(visitor.visit_value())); }
                Some(Field::Y) => { y = Some(try!(visitor.visit_value())); }
                None => { break; }
            }
        }

        let x = match x {
            Some(x) => x,
            None => try!(visitor.missing_field("x")),
        };

        let y = match y {
            Some(y) => y,
            None => try!(visitor.missing_field("y")),
        };

        try!(visitor.end());

        Ok(Point{ x: x, y: y })
    }
}
```
