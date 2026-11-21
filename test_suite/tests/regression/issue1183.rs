#![allow(clippy::trivially_copy_pass_by_ref, dead_code)]

use std::collections::HashMap;

use serde_derive::Deserialize;

// https://github.com/serde-rs/serde/issues/1183
//
// Buffering by serde derive (internally tagged enums, untagged enums,
// #[serde(flatten)]) must not lose format-specific deserialization features.
// serde_json deserializes the string map key "1" into a u32, both when
// deserializing a HashMap<u32, _> directly and when the key goes through
// serde's internal Content buffering.

#[derive(Deserialize, PartialEq, Debug)]
#[serde(tag = "type")]
enum InternallyTagged {
    Map(HashMap<u32, ()>),
}

#[test]
fn test_internally_tagged_enum_non_string_map_key() {
    let j = r#"{"type": "Map", "1": null}"#;
    let expected = InternallyTagged::Map({
        let mut map = HashMap::new();
        map.insert(1u32, ());
        map
    });
    assert_eq!(serde_json::from_str::<InternallyTagged>(j).unwrap(), expected);
}

#[derive(Deserialize, PartialEq, Debug)]
struct Flat {
    #[serde(flatten)]
    extra: HashMap<u32, String>,
}

#[test]
fn test_flatten_non_string_map_key() {
    let j = r#"{"1": "one", "2": "two"}"#;
    let expected = Flat {
        extra: [
            (1u32, "one".to_owned()),
            (2u32, "two".to_owned()),
        ]
        .into_iter()
        .collect(),
    };
    assert_eq!(serde_json::from_str::<Flat>(j).unwrap(), expected);
}

#[derive(Deserialize, PartialEq, Eq, Hash, Debug)]
#[serde(untagged)]
enum Untagged {
    Number(u32),
    Text(String),
}

#[test]
fn test_untagged_enum_numeric_map_key() {
    // Map keys are buffered while trying untagged variants, so the key "3"
    // must still be usable as a u32.
    #[derive(Deserialize, PartialEq, Debug)]
    struct Wrapper {
        #[serde(flatten)]
        map: HashMap<Untagged, ()>,
    }

    let j = r#"{"3": null}"#;
    let expected = Wrapper {
        map: [(Untagged::Number(3), ())].into_iter().collect(),
    };
    assert_eq!(serde_json::from_str::<Wrapper>(j).unwrap(), expected);
}

#[test]
fn test_buffered_float_from_numeric_string() {
    // Self-describing formats like serde_yaml or serde_urlencoded natively
    // deserialize the scalar "1.5" into f64. Buffering through Content must not
    // lose that, so an untagged enum still matches the numeric variant.
    #[derive(Deserialize, PartialEq, Debug)]
    #[serde(untagged)]
    enum UntaggedFloat {
        Float(f64),
        Text(String),
    }

    let v: UntaggedFloat = serde_json::from_str(r#""1.5""#).unwrap();
    assert_eq!(v, UntaggedFloat::Float(1.5));
}

#[test]
fn test_buffered_negative_integer_map_key() {
    #[derive(Deserialize, PartialEq, Debug)]
    #[serde(tag = "type")]
    enum E {
        Map(HashMap<i32, ()>),
    }

    let j = r#"{"type": "Map", "-1": null}"#;
    let expected = E::Map([(-1i32, ())].into_iter().collect());
    assert_eq!(serde_json::from_str::<E>(j).unwrap(), expected);
}

#[test]
fn test_buffered_integer_out_of_range_still_errors() {
    #[derive(Deserialize, PartialEq, Debug)]
    #[serde(tag = "type")]
    enum E {
        Small(HashMap<u8, ()>),
    }

    let j = r#"{"type": "Small", "256": null}"#;
    let err = serde_json::from_str::<E>(j).unwrap_err();
    assert!(
        err.to_string().contains("invalid value: integer `256`, expected u8"),
        "unexpected error: {}",
        err
    );
}

#[test]
fn test_buffered_non_numeric_string_still_errors() {
    #[derive(Deserialize, PartialEq, Debug)]
    #[serde(tag = "type")]
    enum E {
        Map(HashMap<u32, ()>),
    }

    let j = r#"{"type": "Map", "x": null}"#;
    let err = serde_json::from_str::<E>(j).unwrap_err();
    assert!(
        err.to_string().contains("invalid type: string \"x\", expected u32"),
        "unexpected error: {}",
        err
    );
}
