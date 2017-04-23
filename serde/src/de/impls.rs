//! This module contains `Deserialize` and `Visitor` implementations.

#[cfg(feature = "std")]
use std::borrow::Cow;
#[cfg(all(feature = "collections", not(feature = "std")))]
use collections::borrow::Cow;

#[cfg(all(feature = "collections", not(feature = "std")))]
use collections::{BinaryHeap, BTreeMap, BTreeSet, LinkedList, VecDeque, Vec, String};

#[cfg(feature = "std")]
use std::collections::{HashMap, HashSet, BinaryHeap, BTreeMap, BTreeSet, LinkedList, VecDeque};

#[cfg(feature = "collections")]
use collections::borrow::ToOwned;

#[cfg(any(feature = "std", feature = "collections"))]
use core::cmp;
use core::fmt;
#[cfg(feature = "std")]
use core::hash::{Hash, BuildHasher};
use core::marker::PhantomData;
#[cfg(all(feature="unstable"))]
use core::mem;
#[cfg(feature = "std")]
use std::net;
#[cfg(feature = "std")]
use std::path;
#[cfg(all(feature="unstable"))]
use core::slice;
use core::str;
#[cfg(feature = "std")]
use std::ffi::{CString, OsString};
#[cfg(all(feature = "std", feature="unstable"))]
use std::ffi::CStr;

#[cfg(feature = "std")]
use std::rc::Rc;
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::rc::Rc;

#[cfg(feature = "std")]
use std::sync::Arc;
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::arc::Arc;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::boxed::Box;

use core::cell::{Cell, RefCell};

#[cfg(feature = "std")]
use std::sync::{Mutex, RwLock};

#[cfg(feature = "std")]
use std::time::Duration;

#[cfg(feature = "std")]
use std;

#[cfg(feature = "unstable")]
use core::nonzero::{NonZero, Zeroable};

use de::{Deserialize, Deserializer, EnumVisitor, Error, MapVisitor, SeqVisitor, Unexpected,
         VariantVisitor, Visitor};
use de::from_primitive::FromPrimitive;

#[cfg(feature = "std")]
use bytes::ByteBuf;

///////////////////////////////////////////////////////////////////////////////

/// A visitor that produces a `()`.
pub struct UnitVisitor;

impl Visitor for UnitVisitor {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("unit")
    }

    fn visit_unit<E>(self) -> Result<(), E>
        where E: Error
    {
        Ok(())
    }

    fn visit_seq<V>(self, _: V) -> Result<(), V::Error>
        where V: SeqVisitor
    {
        Ok(())
    }
}

