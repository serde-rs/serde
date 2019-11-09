// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Generic data structure serialization framework.

mod seed_impls;
pub use self::seed_impls::{Seeded, Unseeded};

pub use serde::ser::*;
/// Stateful variant of serdeäs `Serialize` trait
pub trait SerializeState<State: ?Sized> {
    /// Serializes `self`
    fn serialize_state<S>(&self, serializer: S, state: &State) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}
