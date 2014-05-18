extern crate collections;

use std::hash::Hash;
use std::result;
use collections::HashMap;

#[deriving(Clone, Eq)]
pub enum Token {
    Null,
    Bool(bool),
    Int(int),
    F64(f64),
    StrBuf(StrBuf),
    CollectionStart,
    CollectionSep,
    CollectionEnd,
}

pub trait Deserializer<E>: Iterator<Result<Token, E>> {
    fn end_of_stream_error(&self) -> E;

    fn syntax_error(&self) -> E;

    #[inline]
    fn expect_null(&mut self) -> Result<(), E> {
        match self.next() {
            Some(Ok(Null)) => Ok(()),
            Some(Ok(CollectionStart)) => self.expect_collection_end(),
            Some(Ok(_)) => Err(self.syntax_error()),
            Some(Err(err)) => Err(err),
            None => Err(self.end_of_stream_error()),
        }
    }

    #[inline]
    fn expect_bool(&mut self) -> Result<bool, E> {
        match self.next() {
            Some(Ok(Bool(value))) => Ok(value),
            Some(Ok(_)) => Err(self.syntax_error()),
            Some(Err(err)) => Err(err),
            None => Err(self.end_of_stream_error()),
        }
    }


    #[inline]
    fn expect_int(&mut self) -> Result<int, E> {
        match self.next() {
            Some(Ok(Int(value))) => Ok(value),
            Some(Ok(_)) => Err(self.syntax_error()),
            Some(Err(err)) => Err(err),
            None => Err(self.end_of_stream_error()),
        }
    }

    #[inline]
    fn expect_f64(&mut self) -> Result<f64, E> {
        match self.next() {
            Some(Ok(F64(value))) => Ok(value),
            Some(Ok(_)) => Err(self.syntax_error()),
            Some(Err(err)) => Err(err),
            None => Err(self.end_of_stream_error()),
        }
    }

    #[inline]
    fn expect_str(&mut self) -> Result<StrBuf, E> {
        match self.next() {
            Some(Ok(StrBuf(value))) => Ok(value),
            Some(Ok(_)) => Err(self.syntax_error()),
            Some(Err(err)) => Err(err),
            None => Err(self.end_of_stream_error()),
        }
    }

    #[inline]
    fn expect_collection<
        T: Deserializable<E, Self>,
        C: FromIterator<T>
    >(&mut self) -> Result<C, E> {
        try!(self.expect_collection_start());

        let iter = self.by_ref().batch(|d| {
            let d = d.iter();

            let token = match d.next() {
                Some(token) => token,
                None => { return None; }
            };

            match token {
                Ok(CollectionSep) => {
                    let value: Result<T, E> = Deserializable::deserialize(d);
                    Some(value)
                }
                Ok(CollectionEnd) => {
                    None
                }
                Ok(_) => {
                    Some(Err(d.syntax_error()))
                }
                Err(e) => {
                    Some(Err(e))
                }
            }
        });

        result::collect(iter)
    }

    #[inline]
    fn expect_collection_start(&mut self) -> Result<(), E> {
        match self.next() {
            Some(Ok(CollectionStart)) => Ok(()),
            Some(Ok(_)) => Err(self.syntax_error()),
            Some(Err(err)) => Err(err),
            None => Err(self.end_of_stream_error()),
        }
    }

    #[inline]
    fn expect_collection_sep(&mut self) -> Result<(), E> {
        match self.next() {
            Some(Ok(CollectionSep)) => Ok(()),
            Some(Ok(_)) => Err(self.syntax_error()),
            Some(Err(err)) => Err(err),
            None => Err(self.end_of_stream_error()),
        }
    }

    #[inline]
    fn expect_collection_end(&mut self) -> Result<(), E> {
        match self.next() {
            Some(Ok(CollectionEnd)) => Ok(()),
            Some(Ok(_)) => Err(self.syntax_error()),
            Some(Err(err)) => Err(err),
            None => Err(self.end_of_stream_error()),
        }
    }

    #[inline]
    fn expect_collection_sep_or_end(&mut self) -> Result<bool, E> {
        match self.next() {
            Some(Ok(CollectionSep)) => Ok(false),
            Some(Ok(CollectionEnd)) => Ok(true),
            Some(Ok(_)) => Err(self.syntax_error()),
            Some(Err(err)) => Err(err),
            None => Err(self.end_of_stream_error()),
        }
    }
}

