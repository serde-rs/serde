use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::ser::{SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
                 SerializeTuple, SerializeTupleStruct, SerializeTupleVariant};

pub struct Readable<T: ?Sized>(T);
pub struct Compact<T: ?Sized>(T);

pub trait Configure {
    fn readable(&self) -> Readable<&Self> {
        Readable(self)
    }
    fn compact(&self) -> Compact<&Self> {
        Compact(self)
    }
}

impl<T: ?Sized> Configure for T {}
pub trait Configuration {
    fn is_human_readable(&self) -> bool {
        true
    }
}
impl<T: ?Sized> Configuration for Readable<T> {}

fn assert<T>(_: T)
where
    T: Configure,
{

}

impl<T: ?Sized> Serialize for Readable<T>
where
    T: Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(Readable(serializer))
    }
}
impl<T: ?Sized> Serialize for Compact<T>
where
    T: Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(Compact(serializer))
    }
}
impl<'de, T> Deserialize<'de> for Readable<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(Readable(deserializer)).map(Readable)
    }
}
impl<'de, T> Deserialize<'de> for Compact<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(Compact(deserializer)).map(Compact)
    }
}


macro_rules! forward_method {
    ($name: ident (self $(, $arg: ident : $arg_type: ty)* ) -> $return_type: ty) => {
        fn $name (self $(, $arg : $arg_type)* ) -> $return_type {
            (self.0).$name( $($arg),* )
        }
    };
}

macro_rules! forward_serialize_methods {
    ( $( $name: ident $arg_type: ty ),* ) => {
        $(
            forward_method!($name(self, v : $arg_type) -> Result<Self::Ok, Self::Error>);
        )*
    };
}

