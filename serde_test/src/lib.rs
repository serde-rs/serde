#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

extern crate serde;

mod assert;
pub use assert::{
    assert_tokens,
    assert_ser_tokens,
    assert_ser_tokens_error,
    assert_de_tokens,
    assert_de_tokens_error,
};

mod ser;
pub use ser::Serializer;

mod de;
pub use de::Deserializer;

mod token;
pub use token::Token;

mod error;
pub use error::Error;