impl Deserialize for () {
    fn deserialize<D>(deserializer: D) -> Result<(), D::Error>
        where D: Deserializer
    {
        deserializer.deserialize_unit(UnitVisitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

/// A visitor that produces a `bool`.
pub struct BoolVisitor;

impl Visitor for BoolVisitor {
    type Value = bool;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a boolean")
    }

    fn visit_bool<E>(self, v: bool) -> Result<bool, E>
        where E: Error
    {
        Ok(v)
    }

    fn visit_str<E>(self, s: &str) -> Result<bool, E>
        where E: Error
    {
        match s.trim_matches(::utils::Pattern_White_Space) {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(Error::invalid_type(Unexpected::Str(s), &self)),
        }
    }
}

impl Deserialize for bool {
    fn deserialize<D>(deserializer: D) -> Result<bool, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize_bool(BoolVisitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

macro_rules! impl_deserialize_num_method {
    ($ty:ident, $src_ty:ident, $method:ident, $from_method:ident, $group:ident, $group_ty:ident) => {
        #[inline]
        fn $method<E>(self, v: $src_ty) -> Result<$ty, E>
            where E: Error,
        {
            match FromPrimitive::$from_method(v) {
                Some(v) => Ok(v),
                None => Err(Error::invalid_value(Unexpected::$group(v as $group_ty), &self)),
            }
        }
    }
}

macro_rules! impl_deserialize_num {
    ($ty:ident, $method:ident) => {
        impl Deserialize for $ty {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<$ty, D::Error>
                where D: Deserializer,
            {
                struct PrimitiveVisitor;

                impl Visitor for PrimitiveVisitor {
                    type Value = $ty;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str(stringify!($ty))
                    }

                    impl_deserialize_num_method!($ty, i8, visit_i8, from_i8, Signed, i64);
                    impl_deserialize_num_method!($ty, i16, visit_i16, from_i16, Signed, i64);
                    impl_deserialize_num_method!($ty, i32, visit_i32, from_i32, Signed, i64);
                    impl_deserialize_num_method!($ty, i64, visit_i64, from_i64, Signed, i64);
                    impl_deserialize_num_method!($ty, u8, visit_u8, from_u8, Unsigned, u64);
                    impl_deserialize_num_method!($ty, u16, visit_u16, from_u16, Unsigned, u64);
                    impl_deserialize_num_method!($ty, u32, visit_u32, from_u32, Unsigned, u64);
                    impl_deserialize_num_method!($ty, u64, visit_u64, from_u64, Unsigned, u64);
                    impl_deserialize_num_method!($ty, f32, visit_f32, from_f32, Float, f64);
                    impl_deserialize_num_method!($ty, f64, visit_f64, from_f64, Float, f64);

                    #[inline]
                    fn visit_str<E>(self, s: &str) -> Result<$ty, E>
                        where E: Error,
                    {
                        str::FromStr::from_str(s.trim_matches(::utils::Pattern_White_Space)).or_else(|_| {
                            Err(Error::invalid_type(Unexpected::Str(s), &self))
                        })
                    }
                }

                deserializer.$method(PrimitiveVisitor)
            }
        }
    }
}

impl_deserialize_num!(isize, deserialize_i64);
impl_deserialize_num!(i8, deserialize_i8);
impl_deserialize_num!(i16, deserialize_i16);
impl_deserialize_num!(i32, deserialize_i32);
impl_deserialize_num!(i64, deserialize_i64);
impl_deserialize_num!(usize, deserialize_u64);
impl_deserialize_num!(u8, deserialize_u8);
impl_deserialize_num!(u16, deserialize_u16);
impl_deserialize_num!(u32, deserialize_u32);
impl_deserialize_num!(u64, deserialize_u64);
impl_deserialize_num!(f32, deserialize_f32);
impl_deserialize_num!(f64, deserialize_f64);

///////////////////////////////////////////////////////////////////////////////

struct CharVisitor;

impl Visitor for CharVisitor {
    type Value = char;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a character")
    }

    #[inline]
    fn visit_char<E>(self, v: char) -> Result<char, E>
        where E: Error
    {
        Ok(v)
    }

    #[inline]
    fn visit_str<E>(self, v: &str) -> Result<char, E>
        where E: Error
    {
        let mut iter = v.chars();
        match (iter.next(), iter.next()) {
            (Some(c), None) => Ok(c),
            _ => Err(Error::invalid_value(Unexpected::Str(v), &self)),
        }
    }
}

impl Deserialize for char {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<char, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize_char(CharVisitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(any(feature = "std", feature = "collections"))]
struct StringVisitor;

#[cfg(any(feature = "std", feature = "collections"))]
impl Visitor for StringVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<String, E>
        where E: Error
    {
        Ok(v.to_owned())
    }

    fn visit_string<E>(self, v: String) -> Result<String, E>
        where E: Error
    {
        Ok(v)
    }

    fn visit_unit<E>(self) -> Result<String, E>
        where E: Error
    {
        Ok(String::new())
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<String, E>
        where E: Error
    {
        match str::from_utf8(v) {
            Ok(s) => Ok(s.to_owned()),
            Err(_) => Err(Error::invalid_value(Unexpected::Bytes(v), &self)),
        }
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<String, E>
        where E: Error
    {
        match String::from_utf8(v) {
            Ok(s) => Ok(s),
            Err(e) => Err(Error::invalid_value(Unexpected::Bytes(&e.into_bytes()), &self)),
        }
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
impl Deserialize for String {
    fn deserialize<D>(deserializer: D) -> Result<String, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize_string(StringVisitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(all(feature = "std", feature="unstable"))]
impl Deserialize for Box<CStr> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        let s = try!(CString::deserialize(deserializer));
        Ok(s.into_boxed_c_str())
    }
}

#[cfg(feature = "std")]
impl Deserialize for CString {
    fn deserialize<D>(deserializer: D) -> Result<CString, D::Error>
        where D: Deserializer
    {
        let bytes = try!(ByteBuf::deserialize(deserializer));
        CString::new(bytes).map_err(Error::custom)
    }
}

///////////////////////////////////////////////////////////////////////////////

struct OptionVisitor<T> {
    marker: PhantomData<T>,
}

impl<T: Deserialize> Visitor for OptionVisitor<T> {
    type Value = Option<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("option")
    }

    #[inline]
    fn visit_unit<E>(self) -> Result<Option<T>, E>
        where E: Error
    {
        Ok(None)
    }

    #[inline]
    fn visit_none<E>(self) -> Result<Option<T>, E>
        where E: Error
    {
        Ok(None)
    }

    #[inline]
    fn visit_some<D>(self, deserializer: D) -> Result<Option<T>, D::Error>
        where D: Deserializer
    {
        Ok(Some(try!(Deserialize::deserialize(deserializer))))
    }
}

impl<T> Deserialize for Option<T>
    where T: Deserialize
{
    fn deserialize<D>(deserializer: D) -> Result<Option<T>, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize_option(OptionVisitor { marker: PhantomData })
    }
}

///////////////////////////////////////////////////////////////////////////////

/// A visitor that produces a `PhantomData`.
pub struct PhantomDataVisitor<T> {
    marker: PhantomData<T>,
}

impl<T> Visitor for PhantomDataVisitor<T> {
    type Value = PhantomData<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("unit")
    }

    #[inline]
    fn visit_unit<E>(self) -> Result<PhantomData<T>, E>
        where E: Error
    {
        Ok(PhantomData)
    }
}

impl<T> Deserialize for PhantomData<T> {
    fn deserialize<D>(deserializer: D) -> Result<PhantomData<T>, D::Error>
        where D: Deserializer
    {
        let visitor = PhantomDataVisitor { marker: PhantomData };
        deserializer.deserialize_unit_struct("PhantomData", visitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

macro_rules! seq_impl {
    (
        $ty:ty,
        $visitor_ty:ident < $($typaram:ident : $bound1:ident $(+ $bound2:ident)*),* >,
        $visitor:ident,
        $ctor:expr,
        $with_capacity:expr,
        $insert:expr
    ) => {
        /// A visitor that produces a sequence.
        pub struct $visitor_ty<$($typaram),*> {
            marker: PhantomData<$ty>,
        }

        impl<$($typaram),*> $visitor_ty<$($typaram),*>
            where $($typaram: $bound1 $(+ $bound2)*),*
        {
            /// Construct a new sequence visitor.
            pub fn new() -> Self {
                $visitor_ty {
                    marker: PhantomData,
                }
            }
        }

        impl<$($typaram),*> Visitor for $visitor_ty<$($typaram),*>
            where $($typaram: $bound1 $(+ $bound2)*),*
        {
            type Value = $ty;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence")
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<$ty, E>
                where E: Error,
            {
                Ok($ctor)
            }

            #[inline]
            fn visit_seq<V>(self, mut $visitor: V) -> Result<$ty, V::Error>
                where V: SeqVisitor,
            {
                let mut values = $with_capacity;

                while let Some(value) = try!($visitor.visit()) {
                    $insert(&mut values, value);
                }

                Ok(values)
            }
        }

        impl<$($typaram),*> Deserialize for $ty
            where $($typaram: $bound1 $(+ $bound2)*),*
        {
            fn deserialize<D>(deserializer: D) -> Result<$ty, D::Error>
                where D: Deserializer,
            {
                deserializer.deserialize_seq($visitor_ty::new())
            }
        }
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
seq_impl!(
    BinaryHeap<T>,
    BinaryHeapVisitor<T: Deserialize + Ord>,
    visitor,
    BinaryHeap::new(),
    BinaryHeap::with_capacity(cmp::min(visitor.size_hint().0, 4096)),
    BinaryHeap::push);

#[cfg(any(feature = "std", feature = "collections"))]
seq_impl!(
    BTreeSet<T>,
    BTreeSetVisitor<T: Deserialize + Eq + Ord>,
    visitor,
    BTreeSet::new(),
    BTreeSet::new(),
    BTreeSet::insert);

#[cfg(any(feature = "std", feature = "collections"))]
seq_impl!(
    LinkedList<T>,
    LinkedListVisitor<T: Deserialize>,
    visitor,
    LinkedList::new(),
    LinkedList::new(),
    LinkedList::push_back);

#[cfg(feature = "std")]
seq_impl!(
    HashSet<T, S>,
    HashSetVisitor<T: Deserialize + Eq + Hash,
                   S: BuildHasher + Default>,
    visitor,
    HashSet::with_hasher(S::default()),
    HashSet::with_capacity_and_hasher(cmp::min(visitor.size_hint().0, 4096), S::default()),
    HashSet::insert);

#[cfg(any(feature = "std", feature = "collections"))]
seq_impl!(
    Vec<T>,
    VecVisitor<T: Deserialize>,
    visitor,
    Vec::new(),
    Vec::with_capacity(cmp::min(visitor.size_hint().0, 4096)),
    Vec::push);

#[cfg(any(feature = "std", feature = "collections"))]
seq_impl!(
    VecDeque<T>,
    VecDequeVisitor<T: Deserialize>,
    visitor,
    VecDeque::new(),
    VecDeque::with_capacity(cmp::min(visitor.size_hint().0, 4096)),
    VecDeque::push_back);

///////////////////////////////////////////////////////////////////////////////

struct ArrayVisitor<A> {
    marker: PhantomData<A>,
}

impl<A> ArrayVisitor<A> {
    pub fn new() -> Self {
        ArrayVisitor { marker: PhantomData }
    }
}

impl<T> Visitor for ArrayVisitor<[T; 0]>
    where T: Deserialize
{
    type Value = [T; 0];

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an empty array")
    }

    #[inline]
    fn visit_unit<E>(self) -> Result<[T; 0], E>
        where E: Error
    {
        Ok([])
    }

    #[inline]
    fn visit_seq<V>(self, _: V) -> Result<[T; 0], V::Error>
        where V: SeqVisitor
    {
        Ok([])
    }
}

impl<T> Deserialize for [T; 0]
    where T: Deserialize
{
    fn deserialize<D>(deserializer: D) -> Result<[T; 0], D::Error>
        where D: Deserializer
    {
        deserializer.deserialize_seq_fixed_size(0, ArrayVisitor::<[T; 0]>::new())
    }
}

macro_rules! array_impls {
    ($($len:expr => ($($n:tt $name:ident)+))+) => {
        $(
            impl<T> Visitor for ArrayVisitor<[T; $len]> where T: Deserialize {
                type Value = [T; $len];

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str(concat!("an array of length ", $len))
                }

                #[inline]
                fn visit_seq<V>(self, mut visitor: V) -> Result<[T; $len], V::Error>
                    where V: SeqVisitor,
                {
                    $(
                        let $name = match try!(visitor.visit()) {
                            Some(val) => val,
                            None => return Err(Error::invalid_length($n, &self)),
                        };
                    )+

                    Ok([$($name),+])
                }
            }

            impl<T> Deserialize for [T; $len]
                where T: Deserialize,
            {
                fn deserialize<D>(deserializer: D) -> Result<[T; $len], D::Error>
                    where D: Deserializer,
                {
                    deserializer.deserialize_seq_fixed_size($len, ArrayVisitor::<[T; $len]>::new())
                }
            }
        )+
    }
}

array_impls! {
    1 => (0 a)
    2 => (0 a 1 b)
    3 => (0 a 1 b 2 c)
    4 => (0 a 1 b 2 c 3 d)
    5 => (0 a 1 b 2 c 3 d 4 e)
    6 => (0 a 1 b 2 c 3 d 4 e 5 f)
    7 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g)
    8 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h)
    9 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i)
    10 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j)
    11 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k)
    12 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l)
    13 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m)
    14 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n)
    15 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o)
    16 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p)
    17 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q)
    18 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r)
    19 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r 18 s)
    20 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r 18 s 19 t)
    21 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r 18 s 19 t 20 u)
    22 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r 18 s 19 t 20 u 21 v)
    23 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r 18 s 19 t 20 u 21 v 22 w)
    24 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r 18 s 19 t 20 u 21 v 22 w 23 x)
    25 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r 18 s 19 t 20 u 21 v 22 w 23 x 24 y)
    26 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r 18 s 19 t 20 u 21 v 22 w 23 x 24 y 25 z)
    27 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r 18 s 19 t 20 u 21 v 22 w 23 x 24 y 25 z 26 aa)
    28 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r 18 s 19 t 20 u 21 v 22 w 23 x 24 y 25 z 26 aa 27 ab)
    29 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r 18 s 19 t 20 u 21 v 22 w 23 x 24 y 25 z 26 aa 27 ab 28 ac)
    30 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r 18 s 19 t 20 u 21 v 22 w 23 x 24 y 25 z 26 aa 27 ab 28 ac 29 ad)
    31 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r 18 s 19 t 20 u 21 v 22 w 23 x 24 y 25 z 26 aa 27 ab 28 ac 29 ad 30 ae)
    32 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r 18 s 19 t 20 u 21 v 22 w 23 x 24 y 25 z 26 aa 27 ab 28 ac 29 ad 30 ae 31 af)
}

