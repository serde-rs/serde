#[cfg(all(not(no_serde_derive), any(feature = "std", feature = "alloc")))]
mod content;
mod seed;

#[doc(hidden)]
pub mod size_hint;

#[doc(hidden)]
pub mod string;

#[cfg(all(not(no_serde_derive), any(feature = "std", feature = "alloc")))]
#[doc(hidden)]
pub use self::content::{Content, ContentVisitor};
#[doc(hidden)]
pub use self::seed::InPlaceSeed;
