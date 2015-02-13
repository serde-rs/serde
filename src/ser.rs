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
use std::rc::Rc;
use std::sync::Arc;

//////////////////////////////////////////////////////////////////////////////

pub trait Serializer<E> {
    fn serialize_null(&mut self) -> Result<(), E>;

    fn serialize_bool(&mut self, v: bool) -> Result<(), E>;

    #[inline]
    fn serialize_isize(&mut self, v: isize) -> Result<(), E> {
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
    fn serialize_usize(&mut self, v: usize) -> Result<(), E> {
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

    fn serialize_tuple_start(&mut self, len: usize) -> Result<(), E>;
    fn serialize_tuple_elt<
        T: Serialize<Self, E>
    >(&mut self, v: &T) -> Result<(), E>;
    fn serialize_tuple_end(&mut self) -> Result<(), E>;

    fn serialize_struct_start(&mut self, name: &str, len: usize) -> Result<(), E>;
    fn serialize_struct_elt<
        T: Serialize<Self, E>
    >(&mut self, name: &str, v: &T) -> Result<(), E>;
    fn serialize_struct_end(&mut self) -> Result<(), E>;

    fn serialize_enum_start(&mut self, name: &str, variant: &str, len: usize) -> Result<(), E>;
    fn serialize_enum_elt<
        T: Serialize<Self, E>
    >(&mut self, v: &T) -> Result<(), E>;
    fn serialize_enum_end(&mut self) -> Result<(), E>;

    fn serialize_option<
        T: Serialize<Self, E>
    >(&mut self, v: &Option<T>) -> Result<(), E>;

    fn serialize_seq<
        T: Serialize<Self, E>,
        Iter: Iterator<Item=T>
    >(&mut self, iter: Iter) -> Result<(), E>;

    fn serialize_map<
        K: Serialize<Self, E>,
        V: Serialize<Self, E>,
        Iter: Iterator<Item=(K, V)>
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
impl_serialize!(isize, serialize_isize);
impl_serialize!(i8, serialize_i8);
impl_serialize!(i16, serialize_i16);
impl_serialize!(i32, serialize_i32);
impl_serialize!(i64, serialize_i64);
impl_serialize!(usize, serialize_usize);
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
        (&self[]).serialize(s)
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
    K: Serialize<S, E> + Eq + Hash<Hasher>,
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
> Serialize<S, E> for BTreeMap<K, V> {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        s.serialize_map(self.iter())
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<
    S: Serializer<E>,
    E,
    T: Serialize<S, E> + Eq + Hash<Hasher>
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
> Serialize<S, E> for BTreeSet<T> {
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
