// Untagged enum deserializing expects human-readable representation even with
// non human-readable data formats.
//
// https://github.com/serde-rs/serde/issues/2172

use serde_derive::{Deserialize, Serialize};
use serde_test::{assert_tokens, Configure, Token};

/// Type that serializes as a string when human-readable, or as bytes when compact.
#[derive(Debug, PartialEq)]
struct HrBytes(Vec<u8>);

impl serde::Serialize for HrBytes {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            hex_encode(&self.0).serialize(serializer)
        } else {
            self.0.serialize(serializer)
        }
    }
}

impl<'de> serde::Deserialize<'de> for HrBytes {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            hex_decode(&s).map(HrBytes).map_err(serde::de::Error::custom)
        } else {
            Vec::deserialize(deserializer).map(HrBytes)
        }
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum Untagged {
    Hr(HrBytes),
}

#[test]
fn human_readable() {
    assert_tokens(
        &Untagged::Hr(HrBytes(vec![1, 2])).readable(),
        &[Token::Str("0102")],
    );
}

#[test]
fn compact() {
    assert_tokens(
        &Untagged::Hr(HrBytes(vec![1, 2])).compact(),
        &[
            Token::Seq { len: Some(2) },
            Token::U8(1),
            Token::U8(2),
            Token::SeqEnd,
        ],
    );
}
