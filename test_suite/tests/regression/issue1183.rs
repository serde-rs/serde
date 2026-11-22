#![allow(clippy::trivially_copy_pass_by_ref, dead_code)]

use std::collections::HashMap;

use serde::Deserialize;

// https://github.com/serde-rs/serde/issues/1183
//
// Buffering by serde derive (internally tagged enums, untagged enums,
// #[serde(flatten)]) must not lose format-specific map-key deserialization.
// serde_json deserializes the string map key "1" into a u32, both when
// deserializing a HashMap<u32, _> directly and when the key goes through
// serde's internal Content buffering. The coercion is limited to the map-key
// replay path: buffered values keep serde's strict string/number distinction.

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
fn test_untagged_enum_string_key_stays_string() {
    // Values keep serde's strict data model: a buffered JSON string is not
    // coerced to a number outside the map-key path, so an untagged enum key
    // still matches its text variant.
    #[derive(Deserialize, PartialEq, Debug)]
    struct Wrapper {
        #[serde(flatten)]
        map: HashMap<Untagged, ()>,
    }

    let j = r#"{"3": null}"#;
    let expected = Wrapper {
        map: [(Untagged::Text("3".to_owned()), ())].into_iter().collect(),
    };
    assert_eq!(serde_json::from_str::<Wrapper>(j).unwrap(), expected);
}

#[derive(Debug)]
struct OrderedFloat(f64);

impl<'de> Deserialize<'de> for OrderedFloat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(OrderedFloat(f64::deserialize(deserializer)?))
    }
}

impl PartialEq for OrderedFloat {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}
impl Eq for OrderedFloat {}
impl std::hash::Hash for OrderedFloat {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

#[test]
fn test_buffered_float_map_key_from_numeric_string() {
    // Map keys are the one place where serde_json accepts numbers written as
    // strings, so the buffered key "2.5" still deserializes into f64.
    #[derive(Deserialize, PartialEq, Debug)]
    struct Flat {
        #[serde(flatten)]
        extra: HashMap<OrderedFloat, String>,
    }

    let j = r#"{"2.5": "x"}"#;
    let expected = Flat {
        extra: [(OrderedFloat(2.5), "x".to_owned())].into_iter().collect(),
    };
    assert_eq!(serde_json::from_str::<Flat>(j).unwrap(), expected);
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

#[test]
fn test_buffered_string_value_is_not_coerced_to_number() {
    // Outside the map-key path a buffered string stays a string, matching
    // serde_json's own behavior for values.
    #[derive(Deserialize, PartialEq, Debug)]
    #[serde(untagged)]
    enum UntaggedFloat {
        Float(f64),
        Text(String),
    }

    let v: UntaggedFloat = serde_json::from_str(r#""1.5""#).unwrap();
    assert_eq!(v, UntaggedFloat::Text("1.5".to_owned()));
}

#[test]
fn test_flatten_bool_map_key() {
    // serde_json parses the map keys "true"/"false" into bool when reading a
    // HashMap<bool, _> directly; the buffered flatten replay must do the same.
    #[derive(Deserialize, PartialEq, Debug)]
    struct Flat {
        #[serde(flatten)]
        extra: HashMap<bool, String>,
    }

    let j = r#"{"true": "yes", "false": "no"}"#;
    let expected = Flat {
        extra: [
            (true, "yes".to_owned()),
            (false, "no".to_owned()),
        ]
        .into_iter()
        .collect(),
    };
    assert_eq!(serde_json::from_str::<Flat>(j).unwrap(), expected);
}

#[test]
fn test_flatten_char_map_key() {
    // serde_json parses a single-character map key into char; the buffered
    // flatten replay must do the same via the visitor's visit_str fallback.
    #[derive(Deserialize, PartialEq, Debug)]
    struct Flat {
        #[serde(flatten)]
        extra: HashMap<char, String>,
    }

    let j = r#"{"a": "letter"}"#;
    let expected = Flat {
        extra: [('a', "letter".to_owned())].into_iter().collect(),
    };
    assert_eq!(serde_json::from_str::<Flat>(j).unwrap(), expected);
}

#[test]
fn test_buffered_non_bool_string_still_errors() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Flat {
        #[serde(flatten)]
        extra: HashMap<bool, ()>,
    }

    let j = r#"{"x": null}"#;
    let err = serde_json::from_str::<Flat>(j).unwrap_err();
    assert!(
        err.to_string().contains("invalid type: string \"x\", expected a boolean"),
        "unexpected error: {}",
        err
    );
}
