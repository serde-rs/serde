#[cfg(feature = "std")]
use std::borrow::Cow;
#[cfg(all(feature = "collections", not(feature = "std")))]
use collections::borrow::Cow;

#[cfg(feature = "std")]
use std::collections::{BinaryHeap, BTreeMap, BTreeSet, LinkedList, HashMap, HashSet, VecDeque};
#[cfg(all(feature = "collections", not(feature = "std")))]
use collections::{BinaryHeap, BTreeMap, BTreeSet, LinkedList, VecDeque, String, Vec};

#[cfg(feature = "collections")]
use collections::borrow::ToOwned;

#[cfg(feature = "std")]
use core::hash::{Hash, BuildHasher};
#[cfg(feature = "unstable")]
use core::iter;
#[cfg(feature = "std")]
use std::net;
#[cfg(feature = "unstable")]
use core::ops;
#[cfg(feature = "std")]
use std::path;
#[cfg(feature = "std")]
use std::rc::Rc;
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::rc::Rc;
#[cfg(feature = "std")]
use std::time::Duration;

#[cfg(feature = "std")]
use std::sync::Arc;
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::arc::Arc;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::boxed::Box;

use core::marker::PhantomData;

#[cfg(feature = "unstable")]
use core::nonzero::{NonZero, Zeroable};

use super::{Serialize, SerializeSeq, SerializeTuple, Serializer};
#[cfg(feature = "std")]
use super::Error;

///////////////////////////////////////////////////////////////////////////////

macro_rules! impl_visit {
    ($ty:ty, $method:ident $($cast:tt)*) => {
        impl Serialize for $ty {
            #[inline]
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer,
            {
                serializer.$method(*self $($cast)*)
            }
        }
    }
}

impl_visit!(bool, serialize_bool);
impl_visit!(isize, serialize_i64 as i64);
impl_visit!(i8, serialize_i8);
impl_visit!(i16, serialize_i16);
impl_visit!(i32, serialize_i32);
impl_visit!(i64, serialize_i64);
impl_visit!(usize, serialize_u64 as u64);
impl_visit!(u8, serialize_u8);
impl_visit!(u16, serialize_u16);
impl_visit!(u32, serialize_u32);
impl_visit!(u64, serialize_u64);
impl_visit!(f32, serialize_f32);
impl_visit!(f64, serialize_f64);
impl_visit!(char, serialize_char);

///////////////////////////////////////////////////////////////////////////////

impl Serialize for str {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(self)
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
impl Serialize for String {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        (&self[..]).serialize(serializer)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<T> Serialize for Option<T>
    where T: Serialize
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        match *self {
            Some(ref value) => serializer.serialize_some(value),
            None => serializer.serialize_none(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<T> Serialize for PhantomData<T> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_unit_struct("PhantomData")
    }
}

///////////////////////////////////////////////////////////////////////////////

macro_rules! array_impls {
    ($len:expr) => {
        impl<T> Serialize for [T; $len] where T: Serialize {
            #[inline]
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer,
            {
                let mut seq = try!(serializer.serialize_seq_fixed_size($len));
                for e in self {
                    try!(seq.serialize_element(e));
                }
                seq.end()
            }
        }
    }
}

array_impls!(0);
array_impls!(1);
array_impls!(2);
array_impls!(3);
array_impls!(4);
array_impls!(5);
array_impls!(6);
array_impls!(7);
array_impls!(8);
array_impls!(9);
array_impls!(10);
array_impls!(11);
array_impls!(12);
array_impls!(13);
array_impls!(14);
array_impls!(15);
array_impls!(16);
array_impls!(17);
array_impls!(18);
array_impls!(19);
array_impls!(20);
array_impls!(21);
array_impls!(22);
array_impls!(23);
array_impls!(24);
array_impls!(25);
array_impls!(26);
array_impls!(27);
array_impls!(28);
array_impls!(29);
array_impls!(30);
array_impls!(31);
array_impls!(32);

///////////////////////////////////////////////////////////////////////////////

macro_rules! serialize_seq {
    () => {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: Serializer,
        {
            serializer.collect_seq(self)
        }
    }
}

impl<T> Serialize for [T]
    where T: Serialize
{
    serialize_seq!();
}

#[cfg(any(feature = "std", feature = "collections"))]
impl<T> Serialize for BinaryHeap<T>
    where T: Serialize + Ord
{
    serialize_seq!();
}

#[cfg(any(feature = "std", feature = "collections"))]
impl<T> Serialize for BTreeSet<T>
    where T: Serialize + Ord
{
    serialize_seq!();
}

#[cfg(feature = "std")]
impl<T, H> Serialize for HashSet<T, H>
    where T: Serialize + Eq + Hash,
          H: BuildHasher
{
    serialize_seq!();
}

#[cfg(any(feature = "std", feature = "collections"))]
impl<T> Serialize for LinkedList<T>
    where T: Serialize
{
    serialize_seq!();
}

#[cfg(any(feature = "std", feature = "collections"))]
impl<T> Serialize for Vec<T>
    where T: Serialize
{
    serialize_seq!();
}

#[cfg(any(feature = "std", feature = "collections"))]
impl<T> Serialize for VecDeque<T>
    where T: Serialize
{
    serialize_seq!();
}

#[cfg(feature = "unstable")]
impl<A> Serialize for ops::Range<A>
    where ops::Range<A>: ExactSizeIterator + iter::Iterator<Item = A> + Clone,
          A: Serialize
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut seq = try!(serializer.serialize_seq(Some(self.len())));
        for e in self.clone() {
            try!(seq.serialize_element(&e));
        }
        seq.end()
    }
}

