// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::{HashMap, HashSet, TreeMap, TreeSet};
use std::gc::{GC, Gc};
use std::hash::Hash;
use std::num;
use std::rc::Rc;
use std::sync::Arc;

#[deriving(Clone, PartialEq, Show)]
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
    StructStart(&'static str, uint),
    EnumStart(&'static str, &'static str, uint),
    SeqStart(uint),
    MapStart(uint),

    End,
}

impl Token {
    pub fn to_kind(&self) -> TokenKind {
        match *self {
            Null => NullKind,
            Bool(_) => BoolKind,
            Int(_) => IntKind,
            I8(_) => I8Kind,
            I16(_) => I16Kind,
            I32(_) => I32Kind,
            I64(_) => I64Kind,
            Uint(_) => UintKind,
            U8(_) => U8Kind,
            U16(_) => U16Kind,
            U32(_) => U32Kind,
            U64(_) => U64Kind,
            F32(_) => F32Kind,
            F64(_) => F64Kind,
            Char(_) => CharKind,
            Str(_) => StrKind,
            String(_) => StringKind,
            Option(_) => OptionKind,
            TupleStart(_) => TupleStartKind,
            StructStart(_, _) => StructStartKind,
            EnumStart(_, _, _) => EnumStartKind,
            SeqStart(_) => SeqStartKind,
            MapStart(_) => MapStartKind,
            End => EndKind,
        }
    }
}

#[deriving(Clone, PartialEq, Eq)]
pub enum TokenKind {
    NullKind,
    BoolKind,
    IntKind,
    I8Kind,
    I16Kind,
    I32Kind,
    I64Kind,
    UintKind,
    U8Kind,
    U16Kind,
    U32Kind,
    U64Kind,
    F32Kind,
    F64Kind,
    CharKind,
    StrKind,
    StringKind,
    OptionKind,

    TupleStartKind,
    StructStartKind,
    EnumStartKind,
    SeqStartKind,
    MapStartKind,

    EndKind,
}

static primitive_token_kinds: [TokenKind, .. 12] = [
    IntKind,
    I8Kind,
    I16Kind,
    I32Kind,
    I64Kind,
    UintKind,
    U8Kind,
    U16Kind,
    U32Kind,
    U64Kind,
    F32Kind,
    F64Kind,
];

static str_token_kinds: [TokenKind, .. 2] = [
    StrKind,
    StringKind,
];

static compound_token_kinds: [TokenKind, .. 6] = [
    OptionKind,
    EnumStartKind,
    StructStartKind,
    TupleStartKind,
    SeqStartKind,
    MapStartKind,
];

