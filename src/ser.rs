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
use std::hash::Hash;
use std::rc::Rc;
use std::sync::Arc;

//////////////////////////////////////////////////////////////////////////////

pub trait Serializer<E> {
    fn serialize_null(&mut self) -> Result<(), E>;

    fn serialize_bool(&mut self, v: bool) -> Result<(), E>;

    #[inline]
    fn serialize_int(&mut self, v: int) -> Result<(), E> {
        self.serialize_i64(v as i64)
    }

    #[inline]
    fn serialize_i8(&mut self, v: i8) -> Result<(), E> {
        self.serialize_i64(v as i64)
    }

    #[inline]
    fn serialize_i16(&mut self, v: i16) -> Result<(), E> {
        self.serialize_i64(v as i64)
    }

    #[inline]
    fn serialize_i32(&mut self, v: i32) -> Result<(), E> {
        self.serialize_i64(v as i64)
    }

    #[inline]
    fn serialize_i64(&mut self, v: i64) -> Result<(), E>;

    #[inline]
    fn serialize_uint(&mut self, v: uint) -> Result<(), E> {
        self.serialize_u64(v as u64)
    }

    #[inline]
    fn serialize_u8(&mut self, v: u8) -> Result<(), E> {
        self.serialize_u64(v as u64)
    }

    #[inline]
    fn serialize_u16(&mut self, v: u16) -> Result<(), E> {
        self.serialize_u64(v as u64)
    }

    #[inline]
    fn serialize_u32(&mut self, v: u32) -> Result<(), E> {
        self.serialize_u64(v as u64)
    }

    #[inline]
    fn serialize_u64(&mut self, v: u64) -> Result<(), E>;

    #[inline]
    fn serialize_f32(&mut self, v: f32) -> Result<(), E> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(&mut self, v: f64) -> Result<(), E>;

    fn serialize_char(&mut self, v: char) -> Result<(), E>;

    fn serialize_str(&mut self, v: &str) -> Result<(), E>;

    fn serialize_tuple_start(&mut self, len: uint) -> Result<(), E>;
    fn serialize_tuple_elt<
        T: Serialize<Self, E>
    >(&mut self, v: &T) -> Result<(), E>;
    fn serialize_tuple_end(&mut self) -> Result<(), E>;

    fn serialize_struct_start(&mut self, name: &str, len: uint) -> Result<(), E>;
    fn serialize_struct_elt<
        T: Serialize<Self, E>
    >(&mut self, name: &str, v: &T) -> Result<(), E>;
    fn serialize_struct_end(&mut self) -> Result<(), E>;

    fn serialize_enum_start(&mut self, name: &str, variant: &str, len: uint) -> Result<(), E>;
    fn serialize_enum_elt<
        T: Serialize<Self, E>
    >(&mut self, v: &T) -> Result<(), E>;
    fn serialize_enum_end(&mut self) -> Result<(), E>;

    fn serialize_option<
        T: Serialize<Self, E>
    >(&mut self, v: &Option<T>) -> Result<(), E>;

    fn serialize_seq<
        T: Serialize<Self, E>,
        Iter: Iterator<T>
    >(&mut self, iter: Iter) -> Result<(), E>;

    fn serialize_map<
        K: Serialize<Self, E>,
        V: Serialize<Self, E>,
        Iter: Iterator<(K, V)>
    >(&mut self, iter: Iter) -> Result<(), E>;
}

//////////////////////////////////////////////////////////////////////////////

pub trait Serialize<S: Serializer<E>, E> {
    fn serialize(&self, s: &mut S) -> Result<(), E>;
}

//////////////////////////////////////////////////////////////////////////////

macro_rules! impl_serialize {
    ($ty:ty, $method:ident) => {
        impl<S: Serializer<E>, E> Serialize<S, E> for $ty {
            #[inline]
            fn serialize(&self, s: &mut S) -> Result<(), E> {
                s.$method(*self)
            }
        }
    }
}

impl_serialize!(bool, serialize_bool);
impl_serialize!(int, serialize_int);
impl_serialize!(i8, serialize_i8);
impl_serialize!(i16, serialize_i16);
impl_serialize!(i32, serialize_i32);
impl_serialize!(i64, serialize_i64);
impl_serialize!(uint, serialize_uint);
impl_serialize!(u8, serialize_u8);
impl_serialize!(u16, serialize_u16);
impl_serialize!(u32, serialize_u32);
impl_serialize!(u64, serialize_u64);
impl_serialize!(f32, serialize_f32);
impl_serialize!(f64, serialize_f64);
impl_serialize!(char, serialize_char);

