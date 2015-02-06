#![feature(plugin)]

extern crate serde;
#[plugin]
extern crate serde_macros;

#[derive(PartialEq, Debug)]
#[derive_serialize]
#[derive_deserialize]
struct Test {
    #[serial_name = "$schema"]
    schema: String,
    title: String,
    #[serial_name = "type"]
    ty: isize
}

#[test]
fn test_json_struct() {
    let input = Test {
        schema: "a".to_string(),
        title: "b".to_string(),
        ty: 3,
    };

    let s = serde::json::to_string(&input).unwrap();
    assert_eq!(&s[], r#"{"$schema":"a","title":"b","type":3}"#);

    let output: Test = serde::json::from_str(&s).unwrap();
    assert_eq!(input, output);
}
