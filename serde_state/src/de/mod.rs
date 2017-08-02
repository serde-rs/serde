// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Generic data structure deserialization framework.

use lib::*;

////////////////////////////////////////////////////////////////////////////////

mod seed_impls;

pub use self::seed_impls::{OptionSeed, SeqSeed, SeqSeedEx};

pub use serde::de::*;

/// `DeserializeState` is a trait which specifies how to deserialize a type which requires extra
/// state to deserialize
pub trait DeserializeState<'de, Seed: ?Sized>: Sized {
    /// Deserializes `Self` using `seed` and the `deserializer`
    fn deserialize_state<D>(seed: &mut Seed, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;
}

/// Wrapper type which implements `DeserializeSeed` for `DeserializeState` instances
#[derive(Debug)]
pub struct Seed<S, T> {
    /// The wrapped seed
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
    /// Constructs a new instance of `Seed`
    pub fn new(seed: S) -> Seed<S, T> {
        Seed {
            seed: seed,
            _marker: PhantomData,
        }
    }
}

impl<'de, 's, S, T> DeserializeSeed<'de> for Seed<&'s mut S, T>
where
    S: ?Sized,
    T: DeserializeState<'de, S>,
{
    type Value = T;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize_state(self.seed, deserializer)
    }
}
