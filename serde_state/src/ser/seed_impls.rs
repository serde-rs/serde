// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use lib::*;

use ser::{Serialize, SerializeTuple, Serializer, SerializeState};

#[cfg(feature = "std")]
use ser::Error;

////////////////////////////////////////////////////////////////////////////////

impl<T> SerializeState for Option<T>
where
    T: SerializeState,
{
    type Seed = T::Seed;

    #[inline]
    fn serialize_state<S>(&self, serializer: S, seed: &Self::Seed) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Some(ref value) => serializer.serialize_some(&Seeded::new(seed, value)),
            None => serializer.serialize_none(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

macro_rules! array_impls {
    ($($len:tt)+) => {
        $(
            impl<T> SerializeState for [T; $len]
            where
                T: SerializeState,
            {
                type Seed = T::Seed;

                #[inline]
                fn serialize_state<S>(&self, serializer: S, seed: &Self::Seed) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    let mut seq = try!(serializer.serialize_tuple($len));
                    for e in self {
                        try!(seq.serialize_element(&Seeded::new(seed, e)));
                    }
                    seq.end()
                }
            }
        )+
    }
}

array_impls!(01 02 03 04 05 06 07 08 09 10
             11 12 13 14 15 16 17 18 19 20
             21 22 23 24 25 26 27 28 29 30
             31 32);

////////////////////////////////////////////////////////////////////////////////

impl<T> SerializeState for [T]
where
    T: SerializeState,
{
    type Seed = T::Seed;

    #[inline]
    fn serialize_state<S>(&self, serializer: S, seed: &Self::Seed) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(self.iter().map(|value| Seeded::new(seed, value)))
    }
}


macro_rules! seq_impl {
    ($ty:ident < T $(: $tbound1:ident $(+ $tbound2:ident)*)* $(, $typaram:ident : $bound:ident)* >) => {
        impl<T $(, $typaram)*> SerializeState for $ty<T $(, $typaram)*>
        where
            T: SerializeState $(+ $tbound1 $(+ $tbound2)*)*,
            $($typaram: $bound,)*
        {
            type Seed = T::Seed;

            #[inline]
            fn serialize_state<S>(&self, serializer: S, seed: &Self::Seed) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.collect_seq(self.iter().map(|value| Seeded::new(seed, value)))
            }
        }
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
seq_impl!(BinaryHeap<T: Ord>);

#[cfg(any(feature = "std", feature = "collections"))]
seq_impl!(BTreeSet<T: Ord>);

#[cfg(feature = "std")]
seq_impl!(HashSet<T: Eq + Hash, H: BuildHasher>);

#[cfg(any(feature = "std", feature = "collections"))]
seq_impl!(LinkedList<T>);

#[cfg(any(feature = "std", feature = "collections"))]
seq_impl!(Vec<T>);

#[cfg(any(feature = "std", feature = "collections"))]
seq_impl!(VecDeque<T>);

////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "std")]
impl<Idx> SerializeState for ops::Range<Idx>
where
    Idx: SerializeState,
{
    type Seed = Idx::Seed;

    fn serialize_state<S>(&self, serializer: S, seed: &Self::Seed) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use super::SerializeStruct;
        let mut state = try!(serializer.serialize_struct("Range", 2));
        try!(state.serialize_field("start", &Seeded::new(seed, &self.start)));
        try!(state.serialize_field("end", &Seeded::new(seed, &self.end)));
        state.end()
    }
}

////////////////////////////////////////////////////////////////////////////////

macro_rules! tuple_impls {
    ($seed: ident; $($len:expr => ($($n:tt $name:ident)+))+) => {
        $(
            impl<$seed, $($name),+> SerializeState for ($($name,)+)
            where
                $($name: SerializeState<Seed = $seed>,)+
            {
                type Seed = $seed;

                #[inline]
                fn serialize_state<S>(&self, serializer: S, seed: &Self::Seed) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    let mut tuple = try!(serializer.serialize_tuple($len));
                    $(
                        try!(tuple.serialize_element(&Seeded::new(seed, &self.$n)));
                    )+
                    tuple.end()
                }
            }
        )+
    }
}

tuple_impls! {
    Seed;
    1 => (0 T0)
    2 => (0 T0 1 T1)
    3 => (0 T0 1 T1 2 T2)
    4 => (0 T0 1 T1 2 T2 3 T3)
    5 => (0 T0 1 T1 2 T2 3 T3 4 T4)
    6 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5)
    7 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6)
    8 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7)
    9 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8)
    10 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9)
    11 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10)
    12 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11)
    13 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12)
    14 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13)
    15 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14)
    16 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15)
}

////////////////////////////////////////////////////////////////////////////////

