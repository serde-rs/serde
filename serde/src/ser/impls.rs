//! Implementations for all of Rust's builtin types.

use std::borrow::Cow;
use std::collections::{
    BinaryHeap,
    BTreeMap,
    BTreeSet,
    LinkedList,
    HashMap,
    HashSet,
    VecDeque,
};
#[cfg(feature = "nightly")]
use collections::enum_set::{CLike, EnumSet};
use std::hash::Hash;
#[cfg(feature = "nightly")]
use std::iter;
#[cfg(feature = "nightly")]
use std::num;
#[cfg(feature = "nightly")]
use std::ops;
use std::path;
use std::rc::Rc;
use std::sync::Arc;

#[cfg(feature = "nightly")]
use core::nonzero::{NonZero, Zeroable};

use super::{
    Serialize,
    Serializer,
    SeqVisitor,
    MapVisitor,
};

///////////////////////////////////////////////////////////////////////////////

macro_rules! impl_visit {
    ($ty:ty, $method:ident) => {
        impl Serialize for $ty {
            #[inline]
            fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                where S: Serializer,
            {
                serializer.$method(*self)
            }
        }
    }
}

impl_visit!(bool, serialize_bool);
impl_visit!(isize, serialize_isize);
impl_visit!(i8, serialize_i8);
impl_visit!(i16, serialize_i16);
impl_visit!(i32, serialize_i32);
impl_visit!(i64, serialize_i64);
impl_visit!(usize, serialize_usize);
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
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        serializer.serialize_str(self)
    }
}

impl Serialize for String {
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        (&self[..]).serialize(serializer)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<T> Serialize for Option<T> where T: Serialize {
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        match *self {
            Some(ref value) => serializer.serialize_some(value),
            None => serializer.serialize_none(),
        }
    }
}

impl<T> SeqVisitor for Option<T> where T: Serialize {
    #[inline]
    fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
        where S: Serializer,
    {
        match self.take() {
            Some(value) => {
                try!(serializer.serialize_seq_elt(value));
                Ok(Some(()))
            }
            None => Ok(None),
        }
    }

    #[inline]
    fn len(&self) -> Option<usize> {
        Some(if self.is_some() { 1 } else { 0 })
    }
}

///////////////////////////////////////////////////////////////////////////////

/// A `serde::Visitor` for sequence iterators.
///
/// # Examples
///
/// ```
/// use serde::{Serialize, Serializer};
/// use serde::ser::impls::SeqIteratorVisitor;
///
/// struct Seq(Vec<u32>);
///
/// impl Serialize for Seq {
///     fn serialize<S>(&self, ser: &mut S) -> Result<(), S::Error>
///         where S: Serializer,
///     {
///         ser.serialize_seq(SeqIteratorVisitor::new(
///             self.0.iter(),
///             Some(self.0.len()),
///         ))
///     }
/// }
/// ```
pub struct SeqIteratorVisitor<Iter> {
    iter: Iter,
    len: Option<usize>,
}

impl<T, Iter> SeqIteratorVisitor<Iter>
    where Iter: Iterator<Item=T>
{
    /// Construct a new `SeqIteratorVisitor<Iter>`.
    #[inline]
    pub fn new(iter: Iter, len: Option<usize>) -> SeqIteratorVisitor<Iter> {
        SeqIteratorVisitor {
            iter: iter,
            len: len,
        }
    }
}

impl<T, Iter> SeqVisitor for SeqIteratorVisitor<Iter>
    where T: Serialize,
          Iter: Iterator<Item=T>,
{
    #[inline]
    fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
        where S: Serializer,
    {
        match self.iter.next() {
            Some(value) => {
                try!(serializer.serialize_seq_elt(value));
                Ok(Some(()))
            }
            None => Ok(None),
        }
    }

    #[inline]
    fn len(&self) -> Option<usize> {
        self.len
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<T> Serialize for [T]
    where T: Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        serializer.serialize_seq(SeqIteratorVisitor::new(self.iter(), Some(self.len())))
    }
}

