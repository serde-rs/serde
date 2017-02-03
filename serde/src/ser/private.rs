use core::fmt::{self, Display};

use ser::{self, Serialize, Serializer, SerializeMap, SerializeStruct};

/// Not public API.
pub fn serialize_tagged_newtype<S, T>(
    serializer: S,
    type_ident: &'static str,
    variant_ident: &'static str,
    tag: &'static str,
    variant_name: &'static str,
    value: T,
) -> Result<S::Ok, S::Error>
    where S: Serializer,
          T: Serialize
{
    value.serialize(TaggedSerializer {
        type_ident: type_ident,
        variant_ident: variant_ident,
        tag: tag,
        variant_name: variant_name,
        delegate: serializer,
    })
}

struct TaggedSerializer<S> {
    type_ident: &'static str,
    variant_ident: &'static str,
    tag: &'static str,
    variant_name: &'static str,
    delegate: S,
}

enum Unsupported {
    Boolean,
    Integer,
    Float,
    Char,
    String,
    ByteArray,
    Optional,
    Unit,
    UnitStruct,
    Sequence,
    Tuple,
    TupleStruct,
    Enum,
}

impl Display for Unsupported {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Unsupported::Boolean => formatter.write_str("a boolean"),
            Unsupported::Integer => formatter.write_str("an integer"),
            Unsupported::Float => formatter.write_str("a float"),
            Unsupported::Char => formatter.write_str("a char"),
            Unsupported::String => formatter.write_str("a string"),
            Unsupported::ByteArray => formatter.write_str("a byte array"),
            Unsupported::Optional => formatter.write_str("an optional"),
            Unsupported::Unit => formatter.write_str("unit"),
            Unsupported::UnitStruct => formatter.write_str("a unit struct"),
            Unsupported::Sequence => formatter.write_str("a sequence"),
            Unsupported::Tuple => formatter.write_str("a tuple"),
            Unsupported::TupleStruct => formatter.write_str("a tuple struct"),
            Unsupported::Enum => formatter.write_str("an enum"),
        }
    }
}

struct Error {
    type_ident: &'static str,
    variant_ident: &'static str,
    ty: Unsupported,
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter,
               "cannot serialize tagged newtype variant {}::{} containing {}",
               self.type_ident, self.variant_ident, self.ty)
    }
}

impl<S> TaggedSerializer<S>
    where S: Serializer
{
    fn bad_type(self, what: Unsupported) -> S::Error {
        ser::Error::custom(Error {
            type_ident: self.type_ident,
            variant_ident: self.variant_ident,
            ty: what,
        })
    }
}

impl<S> Serializer for TaggedSerializer<S>
    where S: Serializer
{
    type Ok = S::Ok;
    type Error = S::Error;

    type SerializeSeq = S::SerializeSeq;
    type SerializeTuple = S::SerializeTuple;
    type SerializeTupleStruct = S::SerializeTupleStruct;
    type SerializeTupleVariant = S::SerializeTupleVariant;
    type SerializeMap = S::SerializeMap;
    type SerializeStruct = S::SerializeStruct;
    type SerializeStructVariant = S::SerializeStructVariant;

    fn serialize_bool(self, _: bool) -> Result<Self::Ok, Self::Error> {
        Err(self.bad_type(Unsupported::Boolean))
    }

    fn serialize_i8(self, _: i8) -> Result<Self::Ok, Self::Error> {
        Err(self.bad_type(Unsupported::Integer))
    }

    fn serialize_i16(self, _: i16) -> Result<Self::Ok, Self::Error> {
        Err(self.bad_type(Unsupported::Integer))
    }

    fn serialize_i32(self, _: i32) -> Result<Self::Ok, Self::Error> {
        Err(self.bad_type(Unsupported::Integer))
    }

    fn serialize_i64(self, _: i64) -> Result<Self::Ok, Self::Error> {
        Err(self.bad_type(Unsupported::Integer))
    }

    fn serialize_u8(self, _: u8) -> Result<Self::Ok, Self::Error> {
        Err(self.bad_type(Unsupported::Integer))
    }

    fn serialize_u16(self, _: u16) -> Result<Self::Ok, Self::Error> {
        Err(self.bad_type(Unsupported::Integer))
    }

    fn serialize_u32(self, _: u32) -> Result<Self::Ok, Self::Error> {
        Err(self.bad_type(Unsupported::Integer))
    }

    fn serialize_u64(self, _: u64) -> Result<Self::Ok, Self::Error> {
        Err(self.bad_type(Unsupported::Integer))
    }

    fn serialize_f32(self, _: f32) -> Result<Self::Ok, Self::Error> {
        Err(self.bad_type(Unsupported::Float))
    }

    fn serialize_f64(self, _: f64) -> Result<Self::Ok, Self::Error> {
        Err(self.bad_type(Unsupported::Float))
    }

    fn serialize_char(self, _: char) -> Result<Self::Ok, Self::Error> {
        Err(self.bad_type(Unsupported::Char))
    }

    fn serialize_str(self, _: &str) -> Result<Self::Ok, Self::Error> {
        Err(self.bad_type(Unsupported::String))
    }

    fn serialize_bytes(self, _: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(self.bad_type(Unsupported::ByteArray))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(self.bad_type(Unsupported::Optional))
    }

    fn serialize_some<T: ?Sized>(self, _: &T) -> Result<Self::Ok, Self::Error>
        where T: Serialize
    {
        Err(self.bad_type(Unsupported::Optional))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(self.bad_type(Unsupported::Unit))
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(self.bad_type(Unsupported::UnitStruct))
    }

    fn serialize_unit_variant(self, _: &'static str, _: usize, _: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(self.bad_type(Unsupported::Enum))
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _: &'static str, value: &T) -> Result<Self::Ok, Self::Error>
        where T: Serialize
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(self, _: &'static str, _: usize, _: &'static str, _: &T) -> Result<Self::Ok, Self::Error>
        where T: Serialize
    {
        Err(self.bad_type(Unsupported::Enum))
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(self.bad_type(Unsupported::Sequence))
    }

    fn serialize_seq_fixed_size(self, _: usize) -> Result<Self::SerializeSeq, Self::Error> {
        Err(self.bad_type(Unsupported::Sequence))
    }

    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(self.bad_type(Unsupported::Tuple))
    }

    fn serialize_tuple_struct(self, _: &'static str, _: usize) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(self.bad_type(Unsupported::TupleStruct))
    }

    fn serialize_tuple_variant(self, _: &'static str, _: usize, _: &'static str, _: usize) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(self.bad_type(Unsupported::Enum))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let mut map = try!(self.delegate.serialize_map(len.map(|len| len + 1)));
        try!(map.serialize_entry(self.tag, self.variant_name));
        Ok(map)
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct, Self::Error> {
        let mut state = try!(self.delegate.serialize_struct(name, len + 1));
        try!(state.serialize_field(self.tag, self.variant_name));
        Ok(state)
    }

    fn serialize_struct_variant(self, _: &'static str, _: usize, _: &'static str, _: usize) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(self.bad_type(Unsupported::Enum))
    }
}
