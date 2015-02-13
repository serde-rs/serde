// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::{HashMap, HashSet, BTreeMap, BTreeSet};
use std::collections::hash_map::Hasher;
use std::hash::Hash;
use std::iter::FromIterator;
use std::num::{self, FromPrimitive};
use std::option;
use std::rc::Rc;
use std::string;
use std::sync::Arc;

#[derive(Clone, PartialEq, Debug)]
pub enum Token {
    Null,
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
    Str(&'static str),
    String(String),
    Option(bool),

    TupleStart(usize),
    StructStart(&'static str, usize),
    EnumStart(&'static str, &'static str, usize),
    SeqStart(usize),
    MapStart(usize),

    End,
}

impl Token {
    pub fn to_kind(&self) -> TokenKind {
        match *self {
            Token::Null => TokenKind::NullKind,
            Token::Bool(_) => TokenKind::BoolKind,
            Token::Isize(_) => TokenKind::IsizeKind,
            Token::I8(_) => TokenKind::I8Kind,
            Token::I16(_) => TokenKind::I16Kind,
            Token::I32(_) => TokenKind::I32Kind,
            Token::I64(_) => TokenKind::I64Kind,
            Token::Usize(_) => TokenKind::UsizeKind,
            Token::U8(_) => TokenKind::U8Kind,
            Token::U16(_) => TokenKind::U16Kind,
            Token::U32(_) => TokenKind::U32Kind,
            Token::U64(_) => TokenKind::U64Kind,
            Token::F32(_) => TokenKind::F32Kind,
            Token::F64(_) => TokenKind::F64Kind,
            Token::Char(_) => TokenKind::CharKind,
            Token::Str(_) => TokenKind::StrKind,
            Token::String(_) => TokenKind::StringKind,
            Token::Option(_) => TokenKind::OptionKind,
            Token::TupleStart(_) => TokenKind::TupleStartKind,
            Token::StructStart(_, _) => TokenKind::StructStartKind,
            Token::EnumStart(_, _, _) => TokenKind::EnumStartKind,
            Token::SeqStart(_) => TokenKind::SeqStartKind,
            Token::MapStart(_) => TokenKind::MapStartKind,
            Token::End => TokenKind::EndKind,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TokenKind {
    NullKind,
    BoolKind,
    IsizeKind,
    I8Kind,
    I16Kind,
    I32Kind,
    I64Kind,
    UsizeKind,
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

static PRIMITIVE_TOKEN_KINDS: &'static [TokenKind] = &[
    TokenKind::IsizeKind,
    TokenKind::I8Kind,
    TokenKind::I16Kind,
    TokenKind::I32Kind,
    TokenKind::I64Kind,
    TokenKind::UsizeKind,
    TokenKind::U8Kind,
    TokenKind::U16Kind,
    TokenKind::U32Kind,
    TokenKind::U64Kind,
    TokenKind::F32Kind,
    TokenKind::F64Kind,
];

static STR_TOKEN_KINDS: &'static [TokenKind] = &[
    TokenKind::StrKind,
    TokenKind::StringKind,
];

static COMPOUND_TOKEN_KINDS: &'static [TokenKind] = &[
    TokenKind::OptionKind,
    TokenKind::EnumStartKind,
    TokenKind::StructStartKind,
    TokenKind::TupleStartKind,
    TokenKind::SeqStartKind,
    TokenKind::MapStartKind,
];

impl ::std::fmt::Debug for TokenKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            TokenKind::NullKind => "Null".fmt(f),
            TokenKind::BoolKind => "Bool".fmt(f),
            TokenKind::IsizeKind => "Isize".fmt(f),
            TokenKind::I8Kind => "I8".fmt(f),
            TokenKind::I16Kind => "I16".fmt(f),
            TokenKind::I32Kind => "I32".fmt(f),
            TokenKind::I64Kind => "I64".fmt(f),
            TokenKind::UsizeKind => "Usize".fmt(f),
            TokenKind::U8Kind => "U8".fmt(f),
            TokenKind::U16Kind => "U16".fmt(f),
            TokenKind::U32Kind => "U32".fmt(f),
            TokenKind::U64Kind => "U64".fmt(f),
            TokenKind::F32Kind => "F32".fmt(f),
            TokenKind::F64Kind => "F64".fmt(f),
            TokenKind::CharKind => "Char".fmt(f),
            TokenKind::StrKind => "Str".fmt(f),
            TokenKind::StringKind => "String".fmt(f),
            TokenKind::OptionKind => "Option".fmt(f),
            TokenKind::TupleStartKind => "TupleStart".fmt(f),
            TokenKind::StructStartKind => "StructStart".fmt(f),
            TokenKind::EnumStartKind => "EnumStart".fmt(f),
            TokenKind::SeqStartKind => "SeqStart".fmt(f),
            TokenKind::MapStartKind => "MapStart".fmt(f),
            TokenKind::EndKind => "Token::End".fmt(f),
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

pub trait Deserializer<E>: Iterator<Item=Result<Token, E>> + Sized {
    /// Called when a `Deserialize` expected more tokens, but the
    /// `Deserializer` was empty.
    fn end_of_stream_error(&mut self) -> E;

    /// Called when a `Deserializer` was unable to properly parse the stream.
    fn syntax_error(&mut self, token: Token, expected: &'static [TokenKind]) -> E;

    /// Called when a named structure or enum got a name that it didn't expect.
    fn unexpected_name_error(&mut self, token: Token) -> E;

    /// Called when a value was unable to be coerced into another value.
    fn conversion_error(&mut self, token: Token) -> E;

    /// Called when a `Deserialize` structure did not deserialize a field
    /// named `field`.
    fn missing_field<
        T: Deserialize<Self, E>
    >(&mut self, field: &'static str) -> Result<T, E>;

    /*
    /// Called when a `Deserialize` has decided to not consume this token.
    fn ignore_field(&mut self, _token: Token) -> Result<(), E> {
        let _: IgnoreTokens = try!(Deserialize::deserialize(self));
        Ok(())
    }
    */

    #[inline]
    fn expect_token(&mut self) -> Result<Token, E> {
        self.next().unwrap_or_else(|| Err(self.end_of_stream_error()))
    }

    #[inline]
    fn expect_null(&mut self, token: Token) -> Result<(), E> {
        match token {
            Token::Null => Ok(()),
            Token::TupleStart(_) | Token::SeqStart(_) => {
                match try!(self.expect_token()) {
                    Token::End => Ok(()),
                    token => {
                        static EXPECTED_TOKENS: &'static [TokenKind] = &[
                            TokenKind::EndKind,
                        ];
                        Err(self.syntax_error(token, EXPECTED_TOKENS))
                    }
                }
            }
            token => {
                static EXPECTED_TOKENS: &'static [TokenKind] = &[
                    TokenKind::NullKind,
                    TokenKind::TupleStartKind,
                    TokenKind::SeqStartKind,
                ];
                Err(self.syntax_error(token, EXPECTED_TOKENS))
            }
        }
    }

    #[inline]
    fn expect_bool(&mut self, token: Token) -> Result<bool, E> {
        match token {
            Token::Bool(value) => Ok(value),
            token => {
                static EXPECTED_TOKENS: &'static [TokenKind] = &[
                    TokenKind::BoolKind,
                ];
                Err(self.syntax_error(token, EXPECTED_TOKENS))
            }
        }
    }

    #[inline]
    fn expect_num<T: num::NumCast>(&mut self, token: Token) -> Result<T, E> {
        match token {
            Token::Isize(x) => to_result!(num::cast(x), self.syntax_error(token, PRIMITIVE_TOKEN_KINDS)),
            Token::I8(x) => to_result!(num::cast(x), self.syntax_error(token, PRIMITIVE_TOKEN_KINDS)),
            Token::I16(x) => to_result!(num::cast(x), self.syntax_error(token, PRIMITIVE_TOKEN_KINDS)),
            Token::I32(x) => to_result!(num::cast(x), self.syntax_error(token, PRIMITIVE_TOKEN_KINDS)),
            Token::I64(x) => to_result!(num::cast(x), self.syntax_error(token, PRIMITIVE_TOKEN_KINDS)),
            Token::Usize(x) => to_result!(num::cast(x), self.syntax_error(token, PRIMITIVE_TOKEN_KINDS)),
            Token::U8(x) => to_result!(num::cast(x), self.syntax_error(token, PRIMITIVE_TOKEN_KINDS)),
            Token::U16(x) => to_result!(num::cast(x), self.syntax_error(token, PRIMITIVE_TOKEN_KINDS)),
            Token::U32(x) => to_result!(num::cast(x), self.syntax_error(token, PRIMITIVE_TOKEN_KINDS)),
            Token::U64(x) => to_result!(num::cast(x), self.syntax_error(token, PRIMITIVE_TOKEN_KINDS)),
            Token::F32(x) => to_result!(num::cast(x), self.syntax_error(token, PRIMITIVE_TOKEN_KINDS)),
            Token::F64(x) => to_result!(num::cast(x), self.syntax_error(token, PRIMITIVE_TOKEN_KINDS)),
            token => Err(self.syntax_error(token, PRIMITIVE_TOKEN_KINDS)),
        }
    }

    #[inline]
    fn expect_from_primitive<T: FromPrimitive>(&mut self, token: Token) -> Result<T, E> {
        match token {
            Token::Isize(x) => to_result!(num::from_int(x), self.conversion_error(token)),
            Token::I8(x) => to_result!(num::from_i8(x), self.conversion_error(token)),
            Token::I16(x) => to_result!(num::from_i16(x), self.conversion_error(token)),
            Token::I32(x) => to_result!(num::from_i32(x), self.conversion_error(token)),
            Token::I64(x) => to_result!(num::from_i64(x), self.conversion_error(token)),
            Token::Usize(x) => to_result!(num::from_uint(x), self.conversion_error(token)),
            Token::U8(x) => to_result!(num::from_u8(x), self.conversion_error(token)),
            Token::U16(x) => to_result!(num::from_u16(x), self.conversion_error(token)),
            Token::U32(x) => to_result!(num::from_u32(x), self.conversion_error(token)),
            Token::U64(x) => to_result!(num::from_u64(x), self.conversion_error(token)),
            Token::F32(x) => to_result!(num::from_f32(x), self.conversion_error(token)),
            Token::F64(x) => to_result!(num::from_f64(x), self.conversion_error(token)),
            token => Err(self.syntax_error(token, PRIMITIVE_TOKEN_KINDS)),
        }
    }

    #[inline]
    fn expect_char(&mut self, token: Token) -> Result<char, E> {
        match token {
            Token::Char(value) => Ok(value),
            Token::Str(value) if value.chars().count() == 1 => {
                Ok(value.char_at(0))
            }
            Token::String(ref value) if value[].chars().count() == 1 => {
                Ok(value[].char_at(0))
            }
            token => {
                static EXPECTED_TOKENS: &'static [TokenKind] = &[
                    TokenKind::CharKind,
                ];
                Err(self.syntax_error(token, EXPECTED_TOKENS))
            }
        }
    }

    #[inline]
    fn expect_str(&mut self, token: Token) -> Result<&'static str, E> {
        match token {
            Token::Str(value) => Ok(value),
            token => Err(self.syntax_error(token, STR_TOKEN_KINDS)),
        }
    }

