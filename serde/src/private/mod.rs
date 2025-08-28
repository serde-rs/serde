#[cfg(not(no_serde_derive))]
pub mod de;
#[cfg(not(no_serde_derive))]
pub mod ser;

// FIXME: #[cfg(doctest)] once https://github.com/rust-lang/rust/issues/67295 is fixed.
pub mod doc;

pub use crate::lib::clone::Clone;
pub use crate::lib::convert::{From, Into, TryFrom};
pub use crate::lib::default::Default;
pub use crate::lib::fmt::{self, Formatter};
pub use crate::lib::marker::PhantomData;
pub use crate::lib::option::Option::{self, None, Some};
pub use crate::lib::ptr;
pub use crate::lib::result::Result::{self, Err, Ok};

pub use self::string::from_utf8_lossy;

#[cfg(any(feature = "alloc", feature = "std"))]
pub use crate::lib::{ToString, Vec};

mod string {
    pub use serde_core::from_utf8_lossy;
}
