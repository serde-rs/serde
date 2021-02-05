// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
#![allow(deprecated)] // For try!

//! # serde_state
//!
//! `serde_state` is a crate which extends the normal `Deserialize` and `Serialize` traits to allow
//! state to be passed to every value which is serialized or deserialized.
//!
//! ## Example
//!
//! ```
//! extern crate serde_json;
//! extern crate serde_state as serde;
//! #[macro_use]
//! extern crate serde_derive;
//! #[macro_use]
//! extern crate serde_derive_state;
//!
//! use std::borrow::BorrowMut;
//! use std::cell::Cell;
//! use serde::ser::{Serialize, Serializer, SerializeState};
//! use serde::de::{Deserialize, Deserializer, DeserializeState};
//!
//! #[derive(Deserialize, Serialize)]
//! struct Inner;
//!
//! impl SerializeState<Cell<i32>> for Inner {
//!
//!     fn serialize_state<S>(&self, serializer: S, seed: &Cell<i32>) -> Result<S::Ok, S::Error>
//!     where
//!         S: Serializer,
//!     {
//!         seed.set(seed.get() + 1);
//!         self.serialize(serializer)
//!     }
//! }
//!
//! impl<'de, S> DeserializeState<'de, S> for Inner where S: BorrowMut<i32> {
//!
//!     fn deserialize_state<D>(seed: &mut S, deserializer: D) -> Result<Self, D::Error>
//!     where
//!         D: Deserializer<'de>,
//!     {
//!         *seed.borrow_mut() += 1;
//!         Self::deserialize(deserializer)
//!     }
//! }
//!
//! #[derive(SerializeState, DeserializeState)]
//!
//! // `serialize_state` or `deserialize_state` is necessary to tell the derived implementation which
//! // seed that is passed
//! #[serde(serialize_state = "Cell<i32>")]
//!
//! // `de_parameters` can be used to specify additional type parameters for the derived instance
//! #[serde(de_parameters = "S")]
//! #[serde(bound(deserialize = "S: BorrowMut<i32>"))]
//! #[serde(deserialize_state = "S")]
//! struct Struct {
//!     // The `serialize_state` attribute must be specified to use seeded serialization
//!     #[serde(serialize_state)]
//!     // The `deserialize_state` attribute must be specified to use seeded deserialization
//!     #[serde(deserialize_state)]
//!     value: Inner,
//!
//!     // The `seed` attribute can be used to specify `deserialize_state` and `serialize_state`
//!     // simultaneously
//!     #[serde(state)]
//!     value2: Inner,
//!
//!     // If no attributes are specified then normal serialization and/or deserialization is used
//!     value3: Inner,
//!
//!     // The `[de]serialize_state_with` attribute can be used to specify a custom function which
//!     // does the serialization or deserialization
//!     #[serde(serialize_state_with = "serialize_inner")]
//!     value4: Inner,
//! }
//!
//! fn serialize_inner<S>(self_: &Inner, serializer: S, seed: &Cell<i32>) -> Result<S::Ok, S::Error>
//!     where S: Serializer
//! {
//!     seed.set(seed.get() + 10);
//!     self_.serialize(serializer)
//! }
//!
//! fn main() {
//!     let s = Struct {
//!         value: Inner,
//!         value2: Inner,
//!         value3: Inner,
//!         value4: Inner,
//!     };
//!
//!     let mut buffer = Vec::new();
//!     {
//!         let mut serializer = serde_json::Serializer::pretty(&mut buffer);
//!         let seed = Cell::new(0);
//!         s.serialize_state(&mut serializer, &seed).unwrap();
//!         assert_eq!(seed.get(), 12);
//!     }
//!     {
//!         let mut deserializer = serde_json::Deserializer::from_slice(&buffer);
//!         let mut seed = 0;
//!         Struct::deserialize_state(&mut seed, &mut deserializer).unwrap();
//!         assert_eq!(seed, 2);
//!     }
//! }
//!
//! ```

////////////////////////////////////////////////////////////////////////////////

