Serde Rust Serialization Framework
==================================

[![Build Status](https://travis-ci.org/erickt/rust-serde.png?branch=master)](https://travis-ci.org/erickt/rust-serde)

Serde is a powerful framework that enables serialization libraries to
generically serialize Rust data structures without the overhead of runtime type
information. In many situations, the handshake protocol between serializers and
serializees can be completely optimized away, leaving serde to perform roughly
the same speed as a hand written serializer for a specific type.

Documentation is available at http://erickt.github.io/rust-serde/serde

Example
=======

Serde works by threading visitors between the serializer and the serializee.
This allows data to be generically shared between the two without needing to
wrap the values in a separate structure. Here's an example struct serializer.
It works by reinterpreting the the structure as a named map, with the keys
being the stringified field name, and a simple state machine to step
through each field:

```rust
struct Point {
    x: i32,
    y: i32,
}

impl serde::Serialize for Point {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serialize::Serializer
    {
        struct MapVisitor<'a> {
            value: &'a Point,
            state: u8,
        }

        impl<'a> serde::ser::MapVisitor for MapVisitor {
            fn visit<S>(&mut self, serializer: &mut S) -> Result<Option(), S::Error> {
                match self.state {
                    0 => {
                        self.state += 1;
                        Ok(Some(try!(serializer.visit_map_elt("x", &self.x)))
                    }
                    1 => {
                        self.state += 1;
                        Ok(Some(try!(serializer.visit_map_elt("y", &self.y))))
                    }
                    _ => {
                        Ok(None)
                    }
                }
            }
        }

        serializer.visit_named_map("Point", MapVisitor {
            value: self,
            state: 0,
        })
    }
}
```

Deserialization is a bit more tricky. We need to deserialize a field from a string, but in order to
avoid some borrow checker issues and in desire to avoid allocations, we deserialize field names
into an enum:

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

        deserializer.visit_named_map("Point", PointVisitor)
    }
}

```

There's a bit of machinery required to write implementations of `Serialize` and
`Deserialize`. Fortunately it is not necessary in most circumstances. Instead,
it's much easier to use the `serde_macros` plugin. The prior code can be
rewritten as:

```rust
#![feature(custom_derive)]
#![plugin(serde_macros)]

extern crate serde;

#[derive(Serialize, Deserialize)]
struct Point {
    x: i32,
    y: i32,
}
```
