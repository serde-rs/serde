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
    let mut idx = 0;

    let iter = d.by_ref().batch(|d| {
        let d = d.iter();

        if idx < len {
            idx += 1;
            let value: Result<T, E> = Deserializable::deserialize(d);
            Some(value)
        } else {
            match d.next() {
                Some(Ok(End)) => None,
                Some(Ok(_)) => Some(Err(d.syntax_error())),
                Some(Err(e)) => Some(Err(e)),
                None => Some(Err(d.end_of_stream_error())),
            }
        }

        /*
        match token {
            Ok(Sep) => {
                let value: Result<T, E> = Deserializable::deserialize(d);
                Some(value)
            }
            Ok(End) => None,
            Ok(_) => Some(Err(d.syntax_error())),
            Err(e) => Some(Err(e)),
        }
        */
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

/*
#[cfg(test)]
mod tests {
    use collections::HashMap;
    use test::Bencher;

    use serialize::Decoder;

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
    fn bench_token_deserializer(b: &mut Bencher) {
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
}
*/
