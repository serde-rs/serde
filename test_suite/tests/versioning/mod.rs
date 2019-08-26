#![allow(clippy::decimal_literal_representation, clippy::unreadable_literal)]
#![cfg_attr(feature = "unstable", feature(never_type))]

use serde::Deserialize;

macro_rules! declare_tests_versions {
    (
        $name:ident ($($version_ty:ty: $version_num:expr),*) { $($value:expr => $tokens:expr,)+ }
        $($tt:tt)*
    ) => {
        #[test]
        fn $name() {
            let version_map = vec![$(("versioning::$version_ty", $version_num)*,)]
                .into_iter().collect::<VersionMap>();
            $(
                // Test ser/de roundtripping
                assert_de_tokens_versions(&$value, $tokens, Some(version_map));

                // Test that the tokens are ignorable
                assert_de_tokens_ignore($tokens);
            )+
        }

        declare_tests_versions! { $($tt)* }
    };
    ($($tt:tt)*) => { }
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
#[serde(versions(Structv1))]
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
#[serde(versions(StructSkipAllv1))]
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
#[derive(PartialEq, Debug, Deserialize)]
#[serde(versions(StructSkipDefaultv1))]
struct StructSkipDefault {
    b: i32
}
impl From<StructSkipDefaultv1> for StructSkipDefault {
    fn from(v: StructSkipDefaulv1) -> Self {
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
#[serde(versions(StructSkipAllDenyUnknownv1))]
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
#[serde(versions(StructDefaultv1))]
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

//////////////////////////////////////////////////////////////////////////

declare_tests_versions! {
    test_struct (Struct: 1) {
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
    test_struct_with_skip (Struct: 1) {
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
    test_struct_skip_all (StructSkipAll:) {
        StructSkipAll { a: 0 } => &[
            Token::Struct { name: "StructSkipAll", len: 0 },
            Token::StructEnd,
        ],
        StructSkipAll { a: 0 } => &[
            Token::Struct { name: "StructSkipAll", len: 0 },
                Token::Str("a"),
                Token::I32(1),

                Token::Str("b"),
                Token::I32(2),
            Token::StructEnd,
        ],
    }
    test_struct_skip_default (StructSkipDefault: 1) {
        StructSkipDefault { a: 16 } => &[
            Token::Struct { name: "StructSkipDefault", len: 0 },
            Token::StructEnd,
        ],
    }
    test_struct_skip_all_deny_unknown (StructSkipAllDenyUnknown: 1) {
        StructSkipAllDenyUnknown { a: 0 } => &[
            Token::Struct { name: "StructSkipAllDenyUnknown", len: 0 },
            Token::StructEnd,
        ],
    }
    test_struct_default (StructDefault: 1) {
        StructDefault { a: 50, b: "overwritten".to_string() } => &[
            Token::Struct { name: "StructDefault", len: 2 },
                Token::Str("a"),
                Token::I32(50),

                Token::Str("b"),
                Token::String("overwritten"),
            Token::StructEnd,
        ],
        StructDefault { a: 100, b: "default".to_string() } => &[
            Token::Struct { name: "StructDefault",  len: 2 },
            Token::StructEnd,
        ],
    }
}