macro_rules! array_impls {
    ($len:expr) => {
        impl<T> Serialize for [T; $len] where T: Serialize {
            #[inline]
            fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                where S: Serializer,
            {
                serializer.serialize_seq(SeqIteratorVisitor::new(self.iter(), Some($len)))
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

impl<T> Serialize for BinaryHeap<T>
    where T: Serialize + Ord
{
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        serializer.serialize_seq(SeqIteratorVisitor::new(self.iter(), Some(self.len())))
    }
}

impl<T> Serialize for BTreeSet<T>
    where T: Serialize + Ord,
{
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        serializer.serialize_seq(SeqIteratorVisitor::new(self.iter(), Some(self.len())))
    }
}

#[cfg(feature = "nightly")]
impl<T> Serialize for EnumSet<T>
    where T: Serialize + CLike
{
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        serializer.serialize_seq(SeqIteratorVisitor::new(self.iter(), Some(self.len())))
    }
}

impl<T> Serialize for HashSet<T>
    where T: Serialize + Eq + Hash,
{
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        serializer.serialize_seq(SeqIteratorVisitor::new(self.iter(), Some(self.len())))
    }
}

impl<T> Serialize for LinkedList<T>
    where T: Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        serializer.serialize_seq(SeqIteratorVisitor::new(self.iter(), Some(self.len())))
    }
}

#[cfg(feature = "nightly")]
impl<A> Serialize for ops::Range<A>
    where A: Serialize + Clone + iter::Step + num::One,
          for<'a> &'a A: ops::Add<&'a A, Output = A>,
{
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        let len = iter::Step::steps_between(&self.start, &self.end, &A::one());
        serializer.serialize_seq(SeqIteratorVisitor::new(self.clone(), len))
    }
}

impl<T> Serialize for Vec<T> where T: Serialize {
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        (&self[..]).serialize(serializer)
    }
}

impl<T> Serialize for VecDeque<T> where T: Serialize {
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        serializer.serialize_seq(SeqIteratorVisitor::new(self.iter(), Some(self.len())))
    }
}

///////////////////////////////////////////////////////////////////////////////

impl Serialize for () {
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        serializer.serialize_unit()
    }
}

///////////////////////////////////////////////////////////////////////////////

// FIXME(rust #19630) Remove this work-around
macro_rules! e {
    ($e:expr) => { $e }
}