    #[inline]
    fn expect_string(&mut self, token: Token) -> Result<string::String, E> {
        match token {
            Token::Char(value) => Ok(value.to_string()),
            Token::Str(value) => Ok(value.to_string()),
            Token::String(value) => Ok(value),
            token => Err(self.syntax_error(token, STR_TOKEN_KINDS)),
        }
    }

    #[inline]
    fn expect_option<
        T: Deserialize<Self, E>
    >(&mut self, token: Token) -> Result<option::Option<T>, E> {
        match token {
            Token::Option(false) => Ok(None),
            Token::Option(true) => {
                let value: T = try!(Deserialize::deserialize(self));
                Ok(Some(value))
            }
            token => {
                static EXPECTED_TOKENS: &'static [TokenKind] = &[
                    TokenKind::OptionKind,
                ];
                Err(self.syntax_error(token, EXPECTED_TOKENS))
            }
        }
    }

    #[inline]
    fn expect_tuple_start(&mut self, token: Token) -> Result<usize, E> {
        match token {
            Token::TupleStart(len) => Ok(len),
            Token::SeqStart(len) => Ok(len),
            token => {
                static EXPECTED_TOKENS: &'static [TokenKind] = &[
                    TokenKind::TupleStartKind,
                    TokenKind::SeqStartKind,
                ];
                Err(self.syntax_error(token, EXPECTED_TOKENS))
            }
        }
    }

    #[inline]
    fn expect_tuple_elt<
        T: Deserialize<Self, E>
    >(&mut self) -> Result<T, E> {
        Deserialize::deserialize(self)
    }

    #[inline]
    fn expect_tuple_end(&mut self) -> Result<(), E> {
        match try!(self.expect_token()) {
            Token::End => Ok(()),
            token => {
                static EXPECTED_TOKENS: &'static [TokenKind] = &[
                    TokenKind::EndKind,
                ];
                Err(self.syntax_error(token, EXPECTED_TOKENS))
            }
        }
    }

    #[inline]
    fn expect_struct_start(&mut self, token: Token, name: &str) -> Result<(), E> {
        match token {
            Token::StructStart(n, _) => {
                if name == n {
                    Ok(())
                } else {
                    Err(self.unexpected_name_error(token))
                }
            }
            _ => {
                static EXPECTED_TOKENS: &'static [TokenKind] = &[
                    TokenKind::StructStartKind,
                ];
                Err(self.syntax_error(token, EXPECTED_TOKENS))
            }
        }
    }

    #[inline]
    fn expect_struct_field_or_end(&mut self,
                                  fields: &'static [&'static str]
                                 ) -> Result<option::Option<option::Option<usize>>, E> {
        match try!(self.expect_token()) {
            Token::End => {
                Ok(None)
            }
            Token::Str(n) => {
                Ok(Some(fields.iter().position(|field| *field == n)))
            }
            Token::String(n) => {
                Ok(Some(fields.iter().position(|field| *field == &n[])))
            }
            token => {
                Err(self.syntax_error(token, STR_TOKEN_KINDS))
            }
        }
    }

    #[inline]
    fn expect_struct_value<
        T: Deserialize<Self, E>
    >(&mut self) -> Result<T, E> {
        Deserialize::deserialize(self)
    }

    #[inline]
    fn expect_struct_end(&mut self) -> Result<(), E> {
        match try!(self.expect_token()) {
            Token::End => Ok(()),
            token => {
                static EXPECTED_TOKENS: &'static [TokenKind] = &[
                    TokenKind::EndKind,
                ];
                Err(self.syntax_error(token, EXPECTED_TOKENS))
            }
        }
    }

    #[inline]
    fn expect_enum_start(&mut self, token: Token, name: &str, variants: &[&str]) -> Result<usize, E> {
        match token {
            Token::EnumStart(n, v, _) => {
                if name == n {
                    match variants.iter().position(|variant| *variant == v) {
                        Some(position) => Ok(position),
                        None => Err(self.unexpected_name_error(token)),
                    }
                } else {
                    Err(self.unexpected_name_error(token))
                }
            }
            token => {
                static EXPECTED_TOKENS: &'static [TokenKind] = &[
                    TokenKind::EnumStartKind,
                ];
                Err(self.syntax_error(token, EXPECTED_TOKENS))
            }
        }
    }

    #[inline]
    fn expect_enum_elt<
        T: Deserialize<Self, E>
    >(&mut self) -> Result<T, E> {
        Deserialize::deserialize(self)
    }

    #[inline]
    fn expect_enum_end(&mut self) -> Result<(), E> {
        match try!(self.expect_token()) {
            Token::End => Ok(()),
            token => {
                static EXPECTED_TOKENS: &'static [TokenKind] = &[
                    TokenKind::EndKind,
                ];
                Err(self.syntax_error(token, EXPECTED_TOKENS))
            }
        }
    }

    #[inline]
    fn expect_seq_start(&mut self, token: Token) -> Result<usize, E> {
        match token {
            Token::TupleStart(len) => Ok(len),
            Token::SeqStart(len) => Ok(len),
            token => {
                static EXPECTED_TOKENS: &'static [TokenKind] = &[
                    TokenKind::TupleStartKind,
                    TokenKind::SeqStartKind,
                ];
                Err(self.syntax_error(token, EXPECTED_TOKENS))
            }
        }
    }

    #[inline]
    fn expect_seq_elt_or_end<
        T: Deserialize<Self, E>
    >(&mut self) -> Result<option::Option<T>, E> {
        match try!(self.expect_token()) {
            Token::End => Ok(None),
            token => {
                let value = try!(Deserialize::deserialize_token(self, token));
                Ok(Some(value))
            }
        }
    }

    #[inline]
    fn expect_seq<
        T: Deserialize<Self, E>,
        C: FromIterator<T>
    >(&mut self, token: Token) -> Result<C, E> {
        let len = try!(self.expect_seq_start(token));
        let mut err = None;

        let collection: C = {
            let d = SeqDeserializer {
                d: self,
                len: len,
                err: &mut err,
            };

            d.collect()
        };

        match err {
            Some(err) => Err(err),
            None => Ok(collection),
        }
    }

    #[inline]
    fn expect_map_start(&mut self, token: Token) -> Result<usize, E> {
        match token {
            Token::MapStart(len) => Ok(len),
            _ => {
                static EXPECTED_TOKENS: &'static [TokenKind] = &[
                    TokenKind::MapStartKind,
                ];
                Err(self.syntax_error(token, EXPECTED_TOKENS))
            }
        }
    }

    #[inline]
    fn expect_map_elt_or_end<
        K: Deserialize<Self, E>,
        V: Deserialize<Self, E>
    >(&mut self) -> Result<option::Option<(K, V)>, E> {
        match try!(self.expect_token()) {
            Token::End => Ok(None),
            token => {
                let key = try!(Deserialize::deserialize_token(self, token));
                let value = try!(Deserialize::deserialize(self));
                Ok(Some((key, value)))
            }
        }
    }

    #[inline]
    fn expect_map<
        K: Deserialize<Self, E>,
        V: Deserialize<Self, E>,
        C: FromIterator<(K, V)>
    >(&mut self, token: Token) -> Result<C, E> {
        let len = try!(self.expect_map_start(token));
        let mut err = None;

        let collection: C = {
            let d = MapDeserializer {
                d: self,
                len: len,
                err: &mut err,
            };

            d.collect()
        };

        match err {
            Some(err) => Err(err),
            None => Ok(collection),
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

struct SeqDeserializer<'a, D: 'a, E: 'a, T> {
    d: &'a mut D,
    len: usize,
    err: &'a mut Option<E>,
}

impl<
    'a,
    D: Deserializer<E>,
    E,
    T: Deserialize<D, E>
> Iterator for SeqDeserializer<'a, D, E, T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> option::Option<T> {
        match self.d.expect_seq_elt_or_end() {
            Ok(next) => {
                self.len -= 1;
                next
            }
            Err(err) => {
                *self.err = Some(err);
                None
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, option::Option<usize>) {
        (self.len, Some(self.len))
    }
}

//////////////////////////////////////////////////////////////////////////////

struct MapDeserializer<'a, D:'a, E: 'a, K, V> {
    d: &'a mut D,
    len: usize,
    err: &'a mut option::Option<E>,
}

