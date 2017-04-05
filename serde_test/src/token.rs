#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Token {
    /// A serialized `bool`.
    Bool(bool),

    /// A serialized `i8`.
    I8(i8),

    /// A serialized `i16`.
    I16(i16),

    /// A serialized `i32`.
    I32(i32),

    /// A serialized `i64`.
    I64(i64),

    /// A serialized `u8`.
    U8(u8),

    /// A serialized `u16`.
    U16(u16),

    /// A serialized `u32`.
    U32(u32),

    /// A serialized `u64`.
    U64(u64),

    /// A serialized `f32`.
    F32(f32),

    /// A serialized `f64`.
    F64(f64),

    /// A serialized `char`.
    Char(char),

    /// A serialized `str`.
    Str(&'static str),

    /// A borrowed `str`.
    BorrowedStr(&'static str),

    /// A serialized `String`.
    String(&'static str),

    /// A serialized `[u8]`
    Bytes(&'static [u8]),

    /// A borrowed `[u8]`.
    BorrowedBytes(&'static [u8]),

    /// A serialized `ByteBuf`
    ByteBuf(&'static [u8]),

    /// The header to a serialized `Option<T>` containing some value.
    ///
    /// The tokens of the value follow after this header.
    Some,

    /// A serialized `Option<T>` containing none.
    None,

    /// A serialized `()`.
    Unit,

    /// A serialized unit struct of the given name.
    UnitStruct(&'static str),

    /// The header to a serialized newtype struct of the given name.
    ///
    /// Newtype structs are serialized with this header, followed by the value contained in the
    /// newtype struct.
    NewtypeStruct(&'static str),

    /// The header to an enum of the given name.
    Enum(&'static str),

    /// A unit variant of an enum of the given name, of the given name.
    ///
    /// The first string represents the name of the enum, and the second represents the name of the
    /// variant.
    UnitVariant(&'static str, &'static str),

    /// The header to a newtype variant of an enum of the given name, of the given name.
    ///
    /// The first string represents the name of the enum, and the second represents the name of the
    /// variant. The value contained within this enum works the same as `StructNewType`.
    NewtypeVariant(&'static str, &'static str),

    /// The header to a sequence of the given length.
    ///
    /// These are serialized via `serialize_seq`, which takes an optional length. After this
    /// header is a list of elements, followed by `SeqEnd`.
    Seq(Option<usize>),

    /// The header to an array of the given length.
    ///
    /// These are serialized via `serialize_seq_fized_size`, which requires a length. After this
    /// header is a list of elements, followed by `SeqEnd`.
    SeqFixedSize(usize),

    /// An indicator of the end of a sequence.
    SeqEnd,

    /// The header to a tuple of the given length, similar to `SeqFixedSize`.
    Tuple(usize),

    /// An indicator of the end of a tuple, similar to `SeqEnd`.
    TupleEnd,

    /// The header to a tuple struct of the given name and length.
    TupleStruct(&'static str, usize),

    /// An indicator of the end of a tuple struct, similar to `TupleEnd`.
    TupleStructEnd,

    /// The header to a map of the given length.
    ///
    /// These are serialized via `serialize_map`, which takes an optional length. After this header
    /// is a list of key-value pairs, followed by `MapEnd`.
    Map(Option<usize>),

    /// An indicator of the end of a map.
    MapEnd,

    /// The header of a struct of the given name and length, similar to `Map`.
    Struct(&'static str, usize),

    /// An indicator of the end of a struct, similar to `MapEnd`.
    StructEnd,

    /// The header to a tuple variant of an enum of the given name, of the given name and length.
    TupleVariant(&'static str, &'static str, usize),

    /// An indicator of the end of a tuple variant, similar to `TupleEnd`.
    TupleVariantEnd,

    /// The header of a struct variant of an enum of the given name, of the given name and length,
    /// similar to `Struct`.
    StructVariant(&'static str, &'static str, usize),

    /// An indicator of the end of a struct, similar to `StructEnd`.
    StructVariantEnd,
}
