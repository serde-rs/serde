#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(not(feature = "with-syntex"), feature(rustc_private, plugin))]

#[cfg(feature = "with-syntex")]
#[macro_use]
extern crate syntex_syntax as syntax;
#[cfg(feature = "with-syntex")]
extern crate syntex_errors as errors;

#[cfg(not(feature = "with-syntex"))]
#[macro_use]
extern crate syntax;
#[cfg(not(feature = "with-syntex"))]
extern crate rustc_errors as errors;

pub mod ast;
pub mod attr;

mod error;
pub use error::Error;
