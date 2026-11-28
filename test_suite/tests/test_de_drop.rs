//! Every value that a derived `Deserialize` impl creates is dropped exactly
//! once, whether deserializing the struct succeeds or fails.

use serde::de::{Deserialize, Deserializer};
use serde_derive::Deserialize;
use serde_test::{assert_de_tokens, assert_de_tokens_error, Token};
use std::cell::Cell;

thread_local! {
    static CREATED: Cell<usize> = const { Cell::new(0) };
    static DROPPED: Cell<usize> = const { Cell::new(0) };
}

#[derive(PartialEq, Debug)]
struct Counted(String);

impl Counted {
    fn new(value: &str) -> Self {
        CREATED.with(|created| created.set(created.get() + 1));
        Counted(value.to_owned())
    }
}

impl<'de> Deserialize<'de> for Counted {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Counted::new(&value))
    }
}

impl Default for Counted {
    fn default() -> Self {
        Counted::new("")
    }
}

impl Drop for Counted {
    fn drop(&mut self) {
        DROPPED.with(|dropped| dropped.set(dropped.get() + 1));
    }
}

#[derive(PartialEq, Debug, Deserialize)]
struct Four {
    a: Counted,
    b: Counted,
    #[serde(default)]
    c: Counted,
    d: Counted,
}

/// Runs `f` and returns how many `Counted` values it created, after checking
/// that it dropped all of them.
fn created_and_dropped(f: impl FnOnce()) -> usize {
    CREATED.with(|created| created.set(0));
    DROPPED.with(|dropped| dropped.set(0));
    f();
    let created = CREATED.with(Cell::get);
    let dropped = DROPPED.with(Cell::get);
    assert_eq!(
        created, dropped,
        "created {created} values, dropped {dropped}"
    );
    created
}

#[test]
fn test_all_fields() {
    created_and_dropped(|| {
        assert_de_tokens(
            &Four {
                a: Counted::new("a"),
                b: Counted::new("b"),
                c: Counted::new("c"),
                d: Counted::new("d"),
            },
            &[
                Token::Struct {
                    name: "Four",
                    len: 4,
                },
                Token::Str("a"),
                Token::Str("a"),
                Token::Str("b"),
                Token::Str("b"),
                Token::Str("c"),
                Token::Str("c"),
                Token::Str("d"),
                Token::Str("d"),
                Token::StructEnd,
            ],
        );
    });
}

#[test]
fn test_missing_field_after_default() {
    let created = created_and_dropped(|| {
        assert_de_tokens_error::<Four>(
            &[
                Token::Map { len: None },
                Token::Str("b"),
                Token::Str("b"),
                Token::Str("a"),
                Token::Str("a"),
                Token::MapEnd,
            ],
            "missing field `d`",
        );
    });
    // `a`, `b` and the default value of `c`.
    assert_eq!(created, 3);
}

#[test]
fn test_missing_first_field() {
    let created = created_and_dropped(|| {
        assert_de_tokens_error::<Four>(
            &[
                Token::Map { len: None },
                Token::Str("d"),
                Token::Str("d"),
                Token::Str("b"),
                Token::Str("b"),
                Token::MapEnd,
            ],
            "missing field `a`",
        );
    });
    assert_eq!(created, 2);
}

#[test]
fn test_duplicate_field() {
    let created = created_and_dropped(|| {
        assert_de_tokens_error::<Four>(
            &[
                Token::Map { len: None },
                Token::Str("a"),
                Token::Str("a"),
                Token::Str("c"),
                Token::Str("c"),
                Token::Str("a"),
            ],
            "duplicate field `a`",
        );
    });
    assert_eq!(created, 2);
}

#[test]
fn test_invalid_field_value() {
    let created = created_and_dropped(|| {
        assert_de_tokens_error::<Four>(
            &[
                Token::Map { len: None },
                Token::Str("a"),
                Token::Str("a"),
                Token::Str("b"),
                Token::Str("b"),
                Token::Str("d"),
                Token::I32(1),
            ],
            "invalid type: integer `1`, expected a string",
        );
    });
    assert_eq!(created, 2);
}

#[derive(PartialEq, Debug, Deserialize)]
struct WithDeserializeWith {
    a: Counted,
    #[serde(deserialize_with = "Counted::deserialize")]
    b: Counted,
    c: Counted,
}

#[test]
fn test_missing_field_with_deserialize_with() {
    let created = created_and_dropped(|| {
        assert_de_tokens_error::<WithDeserializeWith>(
            &[
                Token::Map { len: None },
                Token::Str("c"),
                Token::Str("c"),
                Token::Str("a"),
                Token::Str("a"),
                Token::MapEnd,
            ],
            "missing field `b`",
        );
    });
    assert_eq!(created, 2);
}