//////////////////////////////////////////////////////////////////////////////

impl<'a, S: Serializer<E>, E> Serialize<S, E> for &'a str {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        s.serialize_str(*self)
    }
}

impl<S: Serializer<E>, E> Serialize<S, E> for String {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        self.as_slice().serialize(s)
    }
}

//////////////////////////////////////////////////////////////////////////////

macro_rules! impl_serialize_box {
    ($ty:ty) => {
        impl<
            'a,
            S: Serializer<E>,
            E,
            T: Serialize<S, E>
        > Serialize<S, E> for $ty {
            #[inline]
            fn serialize(&self, s: &mut S) -> Result<(), E> {
                (**self).serialize(s)
            }
        }
    }
}

impl_serialize_box!(&'a T);
impl_serialize_box!(Box<T>);
impl_serialize_box!(Rc<T>);

impl<
    S: Serializer<E>,
    E,
    T: Serialize<S, E> + Send + Sync
> Serialize<S, E> for Arc<T> {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        (**self).serialize(s)
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<
    S: Serializer<E>,
    E,
    T: Serialize<S, E>
> Serialize<S, E> for Option<T> {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        s.serialize_option(self)
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<
    S: Serializer<E>,
    E,
    T: Serialize<S, E>
> Serialize<S, E> for Vec<T> {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        s.serialize_seq(self.iter())
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<
    S: Serializer<E>,
    E,
    K: Serialize<S, E> + Eq + Hash,
    V: Serialize<S, E>
> Serialize<S, E> for HashMap<K, V> {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        s.serialize_map(self.iter())
    }
}

impl<
    S: Serializer<E>,
    E,
    K: Serialize<S, E> + Ord,
    V: Serialize<S, E>
> Serialize<S, E> for TreeMap<K, V> {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        s.serialize_map(self.iter())
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<
    S: Serializer<E>,
    E,
    T: Serialize<S, E> + Eq + Hash
> Serialize<S, E> for HashSet<T> {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        s.serialize_seq(self.iter())
    }
}

impl<
    S: Serializer<E>,
    E,
    T: Serialize<S, E> + Ord
> Serialize<S, E> for TreeSet<T> {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        s.serialize_seq(self.iter())
    }
}

//////////////////////////////////////////////////////////////////////////////

macro_rules! peel {
    ($name:ident, $($other:ident,)*) => ( impl_serialize_tuple!($($other,)*);  )
}

macro_rules! impl_serialize_tuple {
    () => {
        impl<S: Serializer<E>, E> Serialize<S, E> for () {
            #[inline]
            fn serialize(&self, s: &mut S) -> Result<(), E> {
                s.serialize_null()
            }
        }
    };
    ( $($name:ident,)+ ) => {
        impl<
            S: Serializer<E>,
            E,
            $($name:Serialize<S, E>),+
        > Serialize<S, E> for ($($name,)+) {
            #[inline]
            #[allow(non_snake_case)]
            fn serialize(&self, s: &mut S) -> Result<(), E> {
                // FIXME: how can we count macro args?
                let mut len = 0;
                $({ let $name = 1; len += $name; })*;

                let ($(ref $name,)*) = *self;

                try!(s.serialize_tuple_start(len));
                $(
                    try!(s.serialize_tuple_elt($name));
                 )*
                s.serialize_tuple_end()
            }
        }
        peel!($($name,)*);
    }
}

impl_serialize_tuple! { T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, }

//////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, TreeMap};

    use std::{option, string};

    use serialize::Decoder;

    use super::{Serializer, Serialize};

    //////////////////////////////////////////////////////////////////////////////

    #[deriving(Clone, PartialEq, Show, Decodable)]
    #[deriving_serialize]
    struct Inner {
        a: (),
        b: uint,
        c: HashMap<string::String, option::Option<char>>,
    }

    //////////////////////////////////////////////////////////////////////////////

    #[deriving(Clone, PartialEq, Show, Decodable)]
    #[deriving_serialize]
    struct Outer {
        inner: Vec<Inner>,
    }

    //////////////////////////////////////////////////////////////////////////////

    #[deriving(Clone, PartialEq, Show, Decodable)]
    #[deriving_serialize]
    enum Animal {
        Dog,
        Frog(String, int)
    }

    //////////////////////////////////////////////////////////////////////////////

    #[deriving(Clone, PartialEq, Show)]
    pub enum Token<'a> {
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
        Str(&'a str),

        TupleStart(uint),
        TupleSep,
        TupleEnd,

        StructStart(&'a str, uint),
        StructSep(&'a str),
        StructEnd,

        EnumStart(&'a str, &'a str, uint),
        EnumSep,
        EnumEnd,

        Option(bool),

        SeqStart(uint),
        SeqEnd,

        MapStart(uint),
        MapEnd,
    }

    #[deriving(Show)]
    #[allow(dead_code)]
    enum Error {
        EndOfStream,
        SyntaxError,
    }

    //////////////////////////////////////////////////////////////////////////////

    struct AssertSerializer<Iter> {
        iter: Iter,
    }

    impl<'a, Iter: Iterator<Token<'a>>> AssertSerializer<Iter> {
        fn new(iter: Iter) -> AssertSerializer<Iter> {
            AssertSerializer {
                iter: iter,
            }
        }

        fn serialize<'b>(&mut self, token: Token<'b>) -> Result<(), Error> {
            let t = match self.iter.next() {
                Some(t) => t,
                None => { panic!(); }
            };

            assert_eq!(t, token);

            Ok(())
        }
    }

    impl<'a, Iter: Iterator<Token<'a>>> Serializer<Error> for AssertSerializer<Iter> {
        fn serialize_null(&mut self) -> Result<(), Error> {
            self.serialize(Token::Null)
        }
        fn serialize_bool(&mut self, v: bool) -> Result<(), Error> {
            self.serialize(Token::Bool(v))
        }
        fn serialize_int(&mut self, v: int) -> Result<(), Error> {
            self.serialize(Token::Int(v))
        }

        fn serialize_i8(&mut self, v: i8) -> Result<(), Error> {
            self.serialize(Token::I8(v))
        }

        fn serialize_i16(&mut self, v: i16) -> Result<(), Error> {
            self.serialize(Token::I16(v))
        }

        fn serialize_i32(&mut self, v: i32) -> Result<(), Error> {
            self.serialize(Token::I32(v))
        }

        fn serialize_i64(&mut self, v: i64) -> Result<(), Error> {
            self.serialize(Token::I64(v))
        }

        fn serialize_uint(&mut self, v: uint) -> Result<(), Error> {
            self.serialize(Token::Uint(v))
        }

        fn serialize_u8(&mut self, v: u8) -> Result<(), Error> {
            self.serialize(Token::U8(v))
        }

        fn serialize_u16(&mut self, v: u16) -> Result<(), Error> {
            self.serialize(Token::U16(v))
        }

        fn serialize_u32(&mut self, v: u32) -> Result<(), Error> {
            self.serialize(Token::U32(v))
        }

        fn serialize_u64(&mut self, v: u64) -> Result<(), Error> {
            self.serialize(Token::U64(v))
        }

        fn serialize_f32(&mut self, v: f32) -> Result<(), Error> {
            self.serialize(Token::F32(v))
        }

        fn serialize_f64(&mut self, v: f64) -> Result<(), Error> {
            self.serialize(Token::F64(v))
        }

        fn serialize_char(&mut self, v: char) -> Result<(), Error> {
            self.serialize(Token::Char(v))
        }

        fn serialize_str(&mut self, v: &str) -> Result<(), Error> {
            self.serialize(Token::Str(v))
        }

        fn serialize_tuple_start(&mut self, len: uint) -> Result<(), Error> {
            self.serialize(Token::TupleStart(len))
        }

        fn serialize_tuple_elt<
            T: Serialize<AssertSerializer<Iter>, Error>
        >(&mut self, value: &T) -> Result<(), Error> {
            try!(self.serialize(Token::TupleSep));
            value.serialize(self)
        }

        fn serialize_tuple_end(&mut self) -> Result<(), Error> {
            self.serialize(Token::TupleEnd)
        }

        fn serialize_struct_start(&mut self, name: &str, len: uint) -> Result<(), Error> {
            self.serialize(Token::StructStart(name, len))
        }

        fn serialize_struct_elt<
            T: Serialize<AssertSerializer<Iter>, Error>
        >(&mut self, name: &str, value: &T) -> Result<(), Error> {
            try!(self.serialize(Token::StructSep(name)));
            value.serialize(self)
        }

        fn serialize_struct_end(&mut self) -> Result<(), Error> {
            self.serialize(Token::StructEnd)
        }

        fn serialize_enum_start(&mut self, name: &str, variant: &str, len: uint) -> Result<(), Error> {
            self.serialize(Token::EnumStart(name, variant, len))
        }

        fn serialize_enum_elt<
            T: Serialize<AssertSerializer<Iter>, Error>
        >(&mut self, value: &T) -> Result<(), Error> {
            try!(self.serialize(Token::EnumSep));
            value.serialize(self)
        }

        fn serialize_enum_end(&mut self) -> Result<(), Error> {
            self.serialize(Token::EnumEnd)
        }

        fn serialize_option<
            T: Serialize<AssertSerializer<Iter>, Error>
        >(&mut self, v: &option::Option<T>) -> Result<(), Error> {
            match *v {
                Some(ref v) => {
                    try!(self.serialize(Token::Option(true)));
                    v.serialize(self)
                }
                None => {
                    self.serialize(Token::Option(false))
                }
            }
        }

        fn serialize_seq<
            T: Serialize<AssertSerializer<Iter>, Error>,
            SeqIter: Iterator<T>
        >(&mut self, mut iter: SeqIter) -> Result<(), Error> {
            let (len, _) = iter.size_hint();
            try!(self.serialize(Token::SeqStart(len)));
            for elt in iter {
                try!(elt.serialize(self));
            }
            self.serialize(Token::SeqEnd)
        }

        fn serialize_map<
            K: Serialize<AssertSerializer<Iter>, Error>,
            V: Serialize<AssertSerializer<Iter>, Error>,
            MapIter: Iterator<(K, V)>
        >(&mut self, mut iter: MapIter) -> Result<(), Error> {
            let (len, _) = iter.size_hint();
            try!(self.serialize(Token::MapStart(len)));
            for (key, value) in iter {
                try!(key.serialize(self));
                try!(value.serialize(self));
            }
            self.serialize(Token::MapEnd)
        }
    }

    //////////////////////////////////////////////////////////////////////////////

    #[test]
    fn test_tokens_int() {
        let tokens = vec!(
            Token::Int(5)
        );
        let mut serializer = AssertSerializer::new(tokens.into_iter());
        5i.serialize(&mut serializer).unwrap();
        assert_eq!(serializer.iter.next(), None);
    }

    #[test]
    fn test_tokens_str() {
        let tokens = vec!(
            Token::Str("a"),
        );

        let mut serializer = AssertSerializer::new(tokens.into_iter());
        "a".serialize(&mut serializer).unwrap();
        assert_eq!(serializer.iter.next(), None);
    }

    #[test]
    fn test_tokens_null() {
        let tokens = vec!(
            Token::Null,
        );

        let mut serializer = AssertSerializer::new(tokens.into_iter());
        ().serialize(&mut serializer).unwrap();
        assert_eq!(serializer.iter.next(), None);
    }

    #[test]
    fn test_tokens_option_none() {
        let tokens = vec!(
            Token::Option(false),
        );

        let mut serializer = AssertSerializer::new(tokens.into_iter());
        None::<int>.serialize(&mut serializer).unwrap();
        assert_eq!(serializer.iter.next(), None);
    }

    #[test]
    fn test_tokens_option_some() {
        let tokens = vec!(
            Token::Option(true),
            Token::Int(5),
        );

        let mut serializer = AssertSerializer::new(tokens.into_iter());
        Some(5i).serialize(&mut serializer).unwrap();
        assert_eq!(serializer.iter.next(), None);
    }

    #[test]
    fn test_tokens_tuple() {
        let tokens = vec!(
            Token::TupleStart(2),
                Token::TupleSep,
                Token::Int(5),

                Token::TupleSep,
                Token::Str("a"),
            Token::TupleEnd,
        );

        let mut serializer = AssertSerializer::new(tokens.into_iter());
        (5i, "a").serialize(&mut serializer).unwrap();
        assert_eq!(serializer.iter.next(), None);
    }

    #[test]
    fn test_tokens_tuple_compound() {
        let tokens = vec!(
            Token::TupleStart(3),
                Token::TupleSep,
                Token::Null,

                Token::TupleSep,
                Token::Null,

                Token::TupleSep,
                Token::TupleStart(2),
                    Token::TupleSep,
                    Token::Int(5),

                    Token::TupleSep,
                    Token::Str("a"),
                Token::TupleEnd,
            Token::TupleEnd,
        );

        let mut serializer = AssertSerializer::new(tokens.into_iter());
        ((), (), (5i, "a")).serialize(&mut serializer).unwrap();
        assert_eq!(serializer.iter.next(), None);
    }

    #[test]
    fn test_tokens_struct_empty() {
        let tokens = vec!(
            Token::StructStart("Outer", 1),
                Token::StructSep("inner"),
                Token::SeqStart(0),
                Token::SeqEnd,
            Token::StructEnd,
        );

        let mut serializer = AssertSerializer::new(tokens.into_iter());
        Outer { inner: vec!() }.serialize(&mut serializer).unwrap();
        assert_eq!(serializer.iter.next(), None);
    }

    #[test]
    fn test_tokens_struct() {
        let tokens = vec!(
            Token::StructStart("Outer", 1),
                Token::StructSep("inner"),
                Token::SeqStart(1),
                    Token::StructStart("Inner", 3),
                        Token::StructSep("a"),
                        Token::Null,

                        Token::StructSep("b"),
                        Token::Uint(5),

                        Token::StructSep("c"),
                        Token::MapStart(1),
                            Token::Str("abc"),
                            Token::Option(true),
                            Token::Char('c'),
                        Token::MapEnd,
                    Token::StructEnd,
                Token::SeqEnd,
            Token::StructEnd,
        );

        let mut serializer = AssertSerializer::new(tokens.into_iter());

        let mut map = HashMap::new();
        map.insert("abc".to_string(), Some('c'));

        Outer {
            inner: vec!(
                Inner {
                    a: (),
                    b: 5,
                    c: map,
                },
            )
        }.serialize(&mut serializer).unwrap();
        assert_eq!(serializer.iter.next(), None);
    }

    #[test]
    fn test_tokens_enum() {
        let tokens = vec!(
            Token::EnumStart("Animal", "Dog", 0),
            Token::EnumEnd,
        );

        let mut serializer = AssertSerializer::new(tokens.into_iter());
        Animal::Dog.serialize(&mut serializer).unwrap();
        assert_eq!(serializer.iter.next(), None);

        let tokens = vec!(
            Token::EnumStart("Animal", "Frog", 2),
                Token::EnumSep,
                Token::Str("Henry"),

                Token::EnumSep,
                Token::Int(349),
            Token::EnumEnd,
        );

        let mut serializer = AssertSerializer::new(tokens.into_iter());
        Animal::Frog("Henry".to_string(), 349).serialize(&mut serializer).unwrap();
        assert_eq!(serializer.iter.next(), None);
    }

    #[test]
    fn test_tokens_vec_empty() {
        let tokens = vec!(
            Token::SeqStart(0),
            Token::SeqEnd,
        );

        let mut serializer = AssertSerializer::new(tokens.into_iter());
        let v: Vec<int> = vec!();
        v.serialize(&mut serializer).unwrap();
        assert_eq!(serializer.iter.next(), None);
    }

    #[test]
    fn test_tokens_vec() {
        let tokens = vec!(
            Token::SeqStart(3),
                Token::Int(5),
                Token::Int(6),
                Token::Int(7),
            Token::SeqEnd,
        );

        let mut serializer = AssertSerializer::new(tokens.into_iter());
        (vec!(5i, 6, 7)).serialize(&mut serializer).unwrap();
        assert_eq!(serializer.iter.next(), None);
    }

    #[test]
    fn test_tokens_vec_compound() {
        let tokens = vec!(
            Token::SeqStart(3),
                Token::SeqStart(1),
                    Token::Int(1),
                Token::SeqEnd,

                Token::SeqStart(2),
                    Token::Int(2),
                    Token::Int(3),
                Token::SeqEnd,

                Token::SeqStart(3),
                    Token::Int(4),
                    Token::Int(5),
                    Token::Int(6),
                Token::SeqEnd,
            Token::SeqEnd,
        );

        let mut serializer = AssertSerializer::new(tokens.into_iter());
        (vec!(vec!(1i), vec!(2, 3), vec!(4, 5, 6))).serialize(&mut serializer).unwrap();
        assert_eq!(serializer.iter.next(), None);
    }

    #[test]
    fn test_tokens_treemap() {
        let tokens = vec!(
            Token::MapStart(2),
                Token::Int(5),
                Token::Str("a"),

                Token::Int(6),
                Token::Str("b"),
            Token::MapEnd,
        );

        let mut serializer = AssertSerializer::new(tokens.into_iter());

        let mut map = TreeMap::new();
        map.insert(5i, "a".to_string());
        map.insert(6i, "b".to_string());

        map.serialize(&mut serializer).unwrap();
        assert_eq!(serializer.iter.next(), None);
    }
}