#[cfg(feature = "unstable")]
impl<A> Serialize for ops::RangeInclusive<A>
    where ops::RangeInclusive<A>: ExactSizeIterator + iter::Iterator<Item = A> + Clone,
          A: Serialize
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut seq = try!(serializer.serialize_seq(Some(self.len())));
        for e in self.clone() {
            try!(seq.serialize_element(&e));
        }
        seq.end()
    }
}

///////////////////////////////////////////////////////////////////////////////

impl Serialize for () {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_unit()
    }
}

///////////////////////////////////////////////////////////////////////////////

macro_rules! tuple_impls {
    ($(
        $TupleVisitor:ident ($len:expr, $($T:ident),+) {
            $($state:pat => $idx:tt,)+
        }
    )+) => {
        $(
            impl<$($T),+> Serialize for ($($T,)+)
                where $($T: Serialize),+
            {
                #[inline]
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where S: Serializer,
                {
                    let mut tuple = try!(serializer.serialize_tuple($len));
                    $(
                        try!(tuple.serialize_element(&self.$idx));
                    )+
                    tuple.end()
                }
            }
        )+
    }
}

tuple_impls! {
    TupleVisitor1 (1, T0) {
        0 => 0,
    }
    TupleVisitor2 (2, T0, T1) {
        0 => 0,
        1 => 1,
    }
    TupleVisitor3 (3, T0, T1, T2) {
        0 => 0,
        1 => 1,
        2 => 2,
    }
    TupleVisitor4 (4, T0, T1, T2, T3) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
    }
    TupleVisitor5 (5, T0, T1, T2, T3, T4) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
    }
    TupleVisitor6 (6, T0, T1, T2, T3, T4, T5) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
    }
    TupleVisitor7 (7, T0, T1, T2, T3, T4, T5, T6) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
        6 => 6,
    }
    TupleVisitor8 (8, T0, T1, T2, T3, T4, T5, T6, T7) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
        6 => 6,
        7 => 7,
    }
    TupleVisitor9 (9, T0, T1, T2, T3, T4, T5, T6, T7, T8) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
        6 => 6,
        7 => 7,
        8 => 8,
    }
    TupleVisitor10 (10, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
        6 => 6,
        7 => 7,
        8 => 8,
        9 => 9,
    }
    TupleVisitor11 (11, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
        6 => 6,
        7 => 7,
        8 => 8,
        9 => 9,
        10 => 10,
    }
    TupleVisitor12 (12, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
        6 => 6,
        7 => 7,
        8 => 8,
        9 => 9,
        10 => 10,
        11 => 11,
    }
    TupleVisitor13 (13, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
        6 => 6,
        7 => 7,
        8 => 8,
        9 => 9,
        10 => 10,
        11 => 11,
        12 => 12,
    }
    TupleVisitor14 (14, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
        6 => 6,
        7 => 7,
        8 => 8,
        9 => 9,
        10 => 10,
        11 => 11,
        12 => 12,
        13 => 13,
    }
    TupleVisitor15 (15, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
        6 => 6,
        7 => 7,
        8 => 8,
        9 => 9,
        10 => 10,
        11 => 11,
        12 => 12,
        13 => 13,
        14 => 14,
    }
    TupleVisitor16 (16, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
        6 => 6,
        7 => 7,
        8 => 8,
        9 => 9,
        10 => 10,
        11 => 11,
        12 => 12,
        13 => 13,
        14 => 14,
        15 => 15,
    }
}

