// human_readable flag does not get carried through #[serde(flatten)].
//
// https://github.com/serde-rs/serde/issues/2704

use serde_derive::{Deserialize, Serialize};
use serde_test::{assert_tokens, Configure, Token};

/// Type that serializes as a string when human-readable, or as a u32 when compact.
#[derive(Debug, PartialEq)]
struct HrU32(u32);

impl serde::Serialize for HrU32 {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            self.0.to_string().serialize(serializer)
        } else {
            self.0.serialize(serializer)
        }
    }
}

impl<'de> serde::Deserialize<'de> for HrU32 {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            s.parse().map(HrU32).map_err(serde::de::Error::custom)
        } else {
            u32::deserialize(deserializer).map(HrU32)
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Inner {
    value: HrU32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Outer {
    #[serde(flatten)]
    inner: Inner,
}

#[test]
fn human_readable() {
    assert_tokens(
        &Outer {
            inner: Inner {
                value: HrU32(42),
            },
        }
        .readable(),
        &[
            Token::Map { len: None },
            Token::Str("value"),
            Token::Str("42"),
            Token::MapEnd,
        ],
    );
}

#[test]
fn compact() {
    assert_tokens(
        &Outer {
            inner: Inner {
                value: HrU32(42),
            },
        }
        .compact(),
        &[
            Token::Map { len: None },
            Token::Str("value"),
            Token::U32(42),
            Token::MapEnd,
        ],
    );
}
