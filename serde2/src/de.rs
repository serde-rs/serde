use std::marker::PhantomData;
use std::collections::{HashMap, BTreeMap};
use std::hash::Hash;
use std::num::FromPrimitive;
use std::str;

///////////////////////////////////////////////////////////////////////////////

pub trait Error {
    fn syntax_error() -> Self;

    fn end_of_stream_error() -> Self;

    fn missing_field_error(&'static str) -> Self;
}

pub trait Deserialize {
    fn deserialize<
        D: Deserializer,
    >(deserializer: &mut D) -> Result<Self, D::Error>;
}

pub trait Deserializer {
    type Error: Error;

    fn visit<
        V: Visitor,
    >(&mut self, visitor: &mut V) -> Result<V::Value, Self::Error>;

    /// The `visit_option` method allows a `Deserialize` type to inform the
    /// `Deserializer` that it's expecting an optional value. This allows
    /// deserializers that encode an optional value as a nullable value to
    /// convert the null value into a `None`, and a regular value as
    /// `Some(value)`.
    #[inline]
    fn visit_option<
        V: Visitor,
    >(&mut self, visitor: &mut V) -> Result<V::Value, Self::Error> {
        self.visit(visitor)
    }
}

pub trait Visitor {
    type Value;

    fn visit_bool<
        E: Error,
    >(&mut self, _v: bool) -> Result<Self::Value, E> {
        Err(Error::syntax_error())
    }

    fn visit_isize<
        E: Error,
    >(&mut self, v: isize) -> Result<Self::Value, E> {
        self.visit_i64(v as i64)
    }

    fn visit_i8<
        E: Error,
    >(&mut self, v: i8) -> Result<Self::Value, E> {
        self.visit_i64(v as i64)
    }

    fn visit_i16<
        E: Error,
    >(&mut self, v: i16) -> Result<Self::Value, E> {
        self.visit_i64(v as i64)
    }

    fn visit_i32<
        E: Error,
    >(&mut self, v: i32) -> Result<Self::Value, E> {
        self.visit_i64(v as i64)
    }

    fn visit_i64<
        E: Error,
    >(&mut self, _v: i64) -> Result<Self::Value, E> {
        Err(Error::syntax_error())
    }

    fn visit_usize<
        E: Error,
    >(&mut self, v: usize) -> Result<Self::Value, E> {
        self.visit_u64(v as u64)
    }

    fn visit_u8<
        E: Error,
    >(&mut self, v: u8) -> Result<Self::Value, E> {
        self.visit_u64(v as u64)
    }

    fn visit_u16<
        E: Error,
    >(&mut self, v: u16) -> Result<Self::Value, E> {
        self.visit_u64(v as u64)
    }

    fn visit_u32<
        E: Error,
    >(&mut self, v: u32) -> Result<Self::Value, E> {
        self.visit_u64(v as u64)
    }

    fn visit_u64<
        E: Error,
    >(&mut self, _v: u64) -> Result<Self::Value, E> {
        Err(Error::syntax_error())
    }

    fn visit_f32<
        E: Error,
    >(&mut self, v: f32) -> Result<Self::Value, E> {
        self.visit_f64(v as f64)
    }

    fn visit_f64<
        E: Error,
    >(&mut self, _v: f64) -> Result<Self::Value, E> {
        Err(Error::syntax_error())
    }

    #[inline]
    fn visit_char<
        E: Error,
    >(&mut self, v: char) -> Result<Self::Value, E> {
        // The unwraps in here should be safe.
        let mut s = &mut [0; 4];
        let len = v.encode_utf8(s).unwrap();
        self.visit_str(str::from_utf8(&s[..len]).unwrap())
    }

    fn visit_str<
        E: Error,
    >(&mut self, _v: &str) -> Result<Self::Value, E> {
        Err(Error::syntax_error())
    }

    #[inline]
    fn visit_string<
        E: Error,
    >(&mut self, v: String) -> Result<Self::Value, E> {
        self.visit_str(&v)
    }

    fn visit_unit<
        E: Error,
    >(&mut self) -> Result<Self::Value, E> {
        Err(Error::syntax_error())
    }

    #[inline]
    fn visit_named_unit<
        E: Error,
    >(&mut self, _name: &str) -> Result<Self::Value, E> {
        self.visit_unit()
    }

    fn visit_none<
        E: Error,
    >(&mut self) -> Result<Self::Value, E> {
        Err(Error::syntax_error())
    }

    fn visit_some<
        D: Deserializer,
    >(&mut self, _deserializer: &mut D) -> Result<Self::Value, D::Error> {
        Err(Error::syntax_error())
    }

    fn visit_seq<
        V: SeqVisitor,
    >(&mut self, _visitor: V) -> Result<Self::Value, V::Error> {
        Err(Error::syntax_error())
    }

    #[inline]
    fn visit_named_seq<
        V: SeqVisitor,
    >(&mut self, _name: &str, visitor: V) -> Result<Self::Value, V::Error> {
        self.visit_seq(visitor)
    }

    fn visit_map<
        V: MapVisitor,
    >(&mut self, _visitor: V) -> Result<Self::Value, V::Error> {
        Err(Error::syntax_error())
    }

    #[inline]
    fn visit_named_map<
        V: MapVisitor,
    >(&mut self, _name: &str, visitor: V) -> Result<Self::Value, V::Error> {
        self.visit_map(visitor)
    }

    #[inline]
    fn visit_enum<
        V: EnumVisitor,
    >(&mut self, _name: &str, _variant: &str, _visitor: V) -> Result<Self::Value, V::Error> {
        Err(Error::syntax_error())
    }