// Serde types in rustdoc of other crates get linked to here.
#![doc(html_root_url = "https://docs.rs/serde/1.0.8")]
// Support using Serde without the standard library!
#![cfg_attr(not(feature = "std"), no_std)]
// Unstable functionality only if the user asks for it. For tracking and
// discussion of these features please refer to this issue:
//
//    https://github.com/serde-rs/serde/issues/812
#![cfg_attr(feature = "unstable", feature(never_type))]
// Whitelisted clippy lints.
#![cfg_attr(feature = "cargo-clippy", allow(doc_markdown))]
#![cfg_attr(feature = "cargo-clippy", allow(linkedlist))]
#![cfg_attr(feature = "cargo-clippy", allow(type_complexity))]
#![cfg_attr(feature = "cargo-clippy", allow(zero_prefixed_literal))]
// Blacklisted Rust lints.
#![deny(missing_docs, unused_imports)]

////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(all(feature = "unstable", feature = "std"))]
extern crate core;

#[macro_use]
extern crate serde;

/// A facade around all the types we need from the `std`, `core`, `alloc`, and
/// `collections` crates. This avoids elaborate import wrangling having to
/// happen in every module.
mod lib {
    mod core {
        #[cfg(not(feature = "std"))]
        pub use core::*;
        #[cfg(feature = "std")]
        pub use std::*;
    }

    pub use self::core::{cmp, iter, mem, ops, slice, str};
    pub use self::core::{f32, f64};
    pub use self::core::{i16, i32, i64, i8, isize};
    pub use self::core::{u16, u32, u64, u8, usize};

    pub use self::core::cell::{Cell, RefCell};
    pub use self::core::clone::{self, Clone};
    pub use self::core::convert::{self, From, Into};
    pub use self::core::default::{self, Default};
    pub use self::core::fmt::{self, Debug, Display};
    pub use self::core::marker::{self, PhantomData};
    pub use self::core::option::{self, Option};
    pub use self::core::result::{self, Result};

    #[cfg(all(feature = "alloc", not(feature = "std")))]
    pub use collections::borrow::{Cow, ToOwned};
    #[cfg(feature = "std")]
    pub use std::borrow::{Cow, ToOwned};

    #[cfg(all(feature = "alloc", not(feature = "std")))]
    pub use collections::string::{String, ToString};
    #[cfg(feature = "std")]
    pub use std::string::String;

    #[cfg(all(feature = "alloc", not(feature = "std")))]
    pub use collections::vec::Vec;
    #[cfg(feature = "std")]
    pub use std::vec::Vec;

    #[cfg(all(feature = "alloc", not(feature = "std")))]
    pub use alloc::boxed::Box;
    #[cfg(feature = "std")]
    pub use std::boxed::Box;

    #[cfg(all(feature = "rc", feature = "alloc", not(feature = "std")))]
    pub use alloc::rc::Rc;
    #[cfg(all(feature = "rc", feature = "std"))]
    pub use std::rc::Rc;

    #[cfg(all(feature = "rc", feature = "alloc", not(feature = "std")))]
    pub use alloc::arc::Arc;
    #[cfg(all(feature = "rc", feature = "std"))]
    pub use std::sync::Arc;

    #[cfg(all(feature = "alloc", not(feature = "std")))]
    pub use collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque};
    #[cfg(feature = "std")]
    pub use std::collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque};

    #[cfg(feature = "std")]
    pub use std::{error, net};

    #[cfg(feature = "std")]
    pub use std::collections::{HashMap, HashSet};
    #[cfg(feature = "std")]
    pub use std::ffi::{CStr, CString, OsStr, OsString};
    #[cfg(feature = "std")]
    pub use std::hash::{BuildHasher, Hash};
    #[cfg(feature = "std")]
    pub use std::io::Write;
    #[cfg(feature = "std")]
    pub use std::path::{Path, PathBuf};
    #[cfg(feature = "std")]
    pub use std::sync::{Mutex, RwLock};
    #[cfg(feature = "std")]
    pub use std::time::Duration;
}

////////////////////////////////////////////////////////////////////////////////

pub mod de;
pub mod ser;

#[doc(hidden)]
pub mod private;

#[doc(hidden)]
pub use serde::*;

#[doc(inline)]
pub use de::{Deserialize, DeserializeState, Deserializer};
#[doc(inline)]
pub use ser::{Serialize, SerializeState, Serializer};
