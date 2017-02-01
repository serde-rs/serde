#[cfg(all(feature = "collections", not(feature = "std")))]
use collections::String;

#[cfg(feature = "std")]
use std::borrow::Cow;
#[cfg(all(feature = "collections", not(feature = "std")))]
use collections::borrow::Cow;

pub use core::default::Default;
pub use core::fmt;
pub use core::marker::PhantomData;
pub use core::option::Option::{self, None, Some};
pub use core::result::Result::{self, Ok, Err};

#[cfg(any(feature = "collections", feature = "std"))]
pub fn from_utf8_lossy(bytes: &[u8]) -> Cow<str> {
    String::from_utf8_lossy(bytes)
}

// The generated code calls this like:
//
//     let value = &_serde::export::from_utf8_lossy(bytes);
//     Err(_serde::de::Error::unknown_variant(value, VARIANTS))
//
// so it is okay for the return type to be different from the std case as long
// as the above works.
#[cfg(not(any(feature = "collections", feature = "std")))]
pub fn from_utf8_lossy(bytes: &[u8]) -> &str {
    use core::str;
    // Three unicode replacement characters if it fails. They look like a
    // white-on-black question mark. The user will recognize it as invalid
    // UTF-8.
    str::from_utf8(bytes).unwrap_or("\u{fffd}\u{fffd}\u{fffd}")
}
