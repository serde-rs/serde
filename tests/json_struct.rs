#![feature(phase)]

extern crate serde;
#[phase(plugin)]
extern crate serde_macros;

#[deriving(PartialEq, Show)]
#[deriving_serialize]
#[deriving_deserialize]
struct Test {
    #[serial_name = "$schema"]
    schema: String,
    title: String,
    #[serial_name = "type"]
    ty: int
}

#[test]
fn test_json_struct() {
    let input = Test {
        schema: "a".to_string(),
        title: "b".to_string(),
        ty: 3,
    };

    let s = serde::json::to_string(&input).unwrap();
    assert_eq!(s.as_slice(), r#"{"$schema":"a","title":"b","type":3}"#);

    let output: Test = serde::json::from_str(s.as_slice()).unwrap();
    assert_eq!(input, output);
}
