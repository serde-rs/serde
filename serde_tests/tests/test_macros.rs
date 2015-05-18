use std::collections::BTreeMap;
use serde::json::{self, Value};

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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct NamedUnit;

#[derive(Debug, PartialEq, Serialize)]
struct SerNamedTuple<'a, 'b, A: 'a, B: 'b, C>(&'a A, &'b mut B, C);

#[derive(Debug, PartialEq, Deserialize)]
struct DeNamedTuple<A, B, C>(A, B, C);

#[derive(Debug, PartialEq, Serialize)]
struct SerNamedMap<'a, 'b, A: 'a, B: 'b, C> {
    a: &'a A,
    b: &'b mut B,
    c: C,
}

#[derive(Debug, PartialEq, Deserialize)]
struct DeNamedMap<A, B, C> {
    a: A,
    b: B,
    c: C,
}

#[derive(Debug, PartialEq, Serialize)]
enum SerEnum<'a, B: 'a, C: /* Trait + */ 'a, D> where D: /* Trait + */ 'a {
    Unit,
    Seq(
        i8,
        B,
        &'a C,
        //C::Type,
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

    // Make sure we can support more than one variant.
    _Unit2,
    _Seq2(
        i8,
        B,
        &'a C,
        //C::Type,
        &'a mut D,
        //<D as Trait>::Type,
    ),
    _Map2 {
        a: i8,
        b: B,
        c: &'a C,
        //d: C::Type,
        e: &'a mut D,
        //f: <D as Trait>::Type,
    },
}

#[derive(Debug, PartialEq, Deserialize)]
enum DeEnum<B, C: /* Trait */, D> /* where D: Trait */ {
    Unit,
    Seq(
        i8,
        B,
        C,
        //C::Type,
        D,
        //<D as Trait>::Type,
    ),
    Map {
        a: i8,
        b: B,
        c: C,
        //d: C::Type,
        e: D,
        //f: <D as Trait>::Type,
    },

    // Make sure we can support more than one variant.
    _Unit2,
    _Seq2(
        i8,
        B,
        C,
        //C::Type,
        D,
        //<D as Trait>::Type,
    ),
    _Map2 {
        a: i8,
        b: B,
        c: C,
        //d: C::Type,
        e: D,
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

    let v: NamedUnit = json::from_str("null").unwrap();
    assert_eq!(v, named_unit);

    let v: NamedUnit = json::from_value(Value::Null).unwrap();
    assert_eq!(v, named_unit);
}

#[test]
fn test_ser_named_tuple() {
    let a = 5;
    let mut b = 6;
    let c = 7;
    let named_tuple = SerNamedTuple(&a, &mut b, c);

    assert_eq!(
        json::to_string(&named_tuple).unwrap(),
        "[5,6,7]"
    );

    assert_eq!(
        json::to_value(&named_tuple),
        Value::Array(vec![Value::U64(5), Value::U64(6), Value::U64(7)])
    );
}

#[test]
fn test_de_named_tuple() {
    let v: DeNamedTuple<i32, i32, i32> = json::from_str("[1,2,3]").unwrap();
    assert_eq!(
        v,
        DeNamedTuple(1, 2, 3)
    );

    let v: Value = json::from_str("[1,2,3]").unwrap();
    assert_eq!(
        v,
        Value::Array(vec![
            Value::U64(1),
            Value::U64(2),
            Value::U64(3),
        ])
    );
}

#[test]
fn test_ser_named_map() {
    let a = 5;
    let mut b = 6;
    let c = 7;
    let named_map = SerNamedMap {
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
            "a".to_string() => Value::U64(5),
            "b".to_string() => Value::U64(6),
            "c".to_string() => Value::U64(7)
        ])
    );
}

#[test]
fn test_de_named_map() {
    let v = DeNamedMap {
        a: 5,
        b: 6,
        c: 7,
    };

    let v2: DeNamedMap<i32, i32, i32> = json::from_str(
        "{\"a\":5,\"b\":6,\"c\":7}"
    ).unwrap();
    assert_eq!(v, v2);

    let v2 = json::from_value(Value::Object(btreemap![
        "a".to_string() => Value::U64(5),
        "b".to_string() => Value::U64(6),
        "c".to_string() => Value::U64(7)
    ])).unwrap();
    assert_eq!(v, v2);
}

