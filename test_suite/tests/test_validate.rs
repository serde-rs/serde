use serde_derive::{Deserialize, Serialize};
use serde_test::{assert_de_tokens, assert_de_tokens_error, Token};
use std::fmt::Display;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(validate = "validate_struct")]
struct Struct {
    a: u16,
}

fn validate_struct(deserialized: &Struct) -> Result<(), impl Display> {
    if deserialized.a == 0 {
        return Err("field `a` can not be zero");
    }
    Ok(())
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(validate = "validate_tuple_struct")]
struct TupleStruct(u16);

fn validate_tuple_struct(deserialized: &TupleStruct) -> Result<(), impl Display> {
    if deserialized.0 == 0 {
        return Err("field `0` can not be zero");
    }
    Ok(())
}

#[test]
fn test_struct() {
    assert_de_tokens(
        &Struct { a: 1 },
        &[
            Token::Struct {
                name: "Struct",
                len: 1,
            },
            Token::Str("a"),
            Token::U16(1),
            Token::StructEnd,
        ],
    );

    assert_de_tokens_error::<Struct>(
        &[
            Token::Struct {
                name: "Struct",
                len: 1,
            },
            Token::Str("a"),
            Token::U16(0),
            Token::StructEnd,
        ],
        "field `a` can not be zero",
    );
}

#[test]
fn test_tuple_struct() {
    assert_de_tokens(
        &TupleStruct(1),
        &[
            Token::TupleStruct {
                name: "TupleStruct",
                len: 1,
            },
            Token::U16(1),
            Token::TupleStructEnd,
        ],
    );

    assert_de_tokens_error::<TupleStruct>(
        &[
            Token::TupleStruct {
                name: "TupleStruct",
                len: 1,
            },
            Token::U16(0),
            Token::TupleStructEnd,
        ],
        "field `0` can not be zero",
    );
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(validate = "validate_struct_variant")]
enum StructVariant {
    Struct { a: u16 },
}

fn validate_struct_variant(deserialized: &StructVariant) -> Result<(), impl Display> {
    if let StructVariant::Struct { a: 0 } = deserialized {
        return Err("variant `Struct.a` can not be zero");
    }
    Ok(())
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(validate = "validate_tuple_variant")]
enum TupleVariant {
    A(u16, u16),
}

fn validate_tuple_variant(deserialized: &TupleVariant) -> Result<(), impl Display> {
    if let TupleVariant::A(0, _) = deserialized {
        return Err("variant `A.0` can not be zero");
    }
    Ok(())
}

#[test]
fn test_struct_variant() {
    assert_de_tokens(
        &StructVariant::Struct { a: 1 },
        &[
            Token::StructVariant {
                name: "StructVariant",
                variant: "Struct",
                len: 1,
            },
            Token::Str("a"),
            Token::U16(1),
            Token::StructVariantEnd,
        ],
    );

    assert_de_tokens_error::<StructVariant>(
        &[
            Token::StructVariant {
                name: "StructVariant",
                variant: "Struct",
                len: 1,
            },
            Token::Str("a"),
            Token::U16(0),
            Token::StructVariantEnd,
        ],
        "variant `Struct.a` can not be zero",
    );
}

#[test]
fn test_tuple_variant() {
    assert_de_tokens(
        &TupleVariant::A(1, 1),
        &[
            Token::TupleVariant {
                name: "TupleVariant",
                variant: "A",
                len: 2,
            },
            Token::U16(1),
            Token::U16(1),
            Token::TupleVariantEnd,
        ],
    );

    assert_de_tokens_error::<TupleVariant>(
        &[
            Token::TupleVariant {
                name: "TupleVariant",
                variant: "A",
                len: 2,
            },
            Token::U16(0),
            Token::U16(1),
            Token::TupleVariantEnd,
        ],
        "variant `A.0` can not be zero",
    );
}

#[derive(Debug, PartialEq, validator::Validate, Deserialize)]
#[serde(validate = "validator::Validate::validate")]
struct ValidatorDemo {
    #[validate(email)]
    mail: String,
}

#[test]
fn test_validator_demo() {
    assert_de_tokens(
        &ValidatorDemo {
            mail: "email@example.com".into(),
        },
        &[
            Token::Struct {
                name: "ValidatorDemo",
                len: 1,
            },
            Token::Str("mail"),
            Token::Str("email@example.com"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens_error::<ValidatorDemo>(
        &[
            Token::Struct {
                name: "ValidatorDemo",
                len: 1,
            },
            Token::Str("mail"),
            Token::Str("email.example.com"),
            Token::StructEnd,
        ],
        "mail: Validation error: email [{\"value\": String(\"email.example.com\")}]",
    );
}
