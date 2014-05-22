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

    TupleStart(uint),

    StructStart(&'static str),
    StructField(&'static str),

    EnumStart(&'static str),
    EnumVariant(&'static str),

    SeqStart(uint),
    MapStart(uint),

    Sep,
    End,
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
    ($( $variant:pat => $expr:expr ),+) => {
        match expect_token!() {
            $( Ok($variant) => $expr ),+,
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
            TupleStart(_) => {
                match_token! {
                    End => Ok(())
                }
            }
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
    fn expect_tuple_start(&mut self, len: uint) -> Result<(), E> {
        match_token! {
            TupleStart(l) => {
                if len == l {
                    Ok(())
                } else {
                    Err(self.syntax_error())
                }
            }
        }
    }

    #[inline]
    fn expect_tuple_elt<T: Deserializable<E, Self>>(&mut self) -> Result<T, E> {
        match_token! {
            Sep => Deserializable::deserialize(self)
        }
    }

    #[inline]
    fn expect_struct_start(&mut self, name: &str) -> Result<(), E> {
        match_token! {
            StructStart(n) => {
                if name == n {
                    Ok(())
                } else {
                    Err(self.syntax_error())
                }
            }
        }
    }

    #[inline]
    fn expect_struct_field<
        T: Deserializable<E, Self>
    >(&mut self, name: &str) -> Result<T, E> {
        match_token! {
            StructField(n) => {
                if name == n {
                    Deserializable::deserialize(self)
                } else {
                    Err(self.syntax_error())
                }
            }
        }
    }

    #[inline]
    fn expect_enum_start<'a>(&mut self, name: &str, variants: &[&str]) -> Result<uint, E> {
        match_token! {
            EnumStart(n) => {
                if name == n {
                    match_token! {
                        EnumVariant(n) => {
                            match variants.iter().position(|variant| *variant == n) {
                                Some(position) => Ok(position),
                                None => Err(self.syntax_error()),
                            }
                        }
                    }
                } else {
                    Err(self.syntax_error())
                }
            }
        }
    }

    #[inline]
    fn expect_collection<
        T: Deserializable<E, Self>,
        C: FromIterator<T>
    >(&mut self) -> Result<C, E> {
        // By default we don't care what our source input was. We can take
        // anything that's a Collection<T>. We'll error out later if the types
        // are wrong.
        let len = match_token! {
            TupleStart(len) => len,
            SeqStart(len) => len,
            MapStart(len) => len
        };

        expect_rest_of_collection(self, len)
    }

    #[inline]
    fn expect_seq<
        T: Deserializable<E, Self>,
        C: FromIterator<T>
    >(&mut self) -> Result<C, E> {
        let len = match_token! {
            SeqStart(len) => len
        };

        expect_rest_of_collection(self, len)
    }

    #[inline]
    fn expect_map<
        K: Deserializable<E, Self>,
        V: Deserializable<E, Self>,
        C: FromIterator<(K, V)>
    >(&mut self) -> Result<C, E> {
        let len = match_token! {
            MapStart(len) => len
        };

        expect_rest_of_collection(self, len)
    }

    #[inline]
    fn expect_end(&mut self) -> Result<(), E> {
        match_token! {
            End => Ok(())
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

fn expect_rest_of_collection<
    E,
    D: Deserializer<E>,
    T: Deserializable<E, D>,
    C: FromIterator<T>
>(d: &mut D, len: uint) -> Result<C, E> {
    let iter = d.by_ref().batch(|d| {
        let d = d.iter();

        let token = match d.next() {
            Some(token) => token,
            None => { return None; }
        };

        match token {
            Ok(Sep) => {
                let value: Result<T, E> = Deserializable::deserialize(d);
                Some(value)
            }
            Ok(End) => None,
            Ok(_) => Some(Err(d.syntax_error())),
            Err(e) => Some(Err(e)),
        }
    });

    result::collect_with_capacity(iter, len)
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
        d.expect_seq()
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
        d.expect_map()
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
                // FIXME: how can we count macro args?
                let mut len = 0;
                $({ let $name = 1; len += $name; })*;

                try!(d.expect_tuple_start(len));

                let result = ($({
                    let $name = try!(d.expect_tuple_elt());
                    $name
                },)*);

                try!(d.expect_end());

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
    use std::num;
    use std::vec;
    use collections::HashMap;
    use test::Bencher;

    use serialize::{Decoder, Decodable};

    use super::{Token, Null, Int, Uint, Str, StrBuf, Char, Option};
    use super::{TupleStart, StructStart, StructField, EnumStart, EnumVariant};
    use super::{SeqStart, MapStart, Sep, End};
    use super::{Deserializer, Deserializable};

    //////////////////////////////////////////////////////////////////////////////

    #[deriving(Clone, Eq, Show, Decodable)]
    struct Inner {
        a: (),
        b: uint,
        c: HashMap<StrBuf, Option<char>>,
    }

    impl<E, D: Deserializer<E>> Deserializable<E, D> for Inner {
        #[inline]
        fn deserialize(d: &mut D) -> Result<Inner, E> {
            try!(d.expect_struct_start("Inner"));
            let a = try!(d.expect_struct_field("a"));
            let b = try!(d.expect_struct_field("b"));
            let c = try!(d.expect_struct_field("c"));
            try!(d.expect_end());
            Ok(Inner { a: a, b: b, c: c })
        }
    }

    //////////////////////////////////////////////////////////////////////////////

    #[deriving(Clone, Eq, Show, Decodable)]
    struct Outer {
        inner: Vec<Inner>,
    }

    impl<E, D: Deserializer<E>> Deserializable<E, D> for Outer {
        #[inline]
        fn deserialize(d: &mut D) -> Result<Outer, E> {
            try!(d.expect_struct_start("Outer"));
            let inner = try!(d.expect_struct_field("inner"));
            try!(d.expect_end());
            Ok(Outer { inner: inner })
        }
    }

    //////////////////////////////////////////////////////////////////////////////

    #[deriving(Clone, Eq, Show, Decodable)]
    enum Animal {
        Dog,
        Frog(StrBuf, int)
    }

    impl<E, D: Deserializer<E>> Deserializable<E, D> for Animal {
        #[inline]
        fn deserialize(d: &mut D) -> Result<Animal, E> {
            match try!(d.expect_enum_start("Animal", ["Dog", "Frog"])) {
                0 => {
                    try!(d.expect_end());
                    Ok(Dog)
                }
                1 => {
                    let x0 = try!(Deserializable::deserialize(d));
                    let x1 = try!(Deserializable::deserialize(d));
                    try!(d.expect_end());
                    Ok(Frog(x0, x1))
                }
                _ => unreachable!(),
            }
        }
    }

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
        IntsDeserializserStartState,
        IntsDeserializserSepOrEndState,
        IntsDeserializserValueState,
        IntsDeserializserEndState,
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
                state: IntsDeserializserStartState,
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
                IntsDeserializserStartState => {
                    self.state = IntsDeserializserSepOrEndState;
                    Some(Ok(SeqStart(self.len)))
                }
                IntsDeserializserSepOrEndState => {
                    match self.iter.next() {
                        Some(value) => {
                            self.state = IntsDeserializserValueState;
                            self.value = Some(value);
                            Some(Ok(Sep))
                        }
                        None => {
                            self.state = IntsDeserializserEndState;
                            Some(Ok(End))
                        }
                    }
                }
                IntsDeserializserValueState => {
                    self.state = IntsDeserializserSepOrEndState;
                    match self.value.take() {
                        Some(value) => Some(Ok(Int(value))),
                        None => Some(Err(self.end_of_stream_error())),
                    }
                }
                IntsDeserializserEndState => {
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
            assert_eq!(self.state, IntsDeserializserValueState);

            self.state = IntsDeserializserSepOrEndState;

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

    //////////////////////////////////////////////////////////////////////////////

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

    //////////////////////////////////////////////////////////////////////////////

    enum AnimalDecoderState {
        AnimalDecoderAnimalState(Animal),
        AnimalDecoderDogState,
        AnimalDecoderFrogState,
        AnimalDecoderIntState(int),
        AnimalDecoderStrState(StrBuf),
    }

    struct AnimalDecoder {
        stack: Vec<AnimalDecoderState>,

    }

    impl AnimalDecoder {
        #[inline]
        fn new(animal: Animal) -> AnimalDecoder {
            AnimalDecoder {
                stack: vec!(AnimalDecoderAnimalState(animal)),
            }
        }
    }

    impl Decoder<Error> for AnimalDecoder {
        // Primitive types:
        fn read_nil(&mut self) -> Result<(), Error> { Err(SyntaxError) }
        fn read_uint(&mut self) -> Result<uint, Error> { Err(SyntaxError) }
        fn read_u64(&mut self) -> Result<u64, Error> { Err(SyntaxError) }
        fn read_u32(&mut self) -> Result<u32, Error> { Err(SyntaxError) }
        fn read_u16(&mut self) -> Result<u16, Error> { Err(SyntaxError) }
        fn read_u8(&mut self) -> Result<u8, Error> { Err(SyntaxError) }
        #[inline]
        fn read_int(&mut self) -> Result<int, Error> {
            match self.stack.pop() {
                Some(AnimalDecoderIntState(x)) => Ok(x),
                _ => Err(SyntaxError),
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
        #[inline]
        fn read_str(&mut self) -> Result<StrBuf, Error> {
            match self.stack.pop() {
                Some(AnimalDecoderStrState(x)) => Ok(x),
                _ => Err(SyntaxError),
            }
        }

        // Compound types:
        #[inline]
        fn read_enum<T>(&mut self, name: &str, f: |&mut AnimalDecoder| -> Result<T, Error>) -> Result<T, Error> {
            match self.stack.pop() {
                Some(AnimalDecoderAnimalState(animal)) => {
                    self.stack.push(AnimalDecoderAnimalState(animal));
                    if name == "Animal" {
                        f(self)
                    } else {
                        Err(SyntaxError)
                    }
                }
                _ => Err(SyntaxError)
            }
        }

        #[inline]
        fn read_enum_variant<T>(&mut self, names: &[&str], f: |&mut AnimalDecoder, uint| -> Result<T, Error>) -> Result<T, Error> {
            let name = match self.stack.pop() {
                Some(AnimalDecoderAnimalState(Dog)) => "Dog",
                Some(AnimalDecoderAnimalState(Frog(x0, x1))) => {
                    self.stack.push(AnimalDecoderIntState(x1));
                    self.stack.push(AnimalDecoderStrState(x0));
                    "Frog"
                }
                _ => { return Err(SyntaxError); }
            };

            let idx = match names.iter().position(|n| *n == name) {
                Some(idx) => idx,
                None => { return Err(SyntaxError); }
            };

            f(self, idx)
        }
        #[inline]
        fn read_enum_variant_arg<T>(&mut self, _a_idx: uint, f: |&mut AnimalDecoder| -> Result<T, Error>) -> Result<T, Error> {
            f(self)
        }
        fn read_enum_struct_variant<T>(&mut self,
                                       _names: &[&str],
                                       _f: |&mut AnimalDecoder, uint| -> Result<T, Error>)
                                       -> Result<T, Error> { Err(SyntaxError) }
        fn read_enum_struct_variant_field<T>(&mut self,
                                             _f_name: &str,
                                             _f_idx: uint,
                                             _f: |&mut AnimalDecoder| -> Result<T, Error>)
                                             -> Result<T, Error> { Err(SyntaxError) }

        fn read_struct<T>(&mut self, _s_name: &str, _len: uint, _f: |&mut AnimalDecoder| -> Result<T, Error>)
                          -> Result<T, Error> { Err(SyntaxError) }
        fn read_struct_field<T>(&mut self,
                                _f_name: &str,
                                _f_idx: uint,
                                _f: |&mut AnimalDecoder| -> Result<T, Error>)
                                -> Result<T, Error> { Err(SyntaxError) }

        fn read_tuple<T>(&mut self, _f: |&mut AnimalDecoder, uint| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }
        fn read_tuple_arg<T>(&mut self, _a_idx: uint, _f: |&mut AnimalDecoder| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }

        fn read_tuple_struct<T>(&mut self,
                                _s_name: &str,
                                _f: |&mut AnimalDecoder, uint| -> Result<T, Error>)
                                -> Result<T, Error> { Err(SyntaxError) }
        fn read_tuple_struct_arg<T>(&mut self,
                                    _a_idx: uint,
                                    _f: |&mut AnimalDecoder| -> Result<T, Error>)
                                    -> Result<T, Error> { Err(SyntaxError) }

        // Specialized types:
        fn read_option<T>(&mut self, _f: |&mut AnimalDecoder, bool| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }

        #[inline]
        fn read_seq<T>(&mut self, f: |&mut AnimalDecoder, uint| -> Result<T, Error>) -> Result<T, Error> {
            f(self, 3)
        }
        #[inline]
        fn read_seq_elt<T>(&mut self, _idx: uint, f: |&mut AnimalDecoder| -> Result<T, Error>) -> Result<T, Error> {
            f(self)
        }

        fn read_map<T>(&mut self, _f: |&mut AnimalDecoder, uint| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }
        fn read_map_elt_key<T>(&mut self, _idx: uint, _f: |&mut AnimalDecoder| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }
        fn read_map_elt_val<T>(&mut self, _idx: uint, _f: |&mut AnimalDecoder| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }
    }

    //////////////////////////////////////////////////////////////////////////////

    enum AnimalDeserializerState {
        AnimalDeserializerAnimalState(Animal),
        AnimalDeserializerDogState,
        AnimalDeserializerFrogState,
        AnimalDeserializerIntState(int),
        AnimalDeserializerStrState(StrBuf),
        AnimalDeserializerEndState,

    }

    struct AnimalDeserializer {
        stack: Vec<AnimalDeserializerState>,
    }

    impl AnimalDeserializer {
        #[inline]
        fn new(animal: Animal) -> AnimalDeserializer {
            AnimalDeserializer {
                stack: vec!(AnimalDeserializerAnimalState(animal)),
            }
        }
    }

    impl Iterator<Result<Token, Error>> for AnimalDeserializer {
        #[inline]
        fn next(&mut self) -> Option<Result<Token, Error>> {
            match self.stack.pop() {
                Some(AnimalDeserializerAnimalState(Dog)) => {
                    self.stack.push(AnimalDeserializerEndState);
                    self.stack.push(AnimalDeserializerDogState);
                    Some(Ok(EnumStart("Animal")))
                }
                Some(AnimalDeserializerAnimalState(Frog(x0, x1))) => {
                    self.stack.push(AnimalDeserializerEndState);
                    self.stack.push(AnimalDeserializerIntState(x1));
                    self.stack.push(AnimalDeserializerStrState(x0));
                    self.stack.push(AnimalDeserializerFrogState);
                    Some(Ok(EnumStart("Animal")))
                }
                Some(AnimalDeserializerDogState) => {
                    Some(Ok(EnumVariant("Dog")))
                }
                Some(AnimalDeserializerFrogState) => {
                    Some(Ok(EnumVariant("Frog")))
                }
                Some(AnimalDeserializerIntState(x)) => {
                    Some(Ok(Int(x)))
                }
                Some(AnimalDeserializerStrState(x)) => {
                    Some(Ok(StrBuf(x)))
                }
                Some(AnimalDeserializerEndState) => {
                    Some(Ok(End))
                }
                None => None,
            }
        }
    }

    impl Deserializer<Error> for AnimalDeserializer {
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

    #[deriving(Show)]
    enum OuterDecoderState {
        OuterDecoderOuterState(Outer),
        OuterDecoderInnerState(Inner),
        OuterDecoderNullState,
        OuterDecoderUintState(uint),
        OuterDecoderCharState(char),
        OuterDecoderStrState(StrBuf),
        OuterDecoderFieldState(&'static str),
        OuterDecoderVecState(Vec<Inner>),
        OuterDecoderMapState(HashMap<StrBuf, Option<char>>),
        OuterDecoderOptionState(bool),
    }

    struct OuterDecoder {
        stack: Vec<OuterDecoderState>,

    }

    impl OuterDecoder {
        #[inline]
        fn new(animal: Outer) -> OuterDecoder {
            OuterDecoder {
                stack: vec!(OuterDecoderOuterState(animal)),
            }
        }
    }

    impl Decoder<Error> for OuterDecoder {
        // Primitive types:
        #[inline]
        fn read_nil(&mut self) -> Result<(), Error> {
            match self.stack.pop() {
                Some(OuterDecoderNullState) => Ok(()),
                _ => Err(SyntaxError),
            }
        }
        #[inline]
        fn read_uint(&mut self) -> Result<uint, Error> {
            match self.stack.pop() {
                Some(OuterDecoderUintState(value)) => Ok(value),
                _ => Err(SyntaxError),
            }
        }
        fn read_u64(&mut self) -> Result<u64, Error> { Err(SyntaxError) }
        fn read_u32(&mut self) -> Result<u32, Error> { Err(SyntaxError) }
        fn read_u16(&mut self) -> Result<u16, Error> { Err(SyntaxError) }
        fn read_u8(&mut self) -> Result<u8, Error> { Err(SyntaxError) }
        fn read_int(&mut self) -> Result<int, Error> { Err(SyntaxError) }
        fn read_i64(&mut self) -> Result<i64, Error> { Err(SyntaxError) }
        fn read_i32(&mut self) -> Result<i32, Error> { Err(SyntaxError) }
        fn read_i16(&mut self) -> Result<i16, Error> { Err(SyntaxError) }
        fn read_i8(&mut self) -> Result<i8, Error> { Err(SyntaxError) }
        fn read_bool(&mut self) -> Result<bool, Error> { Err(SyntaxError) }
        fn read_f64(&mut self) -> Result<f64, Error> { Err(SyntaxError) }
        fn read_f32(&mut self) -> Result<f32, Error> { Err(SyntaxError) }
        #[inline]
        fn read_char(&mut self) -> Result<char, Error> {
            match self.stack.pop() {
                Some(OuterDecoderCharState(c)) => Ok(c),
                _ => Err(SyntaxError),
            }
        }
        #[inline]
        fn read_str(&mut self) -> Result<StrBuf, Error> {
            match self.stack.pop() {
                Some(OuterDecoderStrState(value)) => Ok(value),
                _ => Err(SyntaxError),
            }
        }

        // Compound types:
        fn read_enum<T>(&mut self, _name: &str, _f: |&mut OuterDecoder| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }

        fn read_enum_variant<T>(&mut self,
                                _names: &[&str],
                                _f: |&mut OuterDecoder, uint| -> Result<T, Error>)
                                -> Result<T, Error> { Err(SyntaxError) }
        fn read_enum_variant_arg<T>(&mut self,
                                    _a_idx: uint,
                                    _f: |&mut OuterDecoder| -> Result<T, Error>)
                                    -> Result<T, Error> { Err(SyntaxError) }

        fn read_enum_struct_variant<T>(&mut self,
                                       _names: &[&str],
                                       _f: |&mut OuterDecoder, uint| -> Result<T, Error>)
                                       -> Result<T, Error> { Err(SyntaxError) }
        fn read_enum_struct_variant_field<T>(&mut self,
                                             _f_name: &str,
                                             _f_idx: uint,
                                             _f: |&mut OuterDecoder| -> Result<T, Error>)
                                             -> Result<T, Error> { Err(SyntaxError) }

        #[inline]
        fn read_struct<T>(&mut self, s_name: &str, _len: uint, f: |&mut OuterDecoder| -> Result<T, Error>) -> Result<T, Error> {
            match self.stack.pop() {
                Some(OuterDecoderOuterState(Outer { inner: inner })) => {
                    if s_name == "Outer" {
                        self.stack.push(OuterDecoderVecState(inner));
                        self.stack.push(OuterDecoderFieldState("inner"));
                        f(self)
                    } else {
                        Err(SyntaxError)
                    }
                }
                Some(OuterDecoderInnerState(Inner { a: (), b: b, c: c })) => {
                    if s_name == "Inner" {
                        self.stack.push(OuterDecoderMapState(c));
                        self.stack.push(OuterDecoderFieldState("c"));

                        self.stack.push(OuterDecoderUintState(b));
                        self.stack.push(OuterDecoderFieldState("b"));

                        self.stack.push(OuterDecoderNullState);
                        self.stack.push(OuterDecoderFieldState("a"));
                        f(self)
                    } else {
                        Err(SyntaxError)
                    }
                }
                _ => Err(SyntaxError),
            }
        }
        #[inline]
        fn read_struct_field<T>(&mut self, f_name: &str, _f_idx: uint, f: |&mut OuterDecoder| -> Result<T, Error>) -> Result<T, Error> {
            match self.stack.pop() {
                Some(OuterDecoderFieldState(name)) => {
                    if f_name == name {
                        f(self)
                    } else {
                        Err(SyntaxError)
                    }
                }
                _ => Err(SyntaxError)
            }
        }

        fn read_tuple<T>(&mut self, _f: |&mut OuterDecoder, uint| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }
        fn read_tuple_arg<T>(&mut self, _a_idx: uint, _f: |&mut OuterDecoder| -> Result<T, Error>) -> Result<T, Error> { Err(SyntaxError) }

        fn read_tuple_struct<T>(&mut self,
                                _s_name: &str,
                                _f: |&mut OuterDecoder, uint| -> Result<T, Error>)
                                -> Result<T, Error> { Err(SyntaxError) }
        fn read_tuple_struct_arg<T>(&mut self,
                                    _a_idx: uint,
                                    _f: |&mut OuterDecoder| -> Result<T, Error>)
                                    -> Result<T, Error> { Err(SyntaxError) }

        // Specialized types:
        #[inline]
        fn read_option<T>(&mut self, f: |&mut OuterDecoder, bool| -> Result<T, Error>) -> Result<T, Error> {
            match self.stack.pop() {
                Some(OuterDecoderOptionState(b)) => f(self, b),
                _ => Err(SyntaxError),
            }
        }

        #[inline]
        fn read_seq<T>(&mut self, f: |&mut OuterDecoder, uint| -> Result<T, Error>) -> Result<T, Error> {
            match self.stack.pop() {
                Some(OuterDecoderVecState(value)) => {
                    let len = value.len();
                    for inner in value.move_iter().rev() {
                        self.stack.push(OuterDecoderInnerState(inner));
                    }
                    f(self, len)
                }
                _ => Err(SyntaxError)
            }
        }
        #[inline]
        fn read_seq_elt<T>(&mut self, _idx: uint, f: |&mut OuterDecoder| -> Result<T, Error>) -> Result<T, Error> {
            f(self)
        }

        #[inline]
        fn read_map<T>(&mut self, f: |&mut OuterDecoder, uint| -> Result<T, Error>) -> Result<T, Error> {
            match self.stack.pop() {
                Some(OuterDecoderMapState(map)) => {
                    let len = map.len();
                    for (key, value) in map.move_iter() {
                        match value {
                            Some(c) => {
                                self.stack.push(OuterDecoderCharState(c));
                                self.stack.push(OuterDecoderOptionState(true));
                            }
                            None => {
                                self.stack.push(OuterDecoderOptionState(false));
                            }
                        }
                        self.stack.push(OuterDecoderStrState(key));
                    }
                    f(self, len)
                }
                _ => Err(SyntaxError),
            }
        }
        #[inline]
        fn read_map_elt_key<T>(&mut self, _idx: uint, f: |&mut OuterDecoder| -> Result<T, Error>) -> Result<T, Error> {
            f(self)
        }
        #[inline]
        fn read_map_elt_val<T>(&mut self, _idx: uint, f: |&mut OuterDecoder| -> Result<T, Error>) -> Result<T, Error> {
            f(self)
        }
    }

    //////////////////////////////////////////////////////////////////////////////

    #[test]
    fn test_tokens_int() {
        let tokens = vec!(
            Int(5),
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: int = Deserializable::deserialize(&mut deserializer).unwrap();

        assert_eq!(value, 5);
    }

    #[test]
    fn test_tokens_str() {
        let tokens = vec!(
            Str("a"),
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: &'static str = Deserializable::deserialize(&mut deserializer).unwrap();

        assert_eq!(value, "a");
    }

    #[test]
    fn test_tokens_strbuf() {
        let tokens = vec!(
            StrBuf("a".to_strbuf()),
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: StrBuf = Deserializable::deserialize(&mut deserializer).unwrap();

        assert_eq!(value, "a".to_strbuf());
    }

    #[test]
    fn test_tokens_null() {
        let tokens = vec!(
            Null,
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: () = Deserializable::deserialize(&mut deserializer).unwrap();

        assert_eq!(value, ());
    }

    #[test]
    fn test_tokens_tuple_empty() {
        let tokens = vec!(
            TupleStart(0),
            End,
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: () = Deserializable::deserialize(&mut deserializer).unwrap();

        assert_eq!(value, ());
    }

    #[test]
    fn test_tokens_option_none() {
        let tokens = vec!(
            Option(false),
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: Option<int> = Deserializable::deserialize(&mut deserializer).unwrap();

        assert_eq!(value, None);
    }

    #[test]
    fn test_tokens_option_some() {
        let tokens = vec!(
            Option(true),
            Int(5),
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: Option<int> = Deserializable::deserialize(&mut deserializer).unwrap();

        assert_eq!(value, Some(5));
    }

    #[test]
    fn test_tokens_tuple() {
        let tokens = vec!(
            TupleStart(2),
                Sep,
                Int(5),

                Sep,
                StrBuf("a".to_strbuf()),
            End,
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: (int, StrBuf) = Deserializable::deserialize(&mut deserializer).unwrap();

        assert_eq!(value, (5, "a".to_strbuf()));
    }

    #[test]
    fn test_tokens_tuple_compound() {
        let tokens = vec!(
            TupleStart(3),
                Sep,
                Null,

                Sep,
                TupleStart(0),
                End,

                Sep,
                TupleStart(2),
                    Sep,
                    Int(5),

                    Sep,
                    StrBuf("a".to_strbuf()),
                End,
            End,
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: ((), (), (int, StrBuf)) = Deserializable::deserialize(&mut deserializer).unwrap();

        assert_eq!(value, ((), (), (5, "a".to_strbuf())));
    }

    #[test]
    fn test_tokens_struct_empty() {
        let tokens = vec!(
            StructStart("Outer"),
                StructField("inner"),
                SeqStart(0),
                End,
            End,
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: Outer = Deserializable::deserialize(&mut deserializer).unwrap();

        assert_eq!(value, Outer { inner: vec!() });
    }

    #[test]
    fn test_tokens_struct() {
        let tokens = vec!(
            StructStart("Outer"),
                StructField("inner"),
                SeqStart(1),
                    Sep,
                    StructStart("Inner"),
                        StructField("a"),
                        Null,

                        StructField("b"),
                        Uint(5),

                        StructField("c"),
                        MapStart(1),
                            Sep,
                            TupleStart(2),
                                Sep,
                                StrBuf("abc".to_strbuf()),

                                Sep,
                                Option(true),
                                Char('c'),
                            End,
                        End,
                    End,
                End,
            End,
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: Outer = Deserializable::deserialize(&mut deserializer).unwrap();

        let mut map = HashMap::new();
        map.insert("abc".to_strbuf(), Some('c'));

        assert_eq!(
            value,
            Outer {
                inner: vec!(
                    Inner {
                        a: (),
                        b: 5,
                        c: map,
                    },
                )
            });
    }

    #[test]
    fn test_tokens_enum() {
        let tokens = vec!(
            EnumStart("Animal"),
                EnumVariant("Dog"),
            End,
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: Animal = Deserializable::deserialize(&mut deserializer).unwrap();

        assert_eq!(value, Dog);

        let tokens = vec!(
            EnumStart("Animal"),
                EnumVariant("Frog"),
                StrBuf("Henry".to_strbuf()),
                Int(349),
            End,
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: Animal = Deserializable::deserialize(&mut deserializer).unwrap();

        assert_eq!(value, Frog("Henry".to_strbuf(), 349));
    }

    #[test]
    fn test_tokens_vec_empty() {
        let tokens = vec!(
            SeqStart(0),
            End,
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: Vec<int> = Deserializable::deserialize(&mut deserializer).unwrap();

        assert_eq!(value, vec!());
    }

    #[test]
    fn test_tokens_vec() {
        let tokens = vec!(
            SeqStart(3),
                Sep,
                Int(5),

                Sep,
                Int(6),

                Sep,
                Int(7),
            End,
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: Vec<int> = Deserializable::deserialize(&mut deserializer).unwrap();

        assert_eq!(value, vec!(5, 6, 7));
    }

    #[test]
    fn test_tokens_vec_compound() {
        let tokens = vec!(
            SeqStart(0),
                Sep,
                SeqStart(1),
                    Sep,
                    Int(1),
                End,

                Sep,
                SeqStart(2),
                    Sep,
                    Int(2),

                    Sep,
                    Int(3),
                End,

                Sep,
                SeqStart(3),
                    Sep,
                    Int(4),

                    Sep,
                    Int(5),

                    Sep,
                    Int(6),
                End,
            End,
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: Vec<Vec<int>> = Deserializable::deserialize(&mut deserializer).unwrap();

        assert_eq!(value, vec!(vec!(1), vec!(2, 3), vec!(4, 5, 6)));
    }

    #[test]
    fn test_tokens_hashmap() {
        let tokens = vec!(
            MapStart(2),
                Sep,
                TupleStart(2),
                    Sep,
                    Int(5),

                    Sep,
                    StrBuf("a".to_strbuf()),
                End,

                Sep,
                TupleStart(2),
                    Sep,
                    Int(6),

                    Sep,
                    StrBuf("b".to_strbuf()),
                End,
            End,
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: HashMap<int, StrBuf> = Deserializable::deserialize(&mut deserializer).unwrap();

        let mut map = HashMap::new();
        map.insert(5, "a".to_strbuf());
        map.insert(6, "b".to_strbuf());

        assert_eq!(value, map);
    }

    #[bench]
    fn bench_dummy_deserializer(b: &mut Bencher) {
        b.iter(|| {
            let tokens = vec!(
                SeqStart(3),
                    Sep,
                    Int(5),

                    Sep,
                    Int(6),

                    Sep,
                    Int(7),
                End,
            );

            let mut d = TokenDeserializer::new(tokens);
            let value: Vec<int> = Deserializable::deserialize(&mut d).unwrap();

            assert_eq!(value, vec!(5, 6, 7));
        })
    }

    #[bench]
    fn bench_ints_deserializer(b: &mut Bencher) {
        b.iter(|| {
            let ints = vec!(5, 6, 7);

            let mut d = IntsDeserializer::new(ints);
            let value: Vec<int> = Deserializable::deserialize(&mut d).unwrap();

            assert_eq!(value, vec!(5, 6, 7));
        })
    }

    #[bench]
    fn bench_ints_decoder(b: &mut Bencher) {
        b.iter(|| {
            let ints = vec!(5, 6, 7);

            let mut d = IntsDecoder::new(ints);
            let value: Vec<int> = Decodable::decode(&mut d).unwrap();

            assert_eq!(value, vec!(5, 6, 7));
        })
    }

    #[bench]
    fn bench_enum_decoder(b: &mut Bencher) {
        b.iter(|| {
            let animal = Frog("Henry".to_strbuf(), 349);

            let mut d = AnimalDecoder::new(animal.clone());
            let value: Animal = Decodable::decode(&mut d).unwrap();

            assert_eq!(value, animal);
        })
    }

    #[bench]
    fn bench_enum_deserializer(b: &mut Bencher) {
        b.iter(|| {
            let animal = Frog("Henry".to_strbuf(), 349);

            let mut d = AnimalDeserializer::new(animal.clone());
            let value: Animal = Deserializable::deserialize(&mut d).unwrap();

            assert_eq!(value, animal);
        })
    }

    #[bench]
    fn bench_struct_decoder(b: &mut Bencher) {
        b.iter(|| {
            let mut map = HashMap::new();
            map.insert("abc".to_strbuf(), Some('c'));

            let outer = Outer {
                inner: vec!(
                    Inner {
                        a: (),
                        b: 5,
                        c: map,
                    },
                )
            };

            let mut d = OuterDecoder::new(outer.clone());
            let value: Outer = Decodable::decode(&mut d).unwrap();

            assert_eq!(value, outer);
        })
    }
}
