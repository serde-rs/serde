// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use lib::*;

use de::{Deserialize, DeserializeSeed, DeserializeState, Deserializer, EnumAccess, Error, Seed,
         SeqAccess, Unexpected, VariantAccess, Visitor};

#[cfg(any(feature = "std", feature = "alloc"))]
use de::MapAccess;

use private::de::size_hint;

////////////////////////////////////////////////////////////////////////////////

macro_rules! deserialize_impl {
    ($($ty: ty),*) => {
        $(
        impl<'de, S> DeserializeState<'de, S> for $ty
        {
            fn deserialize_state<D>(_seed: &mut S, deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                <$ty>::deserialize(deserializer)
            }
        }
        )*
    }
}

deserialize_impl! {
    u8,
    u16,
    u32,
    u64,
    usize,
    i8,
    i16,
    i32,
    i64,
    isize,
    f32,
    f64,
    (),
    bool
}

#[cfg(any(feature = "std", feature = "alloc"))]
deserialize_impl! {
    String
}


macro_rules! forwarded_impl {
    (( $($id: ident),* ), $ty: ty, $func: expr) => {
        impl<'de, S $(, $id)*> DeserializeState<'de, S> for $ty
            where $($id : DeserializeState<'de, S>,)*
        {
            fn deserialize_state<D>(seed: &mut S, deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                DeserializeState::deserialize_state(seed, deserializer).map($func)
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<'de, T, S> DeserializeState<'de, S> for Option<T>
where
    T: DeserializeState<'de, S>,
{
    fn deserialize_state<D>(seed: &mut S, deserializer: D) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        OptionSeed(Seed::new(seed)).deserialize(deserializer)
    }
}

////////////////////////////////////////////////////////////////////////////////

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
    where
        E: Error,
    {
        Ok(PhantomData)
    }
}

impl<'de, T, S> DeserializeState<'de, S> for PhantomData<T> {
    fn deserialize_state<D>(_: &mut S, deserializer: D) -> Result<PhantomData<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let visitor = PhantomDataVisitor {
            marker: PhantomData,
        };
        deserializer.deserialize_unit_struct("PhantomData", visitor)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Implementation of `DeserializeSeed` which deserializes sequences into a container
pub struct SeqSeed<S, F, T> {
    seed: T,
    with_capacity: F,
    _marker: PhantomData<S>,
}

impl<S, F, T> SeqSeed<S, F, T> {
    /// Constructs a new `SeqSeed` with a `seed` and a function which constructs the deserialized
    /// type (`with_capacity`)
    pub fn new(seed: T, with_capacity: F) -> SeqSeed<S, F, T> {
        SeqSeed {
            seed: seed,
            with_capacity: with_capacity,
            _marker: PhantomData,
        }
    }
}

impl<'de, S, F, T> DeserializeSeed<'de> for SeqSeed<S, F, T>
where
    T: DeserializeSeed<'de> + Clone,
    F: FnOnce(usize) -> S,
    S: Extend<T::Value>,
{
    type Value = S;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        impl<'de, S, F, T> Visitor<'de> for SeqSeed<S, F, T>
        where
            T: DeserializeSeed<'de> + Clone,
            F: FnOnce(usize) -> S,
            S: Extend<T::Value>,
        {
            type Value = S;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence")
            }

            #[inline]
            fn visit_seq<A>(self, mut access: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut values = (self.with_capacity)(size_hint::cautious(access.size_hint()));

                while let Some(value) = try!(access.next_element_seed(self.seed.clone())) {
                    values.extend(Some(value));
                }

                Ok(values)
            }
        }

        deserializer.deserialize_seq(self)
    }
}

/// `SeqSeedEx` implements `DeserializeSeed` for sequences whose elements implement
/// `DeserializeState`
pub struct SeqSeedEx<'seed, S, F, T: 'seed, U>
where
    T: ?Sized,
{
    seed: &'seed mut T,
    with_capacity: F,
    _marker: PhantomData<(S, U)>,
}

impl<'seed, 'de, S, F, T, U> SeqSeedEx<'seed, S, F, T, U>
where
    T: ?Sized,
    U: DeserializeState<'de, T>,
    F: FnOnce(usize) -> S,
    S: Extend<U>,
{
    /// Constructs a new instance of `SeqSeedEx`
    pub fn new(seed: &'seed mut T, with_capacity: F) -> SeqSeedEx<'seed, S, F, T, U> {
        SeqSeedEx {
            seed: seed,
            with_capacity: with_capacity,
            _marker: PhantomData,
        }
    }
}


impl<'de, 'seed, S, F, T, U> Visitor<'de> for SeqSeedEx<'seed, S, F, T, U>
where
    T: ?Sized,
    U: DeserializeState<'de, T>,
    F: FnOnce(usize) -> S,
    S: Extend<U>,
{
    type Value = S;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a sequence")
    }

    #[inline]
    fn visit_seq<A>(self, mut access: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = (self.with_capacity)(size_hint::cautious(access.size_hint()));

        while let Some(value) = try!(access.next_element_seed(Seed::new(&mut *&mut *self.seed))) {
            values.extend(Some(value));
        }

        Ok(values)
    }
}

impl<'de, 'seed, S, F, T, U> DeserializeSeed<'de> for SeqSeedEx<'seed, S, F, T, U>
where
    T: ?Sized,
    U: DeserializeState<'de, T>,
    F: FnOnce(usize) -> S,
    S: Extend<U>,
{
    type Value = S;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}


/// `DeserializeSeed` instances for optional values
pub struct OptionSeed<S>(pub S);

impl<'de, S> DeserializeSeed<'de> for OptionSeed<S>
where
    S: DeserializeSeed<'de>,
{
    type Value = Option<S::Value>;

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        impl<'de, S> Visitor<'de> for OptionSeed<S>
        where
            S: DeserializeSeed<'de>,
        {
            type Value = Option<S::Value>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("option")
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(None)
            }


            #[inline]
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(None)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                self.0.deserialize(deserializer).map(Some)
            }
        }

        deserializer.deserialize_option(self)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
macro_rules! seq_impl {
    (
        $ty:ident < T $(: $tbound1:ident $(+ $tbound2:ident)*)* $(, $typaram:ident : $bound1:ident $(+ $bound2:ident)*)* >,
        $access:ident,
        $ctor:expr,
        $with_capacity:expr,
        $insert:expr
    ) => {
        impl<'de, Seed, T $(, $typaram)*> DeserializeState<'de, Seed> for $ty<T $(, $typaram)*>
        where
            Seed: ?Sized,
            T: DeserializeState<'de, Seed> $(+ $tbound1 $(+ $tbound2)*)*,
            $($typaram: $bound1 $(+ $bound2)*,)*
        {
            fn deserialize_state<D>(seed: &mut Seed, deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let visitor = SeqSeedEx::new(seed, $with_capacity);
                deserializer.deserialize_seq(visitor)
            }
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
seq_impl!(
    BinaryHeap<T: Ord>,
    seq,
    BinaryHeap::new(),
    BinaryHeap::with_capacity,
    BinaryHeap::push);

#[cfg(any(feature = "std", feature = "alloc"))]
seq_impl!(
    BTreeSet<T: Eq + Ord>,
    seq,
    BTreeSet::new(),
    |_| BTreeSet::new(),
    BTreeSet::insert);

#[cfg(any(feature = "std", feature = "alloc"))]
seq_impl!(
    LinkedList<T>,
    seq,
    LinkedList::new(),
    |_| LinkedList::new(),
    LinkedList::push_back
);

#[cfg(feature = "std")]
seq_impl!(
    HashSet<T: Eq + Hash, S: BuildHasher + Default>,
    seq,
    HashSet::with_hasher(S::default()),
    |size| HashSet::with_capacity_and_hasher(size, S::default()),
    HashSet::insert);

#[cfg(any(feature = "std", feature = "alloc"))]
seq_impl!(Vec<T>, seq, Vec::new(), Vec::with_capacity, Vec::push);

#[cfg(any(feature = "std", feature = "alloc"))]
seq_impl!(
    VecDeque<T>,
    seq,
    VecDeque::new(),
    VecDeque::with_capacity,
    VecDeque::push_back
);

////////////////////////////////////////////////////////////////////////////////

struct ArrayVisitor<'seed, S: 'seed, A> {
    seed: &'seed mut S,
    marker: PhantomData<A>,
}

impl<'seed, S, A> ArrayVisitor<'seed, S, A> {
    fn new(seed: &'seed mut S) -> Self {
        ArrayVisitor {
            seed: seed,
            marker: PhantomData,
        }
    }
}

impl<'de, 'seed, S, T> Visitor<'de> for ArrayVisitor<'seed, S, [T; 0]> {
    type Value = [T; 0];

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an empty array")
    }

    #[inline]
    fn visit_seq<A>(self, _: A) -> Result<[T; 0], A::Error>
    where
        A: SeqAccess<'de>,
    {
        Ok([])
    }
}

// Does not require T: DeserializeState<'de, S>.
impl<'de, S, T> DeserializeState<'de, S> for [T; 0] {
    fn deserialize_state<D>(seed: &mut S, deserializer: D) -> Result<[T; 0], D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(0, ArrayVisitor::<_, [T; 0]>::new(seed))
    }
}

macro_rules! array_impls {
    ($($len:expr => ($($n:tt $name:ident)+))+) => {
        $(
            impl<'de, 'seed, T, S> Visitor<'de> for ArrayVisitor<'seed, S, [T; $len]>
            where
                T: DeserializeState<'de, S>,
            {
                type Value = [T; $len];

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str(concat!("an array of length ", $len))
                }

                #[inline]
                fn visit_seq<A>(self, mut seq: A) -> Result<[T; $len], A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    $(
                        let $name = match try!(seq.next_element_seed(Seed::new(&mut *self.seed))) {
                            Some(val) => val,
                            None => return Err(Error::invalid_length($n, &self)),
                        };
                    )+

                    Ok([$($name),+])
                }
            }

            impl<'de, S, T> DeserializeState<'de, S> for [T; $len]
            where
                T: DeserializeState<'de, S>,
            {
                fn deserialize_state<D>(seed: &mut S, deserializer: D) -> Result<[T; $len], D::Error>
                where
                    D: Deserializer<'de>,
                {
                        deserializer.deserialize_tuple($len, ArrayVisitor::<_, [T; $len]>::new(seed))
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

////////////////////////////////////////////////////////////////////////////////

macro_rules! tuple_impls {
    ($($len:tt $visitor:ident => ($($n:tt $name:ident)+))+) => {
        $(
            struct $visitor<'seed, S: 'seed, $($name,)+> {
                seed: &'seed mut S,
                marker: PhantomData<($($name,)+)>,
            }

            impl<'seed, S, $($name,)+> $visitor<'seed, S, $($name,)+> {
                fn new(seed: &'seed mut S) -> Self {
                    $visitor { seed: seed, marker: PhantomData }
                }
            }

            impl<'de, 'seed, S, $($name: DeserializeState<'de, S>),+> Visitor<'de> for $visitor<'seed, S, $($name,)+> {
                type Value = ($($name,)+);

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str(concat!("a tuple of size ", $len))
                }

                #[inline]
                #[allow(non_snake_case)]
                fn visit_seq<A>(self, mut seq: A) -> Result<($($name,)+), A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    $(
                        let $name = match try!(seq.next_element_seed(Seed::new(&mut *self.seed))) {
                            Some(value) => value,
                            None => return Err(Error::invalid_length($n, &self)),
                        };
                    )+

                    Ok(($($name,)+))
                }
            }

            impl<'de, S, $($name: DeserializeState<'de, S>),+> DeserializeState<'de, S> for ($($name,)+) {
                #[inline]
                fn deserialize_state<D>(seed: &mut S, deserializer: D) -> Result<($($name,)+), D::Error>
                where
                    D: Deserializer<'de>,
                {
                    deserializer.deserialize_tuple($len, $visitor::new(seed))
                }
            }
        )+
    }
}

tuple_impls! {
    1 TupleVisitor1 => (0 T0)
    2 TupleVisitor2 => (0 T0 1 T1)
    3 TupleVisitor3 => (0 T0 1 T1 2 T2)
    4 TupleVisitor4 => (0 T0 1 T1 2 T2 3 T3)
    5 TupleVisitor5 => (0 T0 1 T1 2 T2 3 T3 4 T4)
    6 TupleVisitor6 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5)
    7 TupleVisitor7 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6)
    8 TupleVisitor8 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7)
    9 TupleVisitor9 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8)
    10 TupleVisitor10 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9)
    11 TupleVisitor11 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10)
    12 TupleVisitor12 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11)
    13 TupleVisitor13 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12)
    14 TupleVisitor14 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13)
    15 TupleVisitor15 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14)
    16 TupleVisitor16 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15)
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(any(feature = "std", feature = "alloc"))]
macro_rules! map_impl {
    (
        $ty:ident < K $(: $kbound1:ident $(+ $kbound2:ident)*)*, V $(, $typaram:ident : $bound1:ident $(+ $bound2:ident)*)* >,
        $access:ident,
        $ctor:expr,
        $with_capacity:expr
    ) => {
        impl<'de, S2, K, V $(, $typaram)*> DeserializeState<'de, S2> for $ty<K, V $(, $typaram)*>
        where
            K: DeserializeState<'de, S2> $(+ $kbound1 $(+ $kbound2)*)*,
            V: DeserializeState<'de, S2>,
            $($typaram: $bound1 $(+ $bound2)*),*
        {
            fn deserialize_state<D>(seed: &mut S2, deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct MapVisitor<'seed, S2: 'seed, K, V $(, $typaram)*> {
                    seed: &'seed mut S2,
                    marker: PhantomData<$ty<K, V $(, $typaram)*>>,
                }

                impl<'de, 'seed, S2, K, V $(, $typaram)*> Visitor<'de> for MapVisitor<'seed, S2, K, V $(, $typaram)*>
                where
                    K: DeserializeState<'de, S2> $(+ $kbound1 $(+ $kbound2)*)*,
                    V: DeserializeState<'de, S2>,
                    $($typaram: $bound1 $(+ $bound2)*),*
                {
                    type Value = $ty<K, V $(, $typaram)*>;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("a map")
                    }

                    #[inline]
                    fn visit_map<A>(self, mut $access: A) -> Result<Self::Value, A::Error>
                    where
                        A: MapAccess<'de>,
                    {
                        let mut values = $with_capacity;

                        while let Some(key) = try!($access.next_key_seed(Seed::new(&mut *self.seed))) {
                            let value = try!($access.next_value_seed(Seed::new(&mut *self.seed)));
                            values.insert(key, value);
                        }

                        Ok(values)
                    }
                }

                let visitor = MapVisitor { seed: seed, marker: PhantomData };
                deserializer.deserialize_map(visitor)
            }
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
map_impl!(
    BTreeMap<K: Ord, V>,
    map,
    BTreeMap::new(),
    BTreeMap::new());

#[cfg(feature = "std")]
map_impl!(
    HashMap<K: Eq + Hash, V, S: BuildHasher + Default>,
    map,
    HashMap::with_hasher(S::default()),
    HashMap::with_capacity_and_hasher(size_hint::cautious(map.size_hint()), S::default()));

////////////////////////////////////////////////////////////////////////////////

#[cfg(any(feature = "std", feature = "alloc"))]
forwarded_impl!((T), Box<T>, Box::new);

#[cfg(any(feature = "std", feature = "alloc"))]
forwarded_impl!((T), Box<[T]>, Vec::into_boxed_slice);

#[cfg(all(feature = "rc", any(feature = "std", feature = "alloc")))]
forwarded_impl!((T), Arc<T>, Arc::new);

#[cfg(all(feature = "rc", any(feature = "std", feature = "alloc")))]
forwarded_impl!((T), Rc<T>, Rc::new);

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'de, 'a, S, T: ?Sized> DeserializeState<'de, S> for Cow<'a, T>
where
    T: ToOwned,
    T::Owned: DeserializeState<'de, S>,
{
    #[inline]
    fn deserialize_state<D>(seed: &mut S, deserializer: D) -> Result<Cow<'a, T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::Owned::deserialize_state(seed, deserializer).map(Cow::Owned)
    }
}

////////////////////////////////////////////////////////////////////////////////

forwarded_impl!((T), Cell<T>, Cell::new);

forwarded_impl!((T), RefCell<T>, RefCell::new);

#[cfg(feature = "std")]
forwarded_impl!((T), Mutex<T>, Mutex::new);

#[cfg(feature = "std")]
forwarded_impl!((T), RwLock<T>, RwLock::new);

////////////////////////////////////////////////////////////////////////////////


// Similar to:
//
//     #[derive(Deserialize)]
//     #[serde(deny_unknown_fields)]
//     struct Range {
//         start: u64,
//         end: u32,
//     }
#[cfg(feature = "std")]
impl<'de, S, Idx> DeserializeState<'de, S> for ops::Range<Idx>
where
    Idx: DeserializeState<'de, S>,
{
    fn deserialize_state<D>(seed: &mut S, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // If this were outside of the serde crate, it would just use:
        //
        //    #[derive(Deserialize)]
        //    #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Start,
            End,
        };

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`start` or `end`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: Error,
                    {
                        match value {
                            "start" => Ok(Field::Start),
                            "end" => Ok(Field::End),
                            _ => Err(Error::unknown_field(value, FIELDS)),
                        }
                    }

                    fn visit_bytes<E>(self, value: &[u8]) -> Result<Field, E>
                    where
                        E: Error,
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

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct RangeVisitor<'seed, S: 'seed, Idx> {
            seed: &'seed mut S,
            phantom: PhantomData<Idx>,
        }

        impl<'de, 'seed, S, Idx> Visitor<'de> for RangeVisitor<'seed, S, Idx>
        where
            Idx: DeserializeState<'de, S>,
        {
            type Value = ops::Range<Idx>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Range")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<ops::Range<Idx>, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let start: Idx = match try!(seq.next_element_seed(Seed::new(&mut *self.seed))) {
                    Some(value) => value,
                    None => {
                        return Err(Error::invalid_length(0, &self));
                    }
                };
                let end: Idx = match try!(seq.next_element_seed(Seed::new(&mut *self.seed))) {
                    Some(value) => value,
                    None => {
                        return Err(Error::invalid_length(1, &self));
                    }
                };
                Ok(start..end)
            }

            fn visit_map<A>(self, mut map: A) -> Result<ops::Range<Idx>, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut start: Option<Idx> = None;
                let mut end: Option<Idx> = None;
                while let Some(key) = try!(map.next_key()) {
                    match key {
                        Field::Start => {
                            if start.is_some() {
                                return Err(<A::Error as Error>::duplicate_field("start"));
                            }
                            start = Some(try!(map.next_value_seed(Seed::new(&mut *self.seed))));
                        }
                        Field::End => {
                            if end.is_some() {
                                return Err(<A::Error as Error>::duplicate_field("end"));
                            }
                            end = Some(try!(map.next_value_seed(Seed::new(&mut *self.seed))));
                        }
                    }
                }
                let start = match start {
                    Some(start) => start,
                    None => return Err(<A::Error as Error>::missing_field("start")),
                };
                let end = match end {
                    Some(end) => end,
                    None => return Err(<A::Error as Error>::missing_field("end")),
                };
                Ok(start..end)
            }
        }

        const FIELDS: &'static [&'static str] = &["start", "end"];
        deserializer.deserialize_struct(
            "Range",
            FIELDS,
            RangeVisitor {
                seed: seed,
                phantom: PhantomData,
            },
        )
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "unstable")]
impl<'de, T> DeserializeState<'de, S> for NonZero<T>
where
    T: DeserializeState<'de, S> + Zeroable,
{
    fn deserialize_state<D>(seed: &mut S, deserializer: D) -> Result<NonZero<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = try!(Deserialize::deserialize_state(seed, deserializer));
        unsafe {
            let ptr = &value as *const T as *const u8;
            if slice::from_raw_parts(ptr, mem::size_of::<T>())
                .iter()
                .all(|&b| b == 0)
            {
                return Err(Error::custom("expected a non-zero value"));
            }
            // Waiting for a safe way to construct NonZero<T>:
            // https://github.com/rust-lang/rust/issues/27730#issuecomment-269726075
            Ok(NonZero::new(value))
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<'de, S, T, E> DeserializeState<'de, S> for Result<T, E>
where
    T: DeserializeState<'de, S>,
    E: DeserializeState<'de, S>,
{
    fn deserialize_state<D>(seed: &mut S, deserializer: D) -> Result<Result<T, E>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // If this were outside of the serde crate, it would just use:
        //
        //    #[derive(Deserialize)]
        //    #[serde(variant_identifier)]
        enum Field {
            Ok,
            Err,
        }

        impl<'de> Deserialize<'de> for Field {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`Ok` or `Err`")
                    }

                    fn visit_u32<E>(self, value: u32) -> Result<Field, E>
                    where
                        E: Error,
                    {
                        match value {
                            0 => Ok(Field::Ok),
                            1 => Ok(Field::Err),
                            _ => Err(Error::invalid_value(
                                Unexpected::Unsigned(value as u64),
                                &self,
                            )),
                        }
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: Error,
                    {
                        match value {
                            "Ok" => Ok(Field::Ok),
                            "Err" => Ok(Field::Err),
                            _ => Err(Error::unknown_variant(value, VARIANTS)),
                        }
                    }

                    fn visit_bytes<E>(self, value: &[u8]) -> Result<Field, E>
                    where
                        E: Error,
                    {
                        match value {
                            b"Ok" => Ok(Field::Ok),
                            b"Err" => Ok(Field::Err),
                            _ => match str::from_utf8(value) {
                                Ok(value) => Err(Error::unknown_variant(value, VARIANTS)),
                                Err(_) => {
                                    Err(Error::invalid_value(Unexpected::Bytes(value), &self))
                                }
                            },
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct ResultVisitor<'seed, S: 'seed, T, E> {
            seed: &'seed mut S,
            _marker: PhantomData<Result<T, E>>,
        }

        impl<'de, 'seed, S, T, E> Visitor<'de> for ResultVisitor<'seed, S, T, E>
        where
            T: DeserializeState<'de, S>,
            E: DeserializeState<'de, S>,
        {
            type Value = Result<T, E>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("enum Result")
            }

            fn visit_enum<A>(self, data: A) -> Result<Result<T, E>, A::Error>
            where
                A: EnumAccess<'de>,
            {
                match try!(data.variant()) {
                    (Field::Ok, v) => v.newtype_variant_seed(Seed::new(&mut *self.seed)).map(Ok),
                    (Field::Err, v) => v.newtype_variant_seed(Seed::new(&mut *self.seed)).map(Err),
                }
            }
        }

        const VARIANTS: &'static [&'static str] = &["Ok", "Err"];

        deserializer.deserialize_enum(
            "Result",
            VARIANTS,
            ResultVisitor {
                seed: seed,
                _marker: PhantomData,
            },
        )
    }
}