///////////////////////////////////////////////////////////////////////////////

macro_rules! tuple_impls {
    ($($len:expr => $visitor:ident => ($($n:tt $name:ident)+))+) => {
        $(
            /// Construct a tuple visitor.
            pub struct $visitor<$($name,)+> {
                marker: PhantomData<($($name,)+)>,
            }

            impl<$($name: Deserialize,)+> $visitor<$($name,)+> {
                /// Construct a `TupleVisitor*<T>`.
                pub fn new() -> Self {
                    $visitor { marker: PhantomData }
                }
            }

            impl<$($name: Deserialize),+> Visitor for $visitor<$($name,)+> {
                type Value = ($($name,)+);

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str(concat!("a tuple of size ", $len))
                }

                #[inline]
                #[allow(non_snake_case)]
                fn visit_seq<V>(self, mut visitor: V) -> Result<($($name,)+), V::Error>
                    where V: SeqVisitor,
                {
                    $(
                        let $name = match try!(visitor.visit()) {
                            Some(value) => value,
                            None => return Err(Error::invalid_length($n, &self)),
                        };
                    )+

                    Ok(($($name,)+))
                }
            }

            impl<$($name: Deserialize),+> Deserialize for ($($name,)+) {
                #[inline]
                fn deserialize<D>(deserializer: D) -> Result<($($name,)+), D::Error>
                    where D: Deserializer,
                {
                    deserializer.deserialize_tuple($len, $visitor::new())
                }
            }
        )+
    }
}