impl ::std::fmt::Show for TokenKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            NullKind => "Null".fmt(f),
            BoolKind => "Bool".fmt(f),
            IntKind => "Int".fmt(f),
            I8Kind => "I8".fmt(f),
            I16Kind => "I16".fmt(f),
            I32Kind => "I32".fmt(f),
            I64Kind => "I64".fmt(f),
            UintKind => "Uint".fmt(f),
            U8Kind => "U8".fmt(f),
            U16Kind => "U16".fmt(f),
            U32Kind => "U32".fmt(f),
            U64Kind => "U64".fmt(f),
            F32Kind => "F32".fmt(f),
            F64Kind => "F64".fmt(f),
            CharKind => "Char".fmt(f),
            StrKind => "Str".fmt(f),
            StringKind => "String".fmt(f),
            OptionKind => "Option".fmt(f),
            TupleStartKind => "TupleStart".fmt(f),
            StructStartKind => "StructStart".fmt(f),
            EnumStartKind => "EnumStart".fmt(f),
            SeqStartKind => "SeqStart".fmt(f),
            MapStartKind => "MapStart".fmt(f),
            EndKind => "End".fmt(f),
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
    /// Called when a `Deserializable` expected more tokens, but the
    /// `Deserializer` was empty.
    fn end_of_stream_error(&mut self) -> E;

    /// Called when a `Deserializer` was unable to properly parse the stream.
    fn syntax_error(&mut self, token: Token, expected: &[TokenKind]) -> E;

    /// Called when a named structure or enum got a name that it didn't expect.
    fn unexpected_name_error(&mut self, token: Token) -> E;

    /// Called when a value was unable to be coerced into another value.
    fn conversion_error(&mut self, token: Token) -> E;

    /// Called when a `Deserializable` structure did not deserialize a field
    /// named `field`.
    fn missing_field<
        T: Deserializable<Self, E>
    >(&mut self, field: &'static str) -> Result<T, E>;

    /// Called when a deserializable has decided to not consume this token.
    fn ignore_field(&mut self, _token: Token) -> Result<(), E> {
        let _: IgnoreTokens = try!(Deserializable::deserialize(self));
        Ok(())
    }

    #[inline]
    fn expect_token(&mut self) -> Result<Token, E> {
        match self.next() {
            Some(Ok(token)) => Ok(token),
            Some(Err(err)) => Err(err),
            None => Err(self.end_of_stream_error()),
        }
    }

    #[inline]
    fn expect_null(&mut self, token: Token) -> Result<(), E> {
        match token {
            Null => Ok(()),
            TupleStart(_) | SeqStart(_) => {
                match try!(self.expect_token()) {
                    End => Ok(()),
                    token => Err(self.syntax_error(token, [EndKind])),
                }
            }
            token => Err(self.syntax_error(token, [NullKind, TupleStartKind, SeqStartKind])),
        }
    }

    #[inline]
    fn expect_bool(&mut self, token: Token) -> Result<bool, E> {
        match token {
            Bool(value) => Ok(value),
            token => Err(self.syntax_error(token, [BoolKind])),
        }
    }

    #[inline]
    fn expect_num<T: NumCast>(&mut self, token: Token) -> Result<T, E> {
        match token {
            Int(x) => to_result!(num::cast(x), self.syntax_error(token, primitive_token_kinds)),
            I8(x) => to_result!(num::cast(x), self.syntax_error(token, primitive_token_kinds)),
            I16(x) => to_result!(num::cast(x), self.syntax_error(token, primitive_token_kinds)),
            I32(x) => to_result!(num::cast(x), self.syntax_error(token, primitive_token_kinds)),
            I64(x) => to_result!(num::cast(x), self.syntax_error(token, primitive_token_kinds)),
            Uint(x) => to_result!(num::cast(x), self.syntax_error(token, primitive_token_kinds)),
            U8(x) =>  to_result!(num::cast(x), self.syntax_error(token, primitive_token_kinds)),
            U16(x) => to_result!(num::cast(x), self.syntax_error(token, primitive_token_kinds)),
            U32(x) => to_result!(num::cast(x), self.syntax_error(token, primitive_token_kinds)),
            U64(x) => to_result!(num::cast(x), self.syntax_error(token, primitive_token_kinds)),
            F32(x) => to_result!(num::cast(x), self.syntax_error(token, primitive_token_kinds)),
            F64(x) => to_result!(num::cast(x), self.syntax_error(token, primitive_token_kinds)),
            token => Err(self.syntax_error(token, primitive_token_kinds)),
        }
    }

    #[inline]
    fn expect_from_primitive<T: FromPrimitive>(&mut self, token: Token) -> Result<T, E> {
        match token {
            Int(x) => to_result!(num::from_int(x), self.conversion_error(token)),
            I8(x) => to_result!(num::from_i8(x), self.conversion_error(token)),
            I16(x) => to_result!(num::from_i16(x), self.conversion_error(token)),
            I32(x) => to_result!(num::from_i32(x), self.conversion_error(token)),
            I64(x) => to_result!(num::from_i64(x), self.conversion_error(token)),
            Uint(x) => to_result!(num::from_uint(x), self.conversion_error(token)),
            U8(x) => to_result!(num::from_u8(x), self.conversion_error(token)),
            U16(x) => to_result!(num::from_u16(x), self.conversion_error(token)),
            U32(x) => to_result!(num::from_u32(x), self.conversion_error(token)),
            U64(x) => to_result!(num::from_u64(x), self.conversion_error(token)),
            F32(x) => to_result!(num::from_f32(x), self.conversion_error(token)),
            F64(x) => to_result!(num::from_f64(x), self.conversion_error(token)),
            token => Err(self.syntax_error(token, primitive_token_kinds)),
        }
    }

    #[inline]
    fn expect_char(&mut self, token: Token) -> Result<char, E> {
        match token {
            Char(value) => Ok(value),
            Str(value) if value.char_len() == 1 => {
                Ok(value.char_at(0))
            }
            String(ref value) if value.as_slice().char_len() == 1 => {
                Ok(value.as_slice().char_at(0))
            }
            token => Err(self.syntax_error(token, [CharKind])),
        }
    }

    #[inline]
    fn expect_str(&mut self, token: Token) -> Result<&'static str, E> {
        match token {
            Str(value) => Ok(value),
            token => Err(self.syntax_error(token, str_token_kinds)),
        }
    }

    #[inline]
    fn expect_string(&mut self, token: Token) -> Result<String, E> {
        match token {
            Char(value) => Ok(value.to_string()),
            Str(value) => Ok(value.to_string()),
            String(value) => Ok(value),
            token => Err(self.syntax_error(token, str_token_kinds)),
        }
    }

    #[inline]
    fn expect_option<
        T: Deserializable<Self, E>
    >(&mut self, token: Token) -> Result<Option<T>, E> {
        match token {
            Option(false) => Ok(None),
            Option(true) => {
                let value: T = try!(Deserializable::deserialize(self));
                Ok(Some(value))
            }
            token => Err(self.syntax_error(token, [OptionKind])),
        }
    }

    #[inline]
    fn expect_tuple_start(&mut self, token: Token) -> Result<uint, E> {
        match token {
            TupleStart(len) => Ok(len),
            SeqStart(len) => Ok(len),
            token => Err(self.syntax_error(token, [TupleStartKind, SeqStartKind])),
        }
    }

    #[inline]
    fn expect_tuple_elt<
        T: Deserializable<Self, E>
    >(&mut self) -> Result<T, E> {
        Deserializable::deserialize(self)
    }

    #[inline]
    fn expect_tuple_end(&mut self) -> Result<(), E> {
        match try!(self.expect_token()) {
            End => Ok(()),
            token => Err(self.syntax_error(token, [EndKind])),
        }
    }

    #[inline]
    fn expect_struct_start(&mut self, token: Token, name: &str) -> Result<(), E> {
        match token {
            StructStart(n, _) => {
                if name == n {
                    Ok(())
                } else {
                    Err(self.unexpected_name_error(token))
                }
            }
            _ => Err(self.syntax_error(token, [StructStartKind])),
        }
    }

    #[inline]
    fn expect_struct_field<
        T: Deserializable<Self, E>
    >(&mut self, name: &str) -> Result<T, E> {
        match try!(self.expect_token()) {
            Str(n) => {
                if name != n {
                    return Err(self.unexpected_name_error(Str(n)));
                }
            }
            String(n) => {
                if name != n.as_slice() {
                    return Err(self.unexpected_name_error(String(n)));
                }
            }
            token => {
                return Err(self.syntax_error(token, str_token_kinds));
            }
        }

        Deserializable::deserialize(self)
    }

    #[inline]
    fn expect_struct_end(&mut self) -> Result<(), E> {
        match try!(self.expect_token()) {
            End => Ok(()),
            token => Err(self.syntax_error(token, [EndKind])),
        }
    }

    #[inline]
    fn expect_enum_start(&mut self, token: Token, name: &str, variants: &[&str]) -> Result<uint, E> {
        match token {
            EnumStart(n, v, _) => {
                if name == n {
                    match variants.iter().position(|variant| *variant == v) {
                        Some(position) => Ok(position),
                        None => Err(self.unexpected_name_error(token)),
                    }
                } else {
                    Err(self.unexpected_name_error(token))
                }
            }
            token => Err(self.syntax_error(token, [EnumStartKind])),
        }
    }

    #[inline]
    fn expect_enum_elt<
        T: Deserializable<Self, E>
    >(&mut self) -> Result<T, E> {
        Deserializable::deserialize(self)
    }

    #[inline]
    fn expect_enum_end(&mut self) -> Result<(), E> {
        match try!(self.expect_token()) {
            End => Ok(()),
            token => Err(self.syntax_error(token, [EndKind])),
        }
    }

    #[inline]
    fn expect_seq_start(&mut self, token: Token) -> Result<uint, E> {
        match token {
            TupleStart(len) => Ok(len),
            SeqStart(len) => Ok(len),
            token => Err(self.syntax_error(token, [TupleStartKind, SeqStartKind])),
        }
    }

    #[inline]
    fn expect_seq_elt_or_end<
        T: Deserializable<Self, E>
    >(&mut self) -> Result<Option<T>, E> {
        match try!(self.expect_token()) {
            End => Ok(None),
            token => {
                let value = try!(Deserializable::deserialize_token(self, token));
                Ok(Some(value))
            }
        }
    }

    #[inline]
    fn expect_seq<
        'a,
        T: Deserializable<Self, E>,
        C: FromIterator<T>
    >(&'a mut self, token: Token) -> Result<C, E> {
        let len = try!(self.expect_seq_start(token));

        let mut d: SeqDeserializer<'a, Self, E> = SeqDeserializer {
            d: self,
            len: len,
            err: None,
        };

        let collection: C = d.collect();

        match d.err {
            Some(err) => Err(err),
            None => Ok(collection),
        }
    }

    #[inline]
    fn expect_map_start(&mut self, token: Token) -> Result<uint, E> {
        match token {
            MapStart(len) => Ok(len),
            _ => Err(self.syntax_error(token, [MapStartKind])),
        }
    }

    #[inline]
    fn expect_map_elt_or_end<
        K: Deserializable<Self, E>,
        V: Deserializable<Self, E>
    >(&mut self) -> Result<Option<(K, V)>, E> {
        match try!(self.expect_token()) {
            End => Ok(None),
            token => {
                let key = try!(Deserializable::deserialize_token(self, token));
                let value = try!(Deserializable::deserialize(self));
                Ok(Some((key, value)))
            }
        }
    }

    #[inline]
    fn expect_map<
        'a,
        K: Deserializable<Self, E>,
        V: Deserializable<Self, E>,
        C: FromIterator<(K, V)>
    >(&'a mut self, token: Token) -> Result<C, E> {
        let len = try!(self.expect_map_start(token));

        let mut d: MapDeserializer<'a, Self, E> = MapDeserializer {
            d: self,
            len: len,
            err: None,
        };

        let collection: C = d.collect();

        match d.err {
            Some(err) => Err(err),
            None => Ok(collection),
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

struct SeqDeserializer<'a, D: 'a, E> {
    d: &'a mut D,
    len: uint,
    err: Option<E>,
}

impl<
    'a,
    D: Deserializer<E>,
    E,
    T: Deserializable<D, E>
> Iterator<T> for SeqDeserializer<'a, D, E> {
    #[inline]
    fn next(&mut self) -> Option<T> {
        match self.d.expect_seq_elt_or_end() {
            Ok(next) => next,
            Err(err) => {
                self.err = Some(err);
                None
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        (self.len, Some(self.len))
    }
}

//////////////////////////////////////////////////////////////////////////////

struct MapDeserializer<'a, D:'a, E> {
    d: &'a mut D,
    len: uint,
    err: Option<E>,
}

impl<
    'a,
    D: Deserializer<E>,
    E,
    K: Deserializable<D, E>,
    V: Deserializable<D, E>
> Iterator<(K, V)> for MapDeserializer<'a, D, E> {
    #[inline]
    fn next(&mut self) -> Option<(K, V)> {
        match self.d.expect_map_elt_or_end() {
            Ok(next) => next,
            Err(err) => {
                self.err = Some(err);
                None
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        (self.len, Some(self.len))
    }
}

//////////////////////////////////////////////////////////////////////////////

pub trait Deserializable<D: Deserializer<E>, E> {
    #[inline]
    fn deserialize(d: &mut D) -> Result<Self, E> {
        let token = try!(d.expect_token());
        Deserializable::deserialize_token(d, token)
    }

    fn deserialize_token(d: &mut D, token: Token) -> Result<Self, E>;
}

//////////////////////////////////////////////////////////////////////////////

macro_rules! impl_deserializable {
    ($ty:ty, $method:ident) => {
        impl<D: Deserializer<E>, E> Deserializable<D, E> for $ty {
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
impl_deserializable!(String, expect_string)

//////////////////////////////////////////////////////////////////////////////

impl<
    D: Deserializer<E>,
    E,
    T: Deserializable<D, E>
> Deserializable<D, E> for Box<T> {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<Box<T>, E> {
        Ok(box try!(Deserializable::deserialize_token(d, token)))
    }
}

impl<
    D: Deserializer<E>,
    E,
    T: Deserializable<D, E> + 'static
> Deserializable<D, E> for Gc<T> {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<Gc<T>, E> {
        Ok(box (GC) try!(Deserializable::deserialize_token(d, token)))
    }
}

impl<
    D: Deserializer<E>,
    E,
    T: Deserializable<D, E>
> Deserializable<D, E> for Rc<T> {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<Rc<T>, E> {
        Ok(Rc::new(try!(Deserializable::deserialize_token(d, token))))
    }
}

impl<
    D: Deserializer<E>,
    E,
    T: Deserializable<D, E> + Send + Sync
> Deserializable<D, E> for Arc<T> {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<Arc<T>, E> {
        Ok(Arc::new(try!(Deserializable::deserialize_token(d, token))))
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<
    D: Deserializer<E>,
    E,
    T: Deserializable<D ,E>
> Deserializable<D, E> for Option<T> {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<Option<T>, E> {
        d.expect_option(token)
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<
    D: Deserializer<E>,
    E,
    T: Deserializable<D ,E>
> Deserializable<D, E> for Vec<T> {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<Vec<T>, E> {
        d.expect_seq(token)
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<
    D: Deserializer<E>,
    E,
    K: Deserializable<D, E> + Eq + Hash,
    V: Deserializable<D, E>
> Deserializable<D, E> for HashMap<K, V> {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<HashMap<K, V>, E> {
        d.expect_map(token)
    }
}

impl<
    D: Deserializer<E>,
    E,
    K: Deserializable<D, E> + Ord,
    V: Deserializable<D, E>
> Deserializable<D, E> for TreeMap<K, V> {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<TreeMap<K, V>, E> {
        d.expect_map(token)
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<
    D: Deserializer<E>,
    E,
    T: Deserializable<D, E> + Eq + Hash
> Deserializable<D, E> for HashSet<T> {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<HashSet<T>, E> {
        d.expect_seq(token)
    }
}

impl<
    D: Deserializer<E>,
    E,
    T: Deserializable<D, E> + Ord
> Deserializable<D, E> for TreeSet<T> {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<TreeSet<T>, E> {
        d.expect_seq(token)
    }
}

//////////////////////////////////////////////////////////////////////////////

macro_rules! peel {
    ($name:ident, $($other:ident,)*) => (impl_deserialize_tuple!($($other,)*))
}

macro_rules! impl_deserialize_tuple {
    () => {
        impl<
            D: Deserializer<E>,
            E
        > Deserializable<D, E> for () {
            #[inline]
            fn deserialize_token(d: &mut D, token: Token) -> Result<(), E> {
                d.expect_null(token)
            }
        }
    };
    ( $($name:ident,)+ ) => {
        impl<
            D: Deserializer<E>,
            E,
            $($name: Deserializable<D, E>),*
        > Deserializable<D, E> for ($($name,)*) {
            #[inline]
            #[allow(non_snake_case)]
            fn deserialize_token(d: &mut D, token: Token) -> Result<($($name,)*), E> {
                try!(d.expect_tuple_start(token));

                let result = ($({
                    let $name = try!(d.expect_tuple_elt());
                    $name
                },)*);

                try!(d.expect_tuple_end());

                Ok(result)
            }
        }
        peel!($($name,)*)
    }
}

impl_deserialize_tuple! { T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, }

//////////////////////////////////////////////////////////////////////////////

/// Helper struct that will ignore tokens while taking in consideration
/// recursive structures.
pub struct IgnoreTokens;

impl<D: Deserializer<E>, E> Deserializable<D, E> for IgnoreTokens {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<IgnoreTokens, E> {
        match token {
            Option(true) => {
                Deserializable::deserialize(d)
            }

            EnumStart(_, _, _) => {
                loop {
                    match try!(d.expect_token()) {
                        End => { return Ok(IgnoreTokens); }
                        token => {
                            let _: IgnoreTokens = try!(Deserializable::deserialize_token(d, token));
                        }
                    }
                }
            }

            StructStart(_, _) => {
                loop {
                    match try!(d.expect_token()) {
                        End => { return Ok(IgnoreTokens); }
                        Str(_) | String(_) => {
                            let _: IgnoreTokens = try!(Deserializable::deserialize(d));
                        }
                        _token => { return Err(d.syntax_error(token, [EndKind, StrKind, StringKind])); }
                    }
                }
            }

            TupleStart(_) => {
                loop {
                    match try!(d.expect_token()) {
                        End => { return Ok(IgnoreTokens); }
                        token => {
                            let _: IgnoreTokens = try!(Deserializable::deserialize_token(d, token));
                        }
                    }
                }
            }

            SeqStart(_) => {
                loop {
                    match try!(d.expect_token()) {
                        End => { return Ok(IgnoreTokens); }
                        token => {
                            let _: IgnoreTokens = try!(Deserializable::deserialize_token(d, token));
                        }
                    }
                }
            }

            MapStart(_) => {
                loop {
                    match try!(d.expect_token()) {
                        End => { return Ok(IgnoreTokens); }
                        token => {
                            let _: IgnoreTokens = try!(Deserializable::deserialize_token(d, token));
                            let _: IgnoreTokens = try!(Deserializable::deserialize(d));
                        }
                    }
                }
            }

            End => {
                Err(d.syntax_error(token, compound_token_kinds))
            }

            _ => Ok(IgnoreTokens),
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

/// Helper struct that will gather tokens while taking in consideration
/// recursive structures.
pub struct GatherTokens {
    tokens: Vec<Token>,
}

impl GatherTokens {
    #[inline]
    pub fn unwrap(self) -> Vec<Token> {
        self.tokens
    }

    #[inline]
    fn gather<D: Deserializer<E>, E>(&mut self, d: &mut D) -> Result<(), E> {
        let token = try!(d.expect_token());
        self.gather_token(d, token)
    }

    #[inline]
    fn gather_token<D: Deserializer<E>, E>(&mut self, d: &mut D, token: Token) -> Result<(), E> {
        match token {
            token @ Option(true) => {
                self.tokens.push(token);
                self.gather(d)
            }
            EnumStart(name, variant, len) => {
                self.tokens.reserve_additional(len + 1);
                self.tokens.push(EnumStart(name, variant, len));
                self.gather_seq(d)
            }
            StructStart(name, len) => {
                self.tokens.reserve_additional(len + 1);
                self.tokens.push(StructStart(name, len));
                self.gather_struct(d)
            }
            TupleStart(len) => {
                self.tokens.reserve_additional(len + 1);
                self.tokens.push(TupleStart(len));
                self.gather_seq(d)
            }
            SeqStart(len) => {
                self.tokens.reserve_additional(len + 1);
                self.tokens.push(SeqStart(len));
                self.gather_seq(d)
            }
            MapStart(len) => {
                self.tokens.reserve_additional(len + 1);
                self.tokens.push(MapStart(len));
                self.gather_map(d)
            }
            End => {
                Err(d.syntax_error(token, compound_token_kinds))
            }
            token => {
                self.tokens.push(token);
                Ok(())
            }
        }
    }

    #[inline]
    fn gather_seq<D: Deserializer<E>, E>(&mut self, d: &mut D) -> Result<(), E> {
        loop {
            match try!(d.expect_token()) {
                token @ End => {
                    self.tokens.push(token);
                    return Ok(());
                }
                token => {
                    try!(self.gather_token(d, token));
                }
            }
        }
    }

    #[inline]
    fn gather_struct<D: Deserializer<E>, E>(&mut self, d: &mut D) -> Result<(), E> {
        loop {
            match try!(d.expect_token()) {
                token @ End => {
                    self.tokens.push(token);
                    return Ok(());
                }
                token @ Str(_) | token @ String(_) => {
                    self.tokens.push(token);
                    try!(self.gather(d))
                }
                token => {
                    return Err(d.syntax_error(token, [EndKind, StrKind, StringKind]));
                }
            }
        }
    }

    #[inline]
    fn gather_map<D: Deserializer<E>, E>(&mut self, d: &mut D) -> Result<(), E> {
        loop {
            match try!(d.expect_token()) {
                End => {
                    self.tokens.push(End);
                    return Ok(());
                }
                token => {
                    try!(self.gather_token(d, token));
                    try!(self.gather(d))
                }
            }
        }
    }
}

impl<D: Deserializer<E>, E> Deserializable<D, E> for GatherTokens {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<GatherTokens, E> {
        let mut tokens = GatherTokens {
            tokens: vec!(),
        };
        try!(tokens.gather_token(d, token));
        Ok(tokens)
    }
}

//////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::collections::TreeMap;
    use serialize::Decoder;

    use super::{Deserializer, Deserializable, Token, TokenKind};
    use super::{
        Null,
        Bool,
        Int,
        I8,
        I16,
        I32,
        I64,
        Uint,
        U8,
        U16,
        U32,
        U64,
        F32,
        F64,
        Char,
        Str,
        String,
        Option,
        TupleStart,
        StructStart,
        EnumStart,
        SeqStart,
        MapStart,
        End,
    };

    macro_rules! treemap {
        ($($k:expr => $v:expr),*) => ({
            let mut _m = ::std::collections::TreeMap::new();
            $(_m.insert($k, $v);)*
            _m
        })
    }

    //////////////////////////////////////////////////////////////////////////////

    #[deriving(Clone, PartialEq, Show, Decodable)]
    struct Inner {
        a: (),
        b: uint,
        c: TreeMap<String, Option<char>>,
    }

    impl<
        D: Deserializer<E>,
        E
    > Deserializable<D, E> for Inner {
        #[inline]
        fn deserialize_token(d: &mut D, token: Token) -> Result<Inner, E> {
            try!(d.expect_struct_start(token, "Inner"));
            let a = try!(d.expect_struct_field("a"));
            let b = try!(d.expect_struct_field("b"));
            let c = try!(d.expect_struct_field("c"));
            try!(d.expect_struct_end());
            Ok(Inner { a: a, b: b, c: c })
        }
    }

    //////////////////////////////////////////////////////////////////////////////

    #[deriving(Clone, PartialEq, Show, Decodable)]
    struct Outer {
        inner: Vec<Inner>,
    }

    impl<D: Deserializer<E>, E> Deserializable<D, E> for Outer {
        #[inline]
        fn deserialize_token(d: &mut D, token: Token) -> Result<Outer, E> {
            try!(d.expect_struct_start(token, "Outer"));
            let inner = try!(d.expect_struct_field("inner"));
            try!(d.expect_struct_end());
            Ok(Outer { inner: inner })
        }
    }

    //////////////////////////////////////////////////////////////////////////////

    #[deriving(Clone, PartialEq, Show, Decodable)]
    enum Animal {
        Dog,
        Frog(String, int)
    }

    impl<D: Deserializer<E>, E> Deserializable<D, E> for Animal {
        #[inline]
        fn deserialize_token(d: &mut D, token: Token) -> Result<Animal, E> {
            match try!(d.expect_enum_start(token, "Animal", ["Dog", "Frog"])) {
                0 => {
                    try!(d.expect_enum_end());
                    Ok(Dog)
                }
                1 => {
                    let x0 = try!(Deserializable::deserialize(d));
                    let x1 = try!(Deserializable::deserialize(d));
                    try!(d.expect_enum_end());
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
        IncompleteValue,
    }

    //////////////////////////////////////////////////////////////////////////////

    struct TokenDeserializer<Iter> {
        tokens: Iter,
    }

    impl<Iter: Iterator<Token>> TokenDeserializer<Iter> {
        #[inline]
        fn new(tokens: Iter) -> TokenDeserializer<Iter> {
            TokenDeserializer {
                tokens: tokens,
            }
        }
    }

    impl<Iter: Iterator<Token>> Iterator<Result<Token, Error>> for TokenDeserializer<Iter> {
        #[inline]
        fn next(&mut self) -> Option<Result<Token, Error>> {
            match self.tokens.next() {
                None => None,
                Some(token) => Some(Ok(token)),
            }
        }
    }

    impl<Iter: Iterator<Token>> Deserializer<Error> for TokenDeserializer<Iter> {
        fn end_of_stream_error(&mut self) -> Error {
            EndOfStream
        }

        fn syntax_error(&mut self, _token: Token, _expected: &[TokenKind]) -> Error {
            SyntaxError
        }

        fn unexpected_name_error(&mut self, _token: Token) -> Error {
            SyntaxError
        }

        fn conversion_error(&mut self, _token: Token) -> Error {
            SyntaxError
        }

        #[inline]
        fn missing_field<
            T: Deserializable<TokenDeserializer<Iter>, Error>
        >(&mut self, _field: &'static str) -> Result<T, Error> {
            Err(SyntaxError)
        }
    }

    //////////////////////////////////////////////////////////////////////////////

    macro_rules! test_value {
        ($name:ident, [$($tokens:expr => $value:expr: $ty:ty),*]) => {
            #[test]
            fn $name() {
                $(
                    let mut deserializer = TokenDeserializer::new($tokens.move_iter());
                    let value: $ty = Deserializable::deserialize(&mut deserializer).unwrap();

                    assert_eq!(value, $value);
                )+
            }
        }
    }

    test_value!(test_primitives, [
        vec!(Null) => (): (),
        vec!(Bool(true)) => true: bool,
        vec!(Bool(false)) => false: bool,
        vec!(Int(5)) => 5: int,
        vec!(I8(5)) => 5: i8,
        vec!(I16(5)) => 5: i16,
        vec!(I32(5)) => 5: i32,
        vec!(I64(5)) => 5: i64,
        vec!(Uint(5)) => 5: uint,
        vec!(U8(5)) => 5: u8,
        vec!(U16(5)) => 5: u16,
        vec!(U32(5)) => 5: u32,
        vec!(U64(5)) => 5: u64,
        vec!(F32(5.0)) => 5.0: f32,
        vec!(F64(5.0)) => 5.0: f64,
        vec!(Char('c')) => 'c': char,
        vec!(Str("abc")) => "abc": &str,
        vec!(String("abc".to_string())) => "abc".to_string(): String
    ])

    test_value!(test_tuples, [
        vec!(
            TupleStart(0),
            End,
        ) => (): (),

        vec!(
            TupleStart(2),
                Int(5),

                Str("a"),
            End,
        ) => (5, "a"): (int, &'static str),

        vec!(
            TupleStart(3),
                Null,

                TupleStart(0),
                End,

                TupleStart(2),
                    Int(5),

                    Str("a"),
                End,
            End,
        ) => ((), (), (5, "a")): ((), (), (int, &'static str))
    ])

    test_value!(test_options, [
        vec!(Option(false)) => None: Option<int>,

        vec!(
            Option(true),
            Int(5),
        ) => Some(5): Option<int>
    ])

    test_value!(test_structs, [
        vec!(
            StructStart("Outer", 1),
                Str("inner"),
                SeqStart(0),
                End,
            End,
        ) => Outer { inner: vec!() }: Outer,

        vec!(
            StructStart("Outer", 1),
                Str("inner"),
                SeqStart(1),
                    StructStart("Inner", 3),
                        Str("a"),
                        Null,

                        Str("b"),
                        Uint(5),

                        Str("c"),
                        MapStart(1),
                            String("abc".to_string()),

                            Option(true),
                            Char('c'),
                        End,
                    End,
                End,
            End,
        ) => Outer {
            inner: vec!(
                Inner {
                    a: (),
                    b: 5,
                    c: treemap!("abc".to_string() => Some('c')),
                },
            ),
        }: Outer
    ])

    test_value!(test_enums, [
        vec!(
            EnumStart("Animal", "Dog", 0),
            End,
        ) => Dog: Animal,

        vec!(
            EnumStart("Animal", "Frog", 2),
                String("Henry".to_string()),
                Int(349),
            End,
        ) => Frog("Henry".to_string(), 349): Animal
    ])

    test_value!(test_vecs, [
        vec!(
            SeqStart(0),
            End,
        ) => vec!(): Vec<int>,

        vec!(
            SeqStart(3),
                Int(5),

                Int(6),

                Int(7),
            End,
        ) => vec!(5, 6, 7): Vec<int>,


        vec!(
            SeqStart(3),
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
        ) => vec!(vec!(1), vec!(2, 3), vec!(4, 5, 6)): Vec<Vec<int>>
    ])

    test_value!(test_treemaps, [
        vec!(
            MapStart(0),
            End,
        ) => treemap!(): TreeMap<int, String>,

        vec!(
            MapStart(2),
                Int(5),
                String("a".to_string()),

                Int(6),
                String("b".to_string()),
            End,
        ) => treemap!(5i => "a".to_string(), 6i => "b".to_string()): TreeMap<int, String>
    ])
}
