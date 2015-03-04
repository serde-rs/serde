#![feature(io, plugin, test)]
#![plugin(serde2_macros)]

extern crate test;
extern crate serde2;

use std::fmt::Debug;
use std::string;
use std::collections::BTreeMap;

use serde2::de;
use serde2::ser;

use serde2::json::{
    self,
    Deserializer,
    Error,
    Serializer,
    Value,
    from_str,
    from_value,
    to_value,
};

use serde2::json::error::Error::{
    SyntaxError,
};

use serde2::json::error::ErrorCode::{
    EOFWhileParsingList,
    EOFWhileParsingObject,
    EOFWhileParsingString,
    EOFWhileParsingValue,
    ExpectedColon,
    ExpectedListCommaOrEnd,
    ExpectedObjectCommaOrEnd,
    ExpectedSomeIdent,
    ExpectedSomeValue,
    InvalidNumber,
    KeyMustBeAString,
    TrailingCharacters,
};

macro_rules! treemap {
    ($($k:expr => $v:expr),*) => ({
        let mut _m = ::std::collections::BTreeMap::new();
        $(_m.insert($k, $v);)*
        _m
    })
}

#[derive(PartialEq, Debug)]
#[derive_serialize]
#[derive_deserialize]
enum Animal {
    Dog,
    Frog(String, Vec<isize>)
}

#[derive(PartialEq, Debug)]
#[derive_serialize]
#[derive_deserialize]
struct Inner {
    a: (),
    b: usize,
    c: Vec<string::String>,
}

#[derive(PartialEq, Debug)]
#[derive_serialize]
#[derive_deserialize]
struct Outer {
    inner: Vec<Inner>,
}

fn test_encode_ok<
    T: PartialEq + Debug + ser::Serialize
>(errors: &[(T, &str)]) {
    for &(ref value, out) in errors {
        let out = out.to_string();

        let s = json::to_string(value).unwrap();
        assert_eq!(s, out);

        let v = to_value(&value);
        let s = json::to_string(&v).unwrap();
        assert_eq!(s, out);
    }
}

/*
fn test_pretty_encode_ok<
    T: PartialEq + Debug + ser::Serialize<json::PrettySerializer<Vec<u8>>, io::Error>
>(errors: &[(T, &str)]) {
    for &(ref value, out) in errors {
        let out = out.to_string();

        let s = json::to_pretty_string(value).unwrap();
        assert_eq!(s, out);

        let s = json::to_pretty_string(&value.to_json()).unwrap();
        assert_eq!(s, out);
    }
}
*/

#[test]
fn test_write_null() {
    let tests = &[
        ((), "null"),
    ];
    test_encode_ok(tests);
    //test_pretty_encode_ok(tests);
}

#[test]
fn test_write_i64() {
    let tests = &[
        (3i64, "3"),
        (-2i64, "-2"),
        (-1234i64, "-1234"),
    ];
    test_encode_ok(tests);
    //test_pretty_encode_ok(tests);
}

#[test]
fn test_write_f64() {
    let tests = &[
        (3.0, "3"),
        (3.1, "3.1"),
        (-1.5, "-1.5"),
        (0.5, "0.5"),
    ];
    test_encode_ok(tests);
    //test_pretty_encode_ok(tests);
}

#[test]
fn test_write_str() {
    let tests = &[
        ("", "\"\""),
        ("foo", "\"foo\""),
    ];
    test_encode_ok(tests);
    //test_pretty_encode_ok(tests);
}

#[test]
fn test_write_bool() {
    let tests = &[
        (true, "true"),
        (false, "false"),
    ];
    test_encode_ok(tests);
    //test_pretty_encode_ok(tests);
}

