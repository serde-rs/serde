extern crate collections;

use std::hash::Hash;
use std::num;
use std::result;
use collections::HashMap;

#[deriving(Clone, Eq)]
pub enum Token {
    Null,
    Bool(bool),
    Int(int),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    Uint(uint),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Char(char),
    Str(&'static str),
    StrBuf(StrBuf),
    Option(bool),
    CollectionStart(uint),
    CollectionSep,
    CollectionEnd,
}

macro_rules! expect_token {
    () => {
        match self.next() {
            Some(token) => token,
            None => { return Err(self.end_of_stream_error()); }
        }
    }
}

macro_rules! match_token {
    ($( $Variant:pat => $E:expr ),+) => {
        match expect_token!() {
            $( Ok($Variant) => $E ),+,
            Ok(_) => { return Err(self.syntax_error()); }
            Err(err) => { return Err(err); }
        }
    }
}

macro_rules! to_result {
    ($expr:expr, $err:expr) => {
        match $expr {
            Some(value) => Ok(value),
            None => Err($err),
        }
    }
}

pub trait Deserializer<E>: Iterator<Result<Token, E>> {
    fn end_of_stream_error(&self) -> E;

    fn syntax_error(&self) -> E;

    #[inline]
    fn expect_null(&mut self) -> Result<(), E> {
        match_token! {
            Null => Ok(()),
            CollectionStart(_) => self.expect_collection_end()
        }
    }

    #[inline]
    fn expect_bool(&mut self) -> Result<bool, E> {
        match_token! {
            Bool(value) => Ok(value)
        }
    }

    #[inline]
    fn expect_num<T: NumCast>(&mut self) -> Result<T, E> {
        match_token! {
            Int(x) => to_result!(num::cast(x), self.syntax_error()),
            I8(x) => to_result!(num::cast(x), self.syntax_error()),
            I16(x) => to_result!(num::cast(x), self.syntax_error()),
            I32(x) => to_result!(num::cast(x), self.syntax_error()),
            I64(x) => to_result!(num::cast(x), self.syntax_error()),
            Uint(x) => to_result!(num::cast(x), self.syntax_error()),
            U8(x) => to_result!(num::cast(x), self.syntax_error()),
            U16(x) => to_result!(num::cast(x), self.syntax_error()),
            U32(x) => to_result!(num::cast(x), self.syntax_error()),
            U64(x) => to_result!(num::cast(x), self.syntax_error()),
            F32(x) => to_result!(num::cast(x), self.syntax_error()),
            F64(x) => to_result!(num::cast(x), self.syntax_error())
        }
    }

    #[inline]
    fn expect_char(&mut self) -> Result<char, E> {
        match_token! {
            Char(value) => Ok(value)
        }
    }

    #[inline]
    fn expect_str(&mut self) -> Result<&'static str, E> {
        match_token! {
            Str(value) => Ok(value)
        }
    }

    #[inline]
    fn expect_strbuf(&mut self) -> Result<StrBuf, E> {
        match_token! {
            Str(value) => Ok(value.to_strbuf()),
            StrBuf(value) => Ok(value)
        }
    }

    #[inline]
    fn expect_option<
        T: Deserializable<E, Self>
    >(&mut self) -> Result<Option<T>, E> {
        match_token! {
            Option(false) => Ok(None),
            Option(true) => {
                let value: T = try!(Deserializable::deserialize(self));
                Ok(Some(value))
            }
        }
    }

    #[inline]
    fn expect_collection<
        T: Deserializable<E, Self>,
        C: FromIterator<T>
    >(&mut self) -> Result<C, E> {
        let len = try!(self.expect_collection_start());

        let iter = self.by_ref().batch(|d| {
            let d = d.iter();

            let token = match d.next() {
                Some(token) => token,
                None => { return None; }
            };

            match token {
                Ok(CollectionEnd) => None,
                Ok(CollectionSep) => {
                    let value: Result<T, E> = Deserializable::deserialize(d);
                    Some(value)
                }
                Ok(_) => Some(Err(d.syntax_error())),
                Err(e) => Some(Err(e)),
            }
        });

        result::collect_with_capacity(iter, len)
    }

    #[inline]
    fn expect_collection_start(&mut self) -> Result<uint, E> {
        match_token! {
            CollectionStart(len) => Ok(len)
        }
    }

    #[inline]
    fn expect_collection_sep(&mut self) -> Result<(), E> {
        match_token! {
            CollectionSep => Ok(())
        }
    }

