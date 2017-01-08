#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", feature(plugin))]

extern crate syn;

pub mod ast;
pub mod attr;

mod ctxt;
pub use ctxt::Ctxt;
