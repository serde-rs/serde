use serde::ser::SerializeSeq;
use serde_derive::Serialize;
use serde_test::{assert_ser_tokens, Token};

struct Hint;

impl serde::Serialize for Hint {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let seq = serializer.serialize_seq(Some(usize::MAX))?;
        seq.end()
    }
}

#[derive(Serialize)]
enum Inner {
    Data { items: Hint },
}

#[derive(Serialize)]
struct Outer {
    #[serde(flatten)]
    inner: Inner,
}

#[test]
fn test_content_serializer_does_not_preallocate_huge_length_hint() {
    // Serializing through `#[serde(flatten)]` buffers the flattened value
    // with `ContentSerializer`, which used to call `Vec::with_capacity`
    // with the untrusted length hint and panicked with "capacity overflow"
    // on a hint of `usize::MAX`.
    assert_ser_tokens(
        &Outer {
            inner: Inner::Data { items: Hint },
        },
        &[
            Token::Map { len: None },
            Token::Str("Data"),
            Token::Struct {
                name: "Data",
                len: 1,
            },
            Token::Str("items"),
            // The flattened value is buffered through `ContentSerializer`
            // first, so the re-serialized seq carries the collected length
            // (0), not the original `usize::MAX` hint.
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::StructEnd,
            Token::MapEnd,
        ],
    );
}
