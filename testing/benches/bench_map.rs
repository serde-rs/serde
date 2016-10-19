use std::fmt::Debug;
use std::fmt;
use std::error;
use std::collections::HashMap;
use test::Bencher;

use rustc_serialize::{Decoder, Decodable};

use serde;
use serde::de::{Deserializer, Deserialize};

//////////////////////////////////////////////////////////////////////////////

#[derive(PartialEq, Debug)]
pub enum Error {
    EndOfStream,
    Syntax,
    MissingField,
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
//////////////////////////////////////////////////////////////////////////////

mod decoder {
    use std::collections::HashMap;
    use std::collections::hash_map::IntoIter;
    use rustc_serialize;

    use super::Error;

    enum Value {
        String(String),
        Isize(isize),
    }

    pub struct IsizeDecoder {
        len: usize,
        iter: IntoIter<String, isize>,
        stack: Vec<Value>,
    }

    impl IsizeDecoder {
        #[inline]
        pub fn new(values: HashMap<String, isize>) -> IsizeDecoder {
            IsizeDecoder {
                len: values.len(),
                iter: values.into_iter(),
                stack: vec!(),
            }
        }
    }

    impl rustc_serialize::Decoder for IsizeDecoder {
        type Error = Error;

        fn error(&mut self, _msg: &str) -> Error {
            Error::Syntax
        }

        // Primitive types:
        fn read_nil(&mut self) -> Result<(), Error> { Err(Error::Syntax) }
        fn read_usize(&mut self) -> Result<usize, Error> { Err(Error::Syntax) }
        fn read_u64(&mut self) -> Result<u64, Error> { Err(Error::Syntax) }
        fn read_u32(&mut self) -> Result<u32, Error> { Err(Error::Syntax) }
        fn read_u16(&mut self) -> Result<u16, Error> { Err(Error::Syntax) }
        fn read_u8(&mut self) -> Result<u8, Error> { Err(Error::Syntax) }
        #[inline]
        fn read_isize(&mut self) -> Result<isize, Error> {
            match self.stack.pop() {
                Some(Value::Isize(x)) => Ok(x),
                Some(_) => Err(Error::Syntax),
                None => Err(Error::EndOfStream),
            }
        }
        fn read_i64(&mut self) -> Result<i64, Error> { Err(Error::Syntax) }
        fn read_i32(&mut self) -> Result<i32, Error> { Err(Error::Syntax) }
        fn read_i16(&mut self) -> Result<i16, Error> { Err(Error::Syntax) }
        fn read_i8(&mut self) -> Result<i8, Error> { Err(Error::Syntax) }
        fn read_bool(&mut self) -> Result<bool, Error> { Err(Error::Syntax) }
        fn read_f64(&mut self) -> Result<f64, Error> { Err(Error::Syntax) }
        fn read_f32(&mut self) -> Result<f32, Error> { Err(Error::Syntax) }
        fn read_char(&mut self) -> Result<char, Error> { Err(Error::Syntax) }
        #[inline]
        fn read_str(&mut self) -> Result<String, Error> {
            match self.stack.pop() {
                Some(Value::String(x)) => Ok(x),
                Some(_) => Err(Error::Syntax),
                None => Err(Error::EndOfStream),
            }
        }

