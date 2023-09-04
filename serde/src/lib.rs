#[doc(inline)]
pub use serde_core::*;

// Re-export #[derive(Serialize, Deserialize)].
//
// The reason re-exporting is not enabled by default is that disabling it would
// be annoying for crates that provide handwritten impls or data formats. They
// would need to disable default features and then explicitly re-enable std.
#[cfg(feature = "serde_derive")]
extern crate serde_derive;

/// Derive macro available if serde is built with `features = ["derive"]`.
#[cfg(feature = "serde_derive")]
pub use serde_derive::{Deserialize, Serialize};
