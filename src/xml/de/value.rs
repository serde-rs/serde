
use xml::value::{Element, Content};
use xml::error::Error;
use de;
use std::{vec, mem};
use std::collections::btree_map;

pub struct Deserializer {
    value: Option<Element>,
}

impl Deserializer {
    /// Creates a new deserializer instance for deserializing the specified JSON value.
    pub fn new(value: Element) -> Deserializer {
        Deserializer {
            value: Some(value),
        }
    }
}
impl de::Deserializer for Deserializer {
    type Error = Error;

    #[inline]
    fn visit<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        use self::MapDeserializerState::*;
        println!("value::Deserializer::visit {:?}", self.value);
        let el = match self.value.take() {
            Some(value) => value,
            None => { return Err(de::Error::end_of_stream_error()); }
        };

        match (el.attributes.is_empty(), el.members) {
            (true, Content::Text(s)) => visitor.visit_string(s),
            (true, Content::Nothing) => visitor.visit_unit(),
            (_, m) => visitor.visit_map( MapDeserializer {
                attributes: el.attributes,
                state: Inner,
                members: m,
            }),
        }
    }

    #[inline]
    fn visit_option<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        println!("value::Deserializer::visit_option");
        let el = match self.value.take() {
            Some(value) => value,
            None => { return Err(de::Error::end_of_stream_error()); }
        };
        match &el.members {
            &Content::Nothing => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    #[inline]
    fn visit_enum<V>(&mut self, _name: &str, _visitor: V) -> Result<V::Value, Error>
        where V: de::EnumVisitor,
    {
        println!("value::Deserializer::visit_enum");
        unimplemented!()
    }

    #[inline]
    fn visit_map<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        use self::MapDeserializerState::*;
        println!("value::Deserializer::visit_map {:?}", self.value);
        let el = match self.value.take() {
            Some(value) => value,
            None => { return Err(de::Error::end_of_stream_error()); }
        };
        visitor.visit_map( MapDeserializer {
            attributes: el.attributes,
            state: Inner,
            members: el.members,
        })
    }
}

struct SeqDeserializer (vec::IntoIter<Element>);

impl de::Deserializer for SeqDeserializer {
    type Error = Error;

    #[inline]
    fn visit<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        if self.0.len() == 0 {
            visitor.visit_unit()
        } else {
            visitor.visit_seq(self)
        }
    }
}

impl de::SeqVisitor for SeqDeserializer {
    type Error = Error;

    fn visit<T>(&mut self) -> Result<Option<T>, Error>
        where T: de::Deserialize
    {
        match self.0.next() {
            Some(value) => {
                de::Deserialize::deserialize(&mut Deserializer::new(value))
                    .map(|v| Some(v))
            }
            None => Ok(None),
        }
    }

    fn end(&mut self) -> Result<(), Error> {
        if self.0.len() == 0 {
            Ok(())
        } else {
            Err(de::Error::end_of_stream_error())
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0.len(), Some(self.0.len()))
    }
}

struct StringDeserializer(Option<String>);

impl de::Deserializer for StringDeserializer {
    type Error = Error;

    #[inline]
    fn visit<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        visitor.visit_string(self.0.take().unwrap())
    }
}

#[derive(PartialEq)]
enum MapDeserializerState {
    Inner,
    Done,
}

struct MapDeserializer {
    attributes: btree_map::BTreeMap<String, String>,
    members: Content,
    state: MapDeserializerState,
}

impl de::MapVisitor for MapDeserializer {
    type Error = Error;

    fn visit_key<T>(&mut self) -> Result<Option<T>, Error>
        where T: de::Deserialize
    {
        Ok(None)
    }

    fn visit_value<T>(&mut self) -> Result<T, Error>
        where T: de::Deserialize
    {
        unreachable!()
    }

    fn end(&mut self) -> Result<(), Error> {
        println!("value::MapDeserializer::end");
        Ok(())
    }

    fn missing_field<V>(&mut self, field: &'static str) -> Result<V, Error>
        where V: de::Deserialize,
    {
        use self::MapDeserializerState::*;
        println!("value::MapDeserializer::missing_field {}", field);

        // See if the type can deserialize from a unit.
        struct UnitDeserializer;

        impl de::Deserializer for UnitDeserializer {
            type Error = Error;

            fn visit<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
                where V: de::Visitor,
            {
                visitor.visit_unit()
            }

            fn visit_option<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
                where V: de::Visitor,
            {
                visitor.visit_none()
            }
        }

        match self.state {
            Inner if field == "$value" => {
                self.state = Done;
                match mem::replace(&mut self.members, Content::Nothing) {
                    Content::Text(s) =>
                        de::Deserialize::deserialize(&mut StringDeserializer(Some(s))),
                    Content::Nothing =>
                        de::Deserialize::deserialize(&mut UnitDeserializer),
                    Content::Members(_) => Err(Error::MissingFieldError("inner text")),
                }
            },
            Inner => if let Some(v) = self.attributes.remove(field) {
                de::Deserialize::deserialize(&mut StringDeserializer(Some(v)))
            } else if let Content::Members(ref mut m) = self.members {
                if let Some(el) = m.remove(field) {
                    de::Deserialize::deserialize(&mut SeqDeserializer(el.into_iter()))
                } else {
                    de::Deserialize::deserialize(&mut UnitDeserializer)
                }
            } else {
                de::Deserialize::deserialize(&mut UnitDeserializer)
            },
            Done => de::Deserialize::deserialize(&mut UnitDeserializer),
        }
    }
}

impl de::Deserializer for MapDeserializer {
    type Error = Error;

    #[inline]
    fn visit<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        println!("MapDeserializer!");
        visitor.visit_map(self)
    }
}