#[test]
fn test_write_list() {
    test_encode_ok(&[
        (vec!(), "[]"),
        (vec!(true), "[true]"),
        (vec!(true, false), "[true,false]"),
    ]);

    /*
    test_pretty_encode_ok(&[
        (vec!(), "[]"),
        (
            vec!(true),
            concat!(
                "[\n",
                "  true\n",
                "]"
            ),
        ),
        (
            vec!(true, false),
            concat!(
                "[\n",
                "  true,\n",
                "  false\n",
                "]"
            ),
        ),
    ]);
    */

    let long_test_list = Value::Array(vec![
        Value::Bool(false),
        Value::Null,
        Value::Array(vec![Value::String("foo\nbar".to_string()), Value::F64(3.5)])]);

    test_encode_ok(&[
        (long_test_list, "[false,null,[\"foo\\nbar\",3.5]]"),
    ]);

    let long_test_list = Value::Array(vec![
        Value::Bool(false),
        Value::Null,
        Value::Array(vec![Value::String("foo\nbar".to_string()), Value::F64(3.5)])]);

    /*
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
            )
        )
    ]);
    */
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

    /*
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
    */

    let complex_obj = Value::Object(treemap!(
        "b".to_string() => Value::Array(vec!(
            Value::Object(treemap!("c".to_string() => Value::String("\x0c\r".to_string()))),
            Value::Object(treemap!("d".to_string() => Value::String("".to_string())))
        ))
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

    /*
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
    */
}

#[test]
fn test_write_tuple() {
    test_encode_ok(&[
        (
            (5,),
            "[5]",
        ),
    ]);

    /*
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
    */

    test_encode_ok(&[
        (
            (5, (6, "abc")),
            "[5,[6,\"abc\"]]",
        ),
    ]);

    /*
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
    */
}

/*
#[test]
fn test_write_enum() {
    test_encode_ok(&[
        (Animal::Dog, "{\"Dog\":[]}"),
        (Animal::Frog("Henry".to_string(), vec!()), "{\"Frog\":[\"Henry\",[]]}"),
        (Animal::Frog("Henry".to_string(), vec!(349)), "{\"Frog\":[\"Henry\",[349]]}"),
        (Animal::Frog("Henry".to_string(), vec!(349, 102)), "{\"Frog\":[\"Henry\",[349,102]]}"),
    ]);

    /*
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
            Animal::Frog("Henry".to_string(), vec!()),
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
            Animal::Frog("Henry".to_string(), vec!(349)),
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
            Animal::Frog("Henry".to_string(), vec!(349, 102)),
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
    */
}

#[test]
fn test_write_option() {
    test_encode_ok(&[
        (None, "null"),
        (Some("jodhpurs"), "\"jodhpurs\""),
    ]);

    test_encode_ok(&[
        (None, "null"),
        (Some(vec!("foo", "bar")), "[\"foo\",\"bar\"]"),
    ]);

    /*
    test_pretty_encode_ok(&[
        (None, "null"),
        (Some("jodhpurs"), "\"jodhpurs\""),
    ]);

    test_pretty_encode_ok(&[
        (None, "null"),
        (
            Some(vec!("foo", "bar")),
            concat!(
                "[\n",
                "  \"foo\",\n",
                "  \"bar\"\n",
                "]"
            ),
        ),
    ]);
    */
}

// FIXME (#5527): these could be merged once UFCS is finished.
fn test_parse_err<
    'a,
    T: Debug + de::Deserialize
>(errors: &[(&'a str, Error)]) {
    for &(s, ref err) in errors {
        let v: Result<T, Error> = from_str(s);
        assert_eq!(v.unwrap_err(), *err);
    }
}

fn test_parse_ok<
    'a,
    T: PartialEq + Debug + de::Deserialize
>(errors: &[(&'a str, T)]) {
    for &(s, ref value) in errors {
        let v: T = from_str(s).unwrap();
        assert_eq!(v, *value);

        let v: Value = from_str(s).unwrap();
        assert_eq!(v, value.to_json());
    }
}

fn test_json_deserialize_ok<
    T: PartialEq + Debug + de::Deserialize
>(errors: &[T]) {
    for value in errors {
        let v: T = from_value(value.to_json()).unwrap();
        assert_eq!(v, *value);

        // Make sure we can round trip back to `Json`.
        let v: Value = from_value(value.to_json()).unwrap();
        assert_eq!(v, value.to_json());
    }
}

#[test]
fn test_parse_null() {
    test_parse_err::<()>(&[
        ("n", SyntaxError(ExpectedSomeIdent, 1, 2)),
        ("nul", SyntaxError(ExpectedSomeIdent, 1, 4)),
        ("nulla", SyntaxError(TrailingCharacters, 1, 5)),
    ]);

    test_parse_ok(&[
        ("null", ()),
    ]);
}

#[test]
fn test_json_deserialize_null() {
    test_json_deserialize_ok(&[
        (),
    ]);
}

#[test]
fn test_parse_bool() {
    test_parse_err::<bool>(&[
        ("t", SyntaxError(ExpectedSomeIdent, 1, 2)),
        ("truz", SyntaxError(ExpectedSomeIdent, 1, 4)),
        ("f", SyntaxError(ExpectedSomeIdent, 1, 2)),
        ("faz", SyntaxError(ExpectedSomeIdent, 1, 3)),
        ("truea", SyntaxError(TrailingCharacters, 1, 5)),
        ("falsea", SyntaxError(TrailingCharacters, 1, 6)),
    ]);

    test_parse_ok(&[
        ("true", true),
        ("false", false),
    ]);
}

#[test]
fn test_json_deserialize_bool() {
    test_json_deserialize_ok(&[
        true,
        false,
    ]);
}

#[test]
fn test_parse_number_errors() {
    test_parse_err::<f64>(&[
        ("+", SyntaxError(ExpectedSomeValue, 1, 1)),
        (".", SyntaxError(ExpectedSomeValue, 1, 1)),
        ("-", SyntaxError(InvalidNumber, 1, 2)),
        ("00", SyntaxError(InvalidNumber, 1, 2)),
        ("1.", SyntaxError(InvalidNumber, 1, 3)),
        ("1e", SyntaxError(InvalidNumber, 1, 3)),
        ("1e+", SyntaxError(InvalidNumber, 1, 4)),
        ("1a", SyntaxError(TrailingCharacters, 1, 2)),
    ]);
}

#[test]
fn test_parse_i64() {
    test_parse_ok(&[
        ("3", 3i64),
        ("-2", -2),
        ("-1234", -1234),
    ]);
}

#[test]
fn test_parse_f64() {
    test_parse_ok(&[
        ("3.0", 3.0f64),
        ("3.1", 3.1),
        ("-1.2", -1.2),
        ("0.4", 0.4),
        ("0.4e5", 0.4e5),
        ("0.4e15", 0.4e15),
        ("0.4e-01", 0.4e-01),
    ]);
}

#[test]
fn test_json_deserialize_numbers() {
    test_json_deserialize_ok(&[
        3.0f64,
        3.1,
        -1.2,
        0.4,
        0.4e5,
        0.4e15,
        0.4e-01,
    ]);
}

#[test]
fn test_parse_string() {
    test_parse_err::<string::String>(&[
        ("\"", SyntaxError(EOFWhileParsingString, 1, 2)),
        ("\"lol", SyntaxError(EOFWhileParsingString, 1, 5)),
        ("\"lol\"a", SyntaxError(TrailingCharacters, 1, 6)),
    ]);

    test_parse_ok(&[
        ("\"\"", "".to_string()),
        ("\"foo\"", "foo".to_string()),
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
fn test_json_deserialize_str() {
    test_json_deserialize_ok(&[
        "".to_string(),
        "foo".to_string(),
        "\"".to_string(),
        "\x08".to_string(),
        "\n".to_string(),
        "\r".to_string(),
        "\t".to_string(),
        "\u{12ab}".to_string(),
        "\u{AB12}".to_string(),
    ]);
}

#[test]
fn test_parse_list() {
    test_parse_err::<Vec<f64>>(&[
        ("[", SyntaxError(EOFWhileParsingValue, 1, 2)),
        ("[ ", SyntaxError(EOFWhileParsingValue, 1, 3)),
        ("[1", SyntaxError(EOFWhileParsingList,  1, 3)),
        ("[1,", SyntaxError(EOFWhileParsingValue, 1, 4)),
        ("[1,]", SyntaxError(ExpectedSomeValue, 1, 4)),
        ("[1 2]", SyntaxError(ExpectedListCommaOrEnd, 1, 4)),
        ("[]a", SyntaxError(TrailingCharacters, 1, 3)),
    ]);

    test_parse_ok(&[
        ("[]", vec!()),
        ("[ ]", vec!()),
        ("[null]", vec!(())),
        ("[ null ]", vec!(())),
    ]);

    test_parse_ok(&[
        ("[true]", vec!(true)),
    ]);

    test_parse_ok(&[
        ("[3,1]", vec!(3, 1)),
        ("[ 3 , 1 ]", vec!(3, 1)),
    ]);

    test_parse_ok(&[
        ("[[3], [1, 2]]", vec!(vec!(3is), vec!(1, 2))),
    ]);

    let v: () = from_str("[]").unwrap();
    assert_eq!(v, ());

    test_parse_ok(&[
        ("[1, 2, 3]", (1us, 2us, 3us)),
    ]);
}

#[test]
fn test_json_deserialize_list() {
    test_json_deserialize_ok(&[
        vec!(),
        vec!(()),
    ]);

    test_json_deserialize_ok(&[
        vec!(true),
    ]);

    test_json_deserialize_ok(&[
        vec!(3, 1),
    ]);

    test_json_deserialize_ok(&[
        vec!(vec!(3is), vec!(1, 2)),
    ]);
}

#[test]
fn test_parse_object() {
    test_parse_err::<BTreeMap<string::String, isize>>(&[
        ("{", SyntaxError(EOFWhileParsingString, 1, 2)),
        ("{ ", SyntaxError(EOFWhileParsingString, 1, 3)),
        ("{1", SyntaxError(KeyMustBeAString, 1, 2)),
        ("{ \"a\"", SyntaxError(EOFWhileParsingObject, 1, 6)),
        ("{\"a\"", SyntaxError(EOFWhileParsingObject, 1, 5)),
        ("{\"a\" ", SyntaxError(EOFWhileParsingObject, 1, 6)),
        ("{\"a\" 1", SyntaxError(ExpectedColon, 1, 6)),
        ("{\"a\":", SyntaxError(EOFWhileParsingValue, 1, 6)),
        ("{\"a\":1", SyntaxError(EOFWhileParsingObject, 1, 7)),
        ("{\"a\":1 1", SyntaxError(ExpectedObjectCommaOrEnd, 1, 8)),
        ("{\"a\":1,", SyntaxError(EOFWhileParsingString, 1, 8)),
        ("{}a", SyntaxError(TrailingCharacters, 1, 3)),
    ]);

    test_parse_ok(&[
        ("{}", treemap!()),
        ("{ }", treemap!()),
        (
            "{\"a\":3}",
            treemap!("a".to_string() => 3is)
        ),
        (
            "{ \"a\" : 3 }",
            treemap!("a".to_string() => 3is)
        ),
        (
            "{\"a\":3,\"b\":4}",
            treemap!("a".to_string() => 3, "b".to_string() => 4)
        ),
        (
            "{ \"a\" : 3 , \"b\" : 4 }",
            treemap!("a".to_string() => 3, "b".to_string() => 4),
        ),
    ]);

    test_parse_ok(&[
        (
            "{\"a\": {\"b\": 3, \"c\": 4}}",
            treemap!("a".to_string() => treemap!("b".to_string() => 3, "c".to_string() => 4is)),
        ),
    ]);
}

#[test]
fn test_json_deserialize_object() {
    test_json_deserialize_ok(&[
        treemap!(),
        treemap!("a".to_string() => 3),
        treemap!("a".to_string() => 3, "b".to_string() => 4),
    ]);

    test_json_deserialize_ok(&[
        treemap!("a".to_string() => treemap!("b".to_string() => 3, "c".to_string() => 4)),
    ]);
}

#[test]
fn test_parse_struct() {
    test_parse_ok(&[
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
}

#[test]
fn test_json_deserialize_struct() {
    test_json_deserialize_ok(&[
        Outer {
            inner: vec![
                Inner { a: (), b: 2, c: vec!["abc".to_string(), "xyz".to_string()] }
            ]
        },
    ]);
}

#[test]
fn test_parse_option() {
    test_parse_ok(&[
        ("null", None),
        ("\"jodhpurs\"", Some("jodhpurs".to_string())),
    ]);

    #[derive(PartialEq, Debug)]
    #[derive_serialize]
    #[derive_deserialize]
    struct Foo {
        x: Option<isize>,
    }

    let value: Foo = from_str("{}").unwrap();
    assert_eq!(value, Foo { x: None });

    let value: Foo = from_str("{ \"x\": 5 }").unwrap();
    assert_eq!(value, Foo { x: Some(5) });
}

#[test]
fn test_json_deserialize_option() {
    test_json_deserialize_ok(&[
        None,
        Some("jodhpurs".to_string()),
    ]);
}

#[test]
fn test_parse_enum() {
    test_parse_ok(&[
        ("{\"Dog\": []}", Animal::Dog),
        (
            "{\"Frog\": [\"Henry\", []]}",
            Animal::Frog("Henry".to_string(), vec!()),
        ),
        (
            "{\"Frog\": [\"Henry\", [349]]}",
            Animal::Frog("Henry".to_string(), vec!(349)),
        ),
        (
            "{\"Frog\": [\"Henry\", [349, 102]]}",
            Animal::Frog("Henry".to_string(), vec!(349, 102)),
        ),
    ]);

    test_parse_ok(&[
        (
            concat!(
                "{",
                "  \"a\": {\"Dog\": []},",
                "  \"b\": {\"Frog\":[\"Henry\", []]}",
                "}"
            ),
            treemap!(
                "a".to_string() => Animal::Dog,
                "b".to_string() => Animal::Frog("Henry".to_string(), vec!())
            )
        ),
    ]);
}

#[test]
fn test_json_deserialize_enum() {
    test_json_deserialize_ok(&[
        Animal::Dog,
        Animal::Frog("Henry".to_string(), vec!()),
        Animal::Frog("Henry".to_string(), vec!(349)),
        Animal::Frog("Henry".to_string(), vec!(349, 102)),
    ]);
}

#[test]
fn test_multiline_errors() {
    test_parse_err::<BTreeMap<string::String, string::String>>(&[
        ("{\n  \"foo\":\n \"bar\"", SyntaxError(EOFWhileParsingObject, 3us, 8us)),
    ]);
}

/*
#[derive(Decodable)]
struct DecodeStruct {
    x: f64,
    y: bool,
    z: String,
    w: Vec<DecodeStruct>
}
#[derive(Decodable)]
enum DecodeEnum {
    A(f64),
    B(String)
}
fn check_err<T: RustcDecodable<Decoder, DecoderError>>(to_parse: &'static str,
                                                  expected: DecoderError) {
    let res: DecodeResult<T> = match from_str(to_parse) {
        Err(e) => Err(ParseError(e)),
        Ok(json) => RustcDecodable::decode(&mut Decoder::new(json))
    };
    match res {
        Ok(_) => panic!("`{}` parsed & decoded ok, expecting error `{}`",
                          to_parse, expected),
        Err(ParseError(e)) => panic!("`{}` is not valid json: {}",
                                       to_parse, e),
        Err(e) => {
            assert_eq!(e, expected);
        }
    }
}
#[test]
fn test_decode_errors_struct() {
    check_err::<DecodeStruct>("[]", ExpectedError("Object".to_string(), "[]".to_string()));
    check_err::<DecodeStruct>("{\"x\": true, \"y\": true, \"z\": \"\", \"w\": []}",
                              ExpectedError("Number".to_string(), "true".to_string()));
    check_err::<DecodeStruct>("{\"x\": 1, \"y\": [], \"z\": \"\", \"w\": []}",
                              ExpectedError("Bool".to_string(), "[]".to_string()));
    check_err::<DecodeStruct>("{\"x\": 1, \"y\": true, \"z\": {}, \"w\": []}",
                              ExpectedError("String".to_string(), "{}".to_string()));
    check_err::<DecodeStruct>("{\"x\": 1, \"y\": true, \"z\": \"\", \"w\": null}",
                              ExpectedError("List".to_string(), "null".to_string()));
    check_err::<DecodeStruct>("{\"x\": 1, \"y\": true, \"z\": \"\"}",
                              MissingFieldError("w".to_string()));
}
#[test]
fn test_decode_errors_enum() {
    check_err::<DecodeEnum>("{}",
                            MissingFieldError("variant".to_string()));
    check_err::<DecodeEnum>("{\"variant\": 1}",
                            ExpectedError("String".to_string(), "1".to_string()));
    check_err::<DecodeEnum>("{\"variant\": \"A\"}",
                            MissingFieldError("fields".to_string()));
    check_err::<DecodeEnum>("{\"variant\": \"A\", \"fields\": null}",
                            ExpectedError("List".to_string(), "null".to_string()));
    check_err::<DecodeEnum>("{\"variant\": \"C\", \"fields\": []}",
                            UnknownVariantError("C".to_string()));
}
*/

#[test]
fn test_find(){
    let json_value: Value = from_str("{\"dog\" : \"cat\"}").unwrap();
    let found_str = json_value.find(&"dog".to_string());
    assert!(found_str.is_some() && found_str.unwrap().as_string().unwrap() == "cat");
}

#[test]
fn test_find_path(){
    let json_value: Value = from_str("{\"dog\":{\"cat\": {\"mouse\" : \"cheese\"}}}").unwrap();
    let found_str = json_value.find_path(&[&"dog".to_string(),
                                         &"cat".to_string(), &"mouse".to_string()]);
    assert!(found_str.is_some() && found_str.unwrap().as_string().unwrap() == "cheese");
}

#[test]
fn test_search(){
    let json_value: Value = from_str("{\"dog\":{\"cat\": {\"mouse\" : \"cheese\"}}}").unwrap();
    let found_str = json_value.search(&"mouse".to_string()).and_then(|j| j.as_string());
    assert!(found_str.is_some());
    assert!(found_str.unwrap() == "cheese");
}

#[test]
fn test_is_object() {
    let json_value: Value = from_str("{}").unwrap();
    assert!(json_value.is_object());
}

#[test]
fn test_as_object() {
    let json_value: Value = from_str("{}").unwrap();
    let map = BTreeMap::<string::String, Value>::new();
    let json_object = json_value.as_object();
    assert_eq!(json_object, Some(&map));
}

#[test]
fn test_is_array() {
    let json_value: Value = from_str("[1, 2, 3]").unwrap();
    assert!(json_value.is_array());
}

#[test]
fn test_as_array() {
    let json_value: Value = from_str("[1, 2, 3]").unwrap();
    let json_list = json_value.as_array();
    let expected_length = 3;
    assert!(json_list.is_some() && json_list.unwrap().len() == expected_length);
}

#[test]
fn test_is_string() {
    let json_value: Value = from_str("\"dog\"").unwrap();
    assert!(json_value.is_string());
}

#[test]
fn test_as_string() {
    let json_value: Value = from_str("\"dog\"").unwrap();
    let json_str = json_value.as_string();
    let expected_str = "dog";
    assert_eq!(json_str, Some(expected_str));
}

#[test]
fn test_is_number() {
    let json_value: Value = from_str("12").unwrap();
    assert!(json_value.is_number());

    let json_value: Value = from_str("12.0").unwrap();
    assert!(json_value.is_number());
}

#[test]
fn test_is_i64() {
    let json_value: Value = from_str("12").unwrap();
    assert!(json_value.is_i64());

    let json_value: Value = from_str("12.0").unwrap();
    assert!(!json_value.is_i64());
}

#[test]
fn test_is_f64() {
    let json_value: Value = from_str("12").unwrap();
    assert!(!json_value.is_f64());

    let json_value: Value = from_str("12.0").unwrap();
    assert!(json_value.is_f64());
}

#[test]
fn test_as_i64() {
    let json_value: Value = from_str("12").unwrap();
    assert_eq!(json_value.as_i64(), Some(12));
}

#[test]
fn test_as_f64() {
    let json_value: Value = from_str("12").unwrap();
    assert_eq!(json_value.as_f64(), Some(12.0));
}

#[test]
fn test_is_boolean() {
    let json_value: Value = from_str("false").unwrap();
    assert!(json_value.is_boolean());
}

#[test]
fn test_as_boolean() {
    let json_value: Value = from_str("false").unwrap();
    let json_bool = json_value.as_boolean();
    let expected_bool = false;
    assert!(json_bool.is_some() && json_bool.unwrap() == expected_bool);
}

#[test]
fn test_is_null() {
    let json_value: Value = from_str("null").unwrap();
    assert!(json_value.is_null());
}

#[test]
fn test_as_null() {
    let json_value: Value = from_str("null").unwrap();
    let json_null = json_value.as_null();
    let expected_null = ();
    assert!(json_null.is_some() && json_null.unwrap() == expected_null);
}

/*
#[test]
fn test_encode_hashmap_with_numeric_key() {
    use std::str::from_utf8;
    use std::collections::HashMap;
    let mut hm: HashMap<usize, bool> = HashMap::new();
    hm.insert(1, true);
    let mut mem_buf = MemWriter::new();
    {
        let mut serializer = Serializer::new(&mut mem_buf as &mut Writer);
        hm.serialize(&mut serializer).unwrap();
    }
    let bytes = mem_buf.unwrap();
    let json_str = from_utf8(&bytes).unwrap();
    let _json_value: Value = from_str(json_str).unwrap();
}

/*
#[test]
fn test_prettyencode_hashmap_with_numeric_key() {
    use std::str::from_utf8;
    use std::collections::HashMap;
    let mut hm: HashMap<usize, bool> = HashMap::new();
    hm.insert(1, true);
    let mut mem_buf = MemWriter::new();
    {
        let mut serializer = PrettySerializer::new(&mut mem_buf as &mut Writer);
        hm.serialize(&mut serializer).unwrap()
    }
    let bytes = mem_buf.unwrap();
    let json_str = from_utf8(&bytes).unwrap();
    let _json_value: Value = from_str(json_str).unwrap();
}
*/

#[test]
fn test_hashmap_with_numeric_key_can_handle_double_quote_delimited_key() {
    use std::collections::HashMap;
    let json_str = "{\"1\":true}";
    let map: HashMap<usize, bool> = from_str(json_str).unwrap();
    let mut m = HashMap::new();
    m.insert(1u, true);
    assert_eq!(map, m);
}
*/

/*
fn assert_stream_equal(src: &str, expected: ~[(JsonEvent, ~[StackElement])]) {
    let mut parser = Deserializer::new(src.chars());
    let mut i = 0;
    loop {
        let evt = match parser.next() {
            Some(e) => e,
            None => { break; }
        };
        let (ref expected_evt, ref expected_stack) = expected[i];
        if !parser.stack().is_equal_to(&expected_stack) {
            panic!("Deserializer stack is not equal to {}", expected_stack);
        }
        assert_eq!(&evt, expected_evt);
        i+=1;
    }
}
#[test]
fn test_streaming_parser() {
    assert_stream_equal(
        r#"{ "foo":"bar", "array" : [0, 1, 2,3 ,4,5], "idents":[null,true,false]}"#,
        ~[
            (ObjectStart,             ~[]),
              (StringValue("bar".to_string()),   ~[Key("foo")]),
              (ListStart,             ~[Key("array")]),
                (NumberValue(0.0),    ~[Key("array"), Index(0)]),
                (NumberValue(1.0),    ~[Key("array"), Index(1)]),
                (NumberValue(2.0),    ~[Key("array"), Index(2)]),
                (NumberValue(3.0),    ~[Key("array"), Index(3)]),
                (NumberValue(4.0),    ~[Key("array"), Index(4)]),
                (NumberValue(5.0),    ~[Key("array"), Index(5)]),
              (ListEnd,               ~[Key("array")]),
              (ListStart,             ~[Key("idents")]),
                (NullValue,           ~[Key("idents"), Index(0)]),
                (BoolValue(true),  ~[Key("idents"), Index(1)]),
                (BoolValue(false), ~[Key("idents"), Index(2)]),
              (ListEnd,               ~[Key("idents")]),
            (ObjectEnd,               ~[]),
        ]
    );
}
fn last_event(src: &str) -> JsonEvent {
    let mut parser = Deserializer::new(src.chars());
    let mut evt = NullValue;
    loop {
        evt = match parser.next() {
            Some(e) => e,
            None => return evt,
        }
    }
}
#[test]
#[ignore(cfg(target_word_size = "32"))] // FIXME(#14064)
fn test_read_object_streaming() {
    assert_eq!(last_event("{ "),      Error(SyntaxError(EOFWhileParsingObject, 1, 3)));
    assert_eq!(last_event("{1"),      Error(SyntaxError(KeyMustBeAString,      1, 2)));
    assert_eq!(last_event("{ \"a\""), Error(SyntaxError(EOFWhileParsingObject, 1, 6)));
    assert_eq!(last_event("{\"a\""),  Error(SyntaxError(EOFWhileParsingObject, 1, 5)));
    assert_eq!(last_event("{\"a\" "), Error(SyntaxError(EOFWhileParsingObject, 1, 6)));

    assert_eq!(last_event("{\"a\" 1"),   Error(SyntaxError(ExpectedColon,         1, 6)));
    assert_eq!(last_event("{\"a\":"),    Error(SyntaxError(EOFWhileParsingValue,  1, 6)));
    assert_eq!(last_event("{\"a\":1"),   Error(SyntaxError(EOFWhileParsingObject, 1, 7)));
    assert_eq!(last_event("{\"a\":1 1"), Error(SyntaxError(InvalidSyntax,         1, 8)));
    assert_eq!(last_event("{\"a\":1,"),  Error(SyntaxError(EOFWhileParsingObject, 1, 8)));

    assert_stream_equal(
        "{}",
        box [(ObjectStart, box []), (ObjectEnd, box [])]
    );
    assert_stream_equal(
        "{\"a\": 3}",
        box [
            (ObjectStart,        box []),
              (F64Value(3.0), box [Key("a")]),
            (ObjectEnd,          box []),
        ]
    );
    assert_stream_equal(
        "{ \"a\": null, \"b\" : true }",
        box [
            (ObjectStart,           box []),
              (NullValue,           box [Key("a")]),
              (BoolValue(true),  box [Key("b")]),
            (ObjectEnd,             box []),
        ]
    );
    assert_stream_equal(
        "{\"a\" : 1.0 ,\"b\": [ true ]}",
        box [
            (ObjectStart,           box []),
              (F64Value(1.0),    box [Key("a")]),
              (ListStart,           box [Key("b")]),
                (BoolValue(true),box [Key("b"), Index(0)]),
              (ListEnd,             box [Key("b")]),
            (ObjectEnd,             box []),
        ]
    );
    assert_stream_equal(
        r#"{
            "a": 1.0,
            "b": [
                true,
                "foo\nbar",
                { "c": {"d": null} }
            ]
        }"#,
        ~[
            (ObjectStart,                   ~[]),
              (F64Value(1.0),            ~[Key("a")]),
              (ListStart,                   ~[Key("b")]),
                (BoolValue(true),        ~[Key("b"), Index(0)]),
                (StringValue("foo\nbar".to_string()),  ~[Key("b"), Index(1)]),
                (ObjectStart,               ~[Key("b"), Index(2)]),
                  (ObjectStart,             ~[Key("b"), Index(2), Key("c")]),
                    (NullValue,             ~[Key("b"), Index(2), Key("c"), Key("d")]),
                  (ObjectEnd,               ~[Key("b"), Index(2), Key("c")]),
                (ObjectEnd,                 ~[Key("b"), Index(2)]),
              (ListEnd,                     ~[Key("b")]),
            (ObjectEnd,                     ~[]),
        ]
    );
}
#[test]
#[ignore(cfg(target_word_size = "32"))] // FIXME(#14064)
fn test_read_list_streaming() {
    assert_stream_equal(
        "[]",
        box [
            (ListStart, box []),
            (ListEnd,   box []),
        ]
    );
    assert_stream_equal(
        "[ ]",
        box [
            (ListStart, box []),
            (ListEnd,   box []),
        ]
    );
    assert_stream_equal(
        "[true]",
        box [
            (ListStart,              box []),
                (BoolValue(true), box [Index(0)]),
            (ListEnd,                box []),
        ]
    );
    assert_stream_equal(
        "[ false ]",
        box [
            (ListStart,               box []),
                (BoolValue(false), box [Index(0)]),
            (ListEnd,                 box []),
        ]
    );
    assert_stream_equal(
        "[null]",
        box [
            (ListStart,     box []),
                (NullValue, box [Index(0)]),
            (ListEnd,       box []),
        ]
    );
    assert_stream_equal(
        "[3, 1]",
        box [
            (ListStart,     box []),
                (F64Value(3.0), box [Index(0)]),
                (F64Value(1.0), box [Index(1)]),
            (ListEnd,       box []),
        ]
    );
    assert_stream_equal(
        "\n[3, 2]\n",
        box [
            (ListStart,     box []),
                (F64Value(3.0), box [Index(0)]),
                (F64Value(2.0), box [Index(1)]),
            (ListEnd,       box []),
        ]
    );
    assert_stream_equal(
        "[2, [4, 1]]",
        box [
            (ListStart,                 box []),
                (F64Value(2.0),      box [Index(0)]),
                (ListStart,             box [Index(1)]),
                    (F64Value(4.0),  box [Index(1), Index(0)]),
                    (F64Value(1.0),  box [Index(1), Index(1)]),
                (ListEnd,               box [Index(1)]),
            (ListEnd,                   box []),
        ]
    );

    assert_eq!(last_event("["), Error(SyntaxError(EOFWhileParsingValue, 1,  2)));

    assert_eq!(from_str("["),     Err(SyntaxError(EOFWhileParsingValue, 1, 2)));
    assert_eq!(from_str("[1"),    Err(SyntaxError(EOFWhileParsingList,  1, 3)));
    assert_eq!(from_str("[1,"),   Err(SyntaxError(EOFWhileParsingValue, 1, 4)));
    assert_eq!(from_str("[1,]"),  Err(SyntaxError(InvalidSyntax,        1, 4)));
    assert_eq!(from_str("[6 7]"), Err(SyntaxError(InvalidSyntax,        1, 4)));

}
#[test]
fn test_trailing_characters_streaming() {
    assert_eq!(last_event("nulla"),  Error(SyntaxError(TrailingCharacters, 1, 5)));
    assert_eq!(last_event("truea"),  Error(SyntaxError(TrailingCharacters, 1, 5)));
    assert_eq!(last_event("falsea"), Error(SyntaxError(TrailingCharacters, 1, 6)));
    assert_eq!(last_event("1a"),     Error(SyntaxError(TrailingCharacters, 1, 2)));
    assert_eq!(last_event("[]a"),    Error(SyntaxError(TrailingCharacters, 1, 3)));
    assert_eq!(last_event("{}a"),    Error(SyntaxError(TrailingCharacters, 1, 3)));
}
#[test]
fn test_read_identifiers_streaming() {
    assert_eq!(Deserializer::new("null".chars()).next(), Some(NullValue));
    assert_eq!(Deserializer::new("true".chars()).next(), Some(BoolValue(true)));
    assert_eq!(Deserializer::new("false".chars()).next(), Some(BoolValue(false)));

    assert_eq!(last_event("n"),    Error(SyntaxError(InvalidSyntax, 1, 2)));
    assert_eq!(last_event("nul"),  Error(SyntaxError(InvalidSyntax, 1, 4)));
    assert_eq!(last_event("t"),    Error(SyntaxError(InvalidSyntax, 1, 2)));
    assert_eq!(last_event("truz"), Error(SyntaxError(InvalidSyntax, 1, 4)));
    assert_eq!(last_event("f"),    Error(SyntaxError(InvalidSyntax, 1, 2)));
    assert_eq!(last_event("faz"),  Error(SyntaxError(InvalidSyntax, 1, 3)));
}

#[test]
fn test_stack() {
    let mut stack = Stack::new();

    assert!(stack.is_empty());
    assert!(stack.len() == 0);
    assert!(!stack.last_is_index());

    stack.push_index(0);
    stack.bump_index();

    assert!(stack.len() == 1);
    assert!(stack.is_equal_to(&[Index(1)]));
    assert!(stack.starts_with(&[Index(1)]));
    assert!(stack.ends_with(&[Index(1)]));
    assert!(stack.last_is_index());
    assert!(stack.get(0) == Index(1));

    stack.push_key("foo".to_string());

    assert!(stack.len() == 2);
    assert!(stack.is_equal_to(&[Index(1), Key("foo")]));
    assert!(stack.starts_with(&[Index(1), Key("foo")]));
    assert!(stack.starts_with(&[Index(1)]));
    assert!(stack.ends_with(&[Index(1), Key("foo")]));
    assert!(stack.ends_with(&[Key("foo")]));
    assert!(!stack.last_is_index());
    assert!(stack.get(0) == Index(1));
    assert!(stack.get(1) == Key("foo"));

    stack.push_key("bar".to_string());

    assert!(stack.len() == 3);
    assert!(stack.is_equal_to(&[Index(1), Key("foo"), Key("bar")]));
    assert!(stack.starts_with(&[Index(1)]));
    assert!(stack.starts_with(&[Index(1), Key("foo")]));
    assert!(stack.starts_with(&[Index(1), Key("foo"), Key("bar")]));
    assert!(stack.ends_with(&[Key("bar")]));
    assert!(stack.ends_with(&[Key("foo"), Key("bar")]));
    assert!(stack.ends_with(&[Index(1), Key("foo"), Key("bar")]));
    assert!(!stack.last_is_index());
    assert!(stack.get(0) == Index(1));
    assert!(stack.get(1) == Key("foo"));
    assert!(stack.get(2) == Key("bar"));

    stack.pop();

    assert!(stack.len() == 2);
    assert!(stack.is_equal_to(&[Index(1), Key("foo")]));
    assert!(stack.starts_with(&[Index(1), Key("foo")]));
    assert!(stack.starts_with(&[Index(1)]));
    assert!(stack.ends_with(&[Index(1), Key("foo")]));
    assert!(stack.ends_with(&[Key("foo")]));
    assert!(!stack.last_is_index());
    assert!(stack.get(0) == Index(1));
    assert!(stack.get(1) == Key("foo"));
}
*/
*/