macro_rules! map_impl {
    ($ty:ident < K $(: $kbound1:ident $(+ $kbound2:ident)*)*, V $(, $typaram:ident : $bound:ident)* >) => {
        impl<K, V $(, $typaram)*> SerializeState for $ty<K, V $(, $typaram)*>
        where
            K: SerializeState $(+ $kbound1 $(+ $kbound2)*)*,
            V: SerializeState<Seed = K::Seed>,
            $($typaram: $bound,)*
        {
            type Seed = K::Seed;

            #[inline]
            fn serialize_state<S>(&self, serializer: S, seed: &Self::Seed) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.collect_map(self.into_iter().map(|(k, v)| (Seeded::new(seed, k), Seeded::new(seed, v))))
            }
        }
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
map_impl!(BTreeMap<K: Ord, V>);

#[cfg(feature = "std")]
map_impl!(HashMap<K: Eq + Hash, V, H: BuildHasher>);

////////////////////////////////////////////////////////////////////////////////
macro_rules! deref_impl {
    ($($desc:tt)+) => {
        impl $($desc)+ {
            type Seed = T::Seed;
            #[inline]
            fn serialize_state<S>(&self, serializer: S, seed: &Self::Seed) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                (**self).serialize_state(serializer, seed)
            }
        }
    };
}

deref_impl!(<'a, T: ?Sized> SerializeState for &'a T where T: SerializeState);
deref_impl!(<'a, T: ?Sized> SerializeState for &'a mut T where T: SerializeState);

#[cfg(any(feature = "std", feature = "alloc"))]
deref_impl!(<T: ?Sized> SerializeState for Box<T> where T: SerializeState);

#[cfg(all(feature = "rc", any(feature = "std", feature = "alloc")))]
deref_impl!(<T> SerializeState for Rc<T> where T: SerializeState);

#[cfg(all(feature = "rc", any(feature = "std", feature = "alloc")))]
deref_impl!(<T> SerializeState for Arc<T> where T: SerializeState);

#[cfg(any(feature = "std", feature = "collections"))]
deref_impl!(<'a, T: ?Sized> SerializeState for Cow<'a, T> where T: SerializeState + ToOwned);

////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "unstable")]
impl<T> SerializeState for NonZero<T>
where
    T: SerializeState + Zeroable + Clone,
{
    type Seed = T::Seed;

    #[inline]
    fn serialize_state<S>(&self, serializer: S, seed: &Self::Seed) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.clone().get().serialize(serializer, seed)
    }
}

impl<T> SerializeState for Cell<T>
where
    T: SerializeState + Copy,
{
    type Seed = T::Seed;

    #[inline]
    fn serialize_state<S>(&self, serializer: S, seed: &Self::Seed) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.get().serialize_state(serializer, seed)
    }
}

impl<T> SerializeState for RefCell<T>
where
    T: SerializeState,
{
    type Seed = T::Seed;

    #[inline]
    fn serialize_state<S>(&self, serializer: S, seed: &Self::Seed) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.borrow().serialize_state(serializer, seed)
    }
}

#[cfg(feature = "std")]
impl<T> SerializeState for Mutex<T>
where
    T: SerializeState,
{
    type Seed = T::Seed;

    #[inline]
    fn serialize_state<S>(&self, serializer: S, seed: &Self::Seed) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.lock() {
            Ok(locked) => locked.serialize_state(serializer, seed),
            Err(_) => Err(S::Error::custom("lock poison error while serializing")),
        }
    }
}

#[cfg(feature = "std")]
impl<T> SerializeState for RwLock<T>
where
    T: SerializeState,
{
    type Seed = T::Seed;

    #[inline]
    fn serialize_state<S>(&self, serializer: S, seed: &Self::Seed) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.read() {
            Ok(locked) => locked.serialize_state(serializer, seed),
            Err(_) => Err(S::Error::custom("lock poison error while serializing")),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<T, E> SerializeState for Result<T, E>
where
    T: SerializeState,
    E: SerializeState<Seed = T::Seed>,
{
    type Seed = T::Seed;

    #[inline]
    fn serialize_state<S>(&self, serializer: S, seed: &Self::Seed) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Result::Ok(ref value) => serializer.serialize_newtype_variant("Result", 0, "Ok", &Seeded::new(seed, value)),
            Result::Err(ref value) => {
                serializer.serialize_newtype_variant("Result", 1, "Err", &Seeded::new(seed, value))
            }
        }
    }
}

/// Placeholder
pub struct Seeded<'seed, S: ?Sized + 'seed, V> {
    /// Placeholder
    pub seed: &'seed S,
    /// Placeholder
    pub value: V,
}

impl<'seed, S: ?Sized, V> Seeded<'seed, S, V> {
    /// Placeholder
    #[inline]
    pub fn new(seed: &'seed S, value: V) -> Self {
        Seeded {
            seed: seed,
            value: value,
        }
    }
}

impl<'seed, T: ?Sized, V> Serialize for Seeded<'seed, T, V>
where
    V: SerializeState<Seed = T>,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.value.serialize_state(serializer, self.seed)
    }
}

/// Placeholder
pub struct Unseeded<T>(pub T);

impl<T> SerializeState for Unseeded<T>
where
    T: Serialize,
{
    type Seed = ();

    #[inline]
    fn serialize_state<S>(&self, serializer: S, _: &Self::Seed) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}