impl<
    'a,
    D: Deserializer<E>,
    E,
    K: Deserialize<D, E>,
    V: Deserialize<D, E>
> Iterator for MapDeserializer<'a, D, E, K, V> {
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> option::Option<(K, V)> {
        match self.d.expect_map_elt_or_end() {
            Ok(next) => next,
            Err(err) => {
                *self.err = Some(err);
                None
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, option::Option<usize>) {
        (self.len, Some(self.len))
    }
}

//////////////////////////////////////////////////////////////////////////////

pub trait Deserialize<D: Deserializer<E>, E>: Sized {
    #[inline]
    fn deserialize(d: &mut D) -> Result<Self, E> {
        let token = try!(d.expect_token());
        Deserialize::deserialize_token(d, token)
    }

    fn deserialize_token(d: &mut D, token: Token) -> Result<Self, E>;
}

//////////////////////////////////////////////////////////////////////////////

macro_rules! impl_deserialize {
    ($ty:ty, $method:ident) => {
        impl<D: Deserializer<E>, E> Deserialize<D, E> for $ty {
            #[inline]
            fn deserialize_token(d: &mut D, token: Token) -> Result<$ty, E> {
                d.$method(token)
            }
        }
    }
}

impl_deserialize!(bool, expect_bool);
impl_deserialize!(isize, expect_num);
impl_deserialize!(i8, expect_num);
impl_deserialize!(i16, expect_num);
impl_deserialize!(i32, expect_num);
impl_deserialize!(i64, expect_num);
impl_deserialize!(usize, expect_num);
impl_deserialize!(u8, expect_num);
impl_deserialize!(u16, expect_num);
impl_deserialize!(u32, expect_num);
impl_deserialize!(u64, expect_num);
impl_deserialize!(f32, expect_num);
impl_deserialize!(f64, expect_num);
impl_deserialize!(char, expect_char);
impl_deserialize!(&'static str, expect_str);
impl_deserialize!(string::String, expect_string);

//////////////////////////////////////////////////////////////////////////////

impl<
    D: Deserializer<E>,
    E,
    T: Deserialize<D, E>
> Deserialize<D, E> for Box<T> {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<Box<T>, E> {
        Ok(Box::new(try!(Deserialize::deserialize_token(d, token))))
    }
}

impl<
    D: Deserializer<E>,
    E,
    T: Deserialize<D, E>
> Deserialize<D, E> for Rc<T> {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<Rc<T>, E> {
        Ok(Rc::new(try!(Deserialize::deserialize_token(d, token))))
    }
}

impl<
    D: Deserializer<E>,
    E,
    T: Deserialize<D, E> + Send + Sync
> Deserialize<D, E> for Arc<T> {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<Arc<T>, E> {
        Ok(Arc::new(try!(Deserialize::deserialize_token(d, token))))
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<
    D: Deserializer<E>,
    E,
    T: Deserialize<D ,E>
> Deserialize<D, E> for option::Option<T> {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<option::Option<T>, E> {
        d.expect_option(token)
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<
    D: Deserializer<E>,
    E,
    T: Deserialize<D ,E>
> Deserialize<D, E> for Vec<T> {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<Vec<T>, E> {
        d.expect_seq(token)
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<
    D: Deserializer<E>,
    E,
    K: Deserialize<D, E> + Eq + Hash<Hasher>,
    V: Deserialize<D, E>
> Deserialize<D, E> for HashMap<K, V> {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<HashMap<K, V>, E> {
        d.expect_map(token)
    }
}

impl<
    D: Deserializer<E>,
    E,
    K: Deserialize<D, E> + Ord,
    V: Deserialize<D, E>
> Deserialize<D, E> for BTreeMap<K, V> {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<BTreeMap<K, V>, E> {
        d.expect_map(token)
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<
    D: Deserializer<E>,
    E,
    T: Deserialize<D, E> + Eq + Hash<Hasher>
> Deserialize<D, E> for HashSet<T> {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<HashSet<T>, E> {
        d.expect_seq(token)
    }
}

impl<
    D: Deserializer<E>,
    E,
    T: Deserialize<D, E> + Ord
> Deserialize<D, E> for BTreeSet<T> {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<BTreeSet<T>, E> {
        d.expect_seq(token)
    }
}

//////////////////////////////////////////////////////////////////////////////

macro_rules! peel {
    ($name:ident, $($other:ident,)*) => ( impl_deserialize_tuple!($($other,)*); )
}

macro_rules! impl_deserialize_tuple {
    () => {
        impl<
            D: Deserializer<E>,
            E
        > Deserialize<D, E> for () {
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
            $($name: Deserialize<D, E>),*
        > Deserialize<D, E> for ($($name,)*) {
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
        peel!($($name,)*);
    }
}

impl_deserialize_tuple! { T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, }

//////////////////////////////////////////////////////////////////////////////

/// Helper struct that will ignore tokens while taking in consideration
/// recursive structures.
#[derive(Copy)]
pub struct IgnoreTokens;

impl<D: Deserializer<E>, E> Deserialize<D, E> for IgnoreTokens {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<IgnoreTokens, E> {
        match token {
            Token::Option(true) => {
                Deserialize::deserialize(d)
            }

            Token::EnumStart(_, _, _) => {
                loop {
                    match try!(d.expect_token()) {
                        Token::End => { return Ok(IgnoreTokens); }
                        token => {
                            let _: IgnoreTokens = try!(Deserialize::deserialize_token(d, token));
                        }
                    }
                }
            }

            Token::StructStart(_, _) => {
                loop {
                    match try!(d.expect_token()) {
                        Token::End => { return Ok(IgnoreTokens); }
                        Token::Str(_) | Token::String(_) => {
                            let _: IgnoreTokens = try!(Deserialize::deserialize(d));
                        }
                        _token => {
                            static EXPECTED_TOKENS: &'static [TokenKind] = &[
                                TokenKind::EndKind,
                                TokenKind::StrKind,
                                TokenKind::StringKind,
                            ];
                            return Err(d.syntax_error(token, EXPECTED_TOKENS));
                        }
                    }
                }
            }

            Token::TupleStart(_) => {
                loop {
                    match try!(d.expect_token()) {
                        Token::End => { return Ok(IgnoreTokens); }
                        token => {
                            let _: IgnoreTokens = try!(Deserialize::deserialize_token(d, token));
                        }
                    }
                }
            }

            Token::SeqStart(_) => {
                loop {
                    match try!(d.expect_token()) {
                        Token::End => { return Ok(IgnoreTokens); }
                        token => {
                            let _: IgnoreTokens = try!(Deserialize::deserialize_token(d, token));
                        }
                    }
                }
            }

            Token::MapStart(_) => {
                loop {
                    match try!(d.expect_token()) {
                        Token::End => { return Ok(IgnoreTokens); }
                        token => {
                            let _: IgnoreTokens = try!(Deserialize::deserialize_token(d, token));
                            let _: IgnoreTokens = try!(Deserialize::deserialize(d));
                        }
                    }
                }
            }

            Token::End => {
                Err(d.syntax_error(token, COMPOUND_TOKEN_KINDS))
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
            token @ Token::Option(true) => {
                self.tokens.push(token);
                self.gather(d)
            }
            Token::EnumStart(name, variant, len) => {
                self.tokens.reserve(len + 1);
                self.tokens.push(Token::EnumStart(name, variant, len));
                self.gather_seq(d)
            }
            Token::StructStart(name, len) => {
                self.tokens.reserve(len + 1);
                self.tokens.push(Token::StructStart(name, len));
                self.gather_struct(d)
            }
            Token::TupleStart(len) => {
                self.tokens.reserve(len + 1);
                self.tokens.push(Token::TupleStart(len));
                self.gather_seq(d)
            }
            Token::SeqStart(len) => {
                self.tokens.reserve(len + 1);
                self.tokens.push(Token::SeqStart(len));
                self.gather_seq(d)
            }
            Token::MapStart(len) => {
                self.tokens.reserve(len + 1);
                self.tokens.push(Token::MapStart(len));
                self.gather_map(d)
            }
            Token::End => {
                Err(d.syntax_error(token, COMPOUND_TOKEN_KINDS))
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
                token @ Token::End => {
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
                token @ Token::End => {
                    self.tokens.push(token);
                    return Ok(());
                }
                token @ Token::Str(_) | token @ Token::String(_) => {
                    self.tokens.push(token);
                    try!(self.gather(d))
                }
                token => {
                    static EXPECTED_TOKENS: &'static [TokenKind] = &[
                        TokenKind::EndKind,
                        TokenKind::StrKind,
                        TokenKind::StringKind,
                    ];
                    return Err(d.syntax_error(token, EXPECTED_TOKENS));
                }
            }
        }
    }

    #[inline]
    fn gather_map<D: Deserializer<E>, E>(&mut self, d: &mut D) -> Result<(), E> {
        loop {
            match try!(d.expect_token()) {
                Token::End => {
                    self.tokens.push(Token::End);
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

impl<D: Deserializer<E>, E> Deserialize<D, E> for GatherTokens {
    #[inline]
    fn deserialize_token(d: &mut D, token: Token) -> Result<GatherTokens, E> {
        let mut tokens = GatherTokens {
            tokens: vec!(),
        };
        try!(tokens.gather_token(d, token));
        Ok(tokens)
    }
}
