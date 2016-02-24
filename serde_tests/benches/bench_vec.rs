use std::fmt::Debug;
use std::fmt;
use std::error;
use test::Bencher;

use rustc_serialize::{Decoder, Decodable};

use serde;
use serde::de::{Deserializer, Deserialize};

//////////////////////////////////////////////////////////////////////////////

#[derive(PartialEq, Debug)]
pub enum Error {
    EndOfStream,
    Syntax,
}

impl serde::de::Error for Error {
    fn custom(_: String) -> Error { Error::Syntax }

    fn end_of_stream() -> Error { Error::EndOfStream }

    fn unknown_field(_: &str) -> Error { Error::Syntax }

    fn missing_field(_: &'static str) -> Error { Error::Syntax }
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
    use std::vec;
    use rustc_serialize;

    use super::Error;

    pub struct UsizeDecoder {
        len: usize,
        iter: vec::IntoIter<usize>,
    }

    impl UsizeDecoder {
        #[inline]
        pub fn new(values: Vec<usize>) -> UsizeDecoder {
            UsizeDecoder {
                len: values.len(),
                iter: values.into_iter(),
            }
        }
    }

    impl rustc_serialize::Decoder for UsizeDecoder {
        type Error = Error;

        fn error(&mut self, _: &str) -> Error { Error::Syntax }

        // Primitive types:
        fn read_nil(&mut self) -> Result<(), Error> { Err(Error::Syntax) }
        #[inline]
        fn read_usize(&mut self) -> Result<usize, Error> {
            match self.iter.next() {
                Some(value) => Ok(value),
                None => Err(Error::EndOfStream),
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
        fn read_char(&mut self) -> Result<char, Error> { Err(Error::Syntax) }
        fn read_str(&mut self) -> Result<String, Error> { Err(Error::Syntax) }

        // Compound types:
        fn read_enum<T, F>(&mut self, _name: &str, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut UsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_enum_variant<T, F>(&mut self, _names: &[&str], _f: F) -> Result<T, Error> where
            F: FnOnce(&mut UsizeDecoder, usize) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_enum_variant_arg<T, F>(&mut self, _a_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut UsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_enum_struct_variant<T, F>(&mut self, _names: &[&str], _f: F) -> Result<T, Error> where
            F: FnOnce(&mut UsizeDecoder, usize) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_enum_struct_variant_field<T, F>(&mut self, _f_name: &str, _f_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut UsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_struct<T, F>(&mut self, _s_name: &str, _len: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut UsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_struct_field<T, F>(&mut self, _f_name: &str, _f_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut UsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_tuple<T, F>(&mut self, _len: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut UsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_tuple_arg<T, F>(&mut self, _a_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut UsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_tuple_struct<T, F>(&mut self, _s_name: &str, _len: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut UsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_tuple_struct_arg<T, F>(&mut self, _a_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut UsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        // Specialized types:
        fn read_option<T, F>(&mut self, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut UsizeDecoder, bool) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        #[inline]
        fn read_seq<T, F>(&mut self, f: F) -> Result<T, Error> where
            F: FnOnce(&mut UsizeDecoder, usize) -> Result<T, Error>,
        {
            let len = self.len;
            f(self, len)
        }
        #[inline]
        fn read_seq_elt<T, F>(&mut self, _idx: usize, f: F) -> Result<T, Error> where
            F: FnOnce(&mut UsizeDecoder) -> Result<T, Error>,
        {
            f(self)
        }

        fn read_map<T, F>(&mut self, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut UsizeDecoder, usize) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_map_elt_key<T, F>(&mut self, _idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut UsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_map_elt_val<T, F>(&mut self, _idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut UsizeDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }
    }


    pub struct U8Decoder {
        len: usize,
        iter: vec::IntoIter<u8>,
    }

    impl U8Decoder {
        #[inline]
        pub fn new(values: Vec<u8>) -> U8Decoder {
            U8Decoder {
                len: values.len(),
                iter: values.into_iter(),
            }
        }
    }

    impl rustc_serialize::Decoder for U8Decoder {
        type Error = Error;

        fn error(&mut self, _: &str) -> Error { Error::Syntax }

        // Primitive types:
        fn read_nil(&mut self) -> Result<(), Error> { Err(Error::Syntax) }
        fn read_usize(&mut self) -> Result<usize, Error> { Err(Error::Syntax) }
        fn read_u64(&mut self) -> Result<u64, Error> { Err(Error::Syntax) }
        fn read_u32(&mut self) -> Result<u32, Error> { Err(Error::Syntax) }
        fn read_u16(&mut self) -> Result<u16, Error> { Err(Error::Syntax) }
        #[inline]
        fn read_u8(&mut self) -> Result<u8, Error> {
            match self.iter.next() {
                Some(value) => Ok(value),
                None => Err(Error::EndOfStream),
            }
        }
        #[inline]
        fn read_isize(&mut self) -> Result<isize, Error> { Err(Error::Syntax) }
        fn read_i64(&mut self) -> Result<i64, Error> { Err(Error::Syntax) }
        fn read_i32(&mut self) -> Result<i32, Error> { Err(Error::Syntax) }
        fn read_i16(&mut self) -> Result<i16, Error> { Err(Error::Syntax) }
        fn read_i8(&mut self) -> Result<i8, Error> { Err(Error::Syntax) }
        fn read_bool(&mut self) -> Result<bool, Error> { Err(Error::Syntax) }
        fn read_f64(&mut self) -> Result<f64, Error> { Err(Error::Syntax) }
        fn read_f32(&mut self) -> Result<f32, Error> { Err(Error::Syntax) }
        fn read_char(&mut self) -> Result<char, Error> { Err(Error::Syntax) }
        fn read_str(&mut self) -> Result<String, Error> { Err(Error::Syntax) }

        // Compound types:
        fn read_enum<T, F>(&mut self, _name: &str, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_enum_variant<T, F>(&mut self, _names: &[&str], _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder, usize) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_enum_variant_arg<T, F>(&mut self, _a_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_enum_struct_variant<T, F>(&mut self, _names: &[&str], _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder, usize) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_enum_struct_variant_field<T, F>(&mut self, _f_name: &str, _f_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_struct<T, F>(&mut self, _s_name: &str, _len: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_struct_field<T, F>(&mut self, _f_name: &str, _f_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_tuple<T, F>(&mut self, _len: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_tuple_arg<T, F>(&mut self, _a_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_tuple_struct<T, F>(&mut self, _s_name: &str, _len: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_tuple_struct_arg<T, F>(&mut self, _a_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        // Specialized types:
        fn read_option<T, F>(&mut self, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder, bool) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        #[inline]
        fn read_seq<T, F>(&mut self, f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder, usize) -> Result<T, Error>,
        {
            let len = self.len;
            f(self, len)
        }
        #[inline]
        fn read_seq_elt<T, F>(&mut self, _idx: usize, f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder) -> Result<T, Error>,
        {
            f(self)
        }

        fn read_map<T, F>(&mut self, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder, usize) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_map_elt_key<T, F>(&mut self, _idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_map_elt_val<T, F>(&mut self, _idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut U8Decoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

mod deserializer {
    //use std::num;
    use std::vec;

    use super::Error;

    use serde::de;

    #[derive(PartialEq, Debug)]
    enum State {
        Start,
        SepOrEnd,
        End,
    }

    pub struct Deserializer<A> {
        state: State,
        iter: vec::IntoIter<A>,
        len: usize,
        value: Option<A>,
    }

    impl<A> Deserializer<A> {
        #[inline]
        pub fn new(values: Vec<A>) -> Deserializer<A> {
            let len = values.len();
            Deserializer {
                state: State::Start,
                iter: values.into_iter(),
                len: len,
                value: None,
            }
        }
    }

    impl de::Deserializer for Deserializer<usize> {
        type Error = Error;

        #[inline]
        fn deserialize<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
            where V: de::Visitor,
        {
            match self.state {
                State::Start => {
                    self.state = State::SepOrEnd;
                    visitor.visit_seq(self)
                }
                State::SepOrEnd => {
                    visitor.visit_usize(self.value.take().unwrap())
                }
                State::End => {
                    Err(Error::EndOfStream)
                }
            }
        }
    }

    impl de::SeqVisitor for Deserializer<usize> {
        type Error = Error;

        #[inline]
        fn visit<T>(&mut self) -> Result<Option<T>, Error>
            where T: de::Deserialize,
        {
            match self.iter.next() {
                Some(value) => {
                    self.len -= 1;
                    self.value = Some(value);
                    Ok(Some(try!(de::Deserialize::deserialize(self))))
                }
                None => {
                    self.state = State::End;
                    Ok(None)
                }
            }
        }

        #[inline]
        fn end(&mut self) -> Result<(), Error> {
            match self.iter.next() {
                Some(_) => Err(Error::Syntax),
                None => {
                    self.state = State::End;
                    Ok(())
                }
            }
        }

        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.len, Some(self.len))
        }
    }

    impl de::Deserializer for Deserializer<u8> {
        type Error = Error;

        #[inline]
        fn deserialize<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
            where V: de::Visitor,
        {
            match self.state {
                State::Start => {
                    self.state = State::SepOrEnd;
                    visitor.visit_seq(self)
                }
                State::SepOrEnd => {
                    visitor.visit_u8(self.value.take().unwrap())
                }
                State::End => {
                    Err(Error::EndOfStream)
                }
            }
        }
    }

    impl de::SeqVisitor for Deserializer<u8> {
        type Error = Error;

        #[inline]
        fn visit<T>(&mut self) -> Result<Option<T>, Error>
            where T: de::Deserialize,
        {
            match self.iter.next() {
                Some(value) => {
                    self.len -= 1;
                    self.value = Some(value);
                    Ok(Some(try!(de::Deserialize::deserialize(self))))
                }
                None => {
                    self.state = State::End;
                    Ok(None)
                }
            }
        }

        #[inline]
        fn end(&mut self) -> Result<(), Error> {
            match self.iter.next() {
                Some(_) => Err(Error::Syntax),
                None => {
                    self.state = State::End;
                    Ok(())
                }
            }
        }

        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.len, Some(self.len))
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

fn run_decoder<
    D: Decoder<Error=Error>,
    T: Clone + PartialEq + Debug + Decodable
>(mut d: D, value: T) {
    let v = Decodable::decode(&mut d);

    assert_eq!(Ok(value), v);
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
fn bench_decoder_usize_000(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<usize> = vec!();
        run_decoder(decoder::UsizeDecoder::new(v.clone()), v)
    })
}

#[bench]
fn bench_decoder_usize_003(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<usize> = vec!(1, 2, 3);
        run_decoder(decoder::UsizeDecoder::new(v.clone()), v)
    })
}

#[bench]
fn bench_decoder_usize_100(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<usize> = (0 .. 100).collect();
        run_decoder(decoder::UsizeDecoder::new(v.clone()), v)
    })
}

#[bench]
fn bench_decoder_u8_000(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<u8> = vec!();
        run_decoder(decoder::U8Decoder::new(v.clone()), v)
    })
}

#[bench]
fn bench_decoder_u8_003(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<u8> = vec!(1, 2, 3);
        run_decoder(decoder::U8Decoder::new(v.clone()), v)
    })
}

#[bench]
fn bench_decoder_u8_100(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<u8> = (0 .. 100).collect();
        run_decoder(decoder::U8Decoder::new(v.clone()), v)
    })
}

#[bench]
fn bench_deserializer_usize_000(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<usize> = vec!();
        run_deserializer(deserializer::Deserializer::new(v.clone()), v)
    })
}

#[bench]
fn bench_deserializer_usize_003(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<usize> = vec!(1, 2, 3);
        run_deserializer(deserializer::Deserializer::new(v.clone()), v)
    })
}

#[bench]
fn bench_deserializer_usize_100(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<usize> = (0 .. 100).collect();
        run_deserializer(deserializer::Deserializer::new(v.clone()), v)
    })
}

#[bench]
fn bench_deserializer_u8_000(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<u8> = vec!();
        run_deserializer(deserializer::Deserializer::new(v.clone()), v)
    })
}

#[bench]
fn bench_deserializer_u8_003(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<u8> = vec!(1, 2, 3);
        run_deserializer(deserializer::Deserializer::new(v.clone()), v)
    })
}

#[bench]
fn bench_deserializer_u8_100(b: &mut Bencher) {
    b.iter(|| {
        let v: Vec<u8> = (0 .. 100).collect();
        run_deserializer(deserializer::Deserializer::new(v.clone()), v)
    })
}
