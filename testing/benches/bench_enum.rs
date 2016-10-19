use test::Bencher;
use std::error;
use std::fmt;
use rustc_serialize::Decodable;
use serde;
use serde::de::Deserialize;

//////////////////////////////////////////////////////////////////////////////

#[derive(Clone, PartialEq, Debug, RustcDecodable, Deserialize)]
pub enum Animal {
    Dog,
    Frog(String, isize)
}

//////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum Error {
    EndOfStream,
    Syntax,
}

impl serde::de::Error for Error {
    fn custom<T: Into<String>>(_: T) -> Error { Error::Syntax }

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
    use rustc_serialize::Decoder;

    use super::{Animal, Error};
    use super::Animal::{Dog, Frog};

    enum State {
        Animal(Animal),
        Isize(isize),
        String(String),
    }

    pub struct AnimalDecoder {
        stack: Vec<State>,

    }

    impl AnimalDecoder {
        #[inline]
        pub fn new(animal: Animal) -> AnimalDecoder {
            AnimalDecoder {
                stack: vec!(State::Animal(animal)),
            }
        }
    }

    impl Decoder for AnimalDecoder {
        type Error = Error;

        fn error(&mut self, _: &str) -> Error { Error::Syntax }

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
                Some(State::Isize(x)) => Ok(x),
                _ => Err(Error::Syntax),
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
                Some(State::String(x)) => Ok(x),
                _ => Err(Error::Syntax),
            }
        }

        // Compound types:
        #[inline]
        fn read_enum<T, F>(&mut self, name: &str, f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder) -> Result<T, Error>,
        {
            match self.stack.pop() {
                Some(State::Animal(animal)) => {
                    self.stack.push(State::Animal(animal));
                    if name == "Animal" {
                        f(self)
                    } else {
                        Err(Error::Syntax)
                    }
                }
                _ => Err(Error::Syntax)
            }
        }

        #[inline]
        fn read_enum_variant<T, F>(&mut self, names: &[&str], f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder, usize) -> Result<T, Error>,
        {
            let name = match self.stack.pop() {
                Some(State::Animal(Dog)) => "Dog",
                Some(State::Animal(Frog(x0, x1))) => {
                    self.stack.push(State::Isize(x1));
                    self.stack.push(State::String(x0));
                    "Frog"
                }
                _ => { return Err(Error::Syntax); }
            };

            let idx = match names.iter().position(|n| *n == name) {
                Some(idx) => idx,
                None => { return Err(Error::Syntax); }
            };

            f(self, idx)
        }

        #[inline]
        fn read_enum_variant_arg<T, F>(&mut self, _a_idx: usize, f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder) -> Result<T, Error>,
        {
            f(self)
        }

        fn read_enum_struct_variant<T, F>(&mut self, _names: &[&str], _f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder, usize) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_enum_struct_variant_field<T, F>(&mut self, _f_name: &str, _f_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_struct<T, F>(&mut self, _s_name: &str, _len: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_struct_field<T, F>(&mut self, _f_name: &str, _f_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_tuple<T, F>(&mut self, _len: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_tuple_arg<T, F>(&mut self, _a_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_tuple_struct<T, F>(&mut self, _s_name: &str, _len: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_tuple_struct_arg<T, F>(&mut self, _a_idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        // Specialized types:
        fn read_option<T, F>(&mut self, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder, bool) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        #[inline]
        fn read_seq<T, F>(&mut self, f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder, usize) -> Result<T, Error>,
        {
            f(self, 3)
        }

        #[inline]
        fn read_seq_elt<T, F>(&mut self, _idx: usize, f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder) -> Result<T, Error>,
        {
            f(self)
        }

        fn read_map<T, F>(&mut self, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder, usize) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_map_elt_key<T, F>(&mut self, _idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }

        fn read_map_elt_val<T, F>(&mut self, _idx: usize, _f: F) -> Result<T, Error> where
            F: FnOnce(&mut AnimalDecoder) -> Result<T, Error>,
        {
            Err(Error::Syntax)
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

mod deserializer {
    use super::{Animal, Error};

    use serde::de::{self, Deserialize};

    #[derive(Debug)]
    enum State {
        Animal(Animal),
        Isize(isize),
        Str(&'static str),
        String(String),
        UnitState,
    }

    pub struct AnimalDeserializer {
        stack: Vec<State>,
    }

    impl AnimalDeserializer {
        #[inline]
        pub fn new(animal: Animal) -> AnimalDeserializer {
            AnimalDeserializer {
                stack: vec!(State::Animal(animal)),
            }
        }
    }

    impl de::Deserializer for AnimalDeserializer {
        type Error = Error;

        #[inline]
        fn deserialize<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
            where V: de::Visitor,
        {
            match self.stack.pop() {
                Some(State::Isize(value)) => {
                    visitor.visit_isize(value)
                }
                Some(State::String(value)) => {
                    visitor.visit_string(value)
                }
                Some(State::Str(value)) => {
                    visitor.visit_str(value)
                }
                Some(State::UnitState) => {
                    visitor.visit_unit()
                }
                Some(_) => {
                    Err(Error::Syntax)
                }
                None => {
                    Err(Error::EndOfStream)
                }
            }
        }

        #[inline]
        fn deserialize_enum<V>(&mut self,
                         _name: &str,
                         _variants: &[&str],
                         mut visitor: V) -> Result<V::Value, Error>
            where V: de::EnumVisitor,
        {
            match self.stack.pop() {
                Some(State::Animal(Animal::Dog)) => {
                    self.stack.push(State::UnitState);
                    self.stack.push(State::Str("Dog"));
                    visitor.visit(DogVisitor {
                        de: self,
                    })
                }
                Some(State::Animal(Animal::Frog(x0, x1))) => {
                    self.stack.push(State::Isize(x1));
                    self.stack.push(State::String(x0));
                    self.stack.push(State::Str("Frog"));
                    visitor.visit(FrogVisitor {
                        de: self,
                        state: 0,
                    })
                }
                Some(_) => {
                    Err(Error::Syntax)
                }
                None => {
                    Err(Error::EndOfStream)
                }
            }
        }

        forward_to_deserialize! {
            bool usize u8 u16 u32 u64 isize i8 i16 i32 i64 f32 f64 char str
            string unit option seq seq_fixed_size bytes map unit_struct
            newtype_struct tuple_struct struct struct_field tuple ignored_any
        }
    }

    struct DogVisitor<'a> {
        de: &'a mut AnimalDeserializer,
    }

    impl<'a> de::VariantVisitor for DogVisitor<'a> {
        type Error = Error;

        fn visit_variant<V>(&mut self) -> Result<V, Error>
            where V: de::Deserialize
        {
            de::Deserialize::deserialize(self.de)
        }

        fn visit_unit(&mut self) -> Result<(), Error> {
            de::Deserialize::deserialize(self.de)
        }

        fn visit_newtype<T>(&mut self) -> Result<T, Self::Error>
            where T: Deserialize
        {
            Err(de::Error::invalid_type(de::Type::TupleVariant))
        }

        fn visit_tuple<V>(&mut self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
            where V: de::Visitor
        {
            Err(de::Error::invalid_type(de::Type::TupleVariant))
        }

        fn visit_struct<V>(&mut self, _fields: &'static [&'static str], _visitor: V) -> Result<V::Value, Self::Error>
            where V: de::Visitor
        {
            Err(de::Error::invalid_type(de::Type::StructVariant))
        }
    }

    struct FrogVisitor<'a> {
        de: &'a mut AnimalDeserializer,
        state: usize,
    }

    impl<'a> de::VariantVisitor for FrogVisitor<'a> {
        type Error = Error;

        fn visit_variant<V>(&mut self) -> Result<V, Error>
            where V: de::Deserialize
        {
            de::Deserialize::deserialize(self.de)
        }

        fn visit_tuple<V>(&mut self,
                          _len: usize,
                          mut visitor: V) -> Result<V::Value, Error>
            where V: de::Visitor,
        {
            visitor.visit_seq(self)
        }

        fn visit_unit(&mut self) -> Result<(), Error> {
            Err(de::Error::invalid_type(de::Type::UnitVariant))
        }

        fn visit_newtype<T>(&mut self) -> Result<T, Self::Error>
            where T: Deserialize
        {
            Err(de::Error::invalid_type(de::Type::TupleVariant))
        }

        fn visit_struct<V>(&mut self, _fields: &'static [&'static str], _visitor: V) -> Result<V::Value, Self::Error>
            where V: de::Visitor
        {
            Err(de::Error::invalid_type(de::Type::StructVariant))
        }
    }

    impl<'a> de::SeqVisitor for FrogVisitor<'a> {
        type Error = Error;

        fn visit<T>(&mut self) -> Result<Option<T>, Error>
            where T: de::Deserialize,
        {
            match self.state {
                0 => {
                    self.state += 1;
                    Ok(Some(try!(de::Deserialize::deserialize(self.de))))
                }
                1 => {
                    self.state += 1;
                    Ok(Some(try!(de::Deserialize::deserialize(self.de))))
                }
                _ => {
                    Ok(None)
                }
            }
        }

        fn end(&mut self) -> Result<(), Error> {
            if self.state == 2 {
                Ok(())
            } else {
                Err(Error::Syntax)
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            let len = 2 - self.state;
            (len, Some(len))
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

#[bench]
fn bench_decoder_dog(b: &mut Bencher) {
    b.iter(|| {
        let animal = Animal::Dog;

        let mut d = decoder::AnimalDecoder::new(animal.clone());
        let value: Animal = Decodable::decode(&mut d).unwrap();

        assert_eq!(value, animal);
    })
}

#[bench]
fn bench_decoder_frog(b: &mut Bencher) {
    b.iter(|| {
        let animal = Animal::Frog("Henry".to_owned(), 349);

        let mut d = decoder::AnimalDecoder::new(animal.clone());
        let value: Animal = Decodable::decode(&mut d).unwrap();

        assert_eq!(value, animal);
    })
}

#[bench]
fn bench_deserializer_dog(b: &mut Bencher) {
    b.iter(|| {
        let animal = Animal::Dog;

        let mut d = deserializer::AnimalDeserializer::new(animal.clone());
        let value: Animal = Deserialize::deserialize(&mut d).unwrap();

        assert_eq!(value, animal);
    })
}

#[bench]
fn bench_deserializer_frog(b: &mut Bencher) {
    b.iter(|| {
        let animal = Animal::Frog("Henry".to_owned(), 349);

        let mut d = deserializer::AnimalDeserializer::new(animal.clone());
        let value: Animal = Deserialize::deserialize(&mut d).unwrap();

        assert_eq!(value, animal);
    })
}