tuple_impls! {
    1 => TupleVisitor1 => (0 T0)
    2 => TupleVisitor2 => (0 T0 1 T1)
    3 => TupleVisitor3 => (0 T0 1 T1 2 T2)
    4 => TupleVisitor4 => (0 T0 1 T1 2 T2 3 T3)
    5 => TupleVisitor5 => (0 T0 1 T1 2 T2 3 T3 4 T4)
    6 => TupleVisitor6 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5)
    7 => TupleVisitor7 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6)
    8 => TupleVisitor8 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7)
    9 => TupleVisitor9 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8)
    10 => TupleVisitor10 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9)
    11 => TupleVisitor11 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10)
    12 => TupleVisitor12 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11)
    13 => TupleVisitor13 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12)
    14 => TupleVisitor14 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13)
    15 => TupleVisitor15 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14)
    16 => TupleVisitor16 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15)
}

///////////////////////////////////////////////////////////////////////////////

macro_rules! map_impl {
    (
        $ty:ty,
        $visitor_ty:ident < $($typaram:ident : $bound1:ident $(+ $bound2:ident)*),* >,
        $visitor:ident,
        $ctor:expr,
        $with_capacity:expr
    ) => {
        /// A visitor that produces a map.
        pub struct $visitor_ty<$($typaram),*> {
            marker: PhantomData<$ty>,
        }

        impl<$($typaram),*> $visitor_ty<$($typaram),*>
            where $($typaram: $bound1 $(+ $bound2)*),*
        {
            /// Construct a `MapVisitor*<T>`.
            pub fn new() -> Self {
                $visitor_ty {
                    marker: PhantomData,
                }
            }
        }

        impl<$($typaram),*> Visitor for $visitor_ty<$($typaram),*>
            where $($typaram: $bound1 $(+ $bound2)*),*
        {
            type Value = $ty;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map")
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<$ty, E>
                where E: Error,
            {
                Ok($ctor)
            }

            #[inline]
            fn visit_map<Visitor>(self, mut $visitor: Visitor) -> Result<$ty, Visitor::Error>
                where Visitor: MapVisitor,
            {
                let mut values = $with_capacity;

                while let Some((key, value)) = try!($visitor.visit()) {
                    values.insert(key, value);
                }

                Ok(values)
            }
        }

        impl<$($typaram),*> Deserialize for $ty
            where $($typaram: $bound1 $(+ $bound2)*),*
        {
            fn deserialize<D>(deserializer: D) -> Result<$ty, D::Error>
                where D: Deserializer,
            {
                deserializer.deserialize_map($visitor_ty::new())
            }
        }
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
map_impl!(
    BTreeMap<K, V>,
    BTreeMapVisitor<K: Deserialize + Ord,
                    V: Deserialize>,
    visitor,
    BTreeMap::new(),
    BTreeMap::new());

#[cfg(feature = "std")]
map_impl!(
    HashMap<K, V, S>,
    HashMapVisitor<K: Deserialize + Eq + Hash,
                   V: Deserialize,
                   S: BuildHasher + Default>,
    visitor,
    HashMap::with_hasher(S::default()),
    HashMap::with_capacity_and_hasher(cmp::min(visitor.size_hint().0, 4096), S::default()));

///////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "std")]
impl Deserialize for net::IpAddr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        let s = try!(String::deserialize(deserializer));
        match s.parse() {
            Ok(s) => Ok(s),
            Err(err) => Err(D::Error::custom(err)),
        }
    }
}