#[test]
fn test_ser_enum_unit() {
    assert_eq!(
        json::to_string(&SerEnum::Unit::<u32, u32, u32>).unwrap(),
        "{\"Unit\":[]}"
    );

    assert_eq!(
        json::to_value(&SerEnum::Unit::<u32, u32, u32>),
        Value::Object(btreemap!(
            "Unit".to_string() => Value::Array(vec![]))
        )
    );
}

#[test]
fn test_ser_enum_seq() {
    let a = 1;
    let b = 2;
    let c = 3;
    //let d = 4;
    let mut e = 5;
    //let f = 6;

    assert_eq!(
        json::to_string(&SerEnum::Seq(
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
        json::to_value(&SerEnum::Seq(
            a,
            b,
            &c,
            //d,
            &mut e,
            //e,
        )),
        Value::Object(btreemap!(
            "Seq".to_string() => Value::Array(vec![
                Value::U64(1),
                Value::U64(2),
                Value::U64(3),
                //Value::U64(4),
                Value::U64(5),
                //Value::U64(6),
            ])
        ))
    );
}

#[test]
fn test_ser_enum_map() {
    let a = 1;
    let b = 2;
    let c = 3;
    //let d = 4;
    let mut e = 5;
    //let f = 6;

    assert_eq!(
        json::to_string(&SerEnum::Map {
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
        json::to_value(&SerEnum::Map {
            a: a,
            b: b,
            c: &c,
            //d: d,
            e: &mut e,
            //f: f,
        }),
        Value::Object(btreemap!(
            "Map".to_string() => Value::Object(btreemap![
                "a".to_string() => Value::U64(1),
                "b".to_string() => Value::U64(2),
                "c".to_string() => Value::U64(3),
                //"d".to_string() => Value::U64(4)
                "e".to_string() => Value::U64(5)
                //"f".to_string() => Value::U64(6)
            ])
        ))
    );
}

#[test]
fn test_de_enum_unit() {
    let v: DeEnum<_, _, _> = json::from_str("{\"Unit\":[]}").unwrap();
    assert_eq!(
        v,
        DeEnum::Unit::<u32, u32, u32>
    );

    let v: DeEnum<_, _, _> = json::from_value(Value::Object(btreemap!(
        "Unit".to_string() => Value::Array(vec![]))
    )).unwrap();
    assert_eq!(
        v,
        DeEnum::Unit::<u32, u32, u32>
    );
}

#[test]
fn test_de_enum_seq() {
    let a = 1;
    let b = 2;
    let c = 3;
    //let d = 4;
    let e = 5;
    //let f = 6;

    let v: DeEnum<_, _, _> = json::from_str("{\"Seq\":[1,2,3,5]}").unwrap();
    assert_eq!(
        v,
        DeEnum::Seq(
            a,
            b,
            c,
            //d,
            e,
            //f,
        )
    );

    let v: DeEnum<_, _, _> = json::from_value(Value::Object(btreemap!(
        "Seq".to_string() => Value::Array(vec![
            Value::U64(1),
            Value::U64(2),
            Value::U64(3),
            //Value::U64(4),
            Value::U64(5),
            //Value::U64(6),
        ])
    ))).unwrap();
    assert_eq!(
        v,
        DeEnum::Seq(
            a,
            b,
            c,
            //d,
            e,
            //e,
        )
    );
}

#[test]
fn test_de_enum_map() {
    let a = 1;
    let b = 2;
    let c = 3;
    //let d = 4;
    let e = 5;
    //let f = 6;

    let v: DeEnum<_, _, _> = json::from_str(
        "{\"Map\":{\"a\":1,\"b\":2,\"c\":3,\"e\":5}}"
    ).unwrap();
    assert_eq!(
        v,
        DeEnum::Map {
            a: a,
            b: b,
            c: c,
            //d: d,
            e: e,
            //f: f,
        }
    );

    let v: DeEnum<_, _, _> = json::from_value(Value::Object(btreemap!(
        "Map".to_string() => Value::Object(btreemap![
            "a".to_string() => Value::U64(1),
            "b".to_string() => Value::U64(2),
            "c".to_string() => Value::U64(3),
            //"d".to_string() => Value::U64(4)
            "e".to_string() => Value::U64(5)
            //"f".to_string() => Value::U64(6)
        ])
    ))).unwrap();

    assert_eq!(
        v,
        DeEnum::Map {
            a: a,
            b: b,
            c: c,
            //d: d,
            e: e,
            //f: f,
        }
    );
}
