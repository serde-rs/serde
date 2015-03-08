#![feature(custom_derive, plugin)]
#![plugin(serde2_macros)]

extern crate serde2;

use std::collections::BTreeMap;
use serde2::json::{self, Value};

macro_rules! btreemap {
    () => {
        BTreeMap::new()
    };
    ($($key:expr => $value:expr),+) => {
        {
            let mut map = BTreeMap::new();
            $(map.insert($key, $value);)+
            map
        }
    }
}

/*
trait Trait {
    type Type;
}
*/

#[derive(Debug, PartialEq)]
#[derive_serialize]
#[derive_deserialize]
struct NamedUnit;

#[derive(Debug, PartialEq)]
#[derive_serialize]
struct NamedMap<'a, 'b, A: 'a, B: 'b, C> {
    a: &'a A,
    b: &'b mut B,
    c: C,
}

#[derive(Debug, PartialEq)]
#[derive_serialize]
//#[derive_deserialize]
enum Enum<'a, B: 'a, C: /* Trait + */ 'a, D> where D: /* Trait + */ 'a {
    Unit,
    Seq(
        i8,
        B,
        &'a C,
        //B::Type,
        &'a mut D,
        //<D as Trait>::Type,
    ),
    Map {
        a: i8,
        b: B,
        c: &'a C,
        //d: C::Type,
        e: &'a mut D,
        //f: <D as Trait>::Type,
    },
}

#[test]
fn test_named_unit() {
    let named_unit = NamedUnit;

    assert_eq!(
        json::to_string(&named_unit).unwrap(),
        "null".to_string()
    );

    assert_eq!(
        json::to_value(&named_unit),
        Value::Null
    );

    let v = json::from_str("null").unwrap();
    assert_eq!(v, named_unit);

    let v = json::from_value(Value::Null).unwrap();
    assert_eq!(v, named_unit);
}

#[test]
fn test_ser_named_tuple() {
    #[derive(Debug, PartialEq)]
    #[derive_serialize]
    struct NamedTuple<'a, 'b, A: 'a, B: 'b, C>(&'a A, &'b mut B, C);

    let a = 5;
    let mut b = 6;
    let c = 7;
    let named_tuple = NamedTuple(&a, &mut b, c);

    assert_eq!(
        json::to_string(&named_tuple).unwrap(),
        "[5,6,7]"
    );

    assert_eq!(
        json::to_value(&named_tuple),
        Value::Array(vec![Value::I64(5), Value::I64(6), Value::I64(7)])
    );
}

#[test]
fn test_de_named_tuple() {
    #[derive(Debug, PartialEq)]
    #[derive_deserialize]
    struct NamedTuple<A, B, C>(A, B, C);

    assert_eq!(
        json::from_str("[1,2,3]").unwrap(),
        NamedTuple(1, 2, 3)
    );

    assert_eq!(
        json::from_str("[1,2,3]").unwrap(),
        Value::Array(vec![
            Value::I64(1),
            Value::I64(2),
            Value::I64(3),
        ])
    );
}

#[test]
fn test_named_map() {
    let a = 5;
    let mut b = 6;
    let c = 7;
    let named_map = NamedMap {
        a: &a,
        b: &mut b,
        c: c,
    };

    assert_eq!(
        json::to_string(&named_map).unwrap(),
        "{\"a\":5,\"b\":6,\"c\":7}"
    );

    assert_eq!(
        json::to_value(&named_map),
        Value::Object(btreemap![
            "a".to_string() => Value::I64(5),
            "b".to_string() => Value::I64(6),
            "c".to_string() => Value::I64(7)
        ])
    );
}

#[test]
fn test_enum_unit() {
    assert_eq!(
        json::to_string(&Enum::Unit::<u32, u32, u32>).unwrap(),
        "{\"Unit\":[]}"
    );

    assert_eq!(
        json::to_value(&Enum::Unit::<u32, u32, u32>),
        Value::Object(btreemap!(
            "Unit".to_string() => Value::Array(vec![]))
        )
    );
}

#[test]
fn test_enum_seq() {
    let a = 1;
    let b = 2;
    let c = 3;
    //let d = 4;
    let mut e = 5;
    //let f = 6;

    assert_eq!(
        json::to_string(&Enum::Seq(
            a,
            b,
            &c,
            //d,
            &mut e,
            //f,
        )).unwrap(),
        "{\"Seq\":[1,2,3,5]}".to_string()
    );

    assert_eq!(
        json::to_value(&Enum::Seq(
            a,
            b,
            &c,
            //d,
            &mut e,
            //e,
        )),
        Value::Object(btreemap!(
            "Seq".to_string() => Value::Array(vec![
                Value::I64(1),
                Value::I64(2),
                Value::I64(3),
                //Value::I64(4),
                Value::I64(5),
                //Value::I64(6),
            ])
        ))
    );
}

#[test]
fn test_enum_map() {
    let a = 1;
    let b = 2;
    let c = 3;
    //let d = 4;
    let mut e = 5;
    //let f = 6;

    assert_eq!(
        json::to_string(&Enum::Map {
            a: a,
            b: b,
            c: &c,
            //d: d,
            e: &mut e,
            //f: f,
        }).unwrap(),
        "{\"Map\":{\"a\":1,\"b\":2,\"c\":3,\"e\":5}}".to_string()
    );

    assert_eq!(
        json::to_value(&Enum::Map {
            a: a,
            b: b,
            c: &c,
            //d: d,
            e: &mut e,
            //f: f,
        }),
        Value::Object(btreemap!(
            "Map".to_string() => Value::Object(btreemap![
                "a".to_string() => Value::I64(1),
                "b".to_string() => Value::I64(2),
                "c".to_string() => Value::I64(3),
                //"d".to_string() => Value::I64(4)
                "e".to_string() => Value::I64(5)
                //"f".to_string() => Value::I64(6)
            ])
        ))
    );
}
