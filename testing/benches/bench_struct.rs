use std::collections::HashMap;
use test::Bencher;
use std::fmt;
use std::error;

use rustc_serialize::Decodable;

use serde;
use serde::de::Deserialize;

//////////////////////////////////////////////////////////////////////////////

#[derive(Clone, PartialEq, Debug, RustcDecodable, Deserialize)]
pub struct Inner {
    a: (),
    b: usize,
    c: HashMap<String, Option<char>>,
}

//////////////////////////////////////////////////////////////////////////////

#[derive(Clone, PartialEq, Debug, RustcDecodable, Deserialize)]
pub struct Outer {
    inner: Vec<Inner>,
}

//////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq)]
pub enum Error {
    EndOfStream,
    Syntax,
    MissingField,
    OtherError,
}

impl serde::de::Error for Error {
    fn custom<T: Into<String>>(_: T) -> Error { Error::Syntax }

    fn end_of_stream() -> Error { Error::EndOfStream }

    fn unknown_field(_: &str) -> Error { Error::Syntax }

    fn missing_field(_: &'static str) -> Error {
        Error::MissingField
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        formatter.write_str(format!("{:?}", self).as_ref())
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "Serde Deserialization Error"
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

mod decoder {
    use std::collections::HashMap;
    use rustc_serialize::Decoder;

    use super::{Outer, Inner, Error};

    #[derive(Debug)]
    enum State {
        Outer(Outer),
        Inner(Inner),
        Null,
        Usize(usize),
        Char(char),
        String(String),
        Field(&'static str),
        Vec(Vec<Inner>),
        Map(HashMap<String, Option<char>>),
        Option(bool),
    }

    pub struct OuterDecoder {
        stack: Vec<State>,

    }

    impl OuterDecoder {
        #[inline]
        pub fn new(animal: Outer) -> OuterDecoder {
            OuterDecoder {
                stack: vec!(State::Outer(animal)),
            }
        }
    }

    impl Decoder for OuterDecoder {
        type Error = Error;

        fn error(&mut self, _msg: &str) -> Error {
            Error::OtherError
        }

        // Primitive types:
        #[inline]
        fn read_nil(&mut self) -> Result<(), Error> {
            match self.stack.pop() {
                Some(State::Null) => Ok(()),
                _ => Err(Error::Syntax),
            }
        }
        #[inline]
        fn read_usize(&mut self) -> Result<usize, Error> {
            match self.stack.pop() {
                Some(State::Usize(value)) => Ok(value),
                _ => Err(Error::Syntax),
            }
        }
        fn read_u64(&mut self) -> Result<u64, Error> { Err(Error::Syntax) }
        fn read_u32(&mut self) -> Result<u32, Error> { Err(Error::Syntax) }
        fn read_u16(&mut self) -> Result<u16, Error> { Err(Error::Syntax) }
        fn read_u8(&mut self) -> Result<u8, Error> { Err(Error::Syntax) }
        fn read_isize(&mut self) -> Result<isize, Error> { Err(Error::Syntax) }
        fn read_i64(&mut self) -> Result<i64, Error> { Err(Error::Syntax) }
        fn read_i32(&mut self) -> Result<i32, Error> { Err(Error::Syntax) }
        fn read_i16(&mut self) -> Result<i16, Error> { Err(Error::Syntax) }
        fn read_i8(&mut self) -> Result<i8, Error> { Err(Error::Syntax) }
        fn read_bool(&mut self) -> Result<bool, Error> { Err(Error::Syntax) }
        fn read_f64(&mut self) -> Result<f64, Error> { Err(Error::Syntax) }
        fn read_f32(&mut self) -> Result<f32, Error> { Err(Error::Syntax) }
        #[inline]
        fn read_char(&mut self) -> Result<char, Error> {
            match self.stack.pop() {
                Some(State::Char(c)) => Ok(c),
                _ => Err(Error::Syntax),
            }
        }
        #[inline]
        fn read_str(&mut self) -> Result<String, Error> {
            match self.stack.pop() {
                Some(State::String(value)) => Ok(value),
                _ => Err(Error::Syntax),
            }
        }

        // Compound types:
        fn read_enum<T, F>(&mut self, _name: &str, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_enum_variant<T, F>(&mut self, _names: &[&str], _f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder, usize) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_enum_variant_arg<T, F>(&mut self, _a_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_enum_struct_variant<T, F>(&mut self, _names: &[&str], _f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder, usize) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_enum_struct_variant_field<T, F>(&mut self, _f_name: &str, _f_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        #[inline]
        fn read_struct<T, F>(&mut self, s_name: &str, _len: usize, f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder) -> Result<T, Error>,
        {
            match self.stack.pop() {
                Some(State::Outer(Outer { inner })) => {
                    if s_name == "Outer" {
                        self.stack.push(State::Vec(inner));
                        self.stack.push(State::Field("inner"));
                        f(self)
                    } else {
                        Err(Error::Syntax)
                    }
                }
                Some(State::Inner(Inner { a: (), b, c })) => {
                    if s_name == "Inner" {
                        self.stack.push(State::Map(c));
                        self.stack.push(State::Field("c"));

                        self.stack.push(State::Usize(b));
                        self.stack.push(State::Field("b"));

                        self.stack.push(State::Null);
                        self.stack.push(State::Field("a"));
                        f(self)
                    } else {
                        Err(Error::Syntax)
                    }
                }
                _ => Err(Error::Syntax),
            }
        }
        #[inline]
        fn read_struct_field<T, F>(&mut self, f_name: &str, _f_idx: usize, f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder) -> Result<T, Error>,
        {
            match self.stack.pop() {
                Some(State::Field(name)) => {
                    if f_name == name {
                        f(self)
                    } else {
                        Err(Error::Syntax)
                    }
                }
                _ => Err(Error::Syntax)
            }
        }

        fn read_tuple<T, F>(&mut self, _len: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_tuple_arg<T, F>(&mut self, _a_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_tuple_struct<T, F>(&mut self, _s_name: &str, _len: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_tuple_struct_arg<T, F>(&mut self, _a_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        // Specialized types:
        #[inline]
        fn read_option<T, F>(&mut self, f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder, bool) -> Result<T, Error>,
        {
            match self.stack.pop() {
                Some(State::Option(b)) => f(self, b),
                _ => Err(Error::Syntax),
            }
        }

        #[inline]
        fn read_seq<T, F>(&mut self, f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder, usize) -> Result<T, Error>,
        {
            match self.stack.pop() {
                Some(State::Vec(value)) => {
                    let len = value.len();
                    for inner in value.into_iter().rev() {
                        self.stack.push(State::Inner(inner));
                    }
                    f(self, len)
                }
                _ => Err(Error::Syntax)
            }
        }
        #[inline]
        fn read_seq_elt<T, F>(&mut self, _idx: usize, f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder) -> Result<T, Error>,
        {
            f(self)
        }

        #[inline]
        fn read_map<T, F>(&mut self, f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder, usize) -> Result<T, Error>,
        {
            match self.stack.pop() {
                Some(State::Map(map)) => {
                    let len = map.len();
                    for (key, value) in map {
                        match value {
                            Some(c) => {
                                self.stack.push(State::Char(c));
                                self.stack.push(State::Option(true));
                            }
                            None => {
                                self.stack.push(State::Option(false));
                            }
                        }
                        self.stack.push(State::String(key));
                    }
                    f(self, len)
                }
                _ => Err(Error::Syntax),
            }
        }
        #[inline]
        fn read_map_elt_key<T, F>(&mut self, _idx: usize, f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder) -> Result<T, Error>,
        {
            f(self)
        }

        #[inline]
        fn read_map_elt_val<T, F>(&mut self, _idx: usize, f: F) -> Result<T, Error> where
            F: FnOnce(&mut OuterDecoder) -> Result<T, Error>,
        {
            f(self)
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

mod deserializer {
    use std::collections::HashMap;
    use std::collections::hash_map;
    use std::vec;
    use super::{Outer, Inner};
    use super::Error;
    use serde::de;

    #[derive(Debug)]
    enum State {
        Outer(Outer),
        Inner(Inner),
        Str(&'static str),
        Null,
        Usize(usize),
        Char(char),
        String(String),
        Option(bool),
        Vec(Vec<Inner>),
        Map(HashMap<String, Option<char>>),
    }

    pub struct OuterDeserializer {
        stack: Vec<State>,
    }

    impl OuterDeserializer {
        #[inline]
        pub fn new(outer: Outer) -> OuterDeserializer {
            OuterDeserializer {
                stack: vec!(State::Outer(outer)),
            }
        }
    }

    impl de::Deserializer for OuterDeserializer {
        type Error = Error;

        fn deserialize<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
            where V: de::Visitor,
        {
            match self.stack.pop() {
                Some(State::Vec(value)) => {
                    visitor.visit_seq(OuterSeqVisitor {
                        de: self,
                        iter: value.into_iter(),
                    })
                }
                Some(State::Map(value)) => {
                    visitor.visit_map(MapVisitor {
                        de: self,
                        iter: value.into_iter(),
                    })
                }
                Some(State::Null) => {
                    visitor.visit_unit()
                }
                Some(State::Usize(x)) => {
                    visitor.visit_usize(x)
                }
                Some(State::Char(x)) => {
                    visitor.visit_char(x)
                }
                Some(State::Str(x)) => {
                    visitor.visit_str(x)
                }
                Some(State::String(x)) => {
                    visitor.visit_string(x)
                }
                Some(State::Option(false)) => {
                    visitor.visit_none()
                }
                Some(State::Option(true)) => {
                    visitor.visit_some(self)
                }
                Some(_) => Err(Error::Syntax),
                None => Err(Error::EndOfStream),
            }
        }

        fn deserialize_struct<V>(&mut self,
                           name: &str,
                           _fields: &'static [&'static str],
                           mut visitor: V) -> Result<V::Value, Error>
            where V: de::Visitor,
        {
            match self.stack.pop() {
                Some(State::Outer(Outer { inner })) => {
                    if name != "Outer" {
                        return Err(Error::Syntax);
                    }

                    self.stack.push(State::Vec(inner));
                    self.stack.push(State::Str("inner"));

                    visitor.visit_map(OuterMapVisitor {
                        de: self,
                        state: 0,
                    })
                }
                Some(State::Inner(Inner { a: (), b, c })) => {
                    if name != "Inner" {
                        return Err(Error::Syntax);
                    }

                    self.stack.push(State::Map(c));
                    self.stack.push(State::Str("c"));

                    self.stack.push(State::Usize(b));
                    self.stack.push(State::Str("b"));

                    self.stack.push(State::Null);
                    self.stack.push(State::Str("a"));

                    visitor.visit_map(InnerMapVisitor {
                        de: self,
                        state: 0,
                    })
                }
                _ => {
                    Err(Error::Syntax)
                }
            }
        }

        forward_to_deserialize! {
            bool usize u8 u16 u32 u64 isize i8 i16 i32 i64 f32 f64 char str
            string unit option seq seq_fixed_size bytes map unit_struct
            newtype_struct tuple_struct struct_field tuple enum ignored_any
        }
    }

    struct OuterMapVisitor<'a> {
        de: &'a mut OuterDeserializer,
        state: usize,
    }

    impl<'a> de::MapVisitor for OuterMapVisitor<'a> {
        type Error = Error;

        fn visit_key<K>(&mut self) -> Result<Option<K>, Error>
            where K: de::Deserialize,
        {
            match self.state {
                0 => {
                    self.state += 1;
                    Ok(Some(try!(de::Deserialize::deserialize(self.de))))
                }
                _ => {
                    Ok(None)
                }
            }
        }

        fn visit_value<V>(&mut self) -> Result<V, Error>
            where V: de::Deserialize,
        {
            de::Deserialize::deserialize(self.de)
        }

        fn end(&mut self) -> Result<(), Error> {
            if self.state == 1 {
                Ok(())
            } else {
                Err(Error::Syntax)
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            let len = 1 - self.state;
            (len, Some(len))
        }
    }

    struct OuterSeqVisitor<'a> {
        de: &'a mut OuterDeserializer,
        iter: vec::IntoIter<Inner>,
    }

    impl<'a> de::SeqVisitor for OuterSeqVisitor<'a> {
        type Error = Error;

        fn visit<T>(&mut self) -> Result<Option<T>, Error>
            where T: de::Deserialize,
        {
            match self.iter.next() {
                Some(value) => {
                    self.de.stack.push(State::Inner(value));
                    Ok(Some(try!(de::Deserialize::deserialize(self.de))))
                }
                None => {
                    Ok(None)
                }
            }
        }

        fn end(&mut self) -> Result<(), Error> {
            match self.iter.next() {
                Some(_) => Err(Error::Syntax),
                None => Ok(()),
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            self.iter.size_hint()
        }
    }

    struct InnerMapVisitor<'a> {
        de: &'a mut OuterDeserializer,
        state: usize,
    }

    impl<'a> de::MapVisitor for InnerMapVisitor<'a> {
        type Error = Error;

        fn visit_key<K>(&mut self) -> Result<Option<K>, Error>
            where K: de::Deserialize,
        {
            match self.state {
                0 ... 2 => {
                    self.state += 1;
                    Ok(Some(try!(de::Deserialize::deserialize(self.de))))
                }
                _ => {
                    Ok(None)
                }
            }
        }

        fn visit_value<V>(&mut self) -> Result<V, Error>
            where V: de::Deserialize,
        {
            de::Deserialize::deserialize(self.de)
        }

        fn end(&mut self) -> Result<(), Error> {
            if self.state == 3 {
                Ok(())
            } else {
                Err(Error::Syntax)
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            let len = 1 - self.state;
            (len, Some(len))
        }
    }

    struct MapVisitor<'a> {
        de: &'a mut OuterDeserializer,
        iter: hash_map::IntoIter<String, Option<char>>,
    }

    impl<'a> de::MapVisitor for MapVisitor<'a> {
        type Error = Error;

        fn visit_key<K>(&mut self) -> Result<Option<K>, Error>
            where K: de::Deserialize,
        {
            match self.iter.next() {
                Some((key, Some(value))) => {
                    self.de.stack.push(State::Char(value));
                    self.de.stack.push(State::Option(true));
                    self.de.stack.push(State::String(key));
                    Ok(Some(try!(de::Deserialize::deserialize(self.de))))
                }
                Some((key, None)) => {
                    self.de.stack.push(State::Option(false));
                    self.de.stack.push(State::String(key));
                    Ok(Some(try!(de::Deserialize::deserialize(self.de))))
                }
                None => {
                    Ok(None)
                }
            }
        }

        fn visit_value<V>(&mut self) -> Result<V, Error>
            where V: de::Deserialize,
        {
            de::Deserialize::deserialize(self.de)
        }

        fn end(&mut self) -> Result<(), Error> {
            match self.iter.next() {
                Some(_) => Err(Error::Syntax),
                None => Ok(()),
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            self.iter.size_hint()
        }
    }
}

#[bench]
fn bench_decoder_0_0(b: &mut Bencher) {
    b.iter(|| {
        let mut map = HashMap::new();
        map.insert("abc".to_owned(), Some('c'));

        let outer = Outer {
            inner: vec!(),
        };

        let mut d = decoder::OuterDecoder::new(outer.clone());
        let value: Result<Outer, Error> = Decodable::decode(&mut d);

        assert_eq!(value, Ok(outer));
    })
}

#[bench]
fn bench_decoder_1_0(b: &mut Bencher) {
    b.iter(|| {
        let map = HashMap::new();

        let outer = Outer {
            inner: vec!(
                Inner {
                    a: (),
                    b: 5,
                    c: map,
                },
            )
        };

        let mut d = decoder::OuterDecoder::new(outer.clone());
        let value: Result<Outer, Error> = Decodable::decode(&mut d);

        assert_eq!(value, Ok(outer));
    })
}

#[bench]
fn bench_decoder_1_5(b: &mut Bencher) {
    b.iter(|| {
        let mut map = HashMap::new();
        map.insert("1".to_owned(), Some('a'));
        map.insert("2".to_owned(), None);
        map.insert("3".to_owned(), Some('b'));
        map.insert("4".to_owned(), None);
        map.insert("5".to_owned(), Some('c'));

        let outer = Outer {
            inner: vec!(
                Inner {
                    a: (),
                    b: 5,
                    c: map,
                },
            )
        };

        let mut d = decoder::OuterDecoder::new(outer.clone());
        let value: Result<Outer, Error> = Decodable::decode(&mut d);

        assert_eq!(value, Ok(outer));
    })
}

#[bench]
fn bench_deserializer_0_0(b: &mut Bencher) {
    b.iter(|| {
        let outer = Outer {
            inner: vec!(),
        };

        let mut d = deserializer::OuterDeserializer::new(outer.clone());
        let value: Result<Outer, Error> = Deserialize::deserialize(&mut d);

        assert_eq!(value, Ok(outer));
    })
}

#[bench]
fn bench_deserializer_1_0(b: &mut Bencher) {
    b.iter(|| {
        let map = HashMap::new();

        let outer = Outer {
            inner: vec!(
                Inner {
                    a: (),
                    b: 5,
                    c: map,
                },
            )
        };

        let mut d = deserializer::OuterDeserializer::new(outer.clone());
        let value: Result<Outer, Error> = Deserialize::deserialize(&mut d);

        assert_eq!(value, Ok(outer));
    })
}

#[bench]
fn bench_deserializer_1_5(b: &mut Bencher) {
    b.iter(|| {
        let mut map = HashMap::new();
        map.insert("1".to_owned(), Some('a'));
        map.insert("2".to_owned(), None);
        map.insert("3".to_owned(), Some('b'));
        map.insert("4".to_owned(), None);
        map.insert("5".to_owned(), Some('c'));

        let outer = Outer {
            inner: vec!(
                Inner {
                    a: (),
                    b: 5,
                    c: map,
                },
            )
        };

        let mut d = deserializer::OuterDeserializer::new(outer.clone());
        let value: Result<Outer, Error> = Deserialize::deserialize(&mut d);

        assert_eq!(value, Ok(outer));
    })
}
