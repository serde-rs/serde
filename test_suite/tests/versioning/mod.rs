use serde::Deserialize;
use serde_test::{Token, assert_de_tokens_versions};

fn assert_de_tokens_ignore(ignorable_tokens: &[Token]) {
    #[derive(PartialEq, Debug, Deserialize)]
    struct IgnoreBase {
        a: i32,
    }

    // Embed the tokens to be ignored in the normal token
    // stream for an IgnoreBase type
    let concated_tokens: Vec<Token> = vec![
        Token::Map { len: Some(2) },
        Token::Str("a"),
        Token::I32(1),
        Token::Str("ignored"),
    ]
        .into_iter()
        .chain(ignorable_tokens.to_vec().into_iter())
        .chain(vec![Token::MapEnd].into_iter())
        .collect();

    let mut de = serde_test::Deserializer::new(&concated_tokens);
    let base = IgnoreBase::deserialize(&mut de).unwrap();
    assert_eq!(base, IgnoreBase { a: 1 });
}

macro_rules! declare_tests_versions {
    (
        $name:ident ($($version_ty:expr => $version_num:expr),*) { $($value:expr => $tokens:expr,)+ }
        $($tt:tt)*
    ) => {
        #[test]
        fn $name() {
            let version_map = vec![$(($version_ty.to_owned(), $version_num),)*]
                .into_iter().collect::<serde::de::VersionMap>();
            $(
                // Test ser/de roundtripping
                assert_de_tokens_versions(&$value, $tokens, Some(version_map.clone()));

                // Test that the tokens are ignorable
                assert_de_tokens_ignore($tokens);
            )+
        }

        declare_tests_versions! { $($tt)* }
    };
    () => { }
}

//////////////////////////////////////////////////////////////////////////

#[derive(Deserialize)]
#[serde(rename = "Struct")]
struct Structv1 {
    a: i32,
    b: i32,
    #[serde(skip_deserializing)]
    c: i32,
}