    #[inline]
    fn expect_collection_end(&mut self) -> Result<(), E> {
        match_token! {
            CollectionEnd => Ok(())
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

pub trait Deserializable<E, D: Deserializer<E>> {
    fn deserialize(d: &mut D) -> Result<Self, E>;
}

//////////////////////////////////////////////////////////////////////////////

macro_rules! impl_deserializable {
    ($ty:ty, $method:ident) => {
        impl<
            E,
            D: Deserializer<E>
        > Deserializable<E, D> for $ty {
            #[inline]
            fn deserialize(d: &mut D) -> Result<$ty, E> {
                d.$method()
            }
        }
    }
}

impl_deserializable!(bool, expect_bool)
impl_deserializable!(int, expect_num)
impl_deserializable!(i8, expect_num)
impl_deserializable!(i16, expect_num)
impl_deserializable!(i32, expect_num)
impl_deserializable!(i64, expect_num)
impl_deserializable!(uint, expect_num)
impl_deserializable!(u8, expect_num)
impl_deserializable!(u16, expect_num)
impl_deserializable!(u32, expect_num)
impl_deserializable!(u64, expect_num)
impl_deserializable!(f32, expect_num)
impl_deserializable!(f64, expect_num)
impl_deserializable!(char, expect_char)
impl_deserializable!(&'static str, expect_str)
impl_deserializable!(StrBuf, expect_strbuf)

//////////////////////////////////////////////////////////////////////////////

impl<
    E,
    D: Deserializer<E>,
    T: Deserializable<E, D>
> Deserializable<E, D> for Option<T> {
    #[inline]
    fn deserialize(d: &mut D) -> Result<Option<T>, E> {
        d.expect_option()
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

macro_rules! peel(($name:ident, $($other:ident,)*) => (deserialize_tuple!($($other,)*)))

macro_rules! deserialize_tuple (
    () => ();
    ( $($name:ident,)+ ) => (
        impl<
            E,
            D: Deserializer<E>,
            $($name:Deserializable<E, D>),*
        > Deserializable<E, D> for ($($name,)*) {
            #[inline]
            #[allow(uppercase_variables)]
            fn deserialize(d: &mut D) -> Result<($($name,)*), E> {
                try!(d.expect_collection_start());

                let result = ($(
                    {
                        try!(d.expect_collection_sep());
                        let $name = try!(Deserializable::deserialize(d));
                        $name
                    }
                    ,)*);

                try!(d.expect_collection_end());

                Ok(result)
            }
        }
        peel!($($name,)*)
    )
)

deserialize_tuple! { T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, }

//////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    extern crate serialize;

    use std::num;
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
        SepOrEnd,
        Value,
        End,
    }

    struct IntsDeserializer {
        state: IntsDeserializerState,
        len: uint,
        iter: vec::MoveItems<int>,
        value: Option<int>
    }

    impl IntsDeserializer {
        #[inline]
        fn new(values: Vec<int>) -> IntsDeserializer {
            IntsDeserializer {
                state: Start,
                len: values.len(),
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
                    self.state = SepOrEnd;
                    Some(Ok(CollectionStart(self.len)))
                }
                SepOrEnd => {
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
                    self.state = SepOrEnd;
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
        fn expect_num<T: NumCast>(&mut self) -> Result<T, Error> {
            assert_eq!(self.state, Value);

            self.state = SepOrEnd;

            match self.value.take() {
                Some(value) => {
                    match num::cast(value) {
                        Some(value) => Ok(value),
                        None => Err(self.syntax_error()),
                    }
                }
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
            CollectionStart(0),
            CollectionEnd,
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: Result<(), Error> = Deserializable::deserialize(&mut deserializer);

        assert_eq!(value.unwrap(), ());
    }

    #[test]
    fn test_tokens_tuple() {
        let tokens = vec!(
            CollectionStart(2),
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
            CollectionStart(2),
                CollectionSep,
                CollectionStart(0),
                CollectionEnd,

                CollectionSep,
                CollectionStart(2),
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
            CollectionStart(0),
            CollectionEnd,
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: Result<Vec<int>, Error> = Deserializable::deserialize(&mut deserializer);

        assert_eq!(value.unwrap(), vec!());
    }

    #[test]
    fn test_tokens_vec() {
        let tokens = vec!(
            CollectionStart(3),
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
            CollectionStart(0),
                CollectionSep,
                CollectionStart(1),
                    CollectionSep,
                    Int(1),
                CollectionEnd,

                CollectionSep,
                CollectionStart(2),
                    CollectionSep,
                    Int(2),

                    CollectionSep,
                    Int(3),
                CollectionEnd,

                CollectionSep,
                CollectionStart(3),
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
            CollectionStart(2),
                CollectionSep,
                CollectionStart(2),
                    CollectionSep,
                    Int(5),

                    CollectionSep,
                    StrBuf("a".to_strbuf()),
                CollectionEnd,

                CollectionSep,
                CollectionStart(2),
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
                CollectionStart(3),
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