///////////////////////////////////////////////////////////////////////////////

macro_rules! serialize_map {
    () => {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: Serializer,
        {
            serializer.collect_map(self)
        }
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
impl<K, V> Serialize for BTreeMap<K, V>
    where K: Serialize + Ord,
          V: Serialize
{
    serialize_map!();
}

#[cfg(feature = "std")]
impl<K, V, H> Serialize for HashMap<K, V, H>
    where K: Serialize + Eq + Hash,
          V: Serialize,
          H: BuildHasher
{
    serialize_map!();
}

///////////////////////////////////////////////////////////////////////////////

impl<'a, T: ?Sized> Serialize for &'a T
    where T: Serialize
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        (**self).serialize(serializer)
    }
}

impl<'a, T: ?Sized> Serialize for &'a mut T
    where T: Serialize
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        (**self).serialize(serializer)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T: ?Sized> Serialize for Box<T>
    where T: Serialize
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        (**self).serialize(serializer)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T> Serialize for Rc<T>
    where T: Serialize
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        (**self).serialize(serializer)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T> Serialize for Arc<T>
    where T: Serialize
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        (**self).serialize(serializer)
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
impl<'a, T: ?Sized> Serialize for Cow<'a, T>
    where T: Serialize + ToOwned
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        (**self).serialize(serializer)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<T, E> Serialize for Result<T, E>
    where T: Serialize,
          E: Serialize
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        match *self {
            Result::Ok(ref value) => serializer.serialize_newtype_variant("Result", 0, "Ok", value),
            Result::Err(ref value) => {
                serializer.serialize_newtype_variant("Result", 1, "Err", value)
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "std")]
impl Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        use super::SerializeStruct;
        let mut state = try!(serializer.serialize_struct("Duration", 2));
        try!(state.serialize_field("secs", &self.as_secs()));
        try!(state.serialize_field("nanos", &self.subsec_nanos()));
        state.end()
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "std")]
impl Serialize for net::IpAddr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        match *self {
            net::IpAddr::V4(ref a) => a.serialize(serializer),
            net::IpAddr::V6(ref a) => a.serialize(serializer),
        }
    }
}

#[cfg(feature = "std")]
impl Serialize for net::Ipv4Addr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        /// "101.102.103.104".len()
        const MAX_LEN: usize = 15;
        serialize_display_bounded_length!(self, MAX_LEN, serializer)
    }
}

#[cfg(feature = "std")]
impl Serialize for net::Ipv6Addr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        /// "1000:1002:1003:1004:1005:1006:1007:1008".len()
        const MAX_LEN: usize = 39;
        serialize_display_bounded_length!(self, MAX_LEN, serializer)
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "std")]
impl Serialize for net::SocketAddr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        match *self {
            net::SocketAddr::V4(ref addr) => addr.serialize(serializer),
            net::SocketAddr::V6(ref addr) => addr.serialize(serializer),
        }
    }
}

#[cfg(feature = "std")]
impl Serialize for net::SocketAddrV4 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        /// "101.102.103.104:65000".len()
        const MAX_LEN: usize = 21;
        serialize_display_bounded_length!(self, MAX_LEN, serializer)
    }
}

#[cfg(feature = "std")]
impl Serialize for net::SocketAddrV6 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        /// "[1000:1002:1003:1004:1005:1006:1007:1008]:65000".len()
        const MAX_LEN: usize = 47;
        serialize_display_bounded_length!(self, MAX_LEN, serializer)
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "std")]
impl Serialize for path::Path {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        match self.to_str() {
            Some(s) => s.serialize(serializer),
            None => Err(Error::custom("path contains invalid UTF-8 characters")),
        }
    }
}

#[cfg(feature = "std")]
impl Serialize for path::PathBuf {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        self.as_path().serialize(serializer)
    }
}

#[cfg(feature = "unstable")]
impl<T> Serialize for NonZero<T>
    where T: Serialize + Zeroable
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        (**self).serialize(serializer)
    }
}