    #[inline]
    fn visit_variant<
        V: EnumVisitor,
    >(&mut self, _name: &str, _visitor: V) -> Result<Self::Value, V::Error> {
        Err(Error::syntax_error())
    }
}

pub trait SeqVisitor {
    type Error: Error;

    fn visit<T>(&mut self) -> Result<Option<T>, Self::Error>
        where T: Deserialize;

    fn end(&mut self) -> Result<(), Self::Error>;

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

pub trait MapVisitor {
    type Error: Error;

    #[inline]
    fn visit<K, V>(&mut self) -> Result<Option<(K, V)>, Self::Error>
        where K: Deserialize,
              V: Deserialize,
    {
        match try!(self.visit_key()) {
            Some(key) => {
                let value = try!(self.visit_value());
                Ok(Some((key, value)))
            }
            None => Ok(None)
        }
    }

    fn visit_key<K>(&mut self) -> Result<Option<K>, Self::Error>
        where K: Deserialize;

    fn visit_value<V>(&mut self) -> Result<V, Self::Error>
        where V: Deserialize;

    fn end(&mut self) -> Result<(), Self::Error>;

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

pub trait EnumVisitor {
    type Error: Error;

    fn visit_unit(&mut self) -> Result<(), Self::Error> {
        Err(Error::syntax_error())
    }

    fn visit_seq<
        V: EnumSeqVisitor,
    >(&mut self, _visitor: &mut V) -> Result<V::Value, Self::Error> {
        Err(Error::syntax_error())
    }

    fn visit_map<
        V: EnumMapVisitor,
    >(&mut self, _visitor: &mut V) -> Result<V::Value, Self::Error> {
        Err(Error::syntax_error())
    }
}

pub trait EnumSeqVisitor {
    type Value;

    fn visit<
        V: SeqVisitor,
    >(&mut self, visitor: V) -> Result<Self::Value, V::Error>;
}

pub trait EnumMapVisitor {
    type Value;

    fn visit<
        V: MapVisitor,
    >(&mut self, visitor: V) -> Result<Self::Value, V::Error>;
}

///////////////////////////////////////////////////////////////////////////////

struct UnitVisitor;

impl Visitor for UnitVisitor {
    type Value = ();

    fn visit_unit<
        E: Error,
    >(&mut self) -> Result<(), E> {
        Ok(())
    }