pub trait Deserializable<E, D: Deserializer<E>> {
    fn deserialize(d: &mut D) -> Result<Self, E>;
}

//////////////////////////////////////////////////////////////////////////////

impl<
    E,
    D: Deserializer<E>
> Deserializable<E, D> for bool {
    #[inline]
    fn deserialize(d: &mut D) -> Result<bool, E> {
        d.expect_bool()
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<
    E,
    D: Deserializer<E>
> Deserializable<E, D> for int {
    #[inline]
    fn deserialize(d: &mut D) -> Result<int, E> {
        d.expect_int()
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<
    E,
    D: Deserializer<E>
> Deserializable<E, D> for f64 {
    #[inline]
    fn deserialize(d: &mut D) -> Result<f64, E> {
        d.expect_f64()
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<
    E,
    D: Deserializer<E>
> Deserializable<E, D> for StrBuf {
    #[inline]
    fn deserialize(d: &mut D) -> Result<StrBuf, E> {
        d.expect_str()
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<
    E,
    D: Deserializer<E>,
    T: Deserializable<E, D>
> Deserializable<E, D> for Option<T> {
    #[inline]
    fn deserialize(_d: &mut D) -> Result<Option<T>, E> {
        fail!()
        //d.expect_collection()
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<
    E,
    D: Deserializer<E>,
    T: Deserializable<E, D>
> Deserializable<E, D> for Vec<T> {
    #[inline]
    fn deserialize(d: &mut D) -> Result<Vec<T>, E> {
        d.expect_collection()
    }
}

impl<
    E,
    D: Deserializer<E>,
    K: Deserializable<E, D> + TotalEq + Hash,
    V: Deserializable<E, D>
> Deserializable<E, D> for HashMap<K, V> {
    #[inline]
    fn deserialize(d: &mut D) -> Result<HashMap<K, V>, E> {
        d.expect_collection()
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<
    E,
    D: Deserializer<E>
> Deserializable<E, D> for () {
    #[inline]
    fn deserialize(d: &mut D) -> Result<(), E> {
        d.expect_null()
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<
    E,
    D: Deserializer<E>,
    T0: Deserializable<E, D>
> Deserializable<E, D> for (T0,) {
    #[inline]
    fn deserialize(d: &mut D) -> Result<(T0,), E> {
        try!(d.expect_collection_start());

        try!(d.expect_collection_sep());
        let x0 = try!(Deserializable::deserialize(d));

        try!(d.expect_collection_end());

        Ok((x0,))
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<
    E,
    D: Deserializer<E>,
    T0: Deserializable<E, D>,
    T1: Deserializable<E, D>
> Deserializable<E, D> for (T0, T1) {
    #[inline]
    fn deserialize(d: &mut D) -> Result<(T0, T1), E> {
        try!(d.expect_collection_start());

        try!(d.expect_collection_sep());
        let x0 = try!(Deserializable::deserialize(d));

        try!(d.expect_collection_sep());
        let x1 = try!(Deserializable::deserialize(d));

        try!(d.expect_collection_end());

        Ok((x0, x1))
    }
}

//////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    extern crate serialize;

    use std::vec;
    use collections::HashMap;
    use test::Bencher;

    use self::serialize::{Decoder, Decodable};

    use super::{Token, Int, StrBuf, CollectionStart, CollectionSep, CollectionEnd};
    use super::{Deserializer, Deserializable};

    //////////////////////////////////////////////////////////////////////////////

    #[deriving(Show)]
    enum Error {
        EndOfStream,
        SyntaxError,
    }

    //////////////////////////////////////////////////////////////////////////////

    struct TokenDeserializer {
        tokens: Vec<Token>,
    }

    impl TokenDeserializer {
        #[inline]
        fn new(tokens: Vec<Token>) -> TokenDeserializer {
            TokenDeserializer {
                tokens: tokens,
            }
        }
    }

    impl Iterator<Result<Token, Error>> for TokenDeserializer {
        #[inline]
        fn next(&mut self) -> Option<Result<Token, Error>> {
            match self.tokens.shift() {
                None => None,
                Some(token) => Some(Ok(token)),
            }
        }
    }

    impl Deserializer<Error> for TokenDeserializer {
        #[inline]
        fn end_of_stream_error(&self) -> Error {
            EndOfStream
        }

        #[inline]
        fn syntax_error(&self) -> Error {
            SyntaxError
        }
    }

    //////////////////////////////////////////////////////////////////////////////

    #[deriving(Eq, Show)]
    enum IntsDeserializerState {
        Start,
        Sep,
        Value,
        End,
    }

    struct IntsDeserializer {
        state: IntsDeserializerState,
        iter: vec::MoveItems<int>,
        value: Option<int>
    }

    impl IntsDeserializer {
        #[inline]
        fn new(values: Vec<int>) -> IntsDeserializer {
            IntsDeserializer {
                state: Start,
                iter: values.move_iter(),
                value: None,
            }
        }
    }

    impl Iterator<Result<Token, Error>> for IntsDeserializer {
        #[inline]
        fn next(&mut self) -> Option<Result<Token, Error>> {
            match self.state {
                Start => {
                    self.state = Sep;
                    Some(Ok(CollectionStart))
                }
                Sep => {
                    match self.iter.next() {
                        Some(value) => {
                            self.state = Value;
                            self.value = Some(value);
                            Some(Ok(CollectionSep))
                        }
                        None => {
                            self.state = End;
                            Some(Ok(CollectionEnd))
                        }
                    }
                }
                Value => {
                    self.state = Sep;
                    match self.value.take() {
                        Some(value) => Some(Ok(Int(value))),
                        None => Some(Err(self.end_of_stream_error())),
                    }
                }
                End => {
                    None
                }
            }
        }
    }

    impl Deserializer<Error> for IntsDeserializer {
        #[inline]
        fn end_of_stream_error(&self) -> Error {
            EndOfStream
        }

        #[inline]
        fn syntax_error(&self) -> Error {
            SyntaxError
        }

        #[inline]
        fn expect_int(&mut self) -> Result<int, Error> {
            assert_eq!(self.state, Value);

            self.state = Sep;

            match self.value.take() {
                Some(value) => Ok(value),
                None => Err(self.end_of_stream_error()),
            }
        }
    }

    struct IntsDecoder {
        iter: vec::MoveItems<int>,
    }

    impl IntsDecoder {
        #[inline]
        fn new(values: Vec<int>) -> IntsDecoder {
            IntsDecoder {
                iter: values.move_iter()
            }
        }
    }

    impl Decoder<Error> for IntsDecoder {
        // Primitive types:
        fn read_nil(&mut self) -> Result<(), Error> { Err(SyntaxError) }
        fn read_uint(&mut self) -> Result<uint, Error> { Err(SyntaxError) }
        fn read_u64(&mut self) -> Result<u64, Error> { Err(SyntaxError) }
        fn read_u32(&mut self) -> Result<u32, Error> { Err(SyntaxError) }
        fn read_u16(&mut self) -> Result<u16, Error> { Err(SyntaxError) }
        fn read_u8(&mut self) -> Result<u8, Error> { Err(SyntaxError) }
        #[inline]
        fn read_int(&mut self) -> Result<int, Error> {
            match self.iter.next() {
                Some(value) => Ok(value),
                None => Err(EndOfStream),
            }
        }
        fn read_i64(&mut self) -> Result<i64, Error> { Err(SyntaxError) }
        fn read_i32(&mut self) -> Result<i32, Error> { Err(SyntaxError) }
        fn read_i16(&mut self) -> Result<i16, Error> { Err(SyntaxError) }
        fn read_i8(&mut self) -> Result<i8, Error> { Err(SyntaxError) }
        fn read_bool(&mut self) -> Result<bool, Error> { Err(SyntaxError) }
        fn read_f64(&mut self) -> Result<f64, Error> { Err(SyntaxError) }
        fn read_f32(&mut self) -> Result<f32, Error> { Err(SyntaxError) }
        fn read_char(&mut self) -> Result<char, Error> { Err(SyntaxError) }
        fn read_str(&mut self) -> Result<StrBuf, Error> { Err(SyntaxError) }

        // Compound types:
        fn read_enum<T>(&mut self, _name: &str, _f: |&mut IntsDecoder| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }

        fn read_enum_variant<T>(&mut self,
                                _names: &[&str],
                                _f: |&mut IntsDecoder, uint| -> Result<T, Error>)
                                -> Result<T, Error> { Err(SyntaxError) }
        fn read_enum_variant_arg<T>(&mut self,
                                    _a_idx: uint,
                                    _f: |&mut IntsDecoder| -> Result<T, Error>)
                                    -> Result<T, Error> { Err(SyntaxError) }

        fn read_enum_struct_variant<T>(&mut self,
                                       _names: &[&str],
                                       _f: |&mut IntsDecoder, uint| -> Result<T, Error>)
                                       -> Result<T, Error> { Err(SyntaxError) }
        fn read_enum_struct_variant_field<T>(&mut self,
                                             _f_name: &str,
                                             _f_idx: uint,
                                             _f: |&mut IntsDecoder| -> Result<T, Error>)
                                             -> Result<T, Error> { Err(SyntaxError) }

        fn read_struct<T>(&mut self, _s_name: &str, _len: uint, _f: |&mut IntsDecoder| -> Result<T, Error>)
                          -> Result<T, Error> { Err(SyntaxError) }
        fn read_struct_field<T>(&mut self,
                                _f_name: &str,
                                _f_idx: uint,
                                _f: |&mut IntsDecoder| -> Result<T, Error>)
                                -> Result<T, Error> { Err(SyntaxError) }

        fn read_tuple<T>(&mut self, _f: |&mut IntsDecoder, uint| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }
        fn read_tuple_arg<T>(&mut self, _a_idx: uint, _f: |&mut IntsDecoder| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }

        fn read_tuple_struct<T>(&mut self,
                                _s_name: &str,
                                _f: |&mut IntsDecoder, uint| -> Result<T, Error>)
                                -> Result<T, Error> { Err(SyntaxError) }
        fn read_tuple_struct_arg<T>(&mut self,
                                    _a_idx: uint,
                                    _f: |&mut IntsDecoder| -> Result<T, Error>)
                                    -> Result<T, Error> { Err(SyntaxError) }

        // Specialized types:
        fn read_option<T>(&mut self, _f: |&mut IntsDecoder, bool| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }

        #[inline]
        fn read_seq<T>(&mut self, f: |&mut IntsDecoder, uint| -> Result<T, Error>) -> Result<T, Error> {
            f(self, 3)
        }
        #[inline]
        fn read_seq_elt<T>(&mut self, _idx: uint, f: |&mut IntsDecoder| -> Result<T, Error>) -> Result<T, Error> {
            f(self)
        }

        fn read_map<T>(&mut self, _f: |&mut IntsDecoder, uint| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }
        fn read_map_elt_key<T>(&mut self, _idx: uint, _f: |&mut IntsDecoder| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }
        fn read_map_elt_val<T>(&mut self, _idx: uint, _f: |&mut IntsDecoder| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }
    }

    #[test]
    fn test_tokens_int() {
        let tokens = vec!(
            Int(5),
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: Result<int, Error> = Deserializable::deserialize(&mut deserializer);

        assert_eq!(value.unwrap(), 5);
    }

    #[test]
    fn test_tokens_strbuf() {
        let tokens = vec!(
            StrBuf("a".to_strbuf()),
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: Result<StrBuf, Error> = Deserializable::deserialize(&mut deserializer);

        assert_eq!(value.unwrap(), "a".to_strbuf());
    }


    #[test]
    fn test_tokens_tuple_empty() {
        let tokens = vec!(
            CollectionStart,
            CollectionEnd,
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: Result<(), Error> = Deserializable::deserialize(&mut deserializer);

        assert_eq!(value.unwrap(), ());
    }

    #[test]
    fn test_tokens_tuple() {
        let tokens = vec!(
            CollectionStart,
                CollectionSep,
                Int(5),

                CollectionSep,
                StrBuf("a".to_strbuf()),
            CollectionEnd,
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: Result<(int, StrBuf), Error> = Deserializable::deserialize(&mut deserializer);

        assert_eq!(value.unwrap(), (5, "a".to_strbuf()));
    }

    #[test]
    fn test_tokens_tuple_compound() {
        let tokens = vec!(
            CollectionStart,
                CollectionSep,
                CollectionStart,
                CollectionEnd,

                CollectionSep,
                CollectionStart,
                    CollectionSep,
                    Int(5),
                    CollectionSep,
                    StrBuf("a".to_strbuf()),
                CollectionEnd,
            CollectionEnd,
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: Result<((), (int, StrBuf)), Error> = Deserializable::deserialize(&mut deserializer);

        assert_eq!(value.unwrap(), ((), (5, "a".to_strbuf())));
    }

    #[test]
    fn test_tokens_vec_empty() {
        let tokens = vec!(
            CollectionStart,
            CollectionEnd,
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: Result<Vec<int>, Error> = Deserializable::deserialize(&mut deserializer);

        assert_eq!(value.unwrap(), vec!());
    }

    #[test]
    fn test_tokens_vec() {
        let tokens = vec!(
            CollectionStart,
                CollectionSep,
                Int(5),

                CollectionSep,
                Int(6),

                CollectionSep,
                Int(7),
            CollectionEnd,
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: Result<Vec<int>, Error> = Deserializable::deserialize(&mut deserializer);

        assert_eq!(value.unwrap(), vec!(5, 6, 7));
    }

    #[test]
    fn test_tokens_vec_compound() {
        let tokens = vec!(
            CollectionStart,
                CollectionSep,
                CollectionStart,
                    CollectionSep,
                    Int(1),
                CollectionEnd,

                CollectionSep,
                CollectionStart,
                    CollectionSep,
                    Int(2),

                    CollectionSep,
                    Int(3),
                CollectionEnd,

                CollectionSep,
                CollectionStart,
                    CollectionSep,
                    Int(4),

                    CollectionSep,
                    Int(5),

                    CollectionSep,
                    Int(6),
                CollectionEnd,
            CollectionEnd,
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: Result<Vec<Vec<int>>, Error> = Deserializable::deserialize(&mut deserializer);

        assert_eq!(value.unwrap(), vec!(vec!(1), vec!(2, 3), vec!(4, 5, 6)));
    }

    #[test]
    fn test_tokens_hashmap() {
        let tokens = vec!(
            CollectionStart,
                CollectionSep,
                CollectionStart,
                    CollectionSep,
                    Int(5),

                    CollectionSep,
                    StrBuf("a".to_strbuf()),
                CollectionEnd,

                CollectionSep,
                CollectionStart,
                    CollectionSep,
                    Int(6),

                    CollectionSep,
                    StrBuf("b".to_strbuf()),
                CollectionEnd,
            CollectionEnd,
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: Result<HashMap<int, StrBuf>, Error> = Deserializable::deserialize(&mut deserializer);

        let mut map = HashMap::new();
        map.insert(5, "a".to_strbuf());
        map.insert(6, "b".to_strbuf());

        assert_eq!(value.unwrap(), map);
    }

    #[bench]
    fn bench_dummy_deserializer(b: &mut Bencher) {
        b.iter(|| {
            let tokens = vec!(
                CollectionStart,
                    CollectionSep,
                    Int(5),

                    CollectionSep,
                    Int(6),

                    CollectionSep,
                    Int(7),
                CollectionEnd,
            );

            let mut d = TokenDeserializer::new(tokens);
            let value: Result<Vec<int>, Error> = Deserializable::deserialize(&mut d);

            assert_eq!(value.unwrap(), vec!(5, 6, 7));
        })
    }

    #[bench]
    fn bench_ints_deserializer(b: &mut Bencher) {
        b.iter(|| {
            let ints = vec!(5, 6, 7);

            let mut d = IntsDeserializer::new(ints);
            let value: Result<Vec<int>, Error> = Deserializable::deserialize(&mut d);

            assert_eq!(value.unwrap(), vec!(5, 6, 7));
        })
    }

    #[bench]
    fn bench_ints_decoder(b: &mut Bencher) {
        b.iter(|| {
            let ints = vec!(5, 6, 7);

            let mut d = IntsDecoder::new(ints);
            let value: Result<Vec<int>, Error> = Decodable::decode(&mut d);

            assert_eq!(value.unwrap(), vec!(5, 6, 7));
        })
    }
}