macro_rules! tuple_impls {
    ($(
        $TupleVisitor:ident ($len:expr, $($T:ident),+) {
            $($state:pat => $idx:tt,)+
        }
    )+) => {
        $(
            /// A tuple visitor.
            pub struct $TupleVisitor<'a, $($T: 'a),+> {
                tuple: &'a ($($T,)+),
                state: u8,
            }

            impl<'a, $($T: 'a),+> $TupleVisitor<'a, $($T),+> {
                /// Construct a new, empty `TupleVisitor`.
                pub fn new(tuple: &'a ($($T,)+)) -> $TupleVisitor<'a, $($T),+> {
                    $TupleVisitor {
                        tuple: tuple,
                        state: 0,
                    }
                }
            }

            impl<'a, $($T),+> SeqVisitor for $TupleVisitor<'a, $($T),+>
                where $($T: Serialize),+
            {
                fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
                    where S: Serializer,
                {
                    match self.state {
                        $(
                            $state => {
                                self.state += 1;
                                Ok(Some(try!(serializer.serialize_tuple_elt(&e!(self.tuple.$idx)))))
                            }
                        )+
                        _ => {
                            Ok(None)
                        }
                    }
                }

                fn len(&self) -> Option<usize> {
                    Some($len)
                }
            }

            impl<$($T),+> Serialize for ($($T,)+)
                where $($T: Serialize),+
            {
                #[inline]
                fn serialize<S: Serializer>(&self, serializer: &mut S) -> Result<(), S::Error> {
                    serializer.serialize_tuple($TupleVisitor::new(self))
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
}

///////////////////////////////////////////////////////////////////////////////

/// A `serde::Visitor` for (key, value) map iterators.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use serde::{Serialize, Serializer};
/// use serde::ser::impls::MapIteratorVisitor;
///
/// struct Map(HashMap<u32, u32>);
///
/// impl Serialize for Map {
///     fn serialize<S>(&self, ser: &mut S) -> Result<(), S::Error>
///         where S: Serializer,
///     {
///         ser.serialize_map(MapIteratorVisitor::new(
///             self.0.iter(),
///             Some(self.0.len()),
///         ))
///     }
/// }
/// ```
pub struct MapIteratorVisitor<Iter> {
    iter: Iter,
    len: Option<usize>,
}

impl<K, V, Iter> MapIteratorVisitor<Iter>
    where Iter: Iterator<Item=(K, V)>
{
    /// Construct a new `MapIteratorVisitor<Iter>`.
    #[inline]
    pub fn new(iter: Iter, len: Option<usize>) -> MapIteratorVisitor<Iter> {
        MapIteratorVisitor {
            iter: iter,
            len: len,
        }
    }
}

impl<K, V, I> MapVisitor for MapIteratorVisitor<I>
    where K: Serialize,
          V: Serialize,
          I: Iterator<Item=(K, V)>,
{
    #[inline]
    fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
        where S: Serializer,
    {
        match self.iter.next() {
            Some((key, value)) => {
                try!(serializer.serialize_map_elt(key, value));
                Ok(Some(()))
            }
            None => Ok(None)
        }
    }

    #[inline]
    fn len(&self) -> Option<usize> {
        self.len
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<K, V> Serialize for BTreeMap<K, V>
    where K: Serialize + Ord,
          V: Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        serializer.serialize_map(MapIteratorVisitor::new(self.iter(), Some(self.len())))
    }
}

impl<K, V> Serialize for HashMap<K, V>
    where K: Serialize + Eq + Hash,
          V: Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        serializer.serialize_map(MapIteratorVisitor::new(self.iter(), Some(self.len())))
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<'a, T: ?Sized> Serialize for &'a T where T: Serialize {
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        (**self).serialize(serializer)
    }
}

impl<'a, T: ?Sized> Serialize for &'a mut T where T: Serialize {
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        (**self).serialize(serializer)
    }
}

impl<T: ?Sized> Serialize for Box<T> where T: Serialize {
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        (**self).serialize(serializer)
    }
}

impl<T> Serialize for Rc<T> where T: Serialize, {
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        (**self).serialize(serializer)
    }
}

impl<T> Serialize for Arc<T> where T: Serialize, {
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        (**self).serialize(serializer)
    }
}

impl<'a, T: ?Sized> Serialize for Cow<'a, T> where T: Serialize + ToOwned, {
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        (**self).serialize(serializer)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<T, E> Serialize for Result<T, E> where T: Serialize, E: Serialize {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        match *self {
            Result::Ok(ref value) => {
                serializer.serialize_newtype_variant("Result", 0, "Ok", value)
            }
            Result::Err(ref value) => {
                serializer.serialize_newtype_variant("Result", 1, "Err", value)
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

impl Serialize for path::Path {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        self.to_str().unwrap().serialize(serializer)
    }
}

impl Serialize for path::PathBuf {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer,
    {
        self.to_str().unwrap().serialize(serializer)
    }
}

#[cfg(feature = "nightly")]
impl<T> Serialize for NonZero<T> where T: Serialize + Zeroable {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        (**self).serialize(serializer)
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "num-bigint")]
impl Serialize for ::num::bigint::BigInt {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        self.to_str_radix(10).serialize(serializer)
    }
}

#[cfg(feature = "num-bigint")]
impl Serialize for ::num::bigint::BigUint {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        self.to_str_radix(10).serialize(serializer)
    }
}

#[cfg(feature = "num-complex")]
impl<T> Serialize for ::num::complex::Complex<T>
    where T: Serialize + Clone + ::num::Num
{
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        (&self.re, &self.im).serialize(serializer)
    }
}

#[cfg(feature = "num-rational")]
impl<T> Serialize for ::num::rational::Ratio<T>
    where T: Serialize + Clone + ::num::Integer + PartialOrd
{
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        (self.numer(), self.denom()).serialize(serializer)
    }
}
