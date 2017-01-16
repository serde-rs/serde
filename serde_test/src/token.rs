#[derive(Clone, PartialEq, Debug)]
pub enum Token<'a> {
    Bool(bool),
    Isize(isize),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    Usize(usize),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Char(char),
    Str(&'a str),
    String(String),
    Bytes(&'a [u8]),
    ByteBuf(Vec<u8>),

    Option(bool),

    Unit,
    UnitStruct(&'a str),

    StructNewType(&'a str),

    EnumStart(&'a str),
    EnumUnit(&'a str, &'a str),
    EnumNewType(&'a str, &'a str),

    SeqStart(Option<usize>),
    SeqArrayStart(usize),
    SeqSep,
    SeqEnd,

    TupleStart(usize),
    TupleSep,
    TupleEnd,

    TupleStructStart(&'a str, usize),
    TupleStructSep,
    TupleStructEnd,

    MapStart(Option<usize>),
    MapSep,
    MapEnd,

    StructStart(&'a str, usize),
    StructSep,
    StructEnd,

    EnumSeqStart(&'a str, &'a str, usize),
    EnumSeqSep,
    EnumSeqEnd,

    EnumMapStart(&'a str, &'a str, usize),
    EnumMapSep,
    EnumMapEnd,
}
