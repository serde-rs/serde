#![feature(custom_derive, plugin, test, custom_attribute)]
#![plugin(serde_macros)]

extern crate test;
extern crate serde;

use std::fmt::Debug;
use std::collections::BTreeMap;

use serde::de;
use serde::ser;

use serde::json::{
    self,
    Value,
    from_str,
    from_value,
    to_value,
};

use serde::json::error::{Error, ErrorCode};

macro_rules! treemap {
    ($($k:expr => $v:expr),*) => ({
        let mut _m = BTreeMap::new();
        $(_m.insert($k, $v);)*
        _m
    })
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum Animal {
    Dog,
    Frog(String, Vec<isize>),
    Cat { age: usize, name: String },

}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Inner {
    a: (),
    b: usize,
    c: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Outer {
    inner: Vec<Inner>,
}

fn test_encode_ok<T>(errors: &[(T, &str)])
    where T: PartialEq + Debug + ser::Serialize,
{
    for &(ref value, out) in errors {
        let out = out.to_string();

        let s = json::to_string(value).unwrap();
        assert_eq!(s, out);

        let v = to_value(&value);
        let s = json::to_string(&v).unwrap();
        assert_eq!(s, out);
    }
}

fn test_pretty_encode_ok<T>(errors: &[(T, &str)])
    where T: PartialEq + Debug + ser::Serialize,
{
    for &(ref value, out) in errors {
        let out = out.to_string();

        let s = json::to_string_pretty(value).unwrap();
        assert_eq!(s, out);

        let v = to_value(&value);
        let s = json::to_string_pretty(&v).unwrap();
        assert_eq!(s, out);
    }
}

#[test]
fn test_write_null() {
    let tests = &[
        ((), "null"),
    ];
    test_encode_ok(tests);
    test_pretty_encode_ok(tests);
}

#[test]
fn test_write_i64() {
    let tests = &[
        (3i64, "3"),
        (-2i64, "-2"),
        (-1234i64, "-1234"),
    ];
    test_encode_ok(tests);
    test_pretty_encode_ok(tests);
}

#[test]
fn test_write_f64() {
    let tests = &[
        (3.0, "3.0"),
        (3.1, "3.1"),
        (-1.5, "-1.5"),
        (0.5, "0.5"),
    ];
    test_encode_ok(tests);
    test_pretty_encode_ok(tests);
}

#[test]
fn test_write_str() {
    let tests = &[
        ("", "\"\""),
        ("foo", "\"foo\""),
    ];
    test_encode_ok(tests);
    test_pretty_encode_ok(tests);
}

#[test]
fn test_write_bool() {
    let tests = &[
        (true, "true"),
        (false, "false"),
    ];
    test_encode_ok(tests);
    test_pretty_encode_ok(tests);
}

#[test]
fn test_write_list() {
    test_encode_ok(&[
        (vec![], "[]"),
        (vec![true], "[true]"),
        (vec![true, false], "[true,false]"),
    ]);

    test_encode_ok(&[
        (vec![vec![], vec![], vec![]], "[[],[],[]]"),
        (vec![vec![1, 2, 3], vec![], vec![]], "[[1,2,3],[],[]]"),
        (vec![vec![], vec![1, 2, 3], vec![]], "[[],[1,2,3],[]]"),
        (vec![vec![], vec![], vec![1, 2, 3]], "[[],[],[1,2,3]]"),
    ]);

    test_pretty_encode_ok(&[
        (
            vec![vec![], vec![], vec![]],
            concat!(
                "[\n",
                "  [],\n",
                "  [],\n",
                "  []\n",
                "]"
            ),
        ),
        (
            vec![vec![1, 2, 3], vec![], vec![]],
            concat!(
                "[\n",
                "  [\n",
                "    1,\n",
                "    2,\n",
                "    3\n",
                "  ],\n",
                "  [],\n",
                "  []\n",
                "]"
            ),
        ),
        (
            vec![vec![], vec![1, 2, 3], vec![]],
            concat!(
                "[\n",
                "  [],\n",
                "  [\n",
                "    1,\n",
                "    2,\n",
                "    3\n",
                "  ],\n",
                "  []\n",
                "]"
            ),
        ),
        (
            vec![vec![], vec![], vec![1, 2, 3]],
            concat!(
                "[\n",
                "  [],\n",
                "  [],\n",
                "  [\n",
                "    1,\n",
                "    2,\n",
                "    3\n",
                "  ]\n",
                "]"
            ),
        ),
    ]);

    test_pretty_encode_ok(&[
        (vec![], "[]"),
        (
            vec![true],
            concat!(
                "[\n",
                "  true\n",
                "]"
            ),
        ),
        (
            vec![true, false],
            concat!(
                "[\n",
                "  true,\n",
                "  false\n",
                "]"
            ),
        ),
    ]);

    let long_test_list = Value::Array(vec![
        Value::Bool(false),
        Value::Null,
        Value::Array(vec![Value::String("foo\nbar".to_string()), Value::F64(3.5)])]);

    test_encode_ok(&[
        (
            long_test_list.clone(),
            "[false,null,[\"foo\\nbar\",3.5]]",
        ),
    ]);

    test_pretty_encode_ok(&[
        (
            long_test_list,
            concat!(
                "[\n",
                "  false,\n",
                "  null,\n",
                "  [\n",
                "    \"foo\\nbar\",\n",
                "    3.5\n",
                "  ]\n",
                "]"
            ),
        )
    ]);
}

#[test]
fn test_write_object() {
    test_encode_ok(&[
        (treemap!(), "{}"),
        (treemap!("a".to_string() => true), "{\"a\":true}"),
        (
            treemap!(
                "a".to_string() => true,
                "b".to_string() => false
            ),
            "{\"a\":true,\"b\":false}"),
    ]);

    test_encode_ok(&[
        (
            treemap![
                "a".to_string() => treemap![],
                "b".to_string() => treemap![],
                "c".to_string() => treemap![]
            ],
            "{\"a\":{},\"b\":{},\"c\":{}}",
        ),
        (
            treemap![
                "a".to_string() => treemap![
                    "a".to_string() => treemap!["a" => vec![1,2,3]],
                    "b".to_string() => treemap![],
                    "c".to_string() => treemap![]
                ],
                "b".to_string() => treemap![],
                "c".to_string() => treemap![]
            ],
            "{\"a\":{\"a\":{\"a\":[1,2,3]},\"b\":{},\"c\":{}},\"b\":{},\"c\":{}}",
        ),
        (
            treemap![
                "a".to_string() => treemap![],
                "b".to_string() => treemap![
                    "a".to_string() => treemap!["a" => vec![1,2,3]],
                    "b".to_string() => treemap![],
                    "c".to_string() => treemap![]
                ],
                "c".to_string() => treemap![]
            ],
            "{\"a\":{},\"b\":{\"a\":{\"a\":[1,2,3]},\"b\":{},\"c\":{}},\"c\":{}}",
        ),
        (
            treemap![
                "a".to_string() => treemap![],
                "b".to_string() => treemap![],
                "c".to_string() => treemap![
                    "a".to_string() => treemap!["a" => vec![1,2,3]],
                    "b".to_string() => treemap![],
                    "c".to_string() => treemap![]
                ]
            ],
            "{\"a\":{},\"b\":{},\"c\":{\"a\":{\"a\":[1,2,3]},\"b\":{},\"c\":{}}}",
        ),
    ]);

    test_pretty_encode_ok(&[
        (
            treemap![
                "a".to_string() => treemap![],
                "b".to_string() => treemap![],
                "c".to_string() => treemap![]
            ],
            concat!(
                "{\n",
                "  \"a\": {},\n",
                "  \"b\": {},\n",
                "  \"c\": {}\n",
                "}",
            ),
        ),
        (
            treemap![
                "a".to_string() => treemap![
                    "a".to_string() => treemap!["a" => vec![1,2,3]],
                    "b".to_string() => treemap![],
                    "c".to_string() => treemap![]
                ],
                "b".to_string() => treemap![],
                "c".to_string() => treemap![]
            ],
            concat!(
                "{\n",
                "  \"a\": {\n",
                "    \"a\": {\n",
                "      \"a\": [\n",
                "        1,\n",
                "        2,\n",
                "        3\n",
                "      ]\n",
                "    },\n",
                "    \"b\": {},\n",
                "    \"c\": {}\n",
                "  },\n",
                "  \"b\": {},\n",
                "  \"c\": {}\n",
                "}"
            ),
        ),
        (
            treemap![
                "a".to_string() => treemap![],
                "b".to_string() => treemap![
                    "a".to_string() => treemap!["a" => vec![1,2,3]],
                    "b".to_string() => treemap![],
                    "c".to_string() => treemap![]
                ],
                "c".to_string() => treemap![]
            ],
            concat!(
                "{\n",
                "  \"a\": {},\n",
                "  \"b\": {\n",
                "    \"a\": {\n",
                "      \"a\": [\n",
                "        1,\n",
                "        2,\n",
                "        3\n",
                "      ]\n",
                "    },\n",
                "    \"b\": {},\n",
                "    \"c\": {}\n",
                "  },\n",
                "  \"c\": {}\n",
                "}"
            ),
        ),
        (
            treemap![
                "a".to_string() => treemap![],
                "b".to_string() => treemap![],
                "c".to_string() => treemap![
                    "a".to_string() => treemap!["a" => vec![1,2,3]],
                    "b".to_string() => treemap![],
                    "c".to_string() => treemap![]
                ]
            ],
            concat!(
                "{\n",
                "  \"a\": {},\n",
                "  \"b\": {},\n",
                "  \"c\": {\n",
                "    \"a\": {\n",
                "      \"a\": [\n",
                "        1,\n",
                "        2,\n",
                "        3\n",
                "      ]\n",
                "    },\n",
                "    \"b\": {},\n",
                "    \"c\": {}\n",
                "  }\n",
                "}"
            ),
        ),
    ]);

    test_pretty_encode_ok(&[
        (treemap!(), "{}"),
        (
            treemap!("a".to_string() => true),
            concat!(
                "{\n",
                "  \"a\": true\n",
                "}"
            ),
        ),
        (
            treemap!(
                "a".to_string() => true,
                "b".to_string() => false
            ),
            concat!(
                "{\n",
                "  \"a\": true,\n",
                "  \"b\": false\n",
                "}"
            ),
        ),
    ]);

    let complex_obj = Value::Object(treemap!(
        "b".to_string() => Value::Array(vec![
            Value::Object(treemap!("c".to_string() => Value::String("\x0c\r".to_string()))),
            Value::Object(treemap!("d".to_string() => Value::String("".to_string())))
        ])
    ));

    test_encode_ok(&[
        (
            complex_obj.clone(),
            "{\
                \"b\":[\
                    {\"c\":\"\\f\\r\"},\
                    {\"d\":\"\"}\
                ]\
            }"
        ),
    ]);

    test_pretty_encode_ok(&[
        (
            complex_obj.clone(),
            concat!(
                "{\n",
                "  \"b\": [\n",
                "    {\n",
                "      \"c\": \"\\f\\r\"\n",
                "    },\n",
                "    {\n",
                "      \"d\": \"\"\n",
                "    }\n",
                "  ]\n",
                "}"
            ),
        )
    ]);
}

#[test]
fn test_write_tuple() {
    test_encode_ok(&[
        (
            (5,),
            "[5]",
        ),
    ]);

    test_pretty_encode_ok(&[
        (
            (5,),
            concat!(
                "[\n",
                "  5\n",
                "]"
            ),
        ),
    ]);

    test_encode_ok(&[
        (
            (5, (6, "abc")),
            "[5,[6,\"abc\"]]",
        ),
    ]);

    test_pretty_encode_ok(&[
        (
            (5, (6, "abc")),
            concat!(
                "[\n",
                "  5,\n",
                "  [\n",
                "    6,\n",
                "    \"abc\"\n",
                "  ]\n",
                "]"
            ),
        ),
    ]);
}

#[test]
fn test_write_enum() {
    test_encode_ok(&[
        (
            Animal::Dog,
            "{\"Dog\":[]}",
        ),
        (
            Animal::Frog("Henry".to_string(), vec![]),
            "{\"Frog\":[\"Henry\",[]]}",
        ),
        (
            Animal::Frog("Henry".to_string(), vec![349]),
            "{\"Frog\":[\"Henry\",[349]]}",
        ),
        (
            Animal::Frog("Henry".to_string(), vec![349, 102]),
            "{\"Frog\":[\"Henry\",[349,102]]}",
        ),
        (
            Animal::Cat { age: 5, name: "Kate".to_string() },
            "{\"Cat\":{\"age\":5,\"name\":\"Kate\"}}"
        ),
    ]);

    test_pretty_encode_ok(&[
        (
            Animal::Dog,
            concat!(
                "{\n",
                "  \"Dog\": []\n",
                "}"
            ),
        ),
        (
            Animal::Frog("Henry".to_string(), vec![]),
            concat!(
                "{\n",
                "  \"Frog\": [\n",
                "    \"Henry\",\n",
                "    []\n",
                "  ]\n",
                "}"
            ),
        ),
        (
            Animal::Frog("Henry".to_string(), vec![349]),
            concat!(
                "{\n",
                "  \"Frog\": [\n",
                "    \"Henry\",\n",
                "    [\n",
                "      349\n",
                "    ]\n",
                "  ]\n",
                "}"
            ),
        ),
        (
            Animal::Frog("Henry".to_string(), vec![349, 102]),
            concat!(
                "{\n",
                "  \"Frog\": [\n",
                "    \"Henry\",\n",
                "    [\n",
                "      349,\n",
                "      102\n",
                "    ]\n",
                "  ]\n",
                "}"
            ),
        ),
    ]);
}

#[test]
fn test_write_option() {
    test_encode_ok(&[
        (None, "null"),
        (Some("jodhpurs"), "\"jodhpurs\""),
    ]);

    test_encode_ok(&[
        (None, "null"),
        (Some(vec!["foo", "bar"]), "[\"foo\",\"bar\"]"),
    ]);

    test_pretty_encode_ok(&[
        (None, "null"),
        (Some("jodhpurs"), "\"jodhpurs\""),
    ]);

    test_pretty_encode_ok(&[
        (None, "null"),
        (
            Some(vec!["foo", "bar"]),
            concat!(
                "[\n",
                "  \"foo\",\n",
                "  \"bar\"\n",
                "]"
            ),
        ),
    ]);
}

fn test_parse_ok<T>(errors: Vec<(&'static str, T)>)
    where T: Clone + Debug + PartialEq + ser::Serialize + de::Deserialize,
{
    for (s, value) in errors {
        let v: T = from_str(s).unwrap();
        assert_eq!(v, value.clone());

        // Make sure we can deserialize into a `Value`.
        let json_value: Value = from_str(s).unwrap();
        assert_eq!(json_value, to_value(&value));

        // Make sure we can deserialize from a `Value`.
        let v: T = from_value(json_value.clone()).unwrap();
        assert_eq!(v, value);

        // Make sure we can round trip back to `Value`.
        let json_value2: Value = from_value(json_value.clone()).unwrap();
        assert_eq!(json_value2, json_value);
    }
}

// FIXME (#5527): these could be merged once UFCS is finished.
fn test_parse_err<T>(errors: Vec<(&'static str, Error)>)
    where T: Debug + PartialEq + de::Deserialize,
{
    for (s, err) in errors {
        match (err, from_str::<T>(s).unwrap_err()) {
            (
                Error::SyntaxError(expected_code, expected_line, expected_col),
                Error::SyntaxError(actual_code, actual_line, actual_col),
            ) => {
                assert_eq!(
                    (expected_code, expected_line, expected_col),
                    (actual_code, actual_line, actual_col)
                )
            }
            (expected_err, actual_err) => {
                panic!("unexpected errors {} != {}", expected_err, actual_err)
            }
        }
    }
}

#[test]
fn test_parse_null() {
    test_parse_err::<()>(vec![
        ("n", Error::SyntaxError(ErrorCode::ExpectedSomeIdent, 1, 1)),
        ("nul", Error::SyntaxError(ErrorCode::ExpectedSomeIdent, 1, 3)),
        ("nulla", Error::SyntaxError(ErrorCode::TrailingCharacters, 1, 5)),
    ]);

    test_parse_ok(vec![
        ("null", ()),
    ]);
}

#[test]
fn test_parse_bool() {
    test_parse_err::<bool>(vec![
        ("t", Error::SyntaxError(ErrorCode::ExpectedSomeIdent, 1, 1)),
        ("truz", Error::SyntaxError(ErrorCode::ExpectedSomeIdent, 1, 4)),
        ("f", Error::SyntaxError(ErrorCode::ExpectedSomeIdent, 1, 1)),
        ("faz", Error::SyntaxError(ErrorCode::ExpectedSomeIdent, 1, 3)),
        ("truea", Error::SyntaxError(ErrorCode::TrailingCharacters, 1, 5)),
        ("falsea", Error::SyntaxError(ErrorCode::TrailingCharacters, 1, 6)),
    ]);

    test_parse_ok(vec![
        ("true", true),
        (" true ", true),
        ("false", false),
        (" false ", false),
    ]);
}

#[test]
fn test_parse_number_errors() {
    test_parse_err::<f64>(vec![
        ("+", Error::SyntaxError(ErrorCode::ExpectedSomeValue, 1, 1)),
        (".", Error::SyntaxError(ErrorCode::ExpectedSomeValue, 1, 1)),
        ("-", Error::SyntaxError(ErrorCode::InvalidNumber, 1, 1)),
        ("00", Error::SyntaxError(ErrorCode::InvalidNumber, 1, 2)),
        ("1.", Error::SyntaxError(ErrorCode::InvalidNumber, 1, 2)),
        ("1e", Error::SyntaxError(ErrorCode::InvalidNumber, 1, 2)),
        ("1e+", Error::SyntaxError(ErrorCode::InvalidNumber, 1, 3)),
        ("1a", Error::SyntaxError(ErrorCode::TrailingCharacters, 1, 2)),
    ]);
}

#[test]
fn test_parse_i64() {
    test_parse_ok(vec![
        ("-2", -2),
        ("-1234", -1234),
        (" -1234 ", -1234),
    ]);
}

#[test]
fn test_parse_u64() {
    test_parse_ok(vec![
        ("3", 3u64),
        ("1234", 1234),
    ]);
}

#[test]
fn test_parse_f64() {
    test_parse_ok(vec![
        ("3.0", 3.0f64),
        ("3.1", 3.1),
        ("-1.2", -1.2),
        ("0.4", 0.4),
        ("0.4e5", 0.4e5),
        ("0.4e15", 0.4e15),
        ("0.4e-01", 0.4e-01),
        (" 0.4e-01 ", 0.4e-01),
    ]);
}

#[test]
fn test_parse_string() {
    test_parse_err::<String>(vec![
        ("\"", Error::SyntaxError(ErrorCode::EOFWhileParsingString, 1, 1)),
        ("\"lol", Error::SyntaxError(ErrorCode::EOFWhileParsingString, 1, 4)),
        ("\"lol\"a", Error::SyntaxError(ErrorCode::TrailingCharacters, 1, 6)),
    ]);

    test_parse_ok(vec![
        ("\"\"", "".to_string()),
        ("\"foo\"", "foo".to_string()),
        (" \"foo\" ", "foo".to_string()),
        ("\"\\\"\"", "\"".to_string()),
        ("\"\\b\"", "\x08".to_string()),
        ("\"\\n\"", "\n".to_string()),
        ("\"\\r\"", "\r".to_string()),
        ("\"\\t\"", "\t".to_string()),
        ("\"\\u12ab\"", "\u{12ab}".to_string()),
        ("\"\\uAB12\"", "\u{AB12}".to_string()),
    ]);
}

#[test]
fn test_parse_list() {
    test_parse_err::<Vec<f64>>(vec![
        ("[", Error::SyntaxError(ErrorCode::EOFWhileParsingValue, 1, 1)),
        ("[ ", Error::SyntaxError(ErrorCode::EOFWhileParsingValue, 1, 2)),
        ("[1", Error::SyntaxError(ErrorCode::EOFWhileParsingList,  1, 2)),
        ("[1,", Error::SyntaxError(ErrorCode::EOFWhileParsingValue, 1, 3)),
        ("[1,]", Error::SyntaxError(ErrorCode::ExpectedSomeValue, 1, 4)),
        ("[1 2]", Error::SyntaxError(ErrorCode::ExpectedListCommaOrEnd, 1, 4)),
        ("[]a", Error::SyntaxError(ErrorCode::TrailingCharacters, 1, 3)),
    ]);

    test_parse_ok(vec![
        ("[]", vec![]),
        ("[ ]", vec![]),
        ("[null]", vec![()]),
        (" [ null ] ", vec![()]),
    ]);

    test_parse_ok(vec![
        ("[true]", vec![true]),
    ]);

    test_parse_ok(vec![
        ("[3,1]", vec![3u64, 1]),
        (" [ 3 , 1 ] ", vec![3, 1]),
    ]);

    test_parse_ok(vec![
        ("[[3], [1, 2]]", vec![vec![3u64], vec![1, 2]]),
    ]);

    test_parse_ok(vec![
        ("[1]", (1u64,)),
    ]);

    test_parse_ok(vec![
        ("[1, 2]", (1u64, 2u64)),
    ]);

    test_parse_ok(vec![
        ("[1, 2, 3]", (1u64, 2u64, 3u64)),
    ]);

    test_parse_ok(vec![
        ("[1, [2, 3]]", (1u64, (2u64, 3u64))),
    ]);

    let v: () = from_str("[]").unwrap();
    assert_eq!(v, ());
}

#[test]
fn test_parse_object() {
    test_parse_err::<BTreeMap<String, u32>>(vec![
        ("{", Error::SyntaxError(ErrorCode::EOFWhileParsingValue, 1, 1)),
        ("{ ", Error::SyntaxError(ErrorCode::EOFWhileParsingValue, 1, 2)),
        ("{1", Error::SyntaxError(ErrorCode::KeyMustBeAString, 1, 2)),
        ("{ \"a\"", Error::SyntaxError(ErrorCode::EOFWhileParsingObject, 1, 5)),
        ("{\"a\"", Error::SyntaxError(ErrorCode::EOFWhileParsingObject, 1, 4)),
        ("{\"a\" ", Error::SyntaxError(ErrorCode::EOFWhileParsingObject, 1, 5)),
        ("{\"a\" 1", Error::SyntaxError(ErrorCode::ExpectedColon, 1, 6)),
        ("{\"a\":", Error::SyntaxError(ErrorCode::EOFWhileParsingValue, 1, 5)),
        ("{\"a\":1", Error::SyntaxError(ErrorCode::EOFWhileParsingObject, 1, 6)),
        ("{\"a\":1 1", Error::SyntaxError(ErrorCode::ExpectedObjectCommaOrEnd, 1, 8)),
        ("{\"a\":1,", Error::SyntaxError(ErrorCode::EOFWhileParsingValue, 1, 7)),
        ("{}a", Error::SyntaxError(ErrorCode::TrailingCharacters, 1, 3)),
    ]);

    test_parse_ok(vec![
        ("{}", treemap!()),
        ("{ }", treemap!()),
        (
            "{\"a\":3}",
            treemap!("a".to_string() => 3u64)
        ),
        (
            "{ \"a\" : 3 }",
            treemap!("a".to_string() => 3)
        ),
        (
            "{\"a\":3,\"b\":4}",
            treemap!("a".to_string() => 3, "b".to_string() => 4)
        ),
        (
            " { \"a\" : 3 , \"b\" : 4 } ",
            treemap!("a".to_string() => 3, "b".to_string() => 4),
        ),
    ]);

    test_parse_ok(vec![
        (
            "{\"a\": {\"b\": 3, \"c\": 4}}",
            treemap!(
                "a".to_string() => treemap!(
                    "b".to_string() => 3u64,
                    "c".to_string() => 4
                )
            ),
        ),
    ]);
}

#[test]
fn test_parse_struct() {
    test_parse_err::<Outer>(vec![
        ("5", Error::SyntaxError(ErrorCode::ExpectedSomeValue, 1, 1)),
        ("\"hello\"", Error::SyntaxError(ErrorCode::ExpectedSomeValue, 1, 7)),
        ("{\"inner\": true}", Error::SyntaxError(ErrorCode::ExpectedSomeValue, 1, 15)),
    ]);

    test_parse_ok(vec![
        (
            "{
                \"inner\": []
            }",
            Outer {
                inner: vec![]
            },
        ),
        (
            "{
                \"inner\": [
                    { \"a\": null, \"b\": 2, \"c\": [\"abc\", \"xyz\"] }
                ]
            }",
            Outer {
                inner: vec![
                    Inner { a: (), b: 2, c: vec!["abc".to_string(), "xyz".to_string()] }
                ]
            },
        )
    ]);

    let v: Outer = from_str("{}").unwrap();

    assert_eq!(
        v,
        Outer {
            inner: vec![],
        }
    );
}

#[test]
fn test_parse_option() {
    test_parse_ok(vec![
        ("null", None::<String>),
        ("\"jodhpurs\"", Some("jodhpurs".to_string())),
    ]);

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct Foo {
        x: Option<isize>,
    }

    let value: Foo = from_str("{}").unwrap();
    assert_eq!(value, Foo { x: None });

    test_parse_ok(vec![
        ("{\"x\": null}", Foo { x: None }),
        ("{\"x\": 5}", Foo { x: Some(5) }),
    ]);
}

#[test]
fn test_parse_enum_errors() {
    test_parse_err::<Animal>(vec![
        ("{}", Error::SyntaxError(ErrorCode::ExpectedSomeValue, 1, 2)),
        ("{\"Dog\":", Error::SyntaxError(ErrorCode::EOFWhileParsingValue, 1, 7)),
        ("{\"Dog\":}", Error::SyntaxError(ErrorCode::ExpectedSomeValue, 1, 8)),
        ("{\"unknown\":[]}", Error::SyntaxError(ErrorCode::UnknownField("unknown".to_string()), 1, 11)),
        ("{\"Dog\":{}}", Error::SyntaxError(ErrorCode::ExpectedSomeValue, 1, 9)),
        ("{\"Frog\":{}}", Error::SyntaxError(ErrorCode::ExpectedSomeValue, 1, 10)),
        ("{\"Cat\":[]}", Error::SyntaxError(ErrorCode::ExpectedSomeValue, 1, 9)),
    ]);
}

#[test]
fn test_parse_enum() {
    test_parse_ok(vec![
        ("{\"Dog\":[]}", Animal::Dog),
        (" { \"Dog\" : [ ] } ", Animal::Dog),
        (
            "{\"Frog\":[\"Henry\",[]]}",
            Animal::Frog("Henry".to_string(), vec![]),
        ),
        (
            " { \"Frog\": [ \"Henry\" , [ 349, 102 ] ] } ",
            Animal::Frog("Henry".to_string(), vec![349, 102]),
        ),
        (
            "{\"Cat\": {\"age\": 5, \"name\": \"Kate\"}}",
            Animal::Cat { age: 5, name: "Kate".to_string() },
        ),
        (
            " { \"Cat\" : { \"age\" : 5 , \"name\" : \"Kate\" } } ",
            Animal::Cat { age: 5, name: "Kate".to_string() },
        ),
    ]);

    test_parse_ok(vec![
        (
            concat!(
                "{",
                "  \"a\": {\"Dog\": []},",
                "  \"b\": {\"Frog\":[\"Henry\", []]}",
                "}"
            ),
            treemap!(
                "a".to_string() => Animal::Dog,
                "b".to_string() => Animal::Frog("Henry".to_string(), vec![])
            )
        ),
    ]);
}

#[test]
fn test_parse_trailing_whitespace() {
    test_parse_ok(vec![
        ("[1, 2] ", vec![1u64, 2]),
        ("[1, 2]\n", vec![1, 2]),
        ("[1, 2]\t", vec![1, 2]),
        ("[1, 2]\t \n", vec![1, 2]),
    ]);
}

#[test]
fn test_multiline_errors() {
    test_parse_err::<BTreeMap<String, String>>(vec![
        ("{\n  \"foo\":\n \"bar\"", Error::SyntaxError(ErrorCode::EOFWhileParsingObject, 3, 6)),
    ]);
}

#[test]
fn test_missing_field() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Foo {
        x: Option<u32>,
    }

    let value: Foo = from_str("{}").unwrap();
    assert_eq!(value, Foo { x: None });

    let value: Foo = from_str("{\"x\": 5}").unwrap();
    assert_eq!(value, Foo { x: Some(5) });

    let value: Foo = from_value(Value::Object(treemap!())).unwrap();
    assert_eq!(value, Foo { x: None });

    let value: Foo = from_value(Value::Object(treemap!(
        "x".to_string() => Value::I64(5)
    ))).unwrap();
    assert_eq!(value, Foo { x: Some(5) });
}

#[test]
fn test_missing_renamed_field() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Foo {
        #[serde(rename="y")]
        x: Option<u32>,
    }

    let value: Foo = from_str("{}").unwrap();
    assert_eq!(value, Foo { x: None });

    let value: Foo = from_str("{\"y\": 5}").unwrap();
    assert_eq!(value, Foo { x: Some(5) });

    let value: Foo = from_value(Value::Object(treemap!())).unwrap();
    assert_eq!(value, Foo { x: None });

    let value: Foo = from_value(Value::Object(treemap!(
        "y".to_string() => Value::I64(5)
    ))).unwrap();
    assert_eq!(value, Foo { x: Some(5) });
}

#[test]
fn test_missing_fmt_renamed_field() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Foo {
        #[serde(rename(json="y"))]
        x: Option<u32>,
    }

    let value: Foo = from_str("{}").unwrap();
    assert_eq!(value, Foo { x: None });

    let value: Foo = from_str("{\"y\": 5}").unwrap();
    assert_eq!(value, Foo { x: Some(5) });

    let value: Foo = from_value(Value::Object(treemap!())).unwrap();
    assert_eq!(value, Foo { x: None });

    let value : Foo = from_value(Value::Object(treemap!(
        "y".to_string() => Value::I64(5)
            ))).unwrap();
    assert_eq!(value, Foo { x: Some(5) });
}