#[cfg(feature = "std")]
impl Deserialize for net::Ipv4Addr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        let s = try!(String::deserialize(deserializer));
        match s.parse() {
            Ok(s) => Ok(s),
            Err(err) => Err(D::Error::custom(err)),
        }
    }
}

#[cfg(feature = "std")]
impl Deserialize for net::Ipv6Addr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        let s = try!(String::deserialize(deserializer));
        match s.parse() {
            Ok(s) => Ok(s),
            Err(err) => Err(D::Error::custom(err)),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "std")]
impl Deserialize for net::SocketAddr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        let s = try!(String::deserialize(deserializer));
        match s.parse() {
            Ok(s) => Ok(s),
            Err(err) => Err(D::Error::custom(err)),
        }
    }
}

#[cfg(feature = "std")]
impl Deserialize for net::SocketAddrV4 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        let s = try!(String::deserialize(deserializer));
        match s.parse() {
            Ok(s) => Ok(s),
            Err(err) => Err(D::Error::custom(err)),
        }
    }
}

#[cfg(feature = "std")]
impl Deserialize for net::SocketAddrV6 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        let s = try!(String::deserialize(deserializer));
        match s.parse() {
            Ok(s) => Ok(s),
            Err(err) => Err(D::Error::custom(err)),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "std")]
struct PathBufVisitor;

#[cfg(feature = "std")]
impl Visitor for PathBufVisitor {
    type Value = path::PathBuf;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("path string")
    }

    fn visit_str<E>(self, v: &str) -> Result<path::PathBuf, E>
        where E: Error
    {
        Ok(From::from(v))
    }

    fn visit_string<E>(self, v: String) -> Result<path::PathBuf, E>
        where E: Error
    {
        Ok(From::from(v))
    }
}


#[cfg(feature = "std")]
impl Deserialize for path::PathBuf {
    fn deserialize<D>(deserializer: D) -> Result<path::PathBuf, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize_string(PathBufVisitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(all(feature = "std", any(unix, windows)))]
enum OsStringKind {
    Unix,
    Windows,
}

#[cfg(all(feature = "std", any(unix, windows)))]
static OSSTR_VARIANTS: &'static [&'static str] = &["Unix", "Windows"];

#[cfg(all(feature = "std", any(unix, windows)))]
impl Deserialize for OsStringKind {
    fn deserialize<D>(deserializer: D) -> Result<OsStringKind, D::Error>
        where D: Deserializer
    {
        struct KindVisitor;

        impl Visitor for KindVisitor {
            type Value = OsStringKind;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("`Unix` or `Windows`")
            }

            fn visit_u32<E>(self, value: u32) -> Result<OsStringKind, E>
                where E: Error,
            {
                match value {
                    0 => Ok(OsStringKind::Unix),
                    1 => Ok(OsStringKind::Windows),
                    _ => Err(Error::invalid_value(Unexpected::Unsigned(value as u64), &self))
                }
            }

            fn visit_str<E>(self, value: &str) -> Result<OsStringKind, E>
                where E: Error,
            {
                match value {
                    "Unix" => Ok(OsStringKind::Unix),
                    "Windows" => Ok(OsStringKind::Windows),
                    _ => Err(Error::unknown_variant(value, OSSTR_VARIANTS)),
                }
            }

            fn visit_bytes<E>(self, value: &[u8]) -> Result<OsStringKind, E>
                where E: Error,
            {
                match value {
                    b"Unix" => Ok(OsStringKind::Unix),
                    b"Windows" => Ok(OsStringKind::Windows),
                    _ => {
                        match str::from_utf8(value) {
                            Ok(value) => Err(Error::unknown_variant(value, OSSTR_VARIANTS)),
                            Err(_) => {
                                Err(Error::invalid_value(Unexpected::Bytes(value), &self))
                            }
                        }
                    }
                }
            }
        }

        deserializer.deserialize(KindVisitor)
    }
}

#[cfg(all(feature = "std", any(unix, windows)))]
struct OsStringVisitor;

