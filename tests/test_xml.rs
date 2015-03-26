#![feature(custom_derive, plugin, test)]
#![plugin(serde_macros)]
#![feature(custom_attribute)]

extern crate test;
extern crate serde;

use std::fmt::Debug;

use serde::xml::{
    from_str,
};

use serde::de;
use serde::ser;

#[derive(PartialEq, Debug, Serialize, Deserialize)]
enum Animal {
    Dog,
    Frog(String, Vec<isize>),
    Cat { age: usize, name: String },
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
struct Simple {
    a: (),
    b: usize,
    c: String,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
struct Inner {
    a: (),
    b: (usize, String, i8),
    c: Vec<String>,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
struct Outer {
    inner: Option<Inner>,
}

fn test_parse_ok<'a, T>(errors: &[(&'a str, T)])
where T: PartialEq + Debug + ser::Serialize + de::Deserialize,
{
    for &(s, ref value) in errors {
        let v: T = from_str(s).unwrap();
        assert_eq!(v, *value);
    }
}

#[test]
fn test_parse_string() {

    test_parse_ok(&[
        (
            "<bla>This is a String</bla>",
            "This is a String".to_string(),
        ),
        (
            "<bla></bla>",
            "".to_string(),
        )
    ]);
}

#[test]
fn test_parse_i64() {
    test_parse_ok(&[
        ("<bla>0</bla>", 0),
        ("<bla>-2</bla>", -2),
        ("<bla>-1234</bla>", -1234),
        ("<bla> -1234 </bla>", -1234),
    ]);
}

#[test]
fn test_parse_u64() {
    test_parse_ok(&[
        ("<bla>0</bla>", 0),
        ("<bla>1234</bla>", 1234),
        ("<bla> 1234 </bla>", 1234),
    ]);
}

#[test]
fn test_parse_bool() {
    test_parse_ok(&[
        ("<bla>true</bla>", true),
        ("<bla>false</bla>", false),
        ("<bla> true </bla>", true),
        ("<bla> false </bla>", false),
    ]);
}

#[test]
fn test_parse_f64() {
    test_parse_ok(&[
        ("<bla>3.0</bla>", 3.0f64),
        ("<bla>3.1</bla>", 3.1),
        ("<bla>-1.2</bla>", -1.2),
        ("<bla>0.4</bla>", 0.4),
        ("<bla>0.4e5</bla>", 0.4e5),
        ("<bla>0.4e15</bla>", 0.4e15),
        //("<bla>0.4e-01</bla>", 0.4e-01), // precision troubles
        //("<bla> 0.4e-01 </bla>", 0.4e-01),
    ]);
}

#[test]
fn test_parse_struct() {

    test_parse_ok(&[
        (
            "<Simple>
                <c>abc</c>
                <a/>
                <b>2</b>
            </Simple>",
            Simple {
                a: (),
                b: 2,
                c: "abc".to_string(),
            },
        )
    ]);
}

#[test]
fn test_parse_xml_value() {
    #[derive(Eq, Debug, PartialEq, Deserialize, Serialize)]
    struct Test {
        #[serde(alias="$value")]
        myval: String,
    }
    test_parse_ok(&[
        (
            "<Test>abc</Test>",
            Test { myval: "abc".to_string() },
        )
    ]);
}

#[test]
fn test_parse_complexstruct() {

    test_parse_ok(&[
        (
            "<Outer>
                <inner>
                    <b>2</b>
                    <b>boom</b>
                    <b>88</b>
                </inner>
            </Outer>",
            Outer {
                inner: Some(Inner {
                    a: (),
                    b: (2, "boom".to_string(), 88),
                    c: vec![]
                })
            },
        ),
        (
            "<Outer>
                <inner>
                    <c>abc</c>
                    <c>xyz</c>
                    <a/>
                    <b>2</b>
                    <b>boom</b>
                    <b>88</b>
                </inner>
            </Outer>",
            Outer {
                inner: Some(Inner {
                    a: (),
                    b: (2, "boom".to_string(), 88),
                    c: vec![
                        "abc".to_string(),
                        "xyz".to_string(),
                    ]
                })
            },
        ),
        (
            "<Outer>
            </Outer>",
            Outer {
                inner: None
            },
        ),
        (
            "<Outer/>",
            Outer {
                inner: None
            },
        )
    ]);
}
