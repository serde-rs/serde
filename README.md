Serde Rust Serialization Framework
==================================

[![Build Status](https://api.travis-ci.org/serde-rs/serde.svg?branch=master)](https://travis-ci.org/serde-rs/serde)
[![Coverage Status](https://coveralls.io/repos/serde-rs/serde/badge.svg?branch=master&service=github)](https://coveralls.io/github/serde-rs/serde?branch=master)
[![Latest Version](https://img.shields.io/crates/v/serde.svg)](https://crates.io/crates/serde)
[![Clippy Linting Result](https://clippy.bashy.io/github/serde-rs/serde/master/badge.svg)](https://clippy.bashy.io/github/serde-rs/serde/master/log)

Serde is a powerful framework that enables serialization libraries to
generically serialize Rust data structures without the overhead of runtime type
information. In many situations, the handshake protocol between serializers and
serializees can be completely optimized away, leaving Serde to perform roughly
the same speed as a hand written serializer for a specific type.

[Documentation](https://serde-rs.github.io/serde/serde/index.html)

Simple Serde Example
====================

Here is a simple example that uses
[serde_json](https://github.com/serde-rs/json), which uses Serde under the
covers, to generate and parse JSON. First, lets start off with the `Cargo.toml`
file:

```toml
[package]
name = "serde_example"
version = "0.1.0"
authors = ["Erick Tryzelaar <erick.tryzelaar@gmail.com>"]

[dependencies]
serde_json = "*"
```

Next, the `src/main.rs` file itself:

```rust,ignore
extern crate serde_json;

use std::collections::HashMap;
use serde_json::Value;
use serde_json::builder::{ArrayBuilder, ObjectBuilder};

fn main() {
    // Serde has support for many of the builtin Rust types, like arrays..:
    let v = vec![1, 2];
    let serialized = serde_json::to_string(&v).unwrap();
    println!("serialized vec: {:?}", serialized);

    let deserialized: Vec<u32> = serde_json::from_str(&serialized).unwrap();
    println!("deserialized vec: {:?}", deserialized);

    // ... and maps:
    let mut map = HashMap::new();
    map.insert("x".to_string(), 1);
    map.insert("y".to_string(), 2);

    let serialized = serde_json::to_string(&map).unwrap();
    println!("serialized map: {:?}", serialized);

    let deserialized: HashMap<String, u32> = serde_json::from_str(&serialized).unwrap();
    println!("deserialized map: {:?}", deserialized);

    // It also can handle complex objects:
    let value = ObjectBuilder::new()
        .insert("int", 1)
        .insert("string", "a string")
        .insert("array", ArrayBuilder::new()
                .push(1)
                .push(2)
                .unwrap())
        .unwrap();

    let serialized = serde_json::to_string(&value).unwrap();
    println!("serialized value: {:?}", serialized);

    let deserialized: serde_json::Value = serde_json::from_str(&serialized).unwrap();
    println!("deserialized value: {:?}", deserialized);
}
```

This produces the following output when run:

```
% cargo run
serialized vec: "[1,2]"
deserialized vec: [1, 2]
serialized map: "{\"y\":2,\"x\":1}"
deserialized map: {"y": 2, "x": 1}
serialized value: "{\"array\":[1,2],\"int\":1,\"string\":\"a string\"}"
deserialized value: {"array":[1,2],"int":1,"string":"a string"}
```

Using Serde with Stable Rust and serde\_codegen
===============================================

The example before used `serde_json::Value` as the in-memory representation of
the JSON value, but it's also possible for Serde to serialize to and from
regular Rust types. However, the code to do this can be a bit complicated to
write. So instead, Serde also has some powerful code generation libraries that
work with Stable and Nightly Rust that eliminate much of the complexity of hand
rolling serialization and deserialization for a given type.

First lets see how we would use Stable Rust, which is currently a tad more
complicated than Nightly Rust due to having to work around compiler plugins
being unstable.  We will use `serde_codegen` which is based on the code
generation library [syntex](https://github.com/serde-rs/syntex). First we need
to setup the `Cargo.toml` that builds the project:

```toml
[package]
name = "serde_example"
version = "0.1.0"
authors = ["Erick Tryzelaar <erick.tryzelaar@gmail.com>"]
build = "build.rs"

[build-dependencies]
serde_codegen = "*"

[dependencies]
serde = "*"
serde_json = "*"
```

Next, we define our source file, `src/main.rs.in`. Note this is a different
extension than usual becaues we need to do code generation:

```rust,ignore
#[derive(Serialize, Deserialize, Debug)]
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let point = Point { x: 1, y: 2 };

    let serialized = serde_json::to_string(&point).unwrap();
    println!("{}", serialized);

    let deserialized: Point = serde_json::from_str(&serialized).unwrap();
    println!("{:?}", deserialized);
}
```

To finish up the main source code, we define a very simple `src/main.rs` that
uses the generated code.

`src/main.rs`:

```rust,ignore
extern crate serde;
extern crate serde_json;

include!(concat!(env!("OUT_DIR"), "/main.rs"));
```

The last step is to actually drive the code generation, with the `build.rs` script:

```rust,ignore
extern crate serde_codegen;

use std::env;
use std::path::Path;

pub fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    let src = Path::new("src/main.rs.in");
    let dst = Path::new(&out_dir).join("main.rs");

    serde_codegen::expand(&src, &dst).unwrap();
}
```

All this produces this when run:

```
% cargo run
{"x":1,"y":2}
Point { x: 1, y: 2 }
```

While this works well with Stable Rust, be aware that the error locations
currently are reported in the generated file instead of in the source file.

Using Serde with Nightly Rust and serde\_macros
===============================================

The prior example is a bit more complicated than it needs to be due to compiler
plugins being unstable. However, if you are already using Nightly Rust, you can
use `serde_macros`, which has a much simpler interface. First, here is the new
`Cargo.toml`:

```toml
[package]
name = "serde_example_nightly"
version = "0.1.0"
authors = ["Erick Tryzelaar <erick.tryzelaar@gmail.com>"]

[dependencies]
serde = "*"
serde_json = "*"
serde_macros = "*"
```

Note that it doesn't need a build script. Now the `src/main.rs`, which enables
the plugin feature, and registers the `serde_macros` plugin:

```rust
#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate serde_json;

#[derive(Serialize, Deserialize, Debug)]
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let point = Point { x: 1, y: 2 };

    let serialized = serde_json::to_string(&point).unwrap();
    println!("{}", serialized);

    let deserialized: Point = serde_json::from_str(&serialized).unwrap();
    println!("{:?}", deserialized);
}
```

This also produces the same output:

```
% cargo run
{"x":1,"y":2}
Point { x: 1, y: 2 }
```

You may find it easier to develop with Nightly Rust and `serde\_macros`, then
deploy with Stable Rust and `serde_codegen`. It's possible to combine both
approaches in one setup:

`Cargo.toml`:

```toml
[package]
name = "serde_example"
version = "0.1.0"
authors = ["Erick Tryzelaar <erick.tryzelaar@gmail.com>"]
build = "build.rs"

[features]
default = ["serde_codegen"]
nightly = ["serde_macros"]

[build-dependencies]
serde_codegen = { version = "*", optional = true }

[dependencies]
serde = "*"
serde_json = "*"
serde_macros = { version = "*", optional = true }
```

`build.rs`:

```rust,ignore
#[cfg(not(feature = "serde_macros"))]
mod inner {
    extern crate serde_codegen;

    use std::env;
    use std::path::Path;

    pub fn main() {
        let out_dir = env::var_os("OUT_DIR").unwrap();

        let src = Path::new("src/main.rs.in");
        let dst = Path::new(&out_dir).join("main.rs");

        serde_codegen::expand(&src, &dst).unwrap();
    }
}

#[cfg(feature = "serde_macros")]
mod inner {
    pub fn main() {}
}

fn main() {
    inner::main();
}
```

`src/main.rs`:

```rust,ignore
#![cfg_attr(feature = "serde_macros", feature(custom_derive, plugin))]
#![cfg_attr(feature = "serde_macros", plugin(serde_macros))]

extern crate serde;
extern crate serde_json;

#[cfg(feature = "serde_macros")]
include!("main.rs.in");

#[cfg(not(feature = "serde_macros"))]
include!(concat!(env!("OUT_DIR"), "/main.rs"));
```

The `src/main.rs.in` is the same as before.

Then to run with stable:

```
% cargo build
...
```

Or with nightly:

```
% cargo build --features nightly --no-default-features
...
```

Serialization without Macros
============================

Under the covers, Serde extensively uses the Visitor pattern to thread state
between the
[Serializer](http://serde-rs.github.io/serde/serde/serde/ser/trait.Serializer.html)
and
[Serialize](http://serde-rs.github.io/serde/serde/serde/ser/trait.Serialize.html)
without the two having specific information about each other's concrete type.
This has many of the same benefits as frameworks that use runtime type
information without the overhead.  In fact, when compiling with optimizations,
Rust is able to remove most or all the visitor state, and generate code that's
nearly as fast as a hand written serializer format for a specific type.

To see it in action, lets look at how a simple type like `i32` is serialized.
The
[Serializer](http://serde-rs.github.io/serde/serde/serde/ser/trait.Serializer.html)
is threaded through the type:

```rust,ignore
impl serde::Serialize for i32 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer,
    {
        serializer.serialize_i32(*self)
    }
}
```

As you can see it's pretty simple. More complex types like `BTreeMap` need to
pass a
[MapVisitor](http://serde-rs.github.io/serde/serde/serde/ser/trait.MapVisitor.html)
to the
[Serializer](http://serde-rs.github.io/serde/serde/serde/ser/trait.Serializer.html)
in order to walk through the type:

```rust,ignore
impl<K, V> Serialize for BTreeMap<K, V>
    where K: Serialize + Ord,
          V: Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        serializer.serialize_map(MapIteratorVisitor::new(self.iter(), Some(self.len())))
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
                let value = try!(serializer.serialize_map_elt(key, value));
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
as a named map. Its visitor uses a simple state machine to iterate through all
the fields:

```rust
extern crate serde;
extern crate serde_json;

struct Point {
    x: i32,
    y: i32,
}

impl serde::Serialize for Point {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        serializer.serialize_struct("Point", PointMapVisitor {
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
                Ok(Some(try!(serializer.serialize_struct_elt("x", &self.value.x))))
            }
            1 => {
                self.state += 1;
                Ok(Some(try!(serializer.serialize_struct_elt("y", &self.value.y))))
            }
            _ => {
                Ok(None)
            }
        }
    }
}

fn main() {
    let point = Point { x: 1, y: 2 };
    let serialized = serde_json::to_string(&point).unwrap();

    println!("{}", serialized);
}
```

Deserialization without Macros
==============================

Deserialization is a little more complicated since there's a bit more error
handling that needs to occur. Let's start with the simple `i32`
[Deserialize](http://serde-rs.github.io/serde/serde/serde/de/trait.Deserialize.html)
implementation. It passes a
[Visitor](http://serde-rs.github.io/serde/serde/serde/de/trait.Visitor.html) to the
[Deserializer](http://serde-rs.github.io/serde/serde/serde/de/trait.Deserializer.html).
The [Visitor](http://serde-rs.github.io/serde/serde/serde/de/trait.Visitor.html)
can create the `i32` from a variety of different types:

```rust,ignore
impl Deserialize for i32 {
    fn deserialize<D>(deserializer: &mut D) -> Result<i32, D::Error>
        where D: serde::Deserializer,
    {
        deserializer.deserialize(I32Visitor)
    }
}

struct I32Visitor;

impl serde::de::Visitor for I32Visitor {
    type Value = i32;

    fn visit_i16<E>(&mut self, value: i16) -> Result<i32, E>
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
[Error](http://serde-rs.github.io/serde/serde/serde/de/trait.Error.html) trait,
which allows a
[Deserialize](http://serde-rs.github.io/serde/serde/serde/de/trait.Deserialize.html)
to generate an error for a few common error conditions. Here's how it could be used:

```rust,ignore
    ...

    fn visit_string<E>(&mut self, _: String) -> Result<i32, E>
        where E: Error,
    {
        Err(serde::de::Error::custom("expect a string"))
    }

    ...

```

Maps follow a similar pattern as before, and use a
[MapVisitor](http://serde-rs.github.io/serde/serde/serde/de/trait.MapVisitor.html)
to walk through the values generated by the
[Deserializer](http://serde-rs.github.io/serde/serde/serde/de/trait.Deserializer.html).

```rust,ignore
impl<K, V> serde::Deserialize for BTreeMap<K, V>
    where K: serde::Deserialize + Eq + Ord,
          V: serde::Deserialize,
{
    fn deserialize<D>(deserializer: &mut D) -> Result<BTreeMap<K, V>, D::Error>
        where D: serde::Deserializer,
    {
        deserializer.deserialize(BTreeMapVisitor::new())
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
extern crate serde;
extern crate serde_json;

#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

enum PointField {
    X,
    Y,
}

impl serde::Deserialize for PointField {
    fn deserialize<D>(deserializer: &mut D) -> Result<PointField, D::Error>
        where D: serde::de::Deserializer
    {
        struct PointFieldVisitor;

        impl serde::de::Visitor for PointFieldVisitor {
            type Value = PointField;

            fn visit_str<E>(&mut self, value: &str) -> Result<PointField, E>
                where E: serde::de::Error
            {
                match value {
                    "x" => Ok(PointField::X),
                    "y" => Ok(PointField::Y),
                    _ => Err(serde::de::Error::custom("expected x or y")),
                }
            }
        }

        deserializer.deserialize(PointFieldVisitor)
    }
}

impl serde::Deserialize for Point {
    fn deserialize<D>(deserializer: &mut D) -> Result<Point, D::Error>
        where D: serde::de::Deserializer
    {
        static FIELDS: &'static [&'static str] = &["x", "y"];
        deserializer.deserialize_struct("Point", FIELDS, PointVisitor)
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
                Some(PointField::X) => { x = Some(try!(visitor.visit_value())); }
                Some(PointField::Y) => { y = Some(try!(visitor.visit_value())); }
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


fn main() {
    let serialized = "{\"x\":1,\"y\":2}";

    let deserialized: Point = serde_json::from_str(&serialized).unwrap();

    println!("{:?}", deserialized);
}
```

Design Considerations and tradeoffs for Serializers and Deserializers
=====================================================================

Serde serialization and deserialization implementations are written in such a
way that they err on being able to represent more values, and also provide
better error messages when they are passed an incorrect type to deserialize
from. For example, by default, it is a syntax error to deserialize a `String`
into an `Option<String>`. This is implemented such that it is possible to
distinguish between the values `None` and `Some(())`, if the serialization
format supports option types.

However, many formats do not have option types, and represents optional values
as either a `null`, or some other value. Serde `Serializer`s and
`Deserializer`s can opt-in support for this. For serialization, this is pretty
easy. Simply implement these methods:

```rust,ignore
...

    fn visit_none(&mut self) -> Result<(), Self::Error> {
        self.visit_unit()
    }

    fn visit_some<T>(&mut self, value: T) -> Result<(), Self::Error> {
        value.serialize(self)
    }
...
```

For deserialization, this can be implemented by way of the
`Deserializer::visit_option` hook, which presumes that there is some ability to peek at what is the
next value in the serialized token stream. This following example is from
[serde_tests::TokenDeserializer](https://github.com/serde-rs/serde/blob/master/serde_tests/tests/token.rs#L435-L454),
where it checks to see if the next value is an `Option`, a `()`, or some other
value:

```rust,ignore
...

    fn visit_option<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        match self.tokens.peek() {
            Some(&Token::Option(false)) => {
                self.tokens.next();
                visitor.visit_none()
            }
            Some(&Token::Option(true)) => {
                self.tokens.next();
                visitor.visit_some(self)
            }
            Some(&Token::Unit) => {
                self.tokens.next();
                visitor.visit_none()
            }
            Some(_) => visitor.visit_some(self),
            None => Err(Error::EndOfStreamError),
        }
    }

...
```

Annotations
===========

`serde_codegen` and `serde_macros` support annotations that help to customize
how types are serialized. Here are the supported annotations:

Container Annotations:

| Annotation                                  | Function                                                                                                                                           |
| ----------                                  | --------                                                                                                                                           |
| `#[serde(rename="name")]`                   | Serialize and deserialize this container with the given name                                                                                       |
| `#[serde(rename(serialize="name1"))]`       | Serialize this container with the given name                                                                                                       |
| `#[serde(rename(deserialize="name1"))]`     | Deserialize this container with the given name                                                                                                     |
| `#[serde(deny_unknown_fields)]`             | Always error during serialization when encountering unknown fields. When absent, unknown fields are ignored for self-describing formats like JSON. |
| `#[serde(bound="T: MyTrait")]`              | Where-clause for the Serialize and Deserialize impls. This replaces any bounds inferred by Serde.                                                  |
| `#[serde(bound(serialize="T: MyTrait"))]`   | Where-clause for the Serialize impl.                                                                                                               |
| `#[serde(bound(deserialize="T: MyTrait"))]` | Where-clause for the Deserialize impl.                                                                                                             |

Variant Annotations:

| Annotation                              | Function                                                   |
| ----------                              | --------                                                   |
| `#[serde(rename="name")]`               | Serialize and deserialize this variant with the given name |
| `#[serde(rename(serialize="name1"))]`   | Serialize this variant with the given name                 |
| `#[serde(rename(deserialize="name1"))]` | Deserialize this variant with the given name               |

Field Annotations:

| Annotation                                  | Function                                                                                                                |
| ----------                                  | --------                                                                                                                |
| `#[serde(rename="name")]`                   | Serialize and deserialize this field with the given name                                                                |
| `#[serde(rename(serialize="name1"))]`       | Serialize this field with the given name                                                                                |
| `#[serde(rename(deserialize="name1"))]`     | Deserialize this field with the given name                                                                              |
| `#[serde(default)]`                         | If the value is not specified, use the `Default::default()`                                                             |
| `#[serde(default="$path")]`                 | Call the path to a function `fn() -> T` to build the value                                                              |
| `#[serde(skip_serializing)]`                | Do not serialize this value                                                                                             |
| `#[serde(skip_deserializing)]`              | Always use `Default::default()` or `#[serde(default="$path")]` instead of deserializing this value                      |
| `#[serde(skip_serializing_if="$path")]`     | Do not serialize this value if this function `fn(&T) -> bool` returns `true`                                            |
| `#[serde(serialize_with="$path")]`          | Call a function `fn<S>(&T, &mut S) -> Result<(), S::Error> where S: Serializer` to serialize this value of type `T`     |
| `#[serde(deserialize_with="$path")]`        | Call a function `fn<D>(&mut D) -> Result<T, D::Error> where D: Deserializer` to deserialize this value of type `T`      |
| `#[serde(bound="T: MyTrait")]`              | Where-clause for the Serialize and Deserialize impls. This replaces any bounds inferred by Serde for the current field. |
| `#[serde(bound(serialize="T: MyTrait"))]`   | Where-clause for the Serialize impl.                                                                                    |
| `#[serde(bound(deserialize="T: MyTrait"))]` | Where-clause for the Deserialize impl.                                                                                  |

Using in `no_std` crates
========================

The core `serde` package defines a number of features to enable usage in a
variety of freestanding environments. Enable any or none of the following
features, and use `default-features = false` in your `Cargo.toml`:

- `alloc` (implies `nightly`)
- `collections` (implies `alloc` and `nightly`)
- `std` (default)

If you only use `default-features = false`, you will receive a stock `no_std`
serde with no support for any of the collection types.

Upgrading from Serde 0.6
========================

* `#[serde(skip_serializing_if_none)]` was replaced with `#[serde(skip_serializing_if="Option::is_none")]`.
* `#[serde(skip_serializing_if_empty)]` was replaced with `#[serde(skip_serializing_if="Vec::is_empty")]`.

Serialization Formats Using Serde
=================================

| Format      | Name                                                 |
| ------      | ----                                                 |
| Bincode     | [bincode](https://crates.io/crates/bincode)          |
| env vars    | [envy](https://crates.io/crates/envy)                |
| Hjson       | [serde\_hjson](https://crates.io/crates/serde-hjson) |
| JSON        | [serde\_json](https://crates.io/crates/serde_json)   |
| MessagePack | [rmp](https://crates.io/crates/rmp)                  |
| XML         | [serde\_xml](https://github.com/serde-rs/xml)        |
| YAML        | [serde\_yaml](https://github.com/dtolnay/serde-yaml) |