    fn visit_seq<
        V: SeqVisitor,
    >(&mut self, mut visitor: V) -> Result<(), V::Error> {
        visitor.end()
    }
}

impl Deserialize for () {
    fn deserialize<
        D: Deserializer,
    >(deserializer: &mut D) -> Result<(), D::Error> {
        deserializer.visit(&mut UnitVisitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

struct BoolVisitor;

impl Visitor for BoolVisitor {
    type Value = bool;

    fn visit_bool<
        E: Error,
    >(&mut self, v: bool) -> Result<bool, E> {
        Ok(v)
    }
}

impl Deserialize for bool {
    fn deserialize<
        D: Deserializer,
    >(deserializer: &mut D) -> Result<bool, D::Error> {
        deserializer.visit(&mut BoolVisitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

macro_rules! impl_deserialize_num_method {
    ($src_ty:ty, $method:ident, $from_method:ident) => {
        #[inline]
        fn $method<
            E: Error,
        >(&mut self, v: $src_ty) -> Result<T, E> {
            match FromPrimitive::$from_method(v) {
                Some(v) => Ok(v),
                None => Err(Error::syntax_error()),
            }
        }
    }
}

pub struct PrimitiveVisitor<T> {
    marker: PhantomData<T>,
}

impl<
    T: Deserialize + FromPrimitive
> self::Visitor for PrimitiveVisitor<T> {
    type Value = T;

    impl_deserialize_num_method!(isize, visit_isize, from_int);
    impl_deserialize_num_method!(i8, visit_i8, from_i8);
    impl_deserialize_num_method!(i16, visit_i16, from_i16);
    impl_deserialize_num_method!(i32, visit_i32, from_i32);
    impl_deserialize_num_method!(i64, visit_i64, from_i64);
    impl_deserialize_num_method!(usize, visit_usize, from_uint);
    impl_deserialize_num_method!(u8, visit_u8, from_u8);
    impl_deserialize_num_method!(u16, visit_u16, from_u16);
    impl_deserialize_num_method!(u32, visit_u32, from_u32);
    impl_deserialize_num_method!(u64, visit_u64, from_u64);
    impl_deserialize_num_method!(f32, visit_f32, from_f32);
    impl_deserialize_num_method!(f64, visit_f64, from_f64);
}

macro_rules! impl_deserialize_num {
    ($ty:ty) => {
        impl Deserialize for $ty {
            #[inline]
            fn deserialize<
                D: Deserializer,
            >(deserializer: &mut D) -> Result<$ty, D::Error> {
                deserializer.visit(&mut PrimitiveVisitor { marker: PhantomData })
            }
        }
    }
}

impl_deserialize_num!(isize);
impl_deserialize_num!(i8);
impl_deserialize_num!(i16);
impl_deserialize_num!(i32);
impl_deserialize_num!(i64);
impl_deserialize_num!(usize);
impl_deserialize_num!(u8);
impl_deserialize_num!(u16);
impl_deserialize_num!(u32);
impl_deserialize_num!(u64);
impl_deserialize_num!(f32);
impl_deserialize_num!(f64);

///////////////////////////////////////////////////////////////////////////////

struct CharVisitor;

impl Visitor for CharVisitor {
    type Value = char;

    #[inline]
    fn visit_char<
        E: Error,
    >(&mut self, v: char) -> Result<char, E> {
        Ok(v)
    }

    #[inline]
    fn visit_str<
        E: Error,
    >(&mut self, v: &str) -> Result<char, E> {
        let mut iter = v.chars();
        if let Some(v) = iter.next() {
            if iter.next().is_some() {
                Err(Error::syntax_error())
            } else {
                Ok(v)
            }
        } else {
            Err(Error::end_of_stream_error())
        }
    }
}

impl Deserialize for char {
    #[inline]
    fn deserialize<
        D: Deserializer,
    >(deserializer: &mut D) -> Result<char, D::Error> {
        deserializer.visit(&mut CharVisitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

struct StringVisitor;

impl Visitor for StringVisitor {
    type Value = String;

    fn visit_str<
        E: Error,
    >(&mut self, v: &str) -> Result<String, E> {
        Ok(v.to_string())
    }

    fn visit_string<
        E: Error,
    >(&mut self, v: String) -> Result<String, E> {
        Ok(v)
    }
}

impl Deserialize for String {
    fn deserialize<
        D: Deserializer,
    >(deserializer: &mut D) -> Result<String, D::Error> {
        deserializer.visit(&mut StringVisitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

struct OptionVisitor<T> {
    marker: PhantomData<T>,
}

impl<
    T: Deserialize,
> Visitor for OptionVisitor<T> {
    type Value = Option<T>;

    #[inline]
    fn visit_none<
        E: Error,
    >(&mut self) -> Result<Option<T>, E> {
        Ok(None)
    }

    #[inline]
    fn visit_some<
        D: Deserializer,
    >(&mut self, deserializer: &mut D) -> Result<Option<T>, D::Error> {
        Ok(Some(try!(Deserialize::deserialize(deserializer))))
    }
}

impl<T> Deserialize for Option<T> where T: Deserialize {
    fn deserialize<
        D: Deserializer,
    >(deserializer: &mut D) -> Result<Option<T>, D::Error> {
        deserializer.visit_option(&mut OptionVisitor { marker: PhantomData })
    }
}

///////////////////////////////////////////////////////////////////////////////

struct VecVisitor<T> {
    marker: PhantomData<T>,
}

impl<T> Visitor for VecVisitor<T> where T: Deserialize {
    type Value = Vec<T>;

    fn visit_seq<
        V: SeqVisitor,
    >(&mut self, mut visitor: V) -> Result<Vec<T>, V::Error> {
        let (len, _) = visitor.size_hint();
        let mut values = Vec::with_capacity(len);

        while let Some(value) = try!(visitor.visit()) {
            values.push(value);
        }

        Ok(values)
    }
}

impl<T: Deserialize> Deserialize for Vec<T> {
    fn deserialize<
        D: Deserializer,
    >(deserializer: &mut D) -> Result<Vec<T>, D::Error> {
        deserializer.visit(&mut VecVisitor { marker: PhantomData })
    }
}

///////////////////////////////////////////////////////////////////////////////

macro_rules! tuple_impls {
    () => {};
    ($($visitor:ident => ($($name:ident),+),)+) => {
        $(
            struct $visitor<$($name,)+> {
                marker: PhantomData<($($name,)+)>,
            }

            impl<
                $($name: Deserialize,)+
            > Visitor for $visitor<$($name,)+> {
                type Value = ($($name,)+);

                #[inline]
                #[allow(non_snake_case)]
                fn visit_seq<
                    V: SeqVisitor,
                >(&mut self, mut visitor: V) -> Result<($($name,)+), V::Error> {
                    $(
                        let $name = match try!(visitor.visit()) {
                            Some(value) => value,
                            None => { return Err(Error::end_of_stream_error()); }
                        };
                     )+;

                    try!(visitor.end());

                    Ok(($($name,)+))
                }
            }

            impl<
                $($name: Deserialize),+
            > Deserialize for ($($name,)+) {
                #[inline]
                fn deserialize<
                    D: Deserializer,
                >(deserializer: &mut D) -> Result<($($name,)+), D::Error> {
                    deserializer.visit(&mut $visitor { marker: PhantomData })
                }
            }
        )+
    }
}

tuple_impls! {
    TupleVisitor1 => (T0),
    TupleVisitor2 => (T0, T1),
    TupleVisitor3 => (T0, T1, T2),
    TupleVisitor4 => (T0, T1, T2, T3),
    TupleVisitor5 => (T0, T1, T2, T3, T4),
    TupleVisitor6 => (T0, T1, T2, T3, T4, T5),
    TupleVisitor7 => (T0, T1, T2, T3, T4, T5, T6),
    TupleVisitor8 => (T0, T1, T2, T3, T4, T5, T6, T7),
    TupleVisitor9 => (T0, T1, T2, T3, T4, T5, T6, T7, T8),
    TupleVisitor10 => (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9),
    TupleVisitor11 => (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10),
    TupleVisitor12 => (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11),
}

///////////////////////////////////////////////////////////////////////////////

struct HashMapVisitor<K, V> {
    marker: PhantomData<HashMap<K, V>>,
}

impl<K, V> Visitor for HashMapVisitor<K, V>
    where K: Deserialize + Eq + Hash,
          V: Deserialize,
{
    type Value = HashMap<K, V>;

    #[inline]
    fn visit_map<
        V_: MapVisitor,
    >(&mut self, mut visitor: V_) -> Result<HashMap<K, V>, V_::Error> {
        let (len, _) = visitor.size_hint();
        let mut values = HashMap::with_capacity(len);

        while let Some((key, value)) = try!(visitor.visit()) {
            values.insert(key, value);
        }

        Ok(values)
    }
}

impl<K, V> Deserialize for HashMap<K, V>
    where K: Deserialize + Eq + Hash,
          V: Deserialize,
{
    fn deserialize<
        D: Deserializer,
    >(deserializer: &mut D) -> Result<HashMap<K, V>, D::Error> {
        deserializer.visit(&mut HashMapVisitor { marker: PhantomData })
    }
}

///////////////////////////////////////////////////////////////////////////////

struct BTreeMapVisitor<K, V> {
    marker: PhantomData<BTreeMap<K, V>>,
}

impl<K, V> Visitor for BTreeMapVisitor<K, V>
    where K: Deserialize + Ord,
          V: Deserialize
{
    type Value = BTreeMap<K, V>;

    #[inline]
    fn visit_map<
        Visitor: MapVisitor,
    >(&mut self, mut visitor: Visitor) -> Result<BTreeMap<K, V>, Visitor::Error> {
        let mut values = BTreeMap::new();

        while let Some((key, value)) = try!(visitor.visit()) {
            values.insert(key, value);
        }

        Ok(values)
    }
}

impl<
    K: Deserialize + Eq + Ord,
    V: Deserialize,
> Deserialize for BTreeMap<K, V> {
    fn deserialize<
        D: Deserializer,
    >(deserializer: &mut D) -> Result<BTreeMap<K, V>, D::Error> {
        deserializer.visit(&mut BTreeMapVisitor { marker: PhantomData })
    }
}

#[cfg(test)]
mod tests {
    use super::{Deserialize, Deserializer, Visitor};
    use std::collections::BTreeMap;
    use std::iter;
    use std::vec;

    #[derive(Debug)]
    enum Token<'a> {
        Bool(bool),
        Isize(isize),
        I8(i8),
        I16(i16),
        I32(i32),
        I64(i64),
        Usize(usize),
        U8(u8),
        U16(u16),
        U32(u32),
        U64(u64),
        F32(f32),
        F64(f64),
        Char(char),
        Str(&'a str),
        String(String),

        Option(bool),

        Unit,
        NamedUnit(&'a str),

        SeqStart(usize),
        NamedSeqStart(&'a str, usize),
        SeqSep(bool),
        SeqEnd,

        MapStart(usize),
        NamedMapStart(&'a str, usize),
        MapSep(bool),
        MapEnd,

        EnumStart(&'a str, &'a str),
        EnumEnd,
    }

    struct TokenDeserializer<'a> {
        tokens: iter::Peekable<vec::IntoIter<Token<'a>>>,
    }

    impl<'a> TokenDeserializer<'a> {
        fn new(tokens: Vec<Token<'a>>) -> TokenDeserializer<'a> {
            TokenDeserializer {
                tokens: tokens.into_iter().peekable(),
            }
        }
    }

    #[derive(Copy, PartialEq, Debug)]
    enum Error {
        SyntaxError,
        EndOfStreamError,
        MissingFieldError(&'static str),
    }

    impl super::Error for Error {
        fn syntax_error() -> Error { Error::SyntaxError }

        fn end_of_stream_error() -> Error { Error::EndOfStreamError }

        fn missing_field_error(field: &'static str) -> Error {
            Error::MissingFieldError(field)
        }
    }

    impl<'a> Deserializer for TokenDeserializer<'a> {
        type Error = Error;

        fn visit<
            V: Visitor,
        >(&mut self, visitor: &mut V) -> Result<V::Value, Error> {
            match self.tokens.next() {
                Some(Token::Bool(v)) => visitor.visit_bool(v),
                Some(Token::Isize(v)) => visitor.visit_isize(v),
                Some(Token::I8(v)) => visitor.visit_i8(v),
                Some(Token::I16(v)) => visitor.visit_i16(v),
                Some(Token::I32(v)) => visitor.visit_i32(v),
                Some(Token::I64(v)) => visitor.visit_i64(v),
                Some(Token::Usize(v)) => visitor.visit_usize(v),
                Some(Token::U8(v)) => visitor.visit_u8(v),
                Some(Token::U16(v)) => visitor.visit_u16(v),
                Some(Token::U32(v)) => visitor.visit_u32(v),
                Some(Token::U64(v)) => visitor.visit_u64(v),
                Some(Token::F32(v)) => visitor.visit_f32(v),
                Some(Token::F64(v)) => visitor.visit_f64(v),
                Some(Token::Char(v)) => visitor.visit_char(v),
                Some(Token::Str(v)) => visitor.visit_str(v),
                Some(Token::String(v)) => visitor.visit_string(v),
                Some(Token::Option(false)) => visitor.visit_none(),
                Some(Token::Option(true)) => visitor.visit_some(self),
                Some(Token::Unit) => visitor.visit_unit(),
                Some(Token::NamedUnit(name)) => visitor.visit_named_unit(name),
                Some(Token::SeqStart(len)) => {
                    visitor.visit_seq(TokenDeserializerSeqVisitor {
                        de: self,
                        len: len,
                        first: true,
                    })
                }
                Some(Token::NamedSeqStart(name, len)) => {
                    visitor.visit_named_seq(name, TokenDeserializerSeqVisitor {
                        de: self,
                        len: len,
                        first: true,
                    })
                }
                Some(Token::MapStart(len)) => {
                    visitor.visit_map(TokenDeserializerMapVisitor {
                        de: self,
                        len: len,
                        first: true,
                    })
                }
                Some(Token::NamedMapStart(name, len)) => {
                    visitor.visit_named_map(name, TokenDeserializerMapVisitor {
                        de: self,
                        len: len,
                        first: true,
                    })
                }
                Some(Token::EnumStart(name, variant)) => {
                    visitor.visit_enum(name, variant, TokenDeserializerEnumVisitor {
                        de: self,
                    })
                }
                Some(_) => Err(Error::SyntaxError),
                None => Err(Error::EndOfStreamError),
            }
        }

        /// Hook into `Option` deserializing so we can treat `Unit` as a
        /// `None`, or a regular value as `Some(value)`.
        #[inline]
        fn visit_option<
            V: Visitor,
        >(&mut self, visitor: &mut V) -> Result<V::Value, Error> {
            match self.tokens.peek() {
                Some(&Token::Option(false)) => {
                    self.tokens.next();
                    visitor.visit_none()
                }
                Some(&Token::Option(true)) => {
                    self.tokens.next();
                    visitor.visit_some(self)
                }
                Some(&Token::Unit) => {
                    self.tokens.next();
                    visitor.visit_none()
                }
                Some(_) => visitor.visit_some(self),
                None => Err(Error::EndOfStreamError),
            }
        }
    }

    //////////////////////////////////////////////////////////////////////////

    struct TokenDeserializerSeqVisitor<'a, 'b: 'a> {
        de: &'a mut TokenDeserializer<'b>,
        len: usize,
        first: bool,
    }

    impl<'a, 'b> super::SeqVisitor for TokenDeserializerSeqVisitor<'a, 'b> {
        type Error = Error;

        fn visit<
            T: Deserialize,
        >(&mut self) -> Result<Option<T>, Error> {
            let first = self.first;
            self.first = false;

            match self.de.tokens.next() {
                Some(Token::SeqSep(first_)) if first_ == first => {
                    self.len -= 1;
                    Ok(Some(try!(Deserialize::deserialize(self.de))))
                }
                Some(Token::SeqEnd) => Ok(None),
                Some(_) => {
                    Err(Error::SyntaxError)
                }
                None => Err(Error::EndOfStreamError),
            }
        }

        fn end(&mut self) -> Result<(), Error> {
            match self.de.tokens.next() {
                Some(Token::SeqEnd) => Ok(()),
                Some(_) => Err(Error::SyntaxError),
                None => Err(Error::EndOfStreamError),
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.len, Some(self.len))
        }
    }

    //////////////////////////////////////////////////////////////////////////

    struct TokenDeserializerMapVisitor<'a, 'b: 'a> {
        de: &'a mut TokenDeserializer<'b>,
        len: usize,
        first: bool,
    }

    impl<'a, 'b> super::MapVisitor for TokenDeserializerMapVisitor<'a, 'b> {
        type Error = Error;

        fn visit_key<
            K: Deserialize,
        >(&mut self) -> Result<Option<K>, Error> {
            let first = self.first;
            self.first = false;

            match self.de.tokens.next() {
                Some(Token::MapSep(first_)) if first_ == first => {
                    Ok(Some(try!(Deserialize::deserialize(self.de))))
                }
                Some(Token::MapEnd) => Ok(None),
                Some(_) => Err(Error::SyntaxError),
                None => Err(Error::EndOfStreamError),
            }
        }

        fn visit_value<
            V: Deserialize,
        >(&mut self) -> Result<V, Error> {
            Ok(try!(Deserialize::deserialize(self.de)))
        }

        fn end(&mut self) -> Result<(), Error> {
            match self.de.tokens.next() {
                Some(Token::MapEnd) => Ok(()),
                Some(_) => Err(Error::SyntaxError),
                None => Err(Error::EndOfStreamError),
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.len, Some(self.len))
        }
    }

    //////////////////////////////////////////////////////////////////////////

    struct TokenDeserializerEnumVisitor<'a, 'b: 'a> {
        de: &'a mut TokenDeserializer<'b>,
    }

    impl<'a, 'b> super::EnumVisitor for TokenDeserializerEnumVisitor<'a, 'b> {
        type Error = Error;

        fn visit_unit(&mut self) -> Result<(), Error> {
            let value = try!(Deserialize::deserialize(self.de));

            match self.de.tokens.next() {
                Some(Token::EnumEnd) => Ok(value),
                Some(_) => Err(Error::SyntaxError),
                None => Err(Error::EndOfStreamError),
            }
        }

        fn visit_seq<
            V: super::EnumSeqVisitor,
        >(&mut self, visitor: &mut V) -> Result<V::Value, Error> {
            let token = self.de.tokens.next();
            match token {
                Some(Token::SeqStart(len)) => {
                    let value = try!(visitor.visit(TokenDeserializerSeqVisitor {
                        de: self.de,
                        len: len,
                        first: true,
                    }));

                    match self.de.tokens.next() {
                        Some(Token::EnumEnd) => Ok(value),
                        Some(_) => Err(Error::SyntaxError),
                        None => Err(Error::EndOfStreamError),
                    }
                }
                Some(_) => Err(Error::SyntaxError),
                None => Err(Error::EndOfStreamError),
            }
        }

        fn visit_map<
            V: super::EnumMapVisitor,
        >(&mut self, visitor: &mut V) -> Result<V::Value, Error> {
            match self.de.tokens.next() {
                Some(Token::MapStart(len)) => {
                    let value = try!(visitor.visit(TokenDeserializerMapVisitor {
                        de: self.de,
                        len: len,
                        first: true,
                    }));

                    match self.de.tokens.next() {
                        Some(Token::EnumEnd) => Ok(value),
                        Some(_) => Err(Error::SyntaxError),
                        None => Err(Error::EndOfStreamError),
                    }
                }
                Some(_) => Err(Error::SyntaxError),
                None => Err(Error::EndOfStreamError),
            }
        }
    }

    //////////////////////////////////////////////////////////////////////////

    #[derive(Copy, PartialEq, Debug)]
    struct NamedUnit;

    impl Deserialize for NamedUnit {
        fn deserialize<
            D: Deserializer,
        >(deserializer: &mut D) -> Result<NamedUnit, D::Error> {
            deserializer.visit(&mut NamedUnitVisitor)
        }
    }

    struct NamedUnitVisitor;

    impl Visitor for NamedUnitVisitor {
        type Value = NamedUnit;

        fn visit_unit<
            E: super::Error,
        >(&mut self) -> Result<NamedUnit, E> {
            Ok(NamedUnit)
        }

        fn visit_named_unit<
            E: super::Error,
        >(&mut self, name: &str) -> Result<NamedUnit, E> {
            if name == "NamedUnit" {
                Ok(NamedUnit)
            } else {
                Err(super::Error::syntax_error())
            }
        }

        fn visit_seq<
            V: super::SeqVisitor,
        >(&mut self, mut visitor: V) -> Result<NamedUnit, V::Error> {
            try!(visitor.end());
            Ok(NamedUnit)
        }
    }

    //////////////////////////////////////////////////////////////////////////

    #[derive(PartialEq, Debug)]
    struct NamedSeq(i32, i32, i32);

    impl Deserialize for NamedSeq {
        fn deserialize<
            D: Deserializer,
        >(deserializer: &mut D) -> Result<NamedSeq, D::Error> {
            deserializer.visit(&mut NamedSeqVisitor)
        }
    }

    struct NamedSeqVisitor;

    impl Visitor for NamedSeqVisitor {
        type Value = NamedSeq;

        fn visit_seq<
            V: super::SeqVisitor,
        >(&mut self, mut visitor: V) -> Result<NamedSeq, V::Error> {
            let a = match try!(visitor.visit()) {
                Some(value) => value,
                None => { return Err(super::Error::end_of_stream_error()); }
            };

            let b = match try!(visitor.visit()) {
                Some(value) => value,
                None => { return Err(super::Error::end_of_stream_error()); }
            };

            let c = match try!(visitor.visit()) {
                Some(value) => value,
                None => { return Err(super::Error::end_of_stream_error()); }
            };

            try!(visitor.end());

            Ok(NamedSeq(a, b, c))
        }

        fn visit_named_seq<
            V: super::SeqVisitor,
        >(&mut self, name: &str, visitor: V) -> Result<NamedSeq, V::Error> {
            if name == "NamedSeq" {
                self.visit_seq(visitor)
            } else {
                Err(super::Error::syntax_error())
            }
        }
    }

    //////////////////////////////////////////////////////////////////////////

    #[derive(PartialEq, Debug)]
    struct NamedMap {
        a: i32,
        b: i32,
        c: i32,
    }

    impl Deserialize for NamedMap {
        fn deserialize<
            D: Deserializer,
        >(deserializer: &mut D) -> Result<NamedMap, D::Error> {
            deserializer.visit(&mut NamedMapVisitor)
        }
    }

    struct NamedMapVisitor;

    impl Visitor for NamedMapVisitor {
        type Value = NamedMap;

        fn visit_map<
            V: super::MapVisitor,
        >(&mut self, mut visitor: V) -> Result<NamedMap, V::Error> {
            let mut a = None;
            let mut b = None;
            let mut c = None;

            while let Some(key) = try!(visitor.visit_key()) {
                match key {
                    NamedMapField::A => { a = Some(try!(visitor.visit_value())); }
                    NamedMapField::B => { b = Some(try!(visitor.visit_value())); }
                    NamedMapField::C => { c = Some(try!(visitor.visit_value())); }
                }
            }

            match (a, b, c) {
                (Some(a), Some(b), Some(c)) => Ok(NamedMap { a: a, b: b, c: c }),
                _ => Err(super::Error::syntax_error()),
            }
        }

        fn visit_named_map<
            V: super::MapVisitor,
        >(&mut self, name: &str, visitor: V) -> Result<NamedMap, V::Error> {
            if name == "NamedMap" {
                self.visit_map(visitor)
            } else {
                Err(super::Error::syntax_error())
            }
        }
    }

    enum NamedMapField {
        A,
        B,
        C,
    }

    impl Deserialize for NamedMapField {
        fn deserialize<
            D: Deserializer,
        >(deserializer: &mut D) -> Result<NamedMapField, D::Error> {
            deserializer.visit(&mut NamedMapFieldVisitor)
        }
    }

    struct NamedMapFieldVisitor;

    impl Visitor for NamedMapFieldVisitor {
        type Value = NamedMapField;

        fn visit_str<
            E: super::Error,
        >(&mut self, value: &str) -> Result<NamedMapField, E> {
            match value {
                "a" => Ok(NamedMapField::A),
                "b" => Ok(NamedMapField::B),
                "c" => Ok(NamedMapField::C),
                _ => Err(super::Error::syntax_error()),
            }
        }
    }

    //////////////////////////////////////////////////////////////////////////

    #[derive(PartialEq, Debug)]
    enum Enum {
        Unit,
        Seq(i32, i32, i32),
        Map { a: i32, b: i32, c: i32 }
    }

    impl Deserialize for Enum {
        fn deserialize<
            D: Deserializer,
        >(deserializer: &mut D) -> Result<Enum, D::Error> {
            deserializer.visit(&mut EnumVisitor)
        }
    }

    struct EnumVisitor;

    impl Visitor for EnumVisitor {
        type Value = Enum;

        fn visit_enum<
            V: super::EnumVisitor,
        >(&mut self, name: &str, variant: &str, visitor: V) -> Result<Enum, V::Error> {
            if name == "Enum" {
                self.visit_variant(variant, visitor)
            } else {
                Err(super::Error::syntax_error())
            }
        }

        fn visit_variant<
            V: super::EnumVisitor,
        >(&mut self, name: &str, mut visitor: V) -> Result<Enum, V::Error> {
            match name {
                "Unit" => {
                    try!(visitor.visit_unit());
                    Ok(Enum::Unit)
                }
                "Seq" => visitor.visit_seq(&mut EnumSeqVisitor),
                "Map" => visitor.visit_map(&mut EnumMapVisitor),
                _ => Err(super::Error::syntax_error()),
            }
        }
    }

    struct EnumSeqVisitor;

    impl super::EnumSeqVisitor for EnumSeqVisitor {
        type Value = Enum;

        fn visit<
            V: super::SeqVisitor,
        >(&mut self, mut visitor: V) -> Result<Enum, V::Error> {
            let a = match try!(visitor.visit()) {
                Some(value) => value,
                None => { return Err(super::Error::end_of_stream_error()); }
            };

            let b = match try!(visitor.visit()) {
                Some(value) => value,
                None => { return Err(super::Error::end_of_stream_error()); }
            };

            let c = match try!(visitor.visit()) {
                Some(value) => value,
                None => { return Err(super::Error::end_of_stream_error()); }
            };

            try!(visitor.end());

            Ok(Enum::Seq(a, b, c))
        }
    }

    struct EnumMapVisitor;

    impl super::EnumMapVisitor for EnumMapVisitor {
        type Value = Enum;

        fn visit<
            V: super::MapVisitor,
        >(&mut self, mut visitor: V) -> Result<Enum, V::Error> {
            let mut a = None;
            let mut b = None;
            let mut c = None;

            while let Some(key) = try!(visitor.visit_key()) {
                match key {
                    EnumMapField::A => { a = Some(try!(visitor.visit_value())); }
                    EnumMapField::B => { b = Some(try!(visitor.visit_value())); }
                    EnumMapField::C => { c = Some(try!(visitor.visit_value())); }
                }
            }

            match (a, b, c) {
                (Some(a), Some(b), Some(c)) => Ok(Enum::Map { a: a, b: b, c: c }),
                _ => Err(super::Error::syntax_error()),
            }
        }
    }

    enum EnumMapField {
        A,
        B,
        C,
    }

    impl Deserialize for EnumMapField {
        fn deserialize<
            D: Deserializer,
        >(deserializer: &mut D) -> Result<EnumMapField, D::Error> {
            deserializer.visit(&mut EnumMapFieldVisitor)
        }
    }

    struct EnumMapFieldVisitor;

    impl Visitor for EnumMapFieldVisitor {
        type Value = EnumMapField;

        fn visit_str<
            E: super::Error,
        >(&mut self, value: &str) -> Result<EnumMapField, E> {
            match value {
                "a" => Ok(EnumMapField::A),
                "b" => Ok(EnumMapField::B),
                "c" => Ok(EnumMapField::C),
                _ => Err(super::Error::syntax_error()),
            }
        }
    }

    //////////////////////////////////////////////////////////////////////////

    macro_rules! btreemap {
        () => {
            BTreeMap::new()
        };
        ($($key:expr => $value:expr),+) => {
            {
                let mut map = BTreeMap::new();
                $(map.insert($key, $value);)+
                map
            }
        }
    }

    macro_rules! declare_test {
        ($name:ident { $($value:expr => $tokens:expr,)+ }) => {
            #[test]
            fn $name() {
                $(
                    let mut de = TokenDeserializer::new($tokens);
                    let value: Result<_, Error> = Deserialize::deserialize(&mut de);
                    assert_eq!(value, Ok($value));
                )+
            }
        }
    }

    macro_rules! declare_tests {
        ($($name:ident { $($value:expr => $tokens:expr,)+ })+) => {
            $(
                declare_test!($name { $($value => $tokens,)+ });
            )+
        }
    }

    //////////////////////////////////////////////////////////////////////////

    declare_tests! {
        test_bool {
            true => vec![Token::Bool(true)],
            false => vec![Token::Bool(false)],
        }
        test_isize {
            0is => vec![Token::Isize(0)],
            0is => vec![Token::I8(0)],
            0is => vec![Token::I16(0)],
            0is => vec![Token::I32(0)],
            0is => vec![Token::I64(0)],
            0is => vec![Token::Usize(0)],
            0is => vec![Token::U8(0)],
            0is => vec![Token::U16(0)],
            0is => vec![Token::U32(0)],
            0is => vec![Token::U64(0)],
            0is => vec![Token::F32(0.)],
            0is => vec![Token::F64(0.)],
        }
        test_ints {
            0is => vec![Token::Isize(0)],
            0i8 => vec![Token::I8(0)],
            0i16 => vec![Token::I16(0)],
            0i32 => vec![Token::I32(0)],
            0i64 => vec![Token::I64(0)],
        }
        test_uints {
            0us => vec![Token::Usize(0)],
            0u8 => vec![Token::U8(0)],
            0u16 => vec![Token::U16(0)],
            0u32 => vec![Token::U32(0)],
            0u64 => vec![Token::U64(0)],
        }
        test_floats {
            0f32 => vec![Token::F32(0.)],
            0f64 => vec![Token::F64(0.)],
        }
        test_char {
            'a' => vec![Token::Char('a')],
            'a' => vec![Token::Str("a")],
            'a' => vec![Token::String("a".to_string())],
        }
        test_string {
            "abc".to_string() => vec![Token::Str("abc")],
            "abc".to_string() => vec![Token::String("abc".to_string())],
            "a".to_string() => vec![Token::Char('a')],
        }
        test_option {
            None::<i32> => vec![Token::Unit],
            None::<i32> => vec![Token::Option(false)],
            Some(1) => vec![Token::I32(1)],
            Some(1) => vec![
                Token::Option(true),
                Token::I32(1),
            ],
        }
        test_unit {
            () => vec![Token::Unit],
            () => vec![
                Token::SeqStart(0),
                Token::SeqEnd,
            ],
            () => vec![
                Token::NamedSeqStart("Anything", 0),
                Token::SeqEnd,
            ],
        }
        test_named_unit {
            NamedUnit => vec![Token::Unit],
            NamedUnit => vec![Token::NamedUnit("NamedUnit")],
            NamedUnit => vec![
                Token::SeqStart(0),
                Token::SeqEnd,
            ],
        }
        test_named_seq {
            NamedSeq(1, 2, 3) => vec![
                Token::SeqStart(3),
                    Token::SeqSep(true),
                    Token::I32(1),

                    Token::SeqSep(false),
                    Token::I32(2),

                    Token::SeqSep(false),
                    Token::I32(3),
                Token::SeqEnd,
            ],
            NamedSeq(1, 2, 3) => vec![
                Token::NamedSeqStart("NamedSeq", 3),
                    Token::SeqSep(true),
                    Token::I32(1),

                    Token::SeqSep(false),
                    Token::I32(2),

                    Token::SeqSep(false),
                    Token::I32(3),
                Token::SeqEnd,
            ],
        }
        test_vec {
            Vec::<isize>::new() => vec![
                Token::SeqStart(0),
                Token::SeqEnd,
            ],
            vec![vec![], vec![1], vec![2, 3]] => vec![
                Token::SeqStart(3),
                    Token::SeqSep(true),
                    Token::SeqStart(0),
                    Token::SeqEnd,

                    Token::SeqSep(false),
                    Token::SeqStart(1),
                        Token::SeqSep(true),
                        Token::I32(1),
                    Token::SeqEnd,

                    Token::SeqSep(false),
                    Token::SeqStart(2),
                        Token::SeqSep(true),
                        Token::I32(2),

                        Token::SeqSep(false),
                        Token::I32(3),
                    Token::SeqEnd,
                Token::SeqEnd,
            ],
        }
        test_tuple {
            (1,) => vec![
                Token::SeqStart(1),
                    Token::SeqSep(true),
                    Token::I32(1),
                Token::SeqEnd,
            ],
            (1, 2, 3) => vec![
                Token::SeqStart(3),
                    Token::SeqSep(true),
                    Token::I32(1),

                    Token::SeqSep(false),
                    Token::I32(2),

                    Token::SeqSep(false),
                    Token::I32(3),
                Token::SeqEnd,
            ],
        }
        test_btreemap {
            btreemap![1 => 2] => vec![
                Token::MapStart(1),
                    Token::MapSep(true),
                    Token::I32(1),
                    Token::I32(2),
                Token::MapEnd,
            ],
            btreemap![1 => 2, 3 => 4] => vec![
                Token::MapStart(2),
                    Token::MapSep(true),
                    Token::I32(1),
                    Token::I32(2),

                    Token::MapSep(false),
                    Token::I32(3),
                    Token::I32(4),
                Token::MapEnd,
            ],
            btreemap![1 => btreemap![], 2 => btreemap![3 => 4, 5 => 6]] => vec![
                Token::MapStart(2),
                    Token::MapSep(true),
                    Token::I32(1),
                    Token::MapStart(0),
                    Token::MapEnd,

                    Token::MapSep(false),
                    Token::I32(2),
                    Token::MapStart(2),
                        Token::MapSep(true),
                        Token::I32(3),
                        Token::I32(4),

                        Token::MapSep(false),
                        Token::I32(5),
                        Token::I32(6),
                    Token::MapEnd,
                Token::MapEnd,
            ],
        }
        test_named_map {
            NamedMap { a: 1, b: 2, c: 3 } => vec![
                Token::MapStart(3),
                    Token::MapSep(true),
                    Token::Str("a"),
                    Token::I32(1),

                    Token::MapSep(false),
                    Token::Str("b"),
                    Token::I32(2),

                    Token::MapSep(false),
                    Token::Str("c"),
                    Token::I32(3),
                Token::MapEnd,
            ],
            NamedMap { a: 1, b: 2, c: 3 } => vec![
                Token::NamedMapStart("NamedMap", 3),
                    Token::MapSep(true),
                    Token::Str("a"),
                    Token::I32(1),

                    Token::MapSep(false),
                    Token::Str("b"),
                    Token::I32(2),

                    Token::MapSep(false),
                    Token::Str("c"),
                    Token::I32(3),
                Token::MapEnd,
            ],
        }
        test_enum {
            Enum::Unit => vec![
                Token::EnumStart("Enum", "Unit"),
                    Token::Unit,
                Token::EnumEnd,
            ],
        }
        test_enum_seq {
            Enum::Seq(1, 2, 3) => vec![
                Token::EnumStart("Enum", "Seq"),
                    Token::SeqStart(3),
                        Token::SeqSep(true),
                        Token::I32(1),

                        Token::SeqSep(false),
                        Token::I32(2),

                        Token::SeqSep(false),
                        Token::I32(3),
                    Token::SeqEnd,
                Token::EnumEnd,
            ],
        }
        test_enum_map {
            Enum::Map { a: 1, b: 2, c: 3 } => vec![
                Token::EnumStart("Enum", "Map"),
                    Token::MapStart(3),
                        Token::MapSep(true),
                        Token::Str("a"),
                        Token::I32(1),

                        Token::MapSep(false),
                        Token::Str("b"),
                        Token::I32(2),

                        Token::MapSep(false),
                        Token::Str("c"),
                        Token::I32(3),
                    Token::MapEnd,
                Token::EnumEnd,
            ],
        }
    }
}