        // Compound types:
        fn read_enum<T, F>(&mut self, _name: &str, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_enum_variant<T, F>(&mut self, _names: &[&str], _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder, usize) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_enum_variant_arg<T, F>(&mut self, _a_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_enum_struct_variant<T, F>(&mut self, _names: &[&str], _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder, usize) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_enum_struct_variant_field<T, F>(&mut self, _f_name: &str, _f_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_struct<T, F>(&mut self, _s_name: &str, _len: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_struct_field<T, F>(&mut self, _f_name: &str, _f_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_tuple<T, F>(&mut self, _len: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_tuple_arg<T, F>(&mut self, _a_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_tuple_struct<T, F>(&mut self, _s_name: &str, _len: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_tuple_struct_arg<T, F>(&mut self, _a_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        // Specialized types:
        fn read_option<T, F>(&mut self, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder, bool) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_seq<T, F>(&mut self, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder, usize) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_seq_elt<T, F>(&mut self, _idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        #[inline]
        fn read_map<T, F>(&mut self, f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder, usize) -> Result<T, Error>,
        {
            let len = self.len;
            f(self, len)
        }
        #[inline]
        fn read_map_elt_key<T, F>(&mut self, _idx: usize, f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder) -> Result<T, Error>,
        {
            match self.iter.next() {
                Some((key, value)) => {
                    self.stack.push(Value::Isize(value));
                    self.stack.push(Value::String(key));
                    f(self)
                }
                None => {
                    Err(Error::Syntax)
                }
            }
        }

        #[inline]
        fn read_map_elt_val<T, F>(&mut self, _idx: usize, f: F) -> Result<T, Error> where
            F: FnOnce(&mut IsizeDecoder) -> Result<T, Error>,
        {
            f(self)
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

mod deserializer {
    use std::collections::HashMap;
    use std::collections::hash_map;

    use super::Error;

    use serde::de;

    #[derive(PartialEq, Debug)]
    enum State {
        StartState,
        Key(String),
        Value(isize),
    }

    pub struct IsizeDeserializer {
        stack: Vec<State>,
        iter: hash_map::IntoIter<String, isize>,
    }

    impl IsizeDeserializer {
        #[inline]
        pub fn new(values: HashMap<String, isize>) -> IsizeDeserializer {
            IsizeDeserializer {
                stack: vec!(State::StartState),
                iter: values.into_iter(),
            }
        }
    }

    impl de::Deserializer for IsizeDeserializer {
        type Error = Error;

        fn deserialize<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
            where V: de::Visitor,
        {
            match self.stack.pop() {
                Some(State::StartState) => {
                    visitor.visit_map(self)
                }
                Some(State::Key(key)) => {
                    visitor.visit_string(key)
                }
                Some(State::Value(value)) => {
                    visitor.visit_isize(value)
                }
                None => {
                    Err(Error::EndOfStream)
                }
            }
        }

        forward_to_deserialize! {
            bool usize u8 u16 u32 u64 isize i8 i16 i32 i64 f32 f64 char str
            string unit option seq seq_fixed_size bytes map unit_struct
            newtype_struct tuple_struct struct struct_field tuple enum
            ignored_any
        }
    }

    impl de::MapVisitor for IsizeDeserializer {
        type Error = Error;

        fn visit_key<K>(&mut self) -> Result<Option<K>, Error>
            where K: de::Deserialize,
        {
            match self.iter.next() {
                Some((key, value)) => {
                    self.stack.push(State::Value(value));
                    self.stack.push(State::Key(key));
                    Ok(Some(try!(de::Deserialize::deserialize(self))))
                }
                None => {
                    Ok(None)
                }
            }
        }

        fn visit_value<V>(&mut self) -> Result<V, Error>
            where V: de::Deserialize,
        {
            de::Deserialize::deserialize(self)
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

/*
    impl Iterator for IsizeDeserializer {
        type Item = Result<de::Token, Error>;

        #[inline]
        fn next(&mut self) -> Option<Result<de::Token, Error>> {
            match self.stack.pop() {
                Some(State::StartState) => {
                    self.stack.push(KeyOrEndState);
                    Some(Ok(de::Token::MapStart(self.len)))
                }
                Some(State::KeyOrEndState) => {
                    match self.iter.next() {
                        Some((key, value)) => {
                            self.stack.push(Value(value));
                            Some(Ok(de::Token::String(key)))
                        }
                        None => {
                            self.stack.push(EndState);
                            Some(Ok(de::Token::End))
                        }
                    }
                }
                Some(State::Value(x)) => {
                    self.stack.push(KeyOrEndState);
                    Some(Ok(de::Token::Isize(x)))
                }
                Some(EndState) => {
                    None
                }
                None => {
                    None
                }
            }
        }
    }

    impl de::Deserializer<Error> for IsizeDeserializer {
        #[inline]
        fn end_of_stream(&mut self) -> Error {
            EndOfStream
        }

        #[inline]
        fn syntax(&mut self, _token: de::Token, _expected: &[de::TokenKind]) -> Error {
            Syntax
        }

        #[inline]
        fn unexpected_name(&mut self, _token: de::Token) -> Error {
            Syntax
        }

        #[inline]
        fn conversion_error(&mut self, _token: de::Token) -> Error {
            Syntax
        }

        #[inline]
        fn missing_field<
            T: de::Deserialize<IsizeDeserializer, Error>
        >(&mut self, _field: &'static str) -> Result<T, Error> {
            Err(Error::Syntax)
        }
    }
*/
}

//////////////////////////////////////////////////////////////////////////////

fn run_decoder<
    D: Decoder<Error=Error>,
    T: Clone + PartialEq + Debug + Decodable
>(mut d: D, value: T) {
    let v = Decodable::decode(&mut d);

    assert_eq!(Ok(value), v);
}

#[bench]
fn bench_decoder_000(b: &mut Bencher) {
    b.iter(|| {
        let m: HashMap<String, isize> = HashMap::new();
        run_decoder(decoder::IsizeDecoder::new(m.clone()), m)
    })
}

#[bench]
fn bench_decoder_003(b: &mut Bencher) {
    b.iter(|| {
        let mut m: HashMap<String, isize> = HashMap::new();
        for i in 0 .. 3 {
            m.insert(i.to_string(), i);
        }
        run_decoder(decoder::IsizeDecoder::new(m.clone()), m)
    })
}

#[bench]
fn bench_decoder_100(b: &mut Bencher) {
    b.iter(|| {
        let mut m: HashMap<String, isize> = HashMap::new();
        for i in 0 .. 100 {
            m.insert(i.to_string(), i);
        }
        run_decoder(decoder::IsizeDecoder::new(m.clone()), m)
    })
}

fn run_deserializer<D, T>(mut d: D, value: T)
    where D: Deserializer,
          D::Error: Debug + PartialEq,
          T: Clone + PartialEq + Debug + Deserialize
{
    let v = T::deserialize(&mut d);

    assert_eq!(Ok(value), v);
}

#[bench]
fn bench_deserializer_000(b: &mut Bencher) {
    b.iter(|| {
        let m: HashMap<String, isize> = HashMap::new();
        run_deserializer(deserializer::IsizeDeserializer::new(m.clone()), m)
    })
}

#[bench]
fn bench_deserializer_003(b: &mut Bencher) {
    b.iter(|| {
        let mut m: HashMap<String, isize> = HashMap::new();
        for i in 0 .. 3 {
            m.insert(i.to_string(), i);
        }
        run_deserializer(deserializer::IsizeDeserializer::new(m.clone()), m)
    })
}

#[bench]
fn bench_deserializer_100(b: &mut Bencher) {
    b.iter(|| {
        let mut m: HashMap<String, isize> = HashMap::new();
        for i in 0 .. 100 {
            m.insert(i.to_string(), i);
        }
        run_deserializer(deserializer::IsizeDeserializer::new(m.clone()), m)
    })
}
