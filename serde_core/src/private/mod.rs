#[cfg(all(not(no_serde_derive), any(feature = "std", feature = "alloc")))]
mod content;

#[cfg(all(not(no_serde_derive), any(feature = "std", feature = "alloc")))]
#[doc(hidden)]
pub use self::content::{Content, ContentVisitor};