#[derive(PartialEq, Debug, Deserialize)]
#[serde(versions("Structv1"))]
struct Struct {
    d: i32,
    e: i32,
    f: i32,
}
impl From<Structv1> for Struct {
    fn from(v: Structv1) -> Self {
        Self {
            d: v.a,
            e: v.b,
            f: v.c,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename = "StructSkipAll")]
struct StructSkipAllv1 {
    #[serde(skip_deserializing)]
    a: i32,
}
#[derive(PartialEq, Debug, Deserialize)]
#[serde(versions("StructSkipAllv1"))]
struct StructSkipAll {
    b: i32,
}
impl From<StructSkipAllv1> for StructSkipAll {
    fn from(v: StructSkipAllv1) -> Self {
        Self {
            b: v.a
        }
    }
}

#[derive(Deserialize)]
#[serde(default, rename = "StructSkipDefault")]
struct StructSkipDefaultv1 {
    #[serde(skip_deserializing)]
    a: i32,

}
impl Default for StructSkipDefaultv1 {
    fn default() -> Self {
        Self { a:  16 }
    }
}
#[derive(PartialEq, Debug, Deserialize)]
#[serde(versions("StructSkipDefaultv1"))]
struct StructSkipDefault {
    b: i32
}
impl From<StructSkipDefaultv1> for StructSkipDefault {
    fn from(v: StructSkipDefaultv1) -> Self {
        Self {
            b: v.a
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename = "StructSkipAllDenyUnknown")]
struct StructSkipAllDenyUnknownv1{
    #[serde(skip_deserializing)]
    a: i32,
}
#[derive(PartialEq, Debug, Deserialize)]
#[serde(versions("StructSkipAllDenyUnknownv1"))]
struct StructSkipAllDenyUnknown {
    b: i32
}
impl From<StructSkipAllDenyUnknownv1> for StructSkipAllDenyUnknown {
    fn from(v: StructSkipAllDenyUnknownv1) -> Self {
        Self {
            b: v.a
        }
    }
}

#[derive(Deserialize)]
#[serde(default, rename = "StructDefault")]
struct StructDefaultv1<T> {
    a: i32,
    b: T,

}
impl Default for StructDefaultv1<String> {
    fn default() -> Self {
        Self {
            a: 100,
            b: "default".to_string(),
        }
    }
}

#[derive(PartialEq, Debug, Deserialize)]
#[serde(versions(version(default, type = "StructDefaultv1<T>")))]
struct StructDefault<T> {
    c: i32,
    d: T
}
impl<T> From<StructDefaultv1<T>> for StructDefault<T> {
    fn from(v: StructDefaultv1<T>) -> Self {
        Self {
            c: v.a,
            d: v.b,
        }
    }
}

#[derive(PartialEq, Debug, Deserialize)]
struct StructInStruct {
    a: Struct,
}

#[derive(Deserialize)]
#[serde(rename(deserialize = "Struct2"))]
struct Struct2v1 {
    a: u8,
}

#[derive(Deserialize)]
#[serde(rename(deserialize = "Struct2"))]
struct Struct2v2 {
    b: u8,
}

#[derive(PartialEq, Debug, Deserialize)]
#[serde(versions("Struct2v1", "Struct2v2"))]
struct Struct2 {
    c: u8
}
impl From<Struct2v1> for Struct2 {
    fn from(v: Struct2v1) -> Self {
        Self {
            c: v.a
        }
    }
}
impl From<Struct2v2> for Struct2 {
    fn from(v: Struct2v2) -> Self {
        Self {
            c: v.b
        }
    }
}

//////////////////////////////////////////////////////////////////////////

declare_tests_versions! {
    test_versioned_struct ("Struct" => 1) {
        Struct { d: 1, e: 2, f: 0 } => &[
            Token::Map { len: Some(3) },
                Token::Str("a"),
                Token::I32(1),

                Token::Str("b"),
                Token::I32(2),
            Token::MapEnd,
        ],
        Struct { d: 1, e: 2, f: 0 } => &[
            Token::Map { len: Some(3) },
                Token::U32(0),
                Token::I32(1),

                Token::U32(1),
                Token::I32(2),
            Token::MapEnd,
        ],
        Struct { d: 1, e: 2, f: 0 } => &[
            Token::Struct { name: "Struct", len: 2 },
                Token::Str("a"),
                Token::I32(1),

                Token::Str("b"),
                Token::I32(2),
            Token::StructEnd,
        ],
        Struct { d: 1, e: 2, f: 0 } => &[
            Token::Seq { len: Some(3) },
                Token::I32(1),
                Token::I32(2),
            Token::SeqEnd,
        ],
    }
    test_versioned_struct_with_skip ("Struct" => 1) {
        Struct { d: 1, e: 2, f: 0 } => &[
            Token::Map { len: Some(3) },
                Token::Str("a"),
                Token::I32(1),

                Token::Str("b"),
                Token::I32(2),

                Token::Str("c"),
                Token::I32(3),

                Token::Str("d"),
                Token::I32(4),
            Token::MapEnd,
        ],
        Struct { d: 1, e: 2, f: 0 } => &[
            Token::Struct { name: "Struct", len: 2 },
                Token::Str("a"),
                Token::I32(1),

                Token::Str("b"),
                Token::I32(2),

                Token::Str("c"),
                Token::I32(3),

                Token::Str("d"),
                Token::I32(4),
            Token::StructEnd,
        ],
    }
    test_versioned_struct_skip_all ("StructSkipAll" => 1) {
        StructSkipAll { b: 0 } => &[
            Token::Struct { name: "StructSkipAll", len: 0 },
            Token::StructEnd,
        ],
        StructSkipAll { b: 0 } => &[
            Token::Struct { name: "StructSkipAll", len: 0 },
                Token::Str("a"),
                Token::I32(1),

                Token::Str("b"),
                Token::I32(2),
            Token::StructEnd,
        ],
    }
    test_versioned_struct_skip_default ("StructSkipDefault" => 1) {
        StructSkipDefault { b: 16 } => &[
            Token::Struct { name: "StructSkipDefault", len: 0 },
            Token::StructEnd,
        ],
    }
    test_versioned_struct_skip_all_deny_unknown ("StructSkipAllDenyUnknown" => 1) {
        StructSkipAllDenyUnknown { b: 0 } => &[
            Token::Struct { name: "StructSkipAllDenyUnknown", len: 0 },
            Token::StructEnd,
        ],
    }
    test_versioned_struct_default ("StructDefault" => 1) {
        StructDefault { c: 50, d: "overwritten".to_string() } => &[
            Token::Struct { name: "StructDefault", len: 2 },
                Token::Str("a"),
                Token::I32(50),

                Token::Str("b"),
                Token::String("overwritten"),
            Token::StructEnd,
        ],
        StructDefault { c: 100, d: "default".to_string() } => &[
            Token::Struct { name: "StructDefault",  len: 2 },
            Token::StructEnd,
        ],
    }
    test_versioned_struct_in_map ("Struct" => 1) {
        StructInStruct { a: Struct { d: 1, e: 2, f: 0 } } => &[
            Token::Map { len: Some(3) },
            Token::Str("a"),
                Token::Map { len: Some(3) },
                    Token::Str("a"),
                    Token::I32(1),

                    Token::Str("b"),
                    Token::I32(2),
                Token::MapEnd,
            Token::MapEnd,
        ],
        StructInStruct { a: Struct { d: 1, e: 2, f: 0 } } => &[
            Token::Map { len: Some(3) },
                Token::U32(0),
                Token::Map { len: Some(3) },
                    Token::U32(0),
                    Token::I32(1),

                    Token::U32(1),
                    Token::I32(2),
                Token::MapEnd,
            Token::MapEnd,
        ],
        StructInStruct { a: Struct { d: 1, e: 2, f: 0 } } => &[
            Token::Struct { name: "StructInStruct", len: 2 },
                Token::Str("a"),
                Token::Struct { name: "Struct", len: 2 },
                    Token::Str("a"),
                    Token::I32(1),

                    Token::Str("b"),
                    Token::I32(2),
                Token::StructEnd,
            Token::StructEnd,
        ],
        StructInStruct { a: Struct { d: 1, e: 2, f: 0 } } => &[
            Token::Seq { len: Some(1) },
                Token::Seq { len: Some(3) },
                    Token::I32(1),
                    Token::I32(2),
                Token::SeqEnd,
            Token::SeqEnd,
        ],
    }
    test_versioned_struct2_v1 ("Struct2" => 1) {
        Struct2 { c: 1 } => &[
            Token::Map { len: Some(1) },
                Token::Str("a"),
                Token::I32(1),
            Token::MapEnd,
        ],
        Struct2 { c: 1 } => &[
            Token::Map { len: Some(1) },
                Token::U32(0),
                Token::I32(1),
            Token::MapEnd,
        ],
        Struct2 { c: 1 } => &[
            Token::Struct { name: "Struct2", len: 2 },
                Token::Str("a"),
                Token::I32(1),
            Token::StructEnd,
        ],
        Struct2 { c: 1 } => &[
            Token::Seq { len: Some(2) },
                Token::I32(1),
            Token::SeqEnd,
        ],
    }
    test_versioned_struct2_v2 ("Struct2" => 2) {
        Struct2 { c: 1 } => &[
            Token::Map { len: Some(1) },
                Token::Str("b"),
                Token::I32(1),
            Token::MapEnd,
        ],
        Struct2 { c: 1 } => &[
            Token::Map { len: Some(1) },
                Token::U32(0),
                Token::I32(1),
            Token::MapEnd,
        ],
        Struct2 { c: 1 } => &[
            Token::Struct { name: "Struct2", len: 2 },
                Token::Str("b"),
                Token::I32(1),
            Token::StructEnd,
        ],
        Struct2 { c: 1 } => &[
            Token::Seq { len: Some(2) },
                Token::I32(1),
            Token::SeqEnd,
        ],
    }
}