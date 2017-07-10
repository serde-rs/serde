// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use lib::*;

////////////////////////////////////////////////////////////////////////////////

mod seed_impls;

pub use self::seed_impls::{OptionSeed, SeqSeed, SeqSeedEx};

use serde::de::*;

/// TODO
pub trait DeserializeSeedEx<'de, Seed>: Sized {
    /// TODO
    fn deserialize_seed<D>(seed: &mut Seed, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;
}

/// TODO
#[derive(Debug)]
pub struct Seed<S, T> {
    /// TODO
    pub seed: S,
    _marker: PhantomData<T>,
}

impl<S, T> Clone for Seed<S, T>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Seed {
            seed: self.seed.clone(),
            _marker: PhantomData,
        }
    }
}

impl<S, T> Copy for Seed<S, T>
where
    S: Copy,
{
}

impl<S, T, U> AsMut<U> for Seed<S, T>
where
    S: AsMut<U>
{
    fn as_mut(&mut self) -> &mut U {
        self.seed.as_mut()
    }
}

impl<S, T> Seed<S, T> {
    /// TODO
    pub fn new(seed: S) -> Seed<S, T> {
        Seed {
            seed: seed,
            _marker: PhantomData,
        }
    }
}

impl<'de, 's, S, T> DeserializeSeed<'de> for Seed<&'s mut S, T>
where
    T: DeserializeSeedEx<'de, S>,
{
    type Value = T;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize_seed(self.seed, deserializer)
    }
}

/// TODO
pub struct SharedSeed<S, T> {
    /// TODO
    pub seed: ::std::cell::RefCell<S>,
    _marker: PhantomData<fn () -> T>,
}

impl<S, T> SharedSeed<S, T> {
    /// TODO
    pub fn new(seed: S) -> Self {
        SharedSeed {
            seed: ::std::cell::RefCell::new(seed),
            _marker: PhantomData,
        }
    }
}

impl<'de, 's, 't, S, T> DeserializeSeed<'de> for &'s SharedSeed<&'t mut S, T>
where
    T: DeserializeSeedEx<'de, S>,
{
    type Value = T;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize_seed(&mut self.seed.borrow_mut(), deserializer)
    }
}