macro_rules! impl_serializer {
    ($wrapper: ident, $is_human_readable : expr) => {
    impl<S> Serializer for $wrapper<S>
    where
        S: Serializer,
    {
        type Ok = S::Ok;
        type Error = S::Error;

        type SerializeSeq = $wrapper<S::SerializeSeq>;
        type SerializeTuple = $wrapper<S::SerializeTuple>;
        type SerializeTupleStruct = $wrapper<S::SerializeTupleStruct>;
        type SerializeTupleVariant = $wrapper<S::SerializeTupleVariant>;
        type SerializeMap = $wrapper<S::SerializeMap>;
        type SerializeStruct = $wrapper<S::SerializeStruct>;
        type SerializeStructVariant = $wrapper<S::SerializeStructVariant>;

        fn is_human_readable(&self) -> bool {
            $is_human_readable
        }


        forward_serialize_methods!{
            serialize_bool bool,
            serialize_i8 i8,
            serialize_i16 i16,
            serialize_i32 i32,
            serialize_i64 i64,
            serialize_u8 u8,
            serialize_u16 u16,
            serialize_u32 u32,
            serialize_u64 u64,
            serialize_f32 f32,
            serialize_f64 f64,
            serialize_char char,
            serialize_str &str,
            serialize_bytes &[u8],
            serialize_unit_struct &'static str

        }

        fn serialize_unit(self) -> Result<S::Ok, S::Error> {
            self.0.serialize_unit()
        }

        fn serialize_unit_variant(
            self,
            name: &'static str,
            variant_index: u32,
            variant: &'static str,
        ) -> Result<S::Ok, S::Error> {
            self.0.serialize_unit_variant(name, variant_index, variant)
        }

        fn serialize_newtype_struct<T: ?Sized>(
            self,
            name: &'static str,
            value: &T,
        ) -> Result<S::Ok, S::Error>
        where
            T: Serialize,
        {
            self.0.serialize_newtype_struct(name, &$wrapper(value))
        }

        fn serialize_newtype_variant<T: ?Sized>(
            self,
            name: &'static str,
            variant_index: u32,
            variant: &'static str,
            value: &T,
        ) -> Result<S::Ok, S::Error>
        where
            T: Serialize,
        {
            self.0
                .serialize_newtype_variant(name, variant_index, variant, &$wrapper(value))
        }

        fn serialize_none(self) -> Result<S::Ok, Self::Error> {
            self.0.serialize_none()
        }

        fn serialize_some<T: ?Sized>(self, value: &T) -> Result<S::Ok, Self::Error>
        where
            T: Serialize,
        {
            self.0.serialize_some(&$wrapper(value))
        }

        fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
            self.0.serialize_seq(len).map($wrapper)
        }

        fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
            self.0.serialize_tuple(len).map($wrapper)
        }

        fn serialize_tuple_struct(
            self,
            name: &'static str,
            len: usize,
        ) -> Result<Self::SerializeTupleStruct, Self::Error> {
            self.0.serialize_tuple_struct(name, len).map($wrapper)
        }

        fn serialize_tuple_variant(
            self,
            name: &'static str,
            variant_index: u32,
            variant: &'static str,
            len: usize,
        ) -> Result<Self::SerializeTupleVariant, Self::Error> {
            self.0
                .serialize_tuple_variant(name, variant_index, variant, len)
                .map($wrapper)
        }

        fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
            self.0.serialize_map(len).map($wrapper)
        }

        fn serialize_struct(
            self,
            name: &'static str,
            len: usize,
        ) -> Result<Self::SerializeStruct, Self::Error> {
            self.0.serialize_struct(name, len).map($wrapper)
        }

        fn serialize_struct_variant(
            self,
            name: &'static str,
            variant_index: u32,
            variant: &'static str,
            len: usize,
        ) -> Result<Self::SerializeStructVariant, Self::Error> {
            self.0
                .serialize_struct_variant(name, variant_index, variant, len)
                .map($wrapper)
        }
    }

    impl<S> SerializeSeq for $wrapper<S>
    where
        S: SerializeSeq,
    {
        type Ok = S::Ok;
        type Error = S::Error;
        fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), S::Error>
        where
            T: Serialize,
        {
            self.0.serialize_element(&$wrapper(value))
        }
        fn end(self) -> Result<S::Ok, S::Error> {
            self.0.end()
        }
    }

    impl<S> SerializeTuple for $wrapper<S>
    where
        S: SerializeTuple,
    {
        type Ok = S::Ok;
        type Error = S::Error;
        fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), S::Error>
        where
            T: Serialize,
        {
            self.0.serialize_element(&$wrapper(value))
        }
        fn end(self) -> Result<S::Ok, S::Error> {
            self.0.end()
        }
    }

    impl<S> SerializeTupleStruct for $wrapper<S>
    where
        S: SerializeTupleStruct,
    {
        type Ok = S::Ok;
        type Error = S::Error;
        fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), S::Error>
        where
            T: Serialize,
        {
            self.0.serialize_field(&$wrapper(value))
        }
        fn end(self) -> Result<S::Ok, S::Error> {
            self.0.end()
        }
    }

    impl<S> SerializeTupleVariant for $wrapper<S>
    where
        S: SerializeTupleVariant,
    {
        type Ok = S::Ok;
        type Error = S::Error;
        fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), S::Error>
        where
            T: Serialize,
        {
            self.0.serialize_field(&$wrapper(value))
        }
        fn end(self) -> Result<S::Ok, S::Error> {
            self.0.end()
        }
    }

    impl<S> SerializeMap for $wrapper<S>
    where
        S: SerializeMap,
    {
        type Ok = S::Ok;
        type Error = S::Error;
        fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), S::Error>
        where
            T: Serialize,
        {
            self.0.serialize_key(&$wrapper(key))
        }
        fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), S::Error>
        where
            T: Serialize,
        {
            self.0.serialize_value(&$wrapper(value))
        }
        fn serialize_entry<K: ?Sized, V: ?Sized>(&mut self, key: &K, value: &V) -> Result<(), S::Error>
        where
            K: Serialize,
            V: Serialize,
        {
            self.0.serialize_entry(key, &$wrapper(value))
        }
        fn end(self) -> Result<S::Ok, S::Error> {
            self.0.end()
        }
    }

    impl<S> SerializeStruct for $wrapper<S>
    where
        S: SerializeStruct,
    {
        type Ok = S::Ok;
        type Error = S::Error;
        fn serialize_field<T: ?Sized>(&mut self, name: &'static str, field: &T) -> Result<(), S::Error>
        where
            T: Serialize,
        {
            self.0.serialize_field(name, &$wrapper(field))
        }
        fn end(self) -> Result<S::Ok, S::Error> {
            self.0.end()
        }
    }

    impl<S> SerializeStructVariant for $wrapper<S>
    where
        S: SerializeStructVariant,
    {
        type Ok = S::Ok;
        type Error = S::Error;
        fn serialize_field<T: ?Sized>(&mut self, name: &'static str, field: &T) -> Result<(), S::Error>
        where
            T: Serialize,
        {
            self.0.serialize_field(name, &$wrapper(field))
        }
        fn end(self) -> Result<S::Ok, S::Error> {
            self.0.end()
        }
    }
    }
}

impl_serializer!(Readable, true);
impl_serializer!(Compact, false);
