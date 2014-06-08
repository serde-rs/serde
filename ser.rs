// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::{HashMap, TreeMap};
use std::hash::Hash;

#[deriving(Clone, PartialEq, Show)]
pub enum Token<'a> {
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
    Option(bool),

    TupleStart(uint),
    StructStart(&'a str, uint),
    EnumStart(&'a str, &'a str, uint),
    SeqStart(uint),
    MapStart(uint),

    End,
}

//////////////////////////////////////////////////////////////////////////////

pub trait Serializer<E> {
    fn serialize<'a>(&mut self, token: Token<'a>) -> Result<(), E>;

    fn syntax_error<T>(&self) -> Result<T, E>;
}

//////////////////////////////////////////////////////////////////////////////

pub trait Serializable<E, S: Serializer<E>> {
    fn serialize(&self, s: &mut S) -> Result<(), E>;
}

//////////////////////////////////////////////////////////////////////////////

macro_rules! impl_serializable {
    ($ty:ty, $variant:expr) => {
        impl<'a, E, S: Serializer<E>> Serializable<E, S> for $ty {
            #[inline]
            fn serialize(&self, s: &mut S) -> Result<(), E> {
                s.serialize($variant)
            }
        }
    }
}

impl_serializable!(bool, Bool(*self))
impl_serializable!(int, Int(*self))
impl_serializable!(i8, I8(*self))
impl_serializable!(i16, I16(*self))
impl_serializable!(i32, I32(*self))
impl_serializable!(i64, I64(*self))
impl_serializable!(uint, Uint(*self))
impl_serializable!(u8, U8(*self))
impl_serializable!(u16, U16(*self))
impl_serializable!(u32, U32(*self))
impl_serializable!(u64, U64(*self))
impl_serializable!(f32, F32(*self))
impl_serializable!(f64, F64(*self))
impl_serializable!(char, Char(*self))
impl_serializable!(&'a str, Str(*self))
impl_serializable!(String, Str(self.as_slice()))

//////////////////////////////////////////////////////////////////////////////

impl<
    E,
    S: Serializer<E>,
    T: Serializable<E, S>
> Serializable<E, S> for Option<T> {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        match *self {
            Some(ref value) => {
                try!(s.serialize(Option(true)));
                value.serialize(s)
            }
            None => {
                s.serialize(Option(false))
            }
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<
    E,
    S: Serializer<E>,
    T: Serializable<E, S>
> Serializable<E, S> for Vec<T> {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        try!(s.serialize(SeqStart(self.len())));
        for elt in self.iter() {
            try!(elt.serialize(s));
        }
        s.serialize(End)
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<
    E,
    S: Serializer<E>,
    K: Serializable<E, S> + Eq + Hash,
    V: Serializable<E, S>
> Serializable<E, S> for HashMap<K, V> {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        try!(s.serialize(MapStart(self.len())));
        for (key, value) in self.iter() {
            try!(key.serialize(s));
            try!(value.serialize(s));
        }
        s.serialize(End)
    }
}

impl<
    E,
    S: Serializer<E>,
    K: Serializable<E, S> + Ord,
    V: Serializable<E, S>
> Serializable<E, S> for TreeMap<K, V> {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        try!(s.serialize(MapStart(self.len())));
        for (key, value) in self.iter() {
            try!(key.serialize(s));
            try!(value.serialize(s));
        }
        s.serialize(End)
    }
}

//////////////////////////////////////////////////////////////////////////////

macro_rules! peel {
    ($name:ident, $($other:ident,)*) => (impl_serialize_tuple!($($other,)*))
}

macro_rules! impl_serialize_tuple {
    () => {
        impl<
            E,
            S: Serializer<E>
        > Serializable<E, S> for () {
            #[inline]
            fn serialize(&self, s: &mut S) -> Result<(), E> {
                try!(s.serialize(TupleStart(0)));
                s.serialize(End)
            }
        }
    };
    ( $($name:ident,)+ ) => {
        impl<
            E,
            S: Serializer<E>,
            $($name:Serializable<E, S>),*
        > Serializable<E, S> for ($($name,)*) {
            #[inline]
            #[allow(uppercase_variables)]
            fn serialize(&self, s: &mut S) -> Result<(), E> {
                // FIXME: how can we count macro args?
                let mut len = 0;
                $({ let $name = 1; len += $name; })*;

                let ($(ref $name,)*) = *self;

                try!(s.serialize(TupleStart(len)));
                $(try!($name.serialize(s));)*
                s.serialize(End)
            }
        }
        peel!($($name,)*)
    }
}

impl_serialize_tuple! { T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, }

//////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serialize::Decoder;

    use super::{Token, Int, Uint, Str, Char, Option};
    use super::{TupleStart, StructStart, EnumStart};
    use super::{SeqStart, MapStart, End};
    use super::{Serializer, Serializable};

    //////////////////////////////////////////////////////////////////////////////

    #[deriving(Clone, PartialEq, Show, Decodable)]
    struct Inner {
        a: (),
        b: uint,
        c: HashMap<String, Option<char>>,
    }

    impl<E, S: Serializer<E>> Serializable<E, S> for Inner {
        #[inline]
        fn serialize(&self, s: &mut S) -> Result<(), E> {
            try!(s.serialize(StructStart("Inner", 3)));
            try!(s.serialize(Str("a")));
            try!(self.a.serialize(s));
            try!(s.serialize(Str("b")));
            try!(self.b.serialize(s));
            try!(s.serialize(Str("c")));
            try!(self.c.serialize(s));
            s.serialize(End)
        }
    }

    //////////////////////////////////////////////////////////////////////////////

    #[deriving(Clone, PartialEq, Show, Decodable)]
    struct Outer {
        inner: Vec<Inner>,
    }

    impl<E, S: Serializer<E>> Serializable<E, S> for Outer {
        #[inline]
        fn serialize(&self, s: &mut S) -> Result<(), E> {
            try!(s.serialize(StructStart("Outer", 1)));
            try!(s.serialize(Str("inner")));
            try!(self.inner.serialize(s));
            s.serialize(End)
        }
    }

    //////////////////////////////////////////////////////////////////////////////

    #[deriving(Clone, PartialEq, Show, Decodable)]
    enum Animal {
        Dog,
        Frog(String, int)
    }

    impl<E, S: Serializer<E>> Serializable<E, S> for Animal {
        #[inline]
        fn serialize(&self, s: &mut S) -> Result<(), E> {
            match *self {
                Dog => {
                    try!(s.serialize(EnumStart("Animal", "Dog", 0)));
                }
                Frog(ref x, y) => {
                    try!(s.serialize(EnumStart("Animal", "Frog", 2)));
                    try!(x.serialize(s));
                    try!(y.serialize(s));
                }
            }
            s.serialize(End)
        }
    }

    //////////////////////////////////////////////////////////////////////////////

    #[deriving(Show)]
    enum Error {
        EndOfStream,
        SyntaxError,
    }

    //////////////////////////////////////////////////////////////////////////////

    struct AssertSerializer<Iter> {
        iter: Iter,
    }

    impl<'a, Iter: Iterator<Token<'a>>> AssertSerializer<Iter> {
        #[inline]
        fn new(iter: Iter) -> AssertSerializer<Iter> {
            AssertSerializer {
                iter: iter,
            }
        }
    }

    impl<'a, Iter: Iterator<Token<'a>>> Serializer<Error> for AssertSerializer<Iter> {
        #[inline]
        fn serialize<'b>(&mut self, token: Token<'b>) -> Result<(), Error> {
            let t = match self.iter.next() {
                Some(t) => t,
                None => { fail!(); }
            };

            assert_eq!(t, token);

            Ok(())
        }

        fn syntax_error<T>(&self) -> Result<T, Error> {
            Err(SyntaxError)
        }
    }

    //////////////////////////////////////////////////////////////////////////////

    #[test]
    fn test_tokens_int() {
        let tokens = vec!(
            Int(5)
        );
        let mut serializer = AssertSerializer::new(tokens.move_iter());
        5i.serialize(&mut serializer).unwrap();
        assert_eq!(serializer.iter.next(), None);
    }

    #[test]
    fn test_tokens_str() {
        let tokens = vec!(
            Str("a"),
        );

        let mut serializer = AssertSerializer::new(tokens.move_iter());
        "a".serialize(&mut serializer).unwrap();
        assert_eq!(serializer.iter.next(), None);
    }

    #[test]
    fn test_tokens_null() {
        let tokens = vec!(
            TupleStart(0),
            End,
        );

        let mut serializer = AssertSerializer::new(tokens.move_iter());
        ().serialize(&mut serializer).unwrap();
        assert_eq!(serializer.iter.next(), None);
    }

    #[test]
    fn test_tokens_option_none() {
        let tokens = vec!(
            Option(false),
        );

        let mut serializer = AssertSerializer::new(tokens.move_iter());
        None::<int>.serialize(&mut serializer).unwrap();
        assert_eq!(serializer.iter.next(), None);
    }

    #[test]
    fn test_tokens_option_some() {
        let tokens = vec!(
            Option(true),
            Int(5),
        );

        let mut serializer = AssertSerializer::new(tokens.move_iter());
        Some(5).serialize(&mut serializer).unwrap();
        assert_eq!(serializer.iter.next(), None);
    }

    #[test]
    fn test_tokens_tuple() {
        let tokens = vec!(
            TupleStart(2),
                Int(5),
                Str("a"),
            End,
        );

        let mut serializer = AssertSerializer::new(tokens.move_iter());
        (5, "a").serialize(&mut serializer).unwrap();
        assert_eq!(serializer.iter.next(), None);
    }

    #[test]
    fn test_tokens_tuple_compound() {
        let tokens = vec!(
            TupleStart(3),
                TupleStart(0),
                End,

                TupleStart(0),
                End,

                TupleStart(2),
                    Int(5),
                    Str("a"),
                End,
            End,
        );

        let mut serializer = AssertSerializer::new(tokens.move_iter());
        ((), (), (5, "a")).serialize(&mut serializer).unwrap();
        assert_eq!(serializer.iter.next(), None);
    }

    #[test]
    fn test_tokens_struct_empty() {
        let tokens = vec!(
            StructStart("Outer", 1),
                Str("inner"),
                SeqStart(0),
                End,
            End,
        );

        let mut serializer = AssertSerializer::new(tokens.move_iter());
        Outer { inner: vec!() }.serialize(&mut serializer).unwrap();
        assert_eq!(serializer.iter.next(), None);
    }

    #[test]
    fn test_tokens_struct() {
        let tokens = vec!(
            StructStart("Outer", 1),
                Str("inner"),
                SeqStart(1),
                    StructStart("Inner", 3),
                        Str("a"),
                        TupleStart(0),
                        End,

                        Str("b"),
                        Uint(5),

                        Str("c"),
                        MapStart(1),
                            Str("abc"),

                            Option(true),
                            Char('c'),
                        End,
                    End,
                End,
            End,
        );

        let mut serializer = AssertSerializer::new(tokens.move_iter());

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
            EnumStart("Animal", "Dog", 0),
            End,
        );

        let mut serializer = AssertSerializer::new(tokens.move_iter());
        Dog.serialize(&mut serializer).unwrap();
        assert_eq!(serializer.iter.next(), None);

        let tokens = vec!(
            EnumStart("Animal", "Frog", 2),
                Str("Henry"),
                Int(349),
            End,
        );

        let mut serializer = AssertSerializer::new(tokens.move_iter());
        Frog("Henry".to_string(), 349).serialize(&mut serializer).unwrap();
        assert_eq!(serializer.iter.next(), None);
    }

    #[test]
    fn test_tokens_vec_empty() {
        let tokens = vec!(
            SeqStart(0),
            End,
        );

        let mut serializer = AssertSerializer::new(tokens.move_iter());
        let v: Vec<int> = vec!();
        v.serialize(&mut serializer).unwrap();
        assert_eq!(serializer.iter.next(), None);
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

        let mut serializer = AssertSerializer::new(tokens.move_iter());
        (vec!(5, 6, 7)).serialize(&mut serializer).unwrap();
        assert_eq!(serializer.iter.next(), None);
    }

    #[test]
    fn test_tokens_vec_compound() {
        let tokens = vec!(
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
        );

        let mut serializer = AssertSerializer::new(tokens.move_iter());
        (vec!(vec!(1), vec!(2, 3), vec!(4, 5, 6))).serialize(&mut serializer).unwrap();
        assert_eq!(serializer.iter.next(), None);
    }

    #[test]
    fn test_tokens_hashmap() {
        let tokens = vec!(
            MapStart(2),
                Int(5),
                Str("a"),

                Int(6),
                Str("b"),
            End,
        );

        let mut serializer = AssertSerializer::new(tokens.move_iter());

        let mut map = HashMap::new();
        map.insert(5, "a".to_string());
        map.insert(6, "b".to_string());

        map.serialize(&mut serializer).unwrap();
        assert_eq!(serializer.iter.next(), None);
    }
}
