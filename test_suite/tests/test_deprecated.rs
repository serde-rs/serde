// Asset that derived `Serialize` and `Deserialize` impls don't emit
// deprecation warnings when the type itself is deprecated.
// This test will fail to compile if a deprecation warning is emitted.

#![deny(deprecated)]

use serde::{Deserialize, Serialize};

#[deprecated]
#[derive(Serialize, Deserialize)]
pub struct DeprecatedItem {}
