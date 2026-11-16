use serde::de::value::{Error, SeqDeserializer};
use serde::de::Deserialize;
use serde_derive::Deserialize;
use serde_test::{assert_de_tokens, Token};

// https://github.com/serde-rs/serde/issues/3112
// Deriving Deserialize for a packed struct must compile when the
// `deserialize_in_place` feature is enabled. The derived in-place impl
// takes `&mut` references to individual fields, which is E0793
// (unaligned reference) for packed structs, so the derive falls back to
// the trait's default `deserialize_in_place` for packed types.
#[derive(Debug, PartialEq, Deserialize)]
#[repr(C, packed)]
struct Packed {
    tag: u8,
    value: u64,
}

#[derive(Debug, PartialEq, Deserialize)]
#[repr(C, packed)]
struct PackedTuple(u8, u64);

// Homogeneous field types so that a SeqDeserializer can drive
// `deserialize_in_place` for these packed types.
#[derive(Debug, PartialEq, Deserialize)]
#[repr(C, packed)]
struct PackedU64 {
    first: u64,
    second: u64,
}

#[derive(Debug, PartialEq, Deserialize)]
#[repr(C, packed)]
struct PackedTupleU64(u64, u64);

#[test]
fn packed_struct_deserialize() {
    assert_de_tokens(
        &Packed { tag: 1, value: 2 },
        &[
            Token::Struct {
                name: "Packed",
                len: 2,
            },
            Token::Str("tag"),
            Token::U8(1),
            Token::Str("value"),
            Token::U64(2),
            Token::StructEnd,
        ],
    );
}

#[test]
fn packed_tuple_struct_deserialize() {
    assert_de_tokens(
        &PackedTuple(3, 4),
        &[
            Token::TupleStruct {
                name: "PackedTuple",
                len: 2,
            },
            Token::U8(3),
            Token::U64(4),
            Token::TupleStructEnd,
        ],
    );
}

#[test]
fn packed_struct_deserialize_in_place() {
    let mut place = PackedU64 {
        first: 0,
        second: 0,
    };
    let seq = SeqDeserializer::<_, Error>::new(vec![1u64, 2].into_iter());
    PackedU64::deserialize_in_place(seq, &mut place).unwrap();
    assert_eq!(
        place,
        PackedU64 {
            first: 1,
            second: 2
        }
    );
}

#[test]
fn packed_tuple_struct_deserialize_in_place() {
    let mut place = PackedTupleU64(0, 0);
    let seq = SeqDeserializer::<_, Error>::new(vec![3u64, 4].into_iter());
    PackedTupleU64::deserialize_in_place(seq, &mut place).unwrap();
    assert_eq!(place, PackedTupleU64(3, 4));
}
