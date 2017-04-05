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
#[cfg(feature = "std")]
use std::net;
#[cfg(feature = "std")]
use std::path;
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

#[cfg(feature = "std")]
use std::time::Duration;

#[cfg(feature = "std")]
use std;

#[cfg(feature = "unstable")]
use core::nonzero::{NonZero, Zeroable};

#[cfg(feature = "unstable")]
#[allow(deprecated)] // required for impl Deserialize for NonZero<T>
use core::num::Zero;

use de::{Deserialize, Deserializer, EnumVisitor, Error, MapVisitor, SeqVisitor, Unexpected,
         VariantVisitor, Visitor};
use de::from_primitive::FromPrimitive;

#[cfg(feature = "std")]
use bytes::ByteBuf;

///////////////////////////////////////////////////////////////////////////////

struct UnitVisitor;

impl<'de> Visitor<'de> for UnitVisitor {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("unit")
    }

    fn visit_unit<E>(self) -> Result<(), E>
        where E: Error
    {
        Ok(())
    }
}

impl<'de> Deserialize<'de> for () {
    fn deserialize<D>(deserializer: D) -> Result<(), D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_unit(UnitVisitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

struct BoolVisitor;

impl<'de> Visitor<'de> for BoolVisitor {
    type Value = bool;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a boolean")
    }

    fn visit_bool<E>(self, v: bool) -> Result<bool, E>
        where E: Error
    {
        Ok(v)
    }
}

impl<'de> Deserialize<'de> for bool {
    fn deserialize<D>(deserializer: D) -> Result<bool, D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_bool(BoolVisitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

macro_rules! visit_integer_method {
    ($src_ty:ident, $method:ident, $from_method:ident, $group:ident, $group_ty:ident) => {
        #[inline]
        fn $method<E>(self, v: $src_ty) -> Result<Self::Value, E>
            where E: Error
        {
            match FromPrimitive::$from_method(v) {
                Some(v) => Ok(v),
                None => Err(Error::invalid_value(Unexpected::$group(v as $group_ty), &self)),
            }
        }
    }
}

macro_rules! visit_float_method {
    ($src_ty:ident, $method:ident) => {
        #[inline]
        fn $method<E>(self, v: $src_ty) -> Result<Self::Value, E>
            where E: Error
        {
            Ok(v as Self::Value)
        }
    }
}

macro_rules! impl_deserialize_num {
    ($ty:ident, $method:ident, $($visit:ident),*) => {
        impl<'de> Deserialize<'de> for $ty {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<$ty, D::Error>
                where D: Deserializer<'de>,
            {
                struct PrimitiveVisitor;

                impl<'de> Visitor<'de> for PrimitiveVisitor {
                    type Value = $ty;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str(stringify!($ty))
                    }

                    $(
                        impl_deserialize_num!($visit $ty);
                    )*
                }

                deserializer.$method(PrimitiveVisitor)
            }
        }
    };
    (integer $ty:ident) => {
        visit_integer_method!(i8, visit_i8, from_i8, Signed, i64);
        visit_integer_method!(i16, visit_i16, from_i16, Signed, i64);
        visit_integer_method!(i32, visit_i32, from_i32, Signed, i64);
        visit_integer_method!(i64, visit_i64, from_i64, Signed, i64);
        visit_integer_method!(u8, visit_u8, from_u8, Unsigned, u64);
        visit_integer_method!(u16, visit_u16, from_u16, Unsigned, u64);
        visit_integer_method!(u32, visit_u32, from_u32, Unsigned, u64);
        visit_integer_method!(u64, visit_u64, from_u64, Unsigned, u64);
    };
    (float $ty:ident) => {
        visit_float_method!(f32, visit_f32);
        visit_float_method!(f64, visit_f64);
    };
}

impl_deserialize_num!(isize, deserialize_i64, integer);
impl_deserialize_num!(i8, deserialize_i8, integer);
impl_deserialize_num!(i16, deserialize_i16, integer);
impl_deserialize_num!(i32, deserialize_i32, integer);
impl_deserialize_num!(i64, deserialize_i64, integer);
impl_deserialize_num!(usize, deserialize_u64, integer);
impl_deserialize_num!(u8, deserialize_u8, integer);
impl_deserialize_num!(u16, deserialize_u16, integer);
impl_deserialize_num!(u32, deserialize_u32, integer);
impl_deserialize_num!(u64, deserialize_u64, integer);

impl_deserialize_num!(f32, deserialize_f32, integer, float);
impl_deserialize_num!(f64, deserialize_f64, integer, float);

///////////////////////////////////////////////////////////////////////////////

struct CharVisitor;

impl<'de> Visitor<'de> for CharVisitor {
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

impl<'de> Deserialize<'de> for char {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<char, D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_char(CharVisitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(any(feature = "std", feature = "collections"))]
struct StringVisitor;

#[cfg(any(feature = "std", feature = "collections"))]
impl<'de> Visitor<'de> for StringVisitor {
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
impl<'de> Deserialize<'de> for String {
    fn deserialize<D>(deserializer: D) -> Result<String, D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_string(StringVisitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

struct StrVisitor;

impl<'a> Visitor<'a> for StrVisitor {
    type Value = &'a str;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a borrowed string")
    }

    fn visit_borrowed_str<E>(self, v: &'a str) -> Result<Self::Value, E>
        where E: Error
    {
        Ok(v) // so easy
    }

    fn visit_borrowed_bytes<E>(self, v: &'a [u8]) -> Result<Self::Value, E>
        where E: Error
    {
        str::from_utf8(v)
            .map_err(|_| Error::invalid_value(Unexpected::Bytes(v), &self))
    }
}

impl<'de: 'a, 'a> Deserialize<'de> for &'a str {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_str(StrVisitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

struct BytesVisitor;

impl<'a> Visitor<'a> for BytesVisitor {
    type Value = &'a [u8];

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a borrowed byte array")
    }

    fn visit_borrowed_bytes<E>(self, v: &'a [u8]) -> Result<Self::Value, E>
        where E: Error
    {
        Ok(v)
    }

    fn visit_borrowed_str<E>(self, v: &'a str) -> Result<Self::Value, E>
        where E: Error
    {
        Ok(v.as_bytes())
    }
}

impl<'de: 'a, 'a> Deserialize<'de> for &'a [u8] {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_bytes(BytesVisitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(all(feature = "std", feature="unstable"))]
impl<'de> Deserialize<'de> for Box<CStr> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = try!(CString::deserialize(deserializer));
        Ok(s.into_boxed_c_str())
    }
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for CString {
    fn deserialize<D>(deserializer: D) -> Result<CString, D::Error>
        where D: Deserializer<'de>
    {
        let bytes = try!(ByteBuf::deserialize(deserializer));
        CString::new(bytes).map_err(Error::custom)
    }
}

///////////////////////////////////////////////////////////////////////////////

struct OptionVisitor<T> {
    marker: PhantomData<T>,
}

impl<'de, T> Visitor<'de> for OptionVisitor<T>
    where T: Deserialize<'de>
{
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
        where D: Deserializer<'de>
    {
        Ok(Some(try!(Deserialize::deserialize(deserializer))))
    }
}

impl<'de, T> Deserialize<'de> for Option<T>
    where T: Deserialize<'de>
{
    fn deserialize<D>(deserializer: D) -> Result<Option<T>, D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_option(OptionVisitor { marker: PhantomData })
    }
}

///////////////////////////////////////////////////////////////////////////////

struct PhantomDataVisitor<T> {
    marker: PhantomData<T>,
}

impl<'de, T> Visitor<'de> for PhantomDataVisitor<T> {
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

impl<'de, T> Deserialize<'de> for PhantomData<T> {
    fn deserialize<D>(deserializer: D) -> Result<PhantomData<T>, D::Error>
        where D: Deserializer<'de>
    {
        let visitor = PhantomDataVisitor { marker: PhantomData };
        deserializer.deserialize_unit_struct("PhantomData", visitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

macro_rules! seq_impl {
    (
        $ty:ident < T $(, $typaram:ident)* >,
        $visitor_ty:ident $( < $($boundparam:ident : $bound1:ident $(+ $bound2:ident)*),* > )*,
        $visitor:ident,
        $ctor:expr,
        $with_capacity:expr,
        $insert:expr
    ) => {
        struct $visitor_ty<T $(, $typaram)*> {
            marker: PhantomData<$ty<T $(, $typaram)*>>,
        }

        impl<T $(, $typaram),*> $visitor_ty<T $(, $typaram)*> {
            fn new() -> Self {
                $visitor_ty {
                    marker: PhantomData,
                }
            }
        }

        impl<'de, T $(, $typaram)*> Visitor<'de> for $visitor_ty<T $(, $typaram)*>
            where T: Deserialize<'de>,
                  $($($boundparam: $bound1 $(+ $bound2)*),*)*
        {
            type Value = $ty<T $(, $typaram)*>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence")
            }

            #[inline]
            fn visit_seq<V>(self, mut $visitor: V) -> Result<Self::Value, V::Error>
                where V: SeqVisitor<'de>,
            {
                let mut values = $with_capacity;

                while let Some(value) = try!($visitor.visit()) {
                    $insert(&mut values, value);
                }

                Ok(values)
            }
        }

        impl<'de, T $(, $typaram)*> Deserialize<'de> for $ty<T $(, $typaram)*>
            where T: Deserialize<'de>,
                  $($($boundparam: $bound1 $(+ $bound2)*),*)*
        {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: Deserializer<'de>,
            {
                deserializer.deserialize_seq($visitor_ty::new())
            }
        }
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
seq_impl!(
    BinaryHeap<T>,
    BinaryHeapVisitor<T: Ord>,
    visitor,
    BinaryHeap::new(),
    BinaryHeap::with_capacity(cmp::min(visitor.size_hint().0, 4096)),
    BinaryHeap::push);

#[cfg(any(feature = "std", feature = "collections"))]
seq_impl!(
    BTreeSet<T>,
    BTreeSetVisitor<T: Eq + Ord>,
    visitor,
    BTreeSet::new(),
    BTreeSet::new(),
    BTreeSet::insert);

#[cfg(any(feature = "std", feature = "collections"))]
seq_impl!(
    LinkedList<T>,
    LinkedListVisitor,
    visitor,
    LinkedList::new(),
    LinkedList::new(),
    LinkedList::push_back);

#[cfg(feature = "std")]
seq_impl!(
    HashSet<T, S>,
    HashSetVisitor<T: Eq + Hash,
                   S: BuildHasher + Default>,
    visitor,
    HashSet::with_hasher(S::default()),
    HashSet::with_capacity_and_hasher(cmp::min(visitor.size_hint().0, 4096), S::default()),
    HashSet::insert);

#[cfg(any(feature = "std", feature = "collections"))]
seq_impl!(
    Vec<T>,
    VecVisitor,
    visitor,
    Vec::new(),
    Vec::with_capacity(cmp::min(visitor.size_hint().0, 4096)),
    Vec::push);

#[cfg(any(feature = "std", feature = "collections"))]
seq_impl!(
    VecDeque<T>,
    VecDequeVisitor,
    visitor,
    VecDeque::new(),
    VecDeque::with_capacity(cmp::min(visitor.size_hint().0, 4096)),
    VecDeque::push_back);

///////////////////////////////////////////////////////////////////////////////

struct ArrayVisitor<A> {
    marker: PhantomData<A>,
}

impl<A> ArrayVisitor<A> {
    fn new() -> Self {
        ArrayVisitor { marker: PhantomData }
    }
}

impl<'de, T> Visitor<'de> for ArrayVisitor<[T; 0]>
    where T: Deserialize<'de>
{
    type Value = [T; 0];

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an empty array")
    }

    #[inline]
    fn visit_seq<V>(self, _: V) -> Result<[T; 0], V::Error>
        where V: SeqVisitor<'de>
    {
        Ok([])
    }
}

impl<'de, T> Deserialize<'de> for [T; 0]
    where T: Deserialize<'de>
{
    fn deserialize<D>(deserializer: D) -> Result<[T; 0], D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_seq_fixed_size(0, ArrayVisitor::<[T; 0]>::new())
    }
}

macro_rules! array_impls {
    ($($len:expr => ($($n:tt $name:ident)+))+) => {
        $(
            impl<'de, T> Visitor<'de> for ArrayVisitor<[T; $len]>
                where T: Deserialize<'de>
            {
                type Value = [T; $len];

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str(concat!("an array of length ", $len))
                }

                #[inline]
                fn visit_seq<V>(self, mut visitor: V) -> Result<[T; $len], V::Error>
                    where V: SeqVisitor<'de>,
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

            impl<'de, T> Deserialize<'de> for [T; $len]
                where T: Deserialize<'de>,
            {
                fn deserialize<D>(deserializer: D) -> Result<[T; $len], D::Error>
                    where D: Deserializer<'de>,
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
            struct $visitor<$($name,)+> {
                marker: PhantomData<($($name,)+)>,
            }

            impl<$($name,)+> $visitor<$($name,)+> {
                fn new() -> Self {
                    $visitor { marker: PhantomData }
                }
            }

            impl<'de, $($name: Deserialize<'de>),+> Visitor<'de> for $visitor<$($name,)+> {
                type Value = ($($name,)+);

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str(concat!("a tuple of size ", $len))
                }

                #[inline]
                #[allow(non_snake_case)]
                fn visit_seq<V>(self, mut visitor: V) -> Result<($($name,)+), V::Error>
                    where V: SeqVisitor<'de>,
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

            impl<'de, $($name: Deserialize<'de>),+> Deserialize<'de> for ($($name,)+) {
                #[inline]
                fn deserialize<D>(deserializer: D) -> Result<($($name,)+), D::Error>
                    where D: Deserializer<'de>,
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
        $ty:ident < K, V $(, $typaram:ident)* >,
        $visitor_ty:ident < $($boundparam:ident : $bound1:ident $(+ $bound2:ident)*),* >,
        $visitor:ident,
        $ctor:expr,
        $with_capacity:expr
    ) => {
        struct $visitor_ty<K, V $(, $typaram)*> {
            marker: PhantomData<$ty<K, V $(, $typaram)*>>,
        }

        impl<K, V $(, $typaram)*> $visitor_ty<K, V $(, $typaram)*> {
            fn new() -> Self {
                $visitor_ty {
                    marker: PhantomData,
                }
            }
        }

        impl<'de, K, V $(, $typaram)*> Visitor<'de> for $visitor_ty<K, V $(, $typaram)*>
            where K: Deserialize<'de>,
                  V: Deserialize<'de>,
                  $($boundparam: $bound1 $(+ $bound2)*),*
        {
            type Value = $ty<K, V $(, $typaram)*>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map")
            }

            #[inline]
            fn visit_map<Visitor>(self, mut $visitor: Visitor) -> Result<Self::Value, Visitor::Error>
                where Visitor: MapVisitor<'de>,
            {
                let mut values = $with_capacity;

                while let Some((key, value)) = try!($visitor.visit()) {
                    values.insert(key, value);
                }

                Ok(values)
            }
        }

        impl<'de, K, V $(, $typaram)*> Deserialize<'de> for $ty<K, V $(, $typaram)*>
            where K: Deserialize<'de>,
                  V: Deserialize<'de>,
                  $($boundparam: $bound1 $(+ $bound2)*),*
        {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: Deserializer<'de>,
            {
                deserializer.deserialize_map($visitor_ty::new())
            }
        }
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
map_impl!(
    BTreeMap<K, V>,
    BTreeMapVisitor<K: Ord>,
    visitor,
    BTreeMap::new(),
    BTreeMap::new());

#[cfg(feature = "std")]
map_impl!(
    HashMap<K, V, S>,
    HashMapVisitor<K: Eq + Hash,
                   S: BuildHasher + Default>,
    visitor,
    HashMap::with_hasher(S::default()),
    HashMap::with_capacity_and_hasher(cmp::min(visitor.size_hint().0, 4096), S::default()));

///////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for net::IpAddr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = try!(String::deserialize(deserializer));
        match s.parse() {
            Ok(s) => Ok(s),
            Err(err) => Err(D::Error::custom(err)),
        }
    }
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for net::Ipv4Addr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = try!(String::deserialize(deserializer));
        match s.parse() {
            Ok(s) => Ok(s),
            Err(err) => Err(D::Error::custom(err)),
        }
    }
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for net::Ipv6Addr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
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
impl<'de> Deserialize<'de> for net::SocketAddr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = try!(String::deserialize(deserializer));
        match s.parse() {
            Ok(s) => Ok(s),
            Err(err) => Err(D::Error::custom(err)),
        }
    }
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for net::SocketAddrV4 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = try!(String::deserialize(deserializer));
        match s.parse() {
            Ok(s) => Ok(s),
            Err(err) => Err(D::Error::custom(err)),
        }
    }
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for net::SocketAddrV6 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
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
impl<'de> Visitor<'de> for PathBufVisitor {
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
impl<'de> Deserialize<'de> for path::PathBuf {
    fn deserialize<D>(deserializer: D) -> Result<path::PathBuf, D::Error>
        where D: Deserializer<'de>
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
impl<'de> Deserialize<'de> for OsStringKind {
    fn deserialize<D>(deserializer: D) -> Result<OsStringKind, D::Error>
        where D: Deserializer<'de>
    {
        struct KindVisitor;

        impl<'de> Visitor<'de> for KindVisitor {
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
impl<'de> Visitor<'de> for OsStringVisitor {
    type Value = OsString;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("os string")
    }

    #[cfg(unix)]
    fn visit_enum<V>(self, visitor: V) -> Result<OsString, V::Error>
        where V: EnumVisitor<'de>,
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
        where V: EnumVisitor<'de>,
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
impl<'de> Deserialize<'de> for OsString {
    fn deserialize<D>(deserializer: D) -> Result<OsString, D::Error>
        where D: Deserializer<'de>
    {
        deserializer.deserialize_enum("OsString", OSSTR_VARIANTS, OsStringVisitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Box<T> {
    fn deserialize<D>(deserializer: D) -> Result<Box<T>, D::Error>
        where D: Deserializer<'de>
    {
        let val = try!(Deserialize::deserialize(deserializer));
        Ok(Box::new(val))
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Box<[T]> {
    fn deserialize<D>(deserializer: D) -> Result<Box<[T]>, D::Error>
        where D: Deserializer<'de>
    {
        let v: Vec<T> = try!(Deserialize::deserialize(deserializer));
        Ok(v.into_boxed_slice())
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
impl<'de> Deserialize<'de> for Box<str> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = try!(String::deserialize(deserializer));
        Ok(s.into_boxed_str())
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Arc<T> {
    fn deserialize<D>(deserializer: D) -> Result<Arc<T>, D::Error>
        where D: Deserializer<'de>
    {
        let val = try!(Deserialize::deserialize(deserializer));
        Ok(Arc::new(val))
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Rc<T> {
    fn deserialize<D>(deserializer: D) -> Result<Rc<T>, D::Error>
        where D: Deserializer<'de>
    {
        let val = try!(Deserialize::deserialize(deserializer));
        Ok(Rc::new(val))
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
impl<'de, 'a, T: ?Sized> Deserialize<'de> for Cow<'a, T>
    where T: ToOwned,
          T::Owned: Deserialize<'de>
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Cow<'a, T>, D::Error>
        where D: Deserializer<'de>
    {
        let val = try!(Deserialize::deserialize(deserializer));
        Ok(Cow::Owned(val))
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
impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        enum Field {
            Secs,
            Nanos,
        };

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
                where D: Deserializer<'de>
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
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

        impl<'de> Visitor<'de> for DurationVisitor {
            type Value = Duration;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Duration")
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<Duration, V::Error>
                where V: SeqVisitor<'de>
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
                where V: MapVisitor<'de>
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
impl<'de, Idx: Deserialize<'de>> Deserialize<'de> for std::ops::Range<Idx> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        enum Field {
            Start,
            End,
        };

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
                where D: Deserializer<'de>
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
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

        impl<'de, Idx: Deserialize<'de>> Visitor<'de> for RangeVisitor<Idx> {
            type Value = std::ops::Range<Idx>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Range")
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<std::ops::Range<Idx>, V::Error>
                where V: SeqVisitor<'de>
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
                where V: MapVisitor<'de>
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
#[allow(deprecated)] // num::Zero is deprecated but there is no replacement
impl<'de, T> Deserialize<'de> for NonZero<T>
    where T: Deserialize<'de> + PartialEq + Zeroable + Zero
{
    fn deserialize<D>(deserializer: D) -> Result<NonZero<T>, D::Error>
        where D: Deserializer<'de>
    {
        let value = try!(Deserialize::deserialize(deserializer));
        if value == Zero::zero() {
            return Err(Error::custom("expected a non-zero value"));
        }
        unsafe { Ok(NonZero::new(value)) }
    }
}

///////////////////////////////////////////////////////////////////////////////


impl<'de, T, E> Deserialize<'de> for Result<T, E>
    where T: Deserialize<'de>,
          E: Deserialize<'de>
{
    fn deserialize<D>(deserializer: D) -> Result<Result<T, E>, D::Error>
        where D: Deserializer<'de>
    {
        enum Field {
            Ok,
            Err,
        }

        impl<'de> Deserialize<'de> for Field {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
                where D: Deserializer<'de>
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
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

        impl<'de, T, E> Visitor<'de> for ResultVisitor<T, E>
            where T: Deserialize<'de>,
                  E: Deserialize<'de>
        {
            type Value = Result<T, E>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("enum Result")
            }

            fn visit_enum<V>(self, visitor: V) -> Result<Result<T, E>, V::Error>
                where V: EnumVisitor<'de>
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
