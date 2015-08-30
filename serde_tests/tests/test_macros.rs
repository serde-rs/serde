use token::{Token, assert_tokens, assert_ser_tokens, assert_de_tokens};

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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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

#[derive(Serialize)]
enum Lifetimes<'a> {
    LifetimeSeq(&'a i32),
    NoLifetimeSeq(i32),
    LifetimeMap { a: &'a i32 },
    NoLifetimeMap { a: i32 },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct GenericStruct<T> {
    x: T,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct GenericNewtypeStruct<T>(T);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct GenericTupleStruct<T, U>(T, U);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum GenericEnum<T, U> {
    Unit,
    Newtype(T),
    Seq(T, U),
    Map { x: T, y: U },
}

#[test]
fn test_named_unit() {
    assert_tokens(
        &NamedUnit,
        vec![Token::UnitStruct("NamedUnit")]
    );
}

#[test]
fn test_ser_named_tuple() {
    let a = 5;
    let mut b = 6;
    let c = 7;
    assert_ser_tokens(
        &SerNamedTuple(&a, &mut b, c),
        &[
            Token::TupleStructStart("SerNamedTuple", Some(3)),
            Token::SeqSep,
            Token::I32(5),

            Token::SeqSep,
            Token::I32(6),

            Token::SeqSep,
            Token::I32(7),

            Token::SeqEnd,
        ],
    );
}

#[test]
fn test_de_named_tuple() {
    assert_de_tokens(
        &DeNamedTuple(5, 6, 7),
        vec![
            Token::TupleStructStart("DeNamedTuple", Some(3)),
            Token::SeqSep,
            Token::I32(5),

            Token::SeqSep,
            Token::I32(6),

            Token::SeqSep,
            Token::I32(7),

            Token::SeqEnd,
        ]
    );
}

#[test]
fn test_ser_named_map() {
    let a = 5;
    let mut b = 6;
    let c = 7;

    assert_ser_tokens(
        &SerNamedMap {
            a: &a,
            b: &mut b,
            c: c,
        },
        &[
            Token::StructStart("SerNamedMap", Some(3)),

            Token::MapSep,
            Token::Str("a"),
            Token::I32(5),

            Token::MapSep,
            Token::Str("b"),
            Token::I32(6),

            Token::MapSep,
            Token::Str("c"),
            Token::I32(7),

            Token::MapEnd,
        ]
    );
}

#[test]
fn test_de_named_map() {
    assert_de_tokens(
        &DeNamedMap {
            a: 5,
            b: 6,
            c: 7,
        },
        vec![
            Token::StructStart("DeNamedMap", Some(3)),

            Token::MapSep,
            Token::Str("a"),
            Token::I32(5),

            Token::MapSep,
            Token::Str("b"),
            Token::I32(6),

            Token::MapSep,
            Token::Str("c"),
            Token::I32(7),

            Token::MapEnd,
        ]
    );
}

#[test]
fn test_ser_enum_unit() {
    assert_ser_tokens(
        &SerEnum::Unit::<u32, u32, u32>,
        &[
            Token::EnumUnit("SerEnum", "Unit"),
        ]
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

    assert_ser_tokens(
        &SerEnum::Seq(
            a,
            b,
            &c,
            //d,
            &mut e,
            //f,
        ),
        &[
            Token::EnumSeqStart("SerEnum", "Seq", Some(4)),

            Token::SeqSep,
            Token::I8(1),

            Token::SeqSep,
            Token::I32(2),

            Token::SeqSep,
            Token::I32(3),

            Token::SeqSep,
            Token::I32(5),

            Token::SeqEnd,
        ],
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

    assert_ser_tokens(
        &SerEnum::Map {
            a: a,
            b: b,
            c: &c,
            //d: d,
            e: &mut e,
            //f: f,
        },
        &[
            Token::EnumMapStart("SerEnum", "Map", Some(4)),

            Token::MapSep,
            Token::Str("a"),
            Token::I8(1),

            Token::MapSep,
            Token::Str("b"),
            Token::I32(2),

            Token::MapSep,
            Token::Str("c"),
            Token::I32(3),

            Token::MapSep,
            Token::Str("e"),
            Token::I32(5),

            Token::MapEnd,
        ],
    );
}

#[test]
fn test_de_enum_unit() {
    assert_tokens(
        &DeEnum::Unit::<u32, u32, u32>,
        vec![
            Token::EnumUnit("DeEnum", "Unit"),
        ],
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

    assert_tokens(
        &DeEnum::Seq(
            a,
            b,
            c,
            //d,
            e,
            //f,
        ),
        vec![
            Token::EnumSeqStart("DeEnum", "Seq", Some(4)),

            Token::SeqSep,
            Token::I8(1),

            Token::SeqSep,
            Token::I32(2),

            Token::SeqSep,
            Token::I32(3),

            Token::SeqSep,
            Token::I32(5),

            Token::SeqEnd,
        ],
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

    assert_tokens(
        &DeEnum::Map {
            a: a,
            b: b,
            c: c,
            //d: d,
            e: e,
            //f: f,
        },
        vec![
            Token::EnumMapStart("DeEnum", "Map", Some(4)),

            Token::MapSep,
            Token::Str("a"),
            Token::I8(1),

            Token::MapSep,
            Token::Str("b"),
            Token::I32(2),

            Token::MapSep,
            Token::Str("c"),
            Token::I32(3),

            Token::MapSep,
            Token::Str("e"),
            Token::I32(5),

            Token::MapEnd,
        ],
    );
}

#[test]
fn test_lifetimes() {
    let value = 5;

    assert_ser_tokens(
        &Lifetimes::LifetimeSeq(&value),
        &[
            Token::EnumNewtype("Lifetimes", "LifetimeSeq"),
            Token::I32(5),
        ]
    );

    assert_ser_tokens(
        &Lifetimes::NoLifetimeSeq(5),
        &[
            Token::EnumNewtype("Lifetimes", "NoLifetimeSeq"),
            Token::I32(5),
        ]
    );

    assert_ser_tokens(
        &Lifetimes::LifetimeMap { a: &value },
        &[
            Token::EnumMapStart("Lifetimes", "LifetimeMap", Some(1)),

            Token::MapSep,
            Token::Str("a"),
            Token::I32(5),

            Token::MapEnd,
        ]
    );

    assert_ser_tokens(
        &Lifetimes::NoLifetimeMap { a: 5 },
        &[
            Token::EnumMapStart("Lifetimes", "NoLifetimeMap", Some(1)),

            Token::MapSep,
            Token::Str("a"),
            Token::I32(5),

            Token::MapEnd,
        ]
    );
}

#[test]
fn test_generic_struct() {
    assert_tokens(
        &GenericStruct { x: 5u32 },
        vec![
            Token::StructStart("GenericStruct", Some(1)),

            Token::MapSep,
            Token::Str("x"),
            Token::U32(5),

            Token::MapEnd,
        ]
    );
}

#[test]
fn test_generic_newtype_struct() {
    assert_tokens(
        &GenericNewtypeStruct(5u32),
        vec![
            Token::StructNewtype("GenericNewtypeStruct"),
            Token::U32(5),
        ]
    );
}

#[test]
fn test_generic_tuple_struct() {
    assert_tokens(
        &GenericTupleStruct(5u32, 6u32),
        vec![
            Token::TupleStructStart("GenericTupleStruct", Some(2)),

            Token::SeqSep,
            Token::U32(5),

            Token::SeqSep,
            Token::U32(6),

            Token::SeqEnd,
        ]
    );
}

#[test]
fn test_generic_enum_unit() {
    assert_tokens(
        &GenericEnum::Unit::<u32, u32>,
        vec![
            Token::EnumUnit("GenericEnum", "Unit"),
        ]
    );
}

#[test]
fn test_generic_enum_newtype() {
    assert_tokens(
        &GenericEnum::Newtype::<u32, u32>(5),
        vec![
            Token::EnumNewtype("GenericEnum", "Newtype"),
            Token::U32(5),
        ]
    );
}

#[test]
fn test_generic_enum_seq() {
    assert_tokens(
        &GenericEnum::Seq::<u32, u32>(5, 6),
        vec![
            Token::EnumSeqStart("GenericEnum", "Seq", Some(2)),

            Token::SeqSep,
            Token::U32(5),

            Token::SeqSep,
            Token::U32(6),

            Token::SeqEnd,
        ]
    );
}

#[test]
fn test_generic_enum_map() {
    assert_tokens(
        &GenericEnum::Map::<u32, u32> { x: 5, y: 6 },
        vec![
            Token::EnumMapStart("GenericEnum", "Map", Some(2)),

            Token::MapSep,
            Token::Str("x"),
            Token::U32(5),

            Token::MapSep,
            Token::Str("y"),
            Token::U32(6),

            Token::MapEnd,
        ]
    );
}
