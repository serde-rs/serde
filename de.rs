extern crate collections;

use std::hash::Hash;
use std::num;
use collections::{HashMap, TreeMap};

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
    String(String),
    Option(bool),

    TupleStart(uint),
    StructStart(&'static str),
    EnumStart(&'static str, &'static str),
    SeqStart(uint),
    MapStart(uint),

    End,
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
    fn expect_null(&mut self, token: Token) -> Result<(), E> {
        match token {
            Null => Ok(()),
            TupleStart(_) => {
                match self.next() {
                    Some(Ok(End)) => Ok(()),
                    Some(Ok(_)) => Err(self.syntax_error()),
                    Some(Err(err)) => Err(err),
                    None => Err(self.end_of_stream_error()),
                }
            }
            _ => Err(self.syntax_error()),
        }
    }

    #[inline]
    fn expect_bool(&mut self, token: Token) -> Result<bool, E> {
        match token {
            Bool(value) => Ok(value),
            _ => Err(self.syntax_error()),
        }
    }

    #[inline]
    fn expect_num<T: NumCast>(&mut self, token: Token) -> Result<T, E> {
        match token {
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
            F64(x) => to_result!(num::cast(x), self.syntax_error()),
            _ => Err(self.syntax_error()),
        }
    }

    #[inline]
    fn expect_char(&mut self, token: Token) -> Result<char, E> {
        match token {
            Char(value) => Ok(value),
            _ => Err(self.syntax_error()),
        }
    }

    #[inline]
    fn expect_str(&mut self, token: Token) -> Result<&'static str, E> {
        match token {
            Str(value) => Ok(value),
            _ => Err(self.syntax_error()),
        }
    }

    #[inline]
    fn expect_strbuf(&mut self, token: Token) -> Result<String, E> {
        match token {
            Str(value) => Ok(value.to_strbuf()),
            String(value) => Ok(value),
            _ => Err(self.syntax_error()),
        }
    }

    #[inline]
    fn expect_option<
        T: Deserializable<E, Self>
    >(&mut self, token: Token) -> Result<Option<T>, E> {
        match token {
            Option(false) => Ok(None),
            Option(true) => {
                let value: T = try!(Deserializable::deserialize(self));
                Ok(Some(value))
            }
            _ => Err(self.syntax_error()),
        }
    }

    #[inline]
    fn expect_tuple_start(&mut self, token: Token, len: uint) -> Result<(), E> {
        match token {
            TupleStart(l) => {
                if len == l {
                    Ok(())
                } else {
                    Err(self.syntax_error())
                }
            }
            _ => Err(self.syntax_error()),
        }
    }

    #[inline]
    fn expect_struct_start(&mut self, token: Token, name: &str) -> Result<(), E> {
        match token {
            StructStart(n) => {
                if name == n {
                    Ok(())
                } else {
                    Err(self.syntax_error())
                }
            }
            _ => Err(self.syntax_error()),
        }
    }

    #[inline]
    fn expect_struct_field<
        T: Deserializable<E, Self>
    >(&mut self, name: &str) -> Result<T, E> {
        let token = match self.next() {
            Some(Ok(token)) => token,
            Some(Err(err)) => { return Err(err); }
            None => { return Err(self.end_of_stream_error()); }
        };

        match token {
            Str(n) => {
                if name != n {
                    return Err(self.syntax_error());
                }
            }
            String(n) => {
                if name != n.as_slice() {
                    return Err(self.syntax_error());
                }
            }
            _ => { return Err(self.syntax_error()); }
        }

        Deserializable::deserialize(self)
    }

    #[inline]
    fn expect_enum_start<'a>(&mut self, token: Token, name: &str, variants: &[&str]) -> Result<uint, E> {
        match token {
            EnumStart(n, v) => {
                if name == n {
                    match variants.iter().position(|variant| *variant == v) {
                        Some(position) => Ok(position),
                        None => Err(self.syntax_error()),
                    }
                } else {
                    Err(self.syntax_error())
                }
            }
            _ => Err(self.syntax_error()),
        }
    }

    /*
    #[inline]
    fn expect_collection<
        T: Deserializable<E, Self>,
        C: FromIterator<T>
    >(&mut self, token: Token) -> Result<C, E> {
        // By default we don't care what our source input was. We can take
        // anything that's a Collection<T>. We'll error out later if the types
        // are wrong.
        let len = match token {
            TupleStart(len) => len,
            SeqStart(len) => len,
            MapStart(len) => len,
            _ => { return Err(self.syntax_error()); }
        };

        expect_rest_of_collection(self, len)
    }
    */

    #[inline]
    fn expect_seq_start(&mut self, token: Token) -> Result<uint, E> {
        match token {
            SeqStart(len) => Ok(len),
            _ => Err(self.syntax_error()),
        }
    }

    #[inline]
    fn expect_map_start(&mut self, token: Token) -> Result<uint, E> {
        match token {
            MapStart(len) => Ok(len),
            _ => Err(self.syntax_error()),
        }
    }

    #[inline]
    fn expect_end(&mut self) -> Result<(), E> {
        match self.next() {
            Some(Ok(End)) => Ok(()),
            Some(Ok(_)) => Err(self.syntax_error()),
            Some(Err(err)) => Err(err),
            None => Err(self.end_of_stream_error()),
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

/*
// FIXME: https://github.com/mozilla/rust/issues/11751
#[inline]
fn expect_rest_of_collection<
    E,
    D: Deserializer<E>,
    T: Deserializable<E, D>,
    C: FromIterator<T>
>(d: &mut D, len: uint) -> Result<C, E> {
    let iter = d.by_ref().batch(|d| {
        let d = d.iter();

        match d.next() {
            Some(Ok(End)) => None,
            Some(Ok(token)) => {
                let value: Result<T, E> = Deserializable::deserialize_token(d, token);
                Some(value)
            }
            Some(Err(e)) => Some(Err(e)),
            None => Some(Err(d.end_of_stream_error())),
        }
    });

    result::collect_with_capacity(iter, len)
}
*/

//////////////////////////////////////////////////////////////////////////////

pub trait Deserializable<E, D: Deserializer<E>> {
    #[inline]
    fn deserialize(d: &mut D) -> Result<Self, E> {
        match d.next() {
            Some(Ok(token)) => Deserializable::deserialize_token(d, token),
            Some(Err(err)) => Err(err),
            None => Err(d.end_of_stream_error()),
        }
    }

    fn deserialize_token(d: &mut D, token: Token) -> Result<Self, E>;
}

//////////////////////////////////////////////////////////////////////////////

macro_rules! impl_deserializable {
    ($ty:ty, $method:ident) => {
        impl<
            E,
            D: Deserializer<E>
        > Deserializable<E, D> for $ty {
            #[inline]
            fn deserialize_token(d: &mut D, token: Token) -> Result<$ty, E> {
                d.$method(token)
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
impl_deserializable!(String, expect_strbuf)

//////////////////////////////////////////////////////////////////////////////

impl<
    E,
    D: Deserializer<E>,
    T: Deserializable<E, D>
> Deserializable<E, D> for Option<T> {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<Option<T>, E> {
        d.expect_option(token)
    }
}

//////////////////////////////////////////////////////////////////////////////

macro_rules! deserialize_seq {
    ($seq:expr) => {
        {
            loop {
                match d.next() {
                    Some(Ok(End)) => { break; }
                    Some(Ok(token)) => {
                        let v = try!(Deserializable::deserialize_token(d, token));
                        $seq.push(v)
                    }
                    Some(Err(err)) => { return Err(err); }
                    None => { return Err(d.end_of_stream_error()); }
                }
            }

            Ok($seq)
        }
    }
}

impl<
    E,
    D: Deserializer<E>,
    T: Deserializable<E, D>
> Deserializable<E, D> for Vec<T> {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<Vec<T>, E> {
        let len = try!(d.expect_seq_start(token));
        let mut value = Vec::with_capacity(len);

        deserialize_seq!(value)
    }
}

//////////////////////////////////////////////////////////////////////////////

macro_rules! deserialize_map {
    ($seq:expr) => {
        {
            loop {
                match d.next() {
                    Some(Ok(End)) => { break; }
                    Some(Ok(token)) => {
                        let k = try!(Deserializable::deserialize_token(d, token));
                        let v = try!(Deserializable::deserialize(d));
                        $seq.insert(k, v);
                    }
                    Some(Err(err)) => { return Err(err); }
                    None => { return Err(d.end_of_stream_error()); }
                }
            }

            Ok($seq)
        }
    }
}

impl<
    E,
    D: Deserializer<E>,
    K: Deserializable<E, D> + TotalEq + Hash,
    V: Deserializable<E, D>
> Deserializable<E, D> for HashMap<K, V> {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<HashMap<K, V>, E> {
        let len = try!(d.expect_map_start(token));
        let mut value = HashMap::with_capacity(len);

        deserialize_map!(value)
    }
}

impl<
    E,
    D: Deserializer<E>,
    K: Deserializable<E, D> + Eq + TotalOrd,
    V: Deserializable<E, D>
> Deserializable<E, D> for TreeMap<K, V> {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<TreeMap<K, V>, E> {
        let _len = try!(d.expect_map_start(token));
        let mut value = TreeMap::new();

        deserialize_map!(value)
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<
    E,
    D: Deserializer<E>
> Deserializable<E, D> for () {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<(), E> {
        d.expect_null(token)
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
            fn deserialize_token(d: &mut D, token: Token) -> Result<($($name,)*), E> {
                // FIXME: how can we count macro args?
                let mut len = 0;
                $({ let $name = 1; len += $name; })*;

                try!(d.expect_tuple_start(token, len));

                let result = ($({
                    let $name = try!(Deserializable::deserialize(d));
                    $name
                },)*);

                match d.next() {
                    Some(Ok(End)) => Ok(result),
                    Some(Ok(_)) => Err(d.syntax_error()),
                    Some(Err(err)) => Err(err),
                    None => Err(d.end_of_stream_error()),
                }
            }
        }
        peel!($($name,)*)
    )
)

deserialize_tuple! { T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, }

//////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use collections::HashMap;

    use serialize::Decoder;

    use super::{Token, Null, Int, Uint, Str, String, Char, Option};
    use super::{TupleStart, StructStart, EnumStart};
    use super::{SeqStart, MapStart, End};
    use super::{Deserializer, Deserializable};

    //////////////////////////////////////////////////////////////////////////////

    #[deriving(Clone, Eq, Show, Decodable)]
    struct Inner {
        a: (),
        b: uint,
        c: HashMap<String, Option<char>>,
    }

    impl<E, D: Deserializer<E>> Deserializable<E, D> for Inner {
        #[inline]
        fn deserialize_token(d: &mut D, token: Token) -> Result<Inner, E> {
            try!(d.expect_struct_start(token, "Inner"));
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
        fn deserialize_token(d: &mut D, token: Token) -> Result<Outer, E> {
            try!(d.expect_struct_start(token, "Outer"));
            let inner = try!(d.expect_struct_field("inner"));
            try!(d.expect_end());
            Ok(Outer { inner: inner })
        }
    }

    //////////////////////////////////////////////////////////////////////////////

    #[deriving(Clone, Eq, Show, Decodable)]
    enum Animal {
        Dog,
        Frog(String, int)
    }

    impl<E, D: Deserializer<E>> Deserializable<E, D> for Animal {
        #[inline]
        fn deserialize_token(d: &mut D, token: Token) -> Result<Animal, E> {
            match try!(d.expect_enum_start(token, "Animal", ["Dog", "Frog"])) {
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
            String("a".to_strbuf()),
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: String = Deserializable::deserialize(&mut deserializer).unwrap();

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
                Int(5),

                String("a".to_strbuf()),
            End,
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: (int, String) = Deserializable::deserialize(&mut deserializer).unwrap();

        assert_eq!(value, (5, "a".to_strbuf()));
    }

    #[test]
    fn test_tokens_tuple_compound() {
        let tokens = vec!(
            TupleStart(3),
                Null,

                TupleStart(0),
                End,

                TupleStart(2),
                    Int(5),

                    String("a".to_strbuf()),
                End,
            End,
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: ((), (), (int, String)) = Deserializable::deserialize(&mut deserializer).unwrap();

        assert_eq!(value, ((), (), (5, "a".to_strbuf())));
    }

    #[test]
    fn test_tokens_struct_empty() {
        let tokens = vec!(
            StructStart("Outer"),
                Str("inner"),
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
                Str("inner"),
                SeqStart(1),
                    StructStart("Inner"),
                        Str("a"),
                        Null,

                        Str("b"),
                        Uint(5),

                        Str("c"),
                        MapStart(1),
                            String("abc".to_strbuf()),

                            Option(true),
                            Char('c'),
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
            EnumStart("Animal", "Dog"),
            End,
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: Animal = Deserializable::deserialize(&mut deserializer).unwrap();

        assert_eq!(value, Dog);

        let tokens = vec!(
            EnumStart("Animal", "Frog"),
                String("Henry".to_strbuf()),
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
                Int(5),

                Int(6),

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
                SeqStart(1),
                    Int(1),
                End,

                SeqStart(2),
                    Int(2),

                    Int(3),
                End,

                SeqStart(3),
                    Int(4),

                    Int(5),

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
                Int(5),

                String("a".to_strbuf()),

                Int(6),

                String("b".to_strbuf()),
            End,
        );

        let mut deserializer = TokenDeserializer::new(tokens);
        let value: HashMap<int, String> = Deserializable::deserialize(&mut deserializer).unwrap();

        let mut map = HashMap::new();
        map.insert(5, "a".to_strbuf());
        map.insert(6, "b".to_strbuf());

        assert_eq!(value, map);
    }
}