#[cfg(all(feature = "std", any(unix, windows)))]
impl Visitor for OsStringVisitor {
    type Value = OsString;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("os string")
    }

    #[cfg(unix)]
    fn visit_enum<V>(self, visitor: V) -> Result<OsString, V::Error>
        where V: EnumVisitor,
    {
        use std::os::unix::ffi::OsStringExt;

        match try!(visitor.visit_variant()) {
            (OsStringKind::Unix, variant) => {
                variant.visit_newtype().map(OsString::from_vec)
            }
            (OsStringKind::Windows, _) => {
                Err(Error::custom("cannot deserialize Windows OS string on Unix"))
            }
        }
    }

    #[cfg(windows)]
    fn visit_enum<V>(self, visitor: V) -> Result<OsString, V::Error>
        where V: EnumVisitor,
    {
        use std::os::windows::ffi::OsStringExt;

        match try!(visitor.visit_variant()) {
            (OsStringKind::Windows, variant) => {
                variant.visit_newtype::<Vec<u16>>().map(|vec| OsString::from_wide(&vec))
            }
            (OsStringKind::Unix, _) => {
                Err(Error::custom("cannot deserialize Unix OS string on Windows"))
            }
        }
    }
}

#[cfg(all(feature = "std", any(unix, windows)))]
impl Deserialize for OsString {
    fn deserialize<D>(deserializer: D) -> Result<OsString, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize_enum("OsString", OSSTR_VARIANTS, OsStringVisitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T: Deserialize> Deserialize for Box<T> {
    fn deserialize<D>(deserializer: D) -> Result<Box<T>, D::Error>
        where D: Deserializer
    {
        let val = try!(Deserialize::deserialize(deserializer));
        Ok(Box::new(val))
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
impl<T: Deserialize> Deserialize for Box<[T]> {
    fn deserialize<D>(deserializer: D) -> Result<Box<[T]>, D::Error>
        where D: Deserializer
    {
        let v: Vec<T> = try!(Deserialize::deserialize(deserializer));
        Ok(v.into_boxed_slice())
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
impl Deserialize for Box<str> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        let s = try!(String::deserialize(deserializer));
        Ok(s.into_boxed_str())
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T: Deserialize> Deserialize for Arc<T> {
    fn deserialize<D>(deserializer: D) -> Result<Arc<T>, D::Error>
        where D: Deserializer
    {
        let val = try!(Deserialize::deserialize(deserializer));
        Ok(Arc::new(val))
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T: Deserialize> Deserialize for Rc<T> {
    fn deserialize<D>(deserializer: D) -> Result<Rc<T>, D::Error>
        where D: Deserializer
    {
        let val = try!(Deserialize::deserialize(deserializer));
        Ok(Rc::new(val))
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
impl<'a, T: ?Sized> Deserialize for Cow<'a, T>
    where T: ToOwned,
          T::Owned: Deserialize
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Cow<'a, T>, D::Error>
        where D: Deserializer
    {
        let val = try!(Deserialize::deserialize(deserializer));
        Ok(Cow::Owned(val))
    }
}

impl<T: Deserialize + Copy> Deserialize for Cell<T> {
    fn deserialize<D>(deserializer: D) -> Result<Cell<T>, D::Error>
        where D: Deserializer
    {
        let val = try!(Deserialize::deserialize(deserializer));
        Ok(Cell::new(val))
    }
}

impl<T: Deserialize> Deserialize for RefCell<T> {
    fn deserialize<D>(deserializer: D) -> Result<RefCell<T>, D::Error>
        where D: Deserializer
    {
        let val = try!(Deserialize::deserialize(deserializer));
        Ok(RefCell::new(val))
    }
}

#[cfg(feature = "std")]
impl<T: Deserialize> Deserialize for Mutex<T> {
    fn deserialize<D>(deserializer: D) -> Result<Mutex<T>, D::Error>
        where D: Deserializer
    {
        let val = try!(Deserialize::deserialize(deserializer));
        Ok(Mutex::new(val))
    }
}

#[cfg(feature = "std")]
impl<T: Deserialize> Deserialize for RwLock<T> {
    fn deserialize<D>(deserializer: D) -> Result<RwLock<T>, D::Error>
        where D: Deserializer
    {
        let val = try!(Deserialize::deserialize(deserializer));
        Ok(RwLock::new(val))
    }
}

///////////////////////////////////////////////////////////////////////////////

// This is a cleaned-up version of the impl generated by:
//
//     #[derive(Deserialize)]
//     #[serde(deny_unknown_fields)]
//     struct Duration {
//         secs: u64,
//         nanos: u32,
//     }
#[cfg(feature = "std")]
impl Deserialize for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        enum Field {
            Secs,
            Nanos,
        };

        impl Deserialize for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
                where D: Deserializer
            {
                struct FieldVisitor;

                impl Visitor for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`secs` or `nanos`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                        where E: Error
                    {
                        match value {
                            "secs" => Ok(Field::Secs),
                            "nanos" => Ok(Field::Nanos),
                            _ => Err(Error::unknown_field(value, FIELDS)),
                        }
                    }

                    fn visit_bytes<E>(self, value: &[u8]) -> Result<Field, E>
                        where E: Error
                    {
                        match value {
                            b"secs" => Ok(Field::Secs),
                            b"nanos" => Ok(Field::Nanos),
                            _ => {
                                let value = String::from_utf8_lossy(value);
                                Err(Error::unknown_field(&value, FIELDS))
                            }
                        }
                    }
                }

                deserializer.deserialize_struct_field(FieldVisitor)
            }
        }

        struct DurationVisitor;

        impl Visitor for DurationVisitor {
            type Value = Duration;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Duration")
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<Duration, V::Error>
                where V: SeqVisitor
            {
                let secs: u64 = match try!(visitor.visit()) {
                    Some(value) => value,
                    None => {
                        return Err(Error::invalid_length(0, &self));
                    }
                };
                let nanos: u32 = match try!(visitor.visit()) {
                    Some(value) => value,
                    None => {
                        return Err(Error::invalid_length(1, &self));
                    }
                };
                Ok(Duration::new(secs, nanos))
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Duration, V::Error>
                where V: MapVisitor
            {
                let mut secs: Option<u64> = None;
                let mut nanos: Option<u32> = None;
                while let Some(key) = try!(visitor.visit_key::<Field>()) {
                    match key {
                        Field::Secs => {
                            if secs.is_some() {
                                return Err(<V::Error as Error>::duplicate_field("secs"));
                            }
                            secs = Some(try!(visitor.visit_value()));
                        }
                        Field::Nanos => {
                            if nanos.is_some() {
                                return Err(<V::Error as Error>::duplicate_field("nanos"));
                            }
                            nanos = Some(try!(visitor.visit_value()));
                        }
                    }
                }
                let secs = match secs {
                    Some(secs) => secs,
                    None => return Err(<V::Error as Error>::missing_field("secs")),
                };
                let nanos = match nanos {
                    Some(nanos) => nanos,
                    None => return Err(<V::Error as Error>::missing_field("nanos")),
                };
                Ok(Duration::new(secs, nanos))
            }
        }

        const FIELDS: &'static [&'static str] = &["secs", "nanos"];
        deserializer.deserialize_struct("Duration", FIELDS, DurationVisitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

// Similar to:
//
//     #[derive(Deserialize)]
//     #[serde(deny_unknown_fields)]
//     struct Range {
//         start: u64,
//         end: u32,
//     }
#[cfg(feature = "std")]
impl<Idx: Deserialize> Deserialize for std::ops::Range<Idx> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        enum Field {
            Start,
            End,
        };

        impl Deserialize for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
                where D: Deserializer
            {
                struct FieldVisitor;

                impl Visitor for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`start` or `end`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                        where E: Error
                    {
                        match value {
                            "start" => Ok(Field::Start),
                            "end" => Ok(Field::End),
                            _ => Err(Error::unknown_field(value, FIELDS)),
                        }
                    }

                    fn visit_bytes<E>(self, value: &[u8]) -> Result<Field, E>
                        where E: Error
                    {
                        match value {
                            b"start" => Ok(Field::Start),
                            b"end" => Ok(Field::End),
                            _ => {
                                let value = String::from_utf8_lossy(value);
                                Err(Error::unknown_field(&value, FIELDS))
                            }
                        }
                    }
                }

                deserializer.deserialize_struct_field(FieldVisitor)
            }
        }

        struct RangeVisitor<Idx> {
            phantom: PhantomData<Idx>,
        }

        impl<Idx: Deserialize> Visitor for RangeVisitor<Idx> {
            type Value = std::ops::Range<Idx>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Range")
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<std::ops::Range<Idx>, V::Error>
                where V: SeqVisitor
            {
                let start: Idx = match try!(visitor.visit()) {
                    Some(value) => value,
                    None => {
                        return Err(Error::invalid_length(0, &self));
                    }
                };
                let end: Idx = match try!(visitor.visit()) {
                    Some(value) => value,
                    None => {
                        return Err(Error::invalid_length(1, &self));
                    }
                };
                Ok(start..end)
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<std::ops::Range<Idx>, V::Error>
                where V: MapVisitor
            {
                let mut start: Option<Idx> = None;
                let mut end: Option<Idx> = None;
                while let Some(key) = try!(visitor.visit_key::<Field>()) {
                    match key {
                        Field::Start => {
                            if start.is_some() {
                                return Err(<V::Error as Error>::duplicate_field("start"));
                            }
                            start = Some(try!(visitor.visit_value()));
                        }
                        Field::End => {
                            if end.is_some() {
                                return Err(<V::Error as Error>::duplicate_field("end"));
                            }
                            end = Some(try!(visitor.visit_value()));
                        }
                    }
                }
                let start = match start {
                    Some(start) => start,
                    None => return Err(<V::Error as Error>::missing_field("start")),
                };
                let end = match end {
                    Some(end) => end,
                    None => return Err(<V::Error as Error>::missing_field("end")),
                };
                Ok(start..end)
            }
        }

        const FIELDS: &'static [&'static str] = &["start", "end"];
        deserializer.deserialize_struct("Range", FIELDS, RangeVisitor { phantom: PhantomData })
    }
}

///////////////////////////////////////////////////////////////////////////////


///////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "unstable")]
impl<T> Deserialize for NonZero<T>
    where T: Deserialize + Zeroable
{
    fn deserialize<D>(deserializer: D) -> Result<NonZero<T>, D::Error>
        where D: Deserializer
    {
        let value = try!(Deserialize::deserialize(deserializer));
        unsafe {
            let ptr = &value as *const T as *const u8;
            if slice::from_raw_parts(ptr, mem::size_of::<T>()).iter().all(|&b| b == 0) {
                return Err(Error::custom("expected a non-zero value"));
            }
            // Waiting for a safe way to construct NonZero<T>:
            // https://github.com/rust-lang/rust/issues/27730#issuecomment-269726075
            Ok(NonZero::new(value))
         }

    }
}

///////////////////////////////////////////////////////////////////////////////


impl<T, E> Deserialize for Result<T, E>
    where T: Deserialize,
          E: Deserialize
{
    fn deserialize<D>(deserializer: D) -> Result<Result<T, E>, D::Error>
        where D: Deserializer
    {
        enum Field {
            Ok,
            Err,
        }

        impl Deserialize for Field {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
                where D: Deserializer
            {
                struct FieldVisitor;

                impl Visitor for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`Ok` or `Err`")
                    }

                    fn visit_u32<E>(self, value: u32) -> Result<Field, E>
                        where E: Error
                    {
                        match value {
                            0 => Ok(Field::Ok),
                            1 => Ok(Field::Err),
                            _ => {
                                Err(Error::invalid_value(Unexpected::Unsigned(value as u64), &self))
                            }
                        }
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                        where E: Error
                    {
                        match value {
                            "Ok" => Ok(Field::Ok),
                            "Err" => Ok(Field::Err),
                            _ => Err(Error::unknown_variant(value, VARIANTS)),
                        }
                    }

                    fn visit_bytes<E>(self, value: &[u8]) -> Result<Field, E>
                        where E: Error
                    {
                        match value {
                            b"Ok" => Ok(Field::Ok),
                            b"Err" => Ok(Field::Err),
                            _ => {
                                match str::from_utf8(value) {
                                    Ok(value) => Err(Error::unknown_variant(value, VARIANTS)),
                                    Err(_) => {
                                        Err(Error::invalid_value(Unexpected::Bytes(value), &self))
                                    }
                                }
                            }
                        }
                    }
                }

                deserializer.deserialize(FieldVisitor)
            }
        }

        struct ResultVisitor<T, E>(PhantomData<Result<T, E>>);

        impl<T, E> Visitor for ResultVisitor<T, E>
            where T: Deserialize,
                  E: Deserialize
        {
            type Value = Result<T, E>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("enum Result")
            }

            fn visit_enum<V>(self, visitor: V) -> Result<Result<T, E>, V::Error>
                where V: EnumVisitor
            {
                match try!(visitor.visit_variant()) {
                    (Field::Ok, variant) => variant.visit_newtype().map(Ok),
                    (Field::Err, variant) => variant.visit_newtype().map(Err),
                }
            }
        }

        const VARIANTS: &'static [&'static str] = &["Ok", "Err"];

        deserializer.deserialize_enum("Result", VARIANTS, ResultVisitor(PhantomData))
    }
}

///////////////////////////////////////////////////////////////////////////////

/// A target for deserializers that want to ignore data. Implements
/// Deserialize and silently eats data given to it.
pub struct IgnoredAny;

impl Deserialize for IgnoredAny {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<IgnoredAny, D::Error>
        where D: Deserializer
    {
        struct IgnoredAnyVisitor;

        impl Visitor for IgnoredAnyVisitor {
            type Value = IgnoredAny;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("anything at all")
            }

            #[inline]
            fn visit_bool<E>(self, _: bool) -> Result<IgnoredAny, E> {
                Ok(IgnoredAny)
            }

            #[inline]
            fn visit_i64<E>(self, _: i64) -> Result<IgnoredAny, E> {
                Ok(IgnoredAny)
            }

            #[inline]
            fn visit_u64<E>(self, _: u64) -> Result<IgnoredAny, E> {
                Ok(IgnoredAny)
            }

            #[inline]
            fn visit_f64<E>(self, _: f64) -> Result<IgnoredAny, E> {
                Ok(IgnoredAny)
            }

            #[inline]
            fn visit_str<E>(self, _: &str) -> Result<IgnoredAny, E>
                where E: Error
            {
                Ok(IgnoredAny)
            }

            #[inline]
            fn visit_none<E>(self) -> Result<IgnoredAny, E> {
                Ok(IgnoredAny)
            }

            #[inline]
            fn visit_some<D>(self, _: D) -> Result<IgnoredAny, D::Error>
                where D: Deserializer
            {
                Ok(IgnoredAny)
            }

            #[inline]
            fn visit_newtype_struct<D>(self, _: D) -> Result<IgnoredAny, D::Error>
                where D: Deserializer
            {
                Ok(IgnoredAny)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<IgnoredAny, E> {
                Ok(IgnoredAny)
            }

            #[inline]
            fn visit_seq<V>(self, mut visitor: V) -> Result<IgnoredAny, V::Error>
                where V: SeqVisitor
            {
                while let Some(_) = try!(visitor.visit::<IgnoredAny>()) {
                    // Gobble
                }
                Ok(IgnoredAny)
            }

            #[inline]
            fn visit_map<V>(self, mut visitor: V) -> Result<IgnoredAny, V::Error>
                where V: MapVisitor
            {
                while let Some((_, _)) = try!(visitor.visit::<IgnoredAny, IgnoredAny>()) {
                    // Gobble
                }
                Ok(IgnoredAny)
            }

            #[inline]
            fn visit_bytes<E>(self, _: &[u8]) -> Result<IgnoredAny, E>
                where E: Error
            {
                Ok(IgnoredAny)
            }
        }

        // TODO maybe not necessary with impl specialization
        deserializer.deserialize_ignored_any(IgnoredAnyVisitor)
    }
}
