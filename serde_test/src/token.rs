use std::fmt::{self, Debug, Display};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Token {
    /// A serialized `bool`.
    ///
    /// ```edition2021
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// assert_tokens(&true, &[Token::Bool(true)]);
    /// ```
    Bool(bool),

    /// A serialized `i8`.
    ///
    /// ```edition2021
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// assert_tokens(&0i8, &[Token::I8(0)]);
    /// ```
    I8(i8),

    /// A serialized `i16`.
    ///
    /// ```edition2021
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// assert_tokens(&0i16, &[Token::I16(0)]);
    /// ```
    I16(i16),

    /// A serialized `i32`.
    ///
    /// ```edition2021
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// assert_tokens(&0i32, &[Token::I32(0)]);
    /// ```
    I32(i32),

    /// A serialized `i64`.
    ///
    /// ```edition2021
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// assert_tokens(&0i64, &[Token::I64(0)]);
    /// ```
    I64(i64),

    /// A serialized `u8`.
    ///
    /// ```edition2021
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// assert_tokens(&0u8, &[Token::U8(0)]);
    /// ```
    U8(u8),

    /// A serialized `u16`.
    ///
    /// ```edition2021
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// assert_tokens(&0u16, &[Token::U16(0)]);
    /// ```
    U16(u16),

    /// A serialized `u32`.
    ///
    /// ```edition2021
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// assert_tokens(&0u32, &[Token::U32(0)]);
    /// ```
    U32(u32),

    /// A serialized `u64`.
    ///
    /// ```edition2021
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// assert_tokens(&0u64, &[Token::U64(0)]);
    /// ```
    U64(u64),

    /// A serialized `f32`.
    ///
    /// ```edition2021
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// assert_tokens(&0f32, &[Token::F32(0.0)]);
    /// ```
    F32(f32),

    /// A serialized `f64`.
    ///
    /// ```edition2021
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// assert_tokens(&0f64, &[Token::F64(0.0)]);
    /// ```
    F64(f64),

    /// A serialized `char`.
    ///
    /// ```edition2021
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// assert_tokens(&'\n', &[Token::Char('\n')]);
    /// ```
    Char(char),

    /// A serialized `str`.
    ///
    /// ```edition2021
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// let s = String::from("transient");
    /// assert_tokens(&s, &[Token::Str("transient")]);
    /// ```
    Str(&'static str),

    /// A borrowed `str`.
    ///
    /// ```edition2021
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// let s: &str = "borrowed";
    /// assert_tokens(&s, &[Token::BorrowedStr("borrowed")]);
    /// ```
    BorrowedStr(&'static str),

    /// A serialized `String`.
    ///
    /// ```edition2021
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// let s = String::from("owned");
    /// assert_tokens(&s, &[Token::String("owned")]);
    /// ```
    String(&'static str),

    /// A serialized `[u8]`
    Bytes(&'static [u8]),

    /// A borrowed `[u8]`.
    BorrowedBytes(&'static [u8]),

    /// A serialized `ByteBuf`
    ByteBuf(&'static [u8]),

    /// A serialized `Option<T>` containing none.
    ///
    /// ```edition2021
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// let opt = None::<char>;
    /// assert_tokens(&opt, &[Token::None]);
    /// ```
    None,

    /// The header to a serialized `Option<T>` containing some value.
    ///
    /// The tokens of the value follow after this header.
    ///
    /// ```edition2021
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// let opt = Some('c');
    /// assert_tokens(&opt, &[Token::Some, Token::Char('c')]);
    /// ```
    Some,

    /// A serialized `()`.
    ///
    /// ```edition2021
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// assert_tokens(&(), &[Token::Unit]);
    /// ```
    Unit,

    /// A serialized unit struct of the given name.
    ///
    /// ```edition2021
    /// # use serde_derive::{Deserialize, Serialize};
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// # fn main() {
    /// #[derive(Serialize, Deserialize, PartialEq, Debug)]
    /// struct X;
    ///
    /// assert_tokens(&X, &[Token::UnitStruct { name: "X" }]);
    /// # }
    /// ```
    UnitStruct { name: &'static str },

    /// A unit variant of an enum.
    ///
    /// ```edition2021
    /// # use serde_derive::{Deserialize, Serialize};
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// # fn main() {
    /// #[derive(Serialize, Deserialize, PartialEq, Debug)]
    /// enum E {
    ///     A,
    /// }
    ///
    /// let a = E::A;
    /// assert_tokens(
    ///     &a,
    ///     &[Token::UnitVariant {
    ///         name: "E",
    ///         variant: "A",
    ///     }],
    /// );
    /// # }
    /// ```
    UnitVariant {
        name: &'static str,
        variant: &'static str,
    },

    /// The header to a serialized newtype struct of the given name.
    ///
    /// After this header is the value contained in the newtype struct.
    ///
    /// ```edition2021
    /// # use serde_derive::{Deserialize, Serialize};
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// # fn main() {
    /// #[derive(Serialize, Deserialize, PartialEq, Debug)]
    /// struct N(String);
    ///
    /// let n = N("newtype".to_owned());
    /// assert_tokens(
    ///     &n,
    ///     &[Token::NewtypeStruct { name: "N" }, Token::String("newtype")],
    /// );
    /// # }
    /// ```
    NewtypeStruct { name: &'static str },

    /// The header to a newtype variant of an enum.
    ///
    /// After this header is the value contained in the newtype variant.
    ///
    /// ```edition2021
    /// # use serde_derive::{Deserialize, Serialize};
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// # fn main() {
    /// #[derive(Serialize, Deserialize, PartialEq, Debug)]
    /// enum E {
    ///     B(u8),
    /// }
    ///
    /// let b = E::B(0);
    /// assert_tokens(
    ///     &b,
    ///     &[
    ///         Token::NewtypeVariant {
    ///             name: "E",
    ///             variant: "B",
    ///         },
    ///         Token::U8(0),
    ///     ],
    /// );
    /// # }
    /// ```
    NewtypeVariant {
        name: &'static str,
        variant: &'static str,
    },

    /// The header to a sequence.
    ///
    /// After this header are the elements of the sequence, followed by
    /// `SeqEnd`.
    ///
    /// ```edition2021
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// let vec = vec!['a', 'b', 'c'];
    /// assert_tokens(
    ///     &vec,
    ///     &[
    ///         Token::Seq { len: Some(3) },
    ///         Token::Char('a'),
    ///         Token::Char('b'),
    ///         Token::Char('c'),
    ///         Token::SeqEnd,
    ///     ],
    /// );
    /// ```
    Seq { len: Option<usize> },

    /// An indicator of the end of a sequence.
    SeqEnd,

    /// The header to a tuple.
    ///
    /// After this header are the elements of the tuple, followed by `TupleEnd`.
    ///
    /// ```edition2021
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// let tuple = ('a', 100);
    /// assert_tokens(
    ///     &tuple,
    ///     &[
    ///         Token::Tuple { len: 2 },
    ///         Token::Char('a'),
    ///         Token::I32(100),
    ///         Token::TupleEnd,
    ///     ],
    /// );
    /// ```
    Tuple { len: usize },

    /// An indicator of the end of a tuple.
    TupleEnd,

    /// The header to a tuple struct.
    ///
    /// After this header are the fields of the tuple struct, followed by
    /// `TupleStructEnd`.
    ///
    /// ```edition2021
    /// # use serde_derive::{Deserialize, Serialize};
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// # fn main() {
    /// #[derive(Serialize, Deserialize, PartialEq, Debug)]
    /// struct T(u8, u8);
    ///
    /// let t = T(0, 0);
    /// assert_tokens(
    ///     &t,
    ///     &[
    ///         Token::TupleStruct { name: "T", len: 2 },
    ///         Token::U8(0),
    ///         Token::U8(0),
    ///         Token::TupleStructEnd,
    ///     ],
    /// );
    /// # }
    /// ```
    TupleStruct { name: &'static str, len: usize },

    /// An indicator of the end of a tuple struct.
    TupleStructEnd,

    /// The header to a tuple variant of an enum.
    ///
    /// After this header are the fields of the tuple variant, followed by
    /// `TupleVariantEnd`.
    ///
    /// ```edition2021
    /// # use serde_derive::{Deserialize, Serialize};
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// # fn main() {
    /// #[derive(Serialize, Deserialize, PartialEq, Debug)]
    /// enum E {
    ///     C(u8, u8),
    /// }
    ///
    /// let c = E::C(0, 0);
    /// assert_tokens(
    ///     &c,
    ///     &[
    ///         Token::TupleVariant {
    ///             name: "E",
    ///             variant: "C",
    ///             len: 2,
    ///         },
    ///         Token::U8(0),
    ///         Token::U8(0),
    ///         Token::TupleVariantEnd,
    ///     ],
    /// );
    /// # }
    /// ```
    TupleVariant {
        name: &'static str,
        variant: &'static str,
        len: usize,
    },

    /// An indicator of the end of a tuple variant.
    TupleVariantEnd,

    /// The header to a map.
    ///
    /// After this header are the entries of the map, followed by `MapEnd`.
    ///
    /// ```edition2021
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// use std::collections::BTreeMap;
    ///
    /// let mut map = BTreeMap::new();
    /// map.insert('A', 65);
    /// map.insert('Z', 90);
    ///
    /// assert_tokens(
    ///     &map,
    ///     &[
    ///         Token::Map { len: Some(2) },
    ///         Token::Char('A'),
    ///         Token::I32(65),
    ///         Token::Char('Z'),
    ///         Token::I32(90),
    ///         Token::MapEnd,
    ///     ],
    /// );
    /// ```
    Map { len: Option<usize> },

    /// An indicator of the end of a map.
    MapEnd,

    /// The header of a struct.
    ///
    /// After this header are the fields of the struct, followed by `StructEnd`.
    ///
    /// ```edition2021
    /// # use serde_derive::{Deserialize, Serialize};
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// # fn main() {
    /// #[derive(Serialize, Deserialize, PartialEq, Debug)]
    /// struct S {
    ///     a: u8,
    ///     b: u8,
    /// }
    ///
    /// let s = S { a: 0, b: 0 };
    /// assert_tokens(
    ///     &s,
    ///     &[
    ///         Token::Struct { name: "S", len: 2 },
    ///         Token::Str("a"),
    ///         Token::U8(0),
    ///         Token::Str("b"),
    ///         Token::U8(0),
    ///         Token::StructEnd,
    ///     ],
    /// );
    /// # }
    /// ```
    Struct { name: &'static str, len: usize },

    /// An indicator of the end of a struct.
    StructEnd,

    /// The header of a struct variant of an enum.
    ///
    /// After this header are the fields of the struct variant, followed by
    /// `StructVariantEnd`.
    ///
    /// ```edition2021
    /// # use serde_derive::{Deserialize, Serialize};
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// # fn main() {
    /// #[derive(Serialize, Deserialize, PartialEq, Debug)]
    /// enum E {
    ///     D { d: u8 },
    /// }
    ///
    /// let d = E::D { d: 0 };
    /// assert_tokens(
    ///     &d,
    ///     &[
    ///         Token::StructVariant {
    ///             name: "E",
    ///             variant: "D",
    ///             len: 1,
    ///         },
    ///         Token::Str("d"),
    ///         Token::U8(0),
    ///         Token::StructVariantEnd,
    ///     ],
    /// );
    /// # }
    /// ```
    StructVariant {
        name: &'static str,
        variant: &'static str,
        len: usize,
    },

    /// An indicator of the end of a struct variant.
    StructVariantEnd,

    /// The header to an enum of the given name.
    ///
    /// ```edition2021
    /// # use serde_derive::{Deserialize, Serialize};
    /// # use serde_test::{assert_tokens, Token};
    /// #
    /// # fn main() {
    /// #[derive(Serialize, Deserialize, PartialEq, Debug)]
    /// enum E {
    ///     A,
    ///     B(u8),
    ///     C(u8, u8),
    ///     D { d: u8 },
    /// }
    ///
    /// let a = E::A;
    /// assert_tokens(
    ///     &a,
    ///     &[Token::Enum { name: "E" }, Token::Str("A"), Token::Unit],
    /// );
    ///
    /// let b = E::B(0);
    /// assert_tokens(
    ///     &b,
    ///     &[Token::Enum { name: "E" }, Token::Str("B"), Token::U8(0)],
    /// );
    ///
    /// let c = E::C(0, 0);
    /// assert_tokens(
    ///     &c,
    ///     &[
    ///         Token::Enum { name: "E" },
    ///         Token::Str("C"),
    ///         Token::Seq { len: Some(2) },
    ///         Token::U8(0),
    ///         Token::U8(0),
    ///         Token::SeqEnd,
    ///     ],
    /// );
    ///
    /// let d = E::D { d: 0 };
    /// assert_tokens(
    ///     &d,
    ///     &[
    ///         Token::Enum { name: "E" },
    ///         Token::Str("D"),
    ///         Token::Map { len: Some(1) },
    ///         Token::Str("d"),
    ///         Token::U8(0),
    ///         Token::MapEnd,
    ///     ],
    /// );
    /// # }
    /// ```
    Enum { name: &'static str },
}

impl Display for Token {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, formatter)
    }
}
