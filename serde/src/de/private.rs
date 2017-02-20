use core::marker::PhantomData;

use de::{Deserialize, Deserializer, Error, Visitor};

#[cfg(any(feature = "std", feature = "collections"))]
pub use de::content::{Content, ContentRefDeserializer, ContentDeserializer, TaggedContentVisitor,
                      TagOrContentField, TagOrContentFieldVisitor, InternallyTaggedUnitVisitor,
                      UntaggedUnitVisitor};

/// If the missing field is of type `Option<T>` then treat is as `None`,
/// otherwise it is an error.
pub fn missing_field<V, E>(field: &'static str) -> Result<V, E>
    where V: Deserialize,
          E: Error
{
    struct MissingFieldDeserializer<E>(&'static str, PhantomData<E>);

    impl<E> Deserializer for MissingFieldDeserializer<E>
        where E: Error
    {
        type Error = E;

        fn deserialize<V>(self, _visitor: V) -> Result<V::Value, E>
            where V: Visitor
        {
            Err(Error::missing_field(self.0))
        }

        fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, E>
            where V: Visitor
        {
            visitor.visit_none()
        }

        forward_to_deserialize! {
            bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit seq
            seq_fixed_size bytes byte_buf map unit_struct newtype_struct
            tuple_struct struct struct_field tuple enum ignored_any
        }
    }

    let deserializer = MissingFieldDeserializer(field, PhantomData);
    Deserialize::deserialize(deserializer)
}
