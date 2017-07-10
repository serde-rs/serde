// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


mod seed_impls;
pub use self::seed_impls::{Seeded, Unseeded};

pub use serde::ser::*;
/// Placeholder
pub trait SerializeSeed {
    /// TODO Placeholder (use a type parameter instead?)
    type Seed: ?Sized;

    /// Placeholder
    fn serialize_seed<S>(&self, serializer: S, seed: &Self::Seed) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}
