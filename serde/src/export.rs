pub use lib::clone::Clone;
pub use lib::convert::{From, Into};
pub use lib::default::Default;
pub use lib::fmt::{self, Formatter};
pub use lib::marker::PhantomData;
pub use lib::option::Option::{self, None, Some};
pub use lib::result::Result::{self, Err, Ok};

pub use self::string::from_utf8_lossy;

#[cfg(any(feature = "alloc", feature = "std"))]
pub use lib::Vec;

mod string {
    use lib::*;

    #[cfg(any(feature = "std", feature = "alloc"))]
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
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    pub fn from_utf8_lossy(bytes: &[u8]) -> &str {
        // Three unicode replacement characters if it fails. They look like a
        // white-on-black question mark. The user will recognize it as invalid
        // UTF-8.
        str::from_utf8(bytes).unwrap_or("\u{fffd}\u{fffd}\u{fffd}")
    }
}
