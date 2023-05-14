//! Building blocks for conversion `Visitor` into `Deserializer`.
//!
//! Those deserializers can be used to temporary save the argument of a `Visitor`
//! method and pass it to the same method in which it was captured, later.

use lib::*;

use de::{
    self, Deserialize, Deserializer, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, Visitor,
};

primitive_deserializer!(
    /// A deserializer holding a `bool`.
    ///
    /// This deserializer will call [`Visitor::visit_bool`] for all requests.
    pub BoolDeserializer, bool, visit_bool
);

primitive_deserializer!(
    /// A deserializer holding an `i8`.
    ///
    /// This deserializer will call [`Visitor::visit_i8`] for all requests.
    pub I8Deserializer, i8, visit_i8
);
primitive_deserializer!(
    /// A deserializer holding an `i16`.
    ///
    /// This deserializer will call [`Visitor::visit_i16`] for all requests.
    pub I16Deserializer, i16, visit_i16
);
primitive_deserializer!(
    /// A deserializer holding an `i32`.
    ///
    /// This deserializer will call [`Visitor::visit_i32`] for all requests.
    pub I32Deserializer, i32, visit_i32
);
primitive_deserializer!(
    /// A deserializer holding an `i64`.
    ///
    /// This deserializer will call [`Visitor::visit_i64`] for all requests.
    pub I64Deserializer, i64, visit_i64
);

primitive_deserializer!(
    /// A deserializer holding a `u8`.
    ///
    /// This deserializer will call [`Visitor::visit_u8`] for all requests.
    pub U8Deserializer, u8, visit_u8
);
primitive_deserializer!(
    /// A deserializer holding a `u16`.
    ///
    /// This deserializer will call [`Visitor::visit_u16`] for all requests.
    pub U16Deserializer, u16, visit_u16
);
primitive_deserializer!(
    /// A deserializer holding a `u32`.
    ///
    /// This deserializer will call [`Visitor::visit_u32`] for all requests.
    pub U32Deserializer, u32, visit_u32
);
primitive_deserializer!(
    /// A deserializer holding a `u64`.
    ///
    /// This deserializer will call [`Visitor::visit_u64`] for all requests.
    pub U64Deserializer, u64, visit_u64
);

primitive_deserializer!(
    /// A deserializer holding an `f32`.
    ///
    /// This deserializer will call [`Visitor::visit_f32`] for all requests.
    pub F32Deserializer, f32, visit_f32
);
primitive_deserializer!(
    /// A deserializer holding an `f64`.
    ///
    /// This deserializer will call [`Visitor::visit_f64`] for all requests.
    pub F64Deserializer, f64, visit_f64
);
primitive_deserializer!(
    /// A deserializer holding a `char`.
    ///
    /// This deserializer will call [`Visitor::visit_char`] for all requests.
    pub CharDeserializer, char, visit_char
);

serde_if_integer128! {
    primitive_deserializer!(
        /// A deserializer holding an `i128`.
        ///
        /// This deserializer will call [`Visitor::visit_i128`] for all requests.
        pub I128Deserializer, i128, visit_i128
    );
    primitive_deserializer!(
        /// A deserializer holding a `u128`.
        ///
        /// This deserializer will call [`Visitor::visit_u128`] for all requests.
        pub U128Deserializer, u128, visit_u128
    );
}

////////////////////////////////////////////////////////////////////////////////

/// A deserializer holding a `&str`.
///
/// This deserializer will call [`Visitor::visit_str`] for all requests.
pub struct StrDeserializer<'a, E> {
    value: &'a str,
    marker: PhantomData<E>,
}

impl<'a, E> StrDeserializer<'a, E> {
    #[allow(missing_docs)]
    pub fn new(value: &'a str) -> Self {
        StrDeserializer {
            value,
            marker: PhantomData,
        }
    }
}

impl<'de, 'a, E> Deserializer<'de> for StrDeserializer<'a, E>
where
    E: de::Error,
{
    type Error = E;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(self.value)
    }
}

impl_copy_clone!(StrDeserializer<'de>);

impl<'a, E> Debug for StrDeserializer<'a, E> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("StrDeserializer")
            .field("value", &self.value)
            .finish()
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A deserializer holding a `&str` with a lifetime tied to another
/// deserializer.
///
/// This deserializer will call [`Visitor::visit_borrowed_str`] for all requests.
pub struct BorrowedStrDeserializer<'de, E> {
    value: &'de str,
    marker: PhantomData<E>,
}

impl<'de, E> BorrowedStrDeserializer<'de, E> {
    #[allow(missing_docs)]
    pub fn new(value: &'de str) -> Self {
        BorrowedStrDeserializer {
            value,
            marker: PhantomData,
        }
    }
}

impl<'de, E> Deserializer<'de> for BorrowedStrDeserializer<'de, E>
where
    E: de::Error,
{
    type Error = E;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.value)
    }
}

impl_copy_clone!(BorrowedStrDeserializer<'de>);

impl<'de, E> Debug for BorrowedStrDeserializer<'de, E> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("BorrowedStrDeserializer")
            .field("value", &self.value)
            .finish()
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A deserializer holding a `String`.
///
/// This deserializer will call [`Visitor::visit_string`] for all requests.
#[cfg(any(feature = "std", feature = "alloc"))]
pub struct StringDeserializer<E> {
    value: String,
    marker: PhantomData<E>,
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<E> StringDeserializer<E> {
    #[allow(missing_docs)]
    pub fn new(value: String) -> Self {
        StringDeserializer {
            value,
            marker: PhantomData,
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'de, E> Deserializer<'de> for StringDeserializer<E>
where
    E: de::Error,
{
    type Error = E;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.value)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<E> Clone for StringDeserializer<E> {
    fn clone(&self) -> Self {
        StringDeserializer {
            value: self.value.clone(),
            marker: PhantomData,
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<E> Debug for StringDeserializer<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("StringDeserializer")
            .field("value", &self.value)
            .finish()
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A deserializer holding a `&[u8]`.
///
/// This deserializer will call [`Visitor::visit_bytes`] for all requests.
pub struct BytesDeserializer<'a, E> {
    value: &'a [u8],
    marker: PhantomData<E>,
}

impl<'a, E> BytesDeserializer<'a, E> {
    #[allow(missing_docs)]
    pub fn new(value: &'a [u8]) -> Self {
        BytesDeserializer {
            value,
            marker: PhantomData,
        }
    }
}

impl<'de, 'a, E> Deserializer<'de> for BytesDeserializer<'a, E>
where
    E: de::Error,
{
    type Error = E;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bytes(self.value)
    }
}

impl_copy_clone!(BytesDeserializer<'a>);

impl<'a, E> Debug for BytesDeserializer<'a, E> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("BytesDeserializer")
            .field("value", &self.value)
            .finish()
    }
}

impl<'de, 'a, E> IntoDeserializer<'de, E> for &'a [u8]
where
    E: de::Error,
{
    type Deserializer = BytesDeserializer<'a, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        BytesDeserializer::new(self)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A deserializer holding a `&[u8]` with a lifetime tied to another
/// deserializer.
///
/// This deserializer will call [`Visitor::visit_borrowed_bytes`] for all requests.
pub struct BorrowedBytesDeserializer<'de, E> {
    value: &'de [u8],
    marker: PhantomData<E>,
}

impl<'de, E> BorrowedBytesDeserializer<'de, E> {
    #[allow(missing_docs)]
    pub fn new(value: &'de [u8]) -> Self {
        BorrowedBytesDeserializer {
            value,
            marker: PhantomData,
        }
    }
}

impl<'de, E> Deserializer<'de> for BorrowedBytesDeserializer<'de, E>
where
    E: de::Error,
{
    type Error = E;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_bytes(self.value)
    }
}

impl_copy_clone!(BorrowedBytesDeserializer<'de>);

impl<'de, E> Debug for BorrowedBytesDeserializer<'de, E> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("BorrowedBytesDeserializer")
            .field("value", &self.value)
            .finish()
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A deserializer holding a `Vec<u8>`.
///
/// This deserializer will call [`Visitor::visit_byte_buf`] for all requests.
#[cfg(any(feature = "std", feature = "alloc"))]
pub struct ByteBufDeserializer<E> {
    value: Vec<u8>,
    marker: PhantomData<E>,
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<E> ByteBufDeserializer<E> {
    #[allow(missing_docs)]
    pub fn new(value: Vec<u8>) -> Self {
        ByteBufDeserializer {
            value,
            marker: PhantomData,
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'de, E> Deserializer<'de> for ByteBufDeserializer<E>
where
    E: de::Error,
{
    type Error = E;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_byte_buf(self.value)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<E> Clone for ByteBufDeserializer<E> {
    fn clone(&self) -> Self {
        ByteBufDeserializer {
            value: self.value.clone(),
            marker: PhantomData,
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<E> Debug for ByteBufDeserializer<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("ByteBufDeserializer")
            .field("value", &self.value)
            .finish()
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A deserializer holding a [`None`].
///
/// This deserializer will call [`Visitor::visit_none`] for all requests.
pub struct NoneDeserializer<E> {
    marker: PhantomData<E>,
}

impl<E> NoneDeserializer<E> {
    #[allow(missing_docs)]
    pub fn new() -> Self {
        NoneDeserializer {
            marker: PhantomData,
        }
    }
}

impl<'de, E> Deserializer<'de> for NoneDeserializer<E>
where
    E: de::Error,
{
    type Error = E;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_none()
    }
}

impl_copy_clone!(NoneDeserializer);

impl<E> Debug for NoneDeserializer<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_struct("NoneDeserializer").finish()
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A deserializer holding a [`Some`].
///
/// This deserializer will call [`Visitor::visit_some`] with provided inner
/// deserializer for all requests.
#[derive(Clone, Copy)]
pub struct SomeDeserializer<T> {
    deserializer: T,
}

impl<T> SomeDeserializer<T> {
    #[allow(missing_docs)]
    pub fn new(deserializer: T) -> Self {
        SomeDeserializer {
            deserializer: deserializer,
        }
    }
}

impl<'de, T> Deserializer<'de> for SomeDeserializer<T>
where
    T: Deserializer<'de>,
{
    type Error = T::Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self.deserializer)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A deserializer holding a `()`.
///
/// This deserializer will call [`Visitor::visit_unit`] for all requests.
pub struct UnitDeserializer<E> {
    marker: PhantomData<E>,
}

impl<E> UnitDeserializer<E> {
    #[allow(missing_docs)]
    pub fn new() -> Self {
        UnitDeserializer {
            marker: PhantomData,
        }
    }
}

impl<'de, E> Deserializer<'de> for UnitDeserializer<E>
where
    E: de::Error,
{
    type Error = E;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

impl_copy_clone!(UnitDeserializer);

impl<E> Debug for UnitDeserializer<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_struct("UnitDeserializer").finish()
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A deserializer holding a newtype.
///
/// This deserializer will call [`Visitor::visit_newtype_struct`] with provided
/// inner deserializer for all requests.
#[derive(Clone, Copy)]
pub struct NewtypeDeserializer<T> {
    deserializer: T,
}

impl<T> NewtypeDeserializer<T> {
    #[allow(missing_docs)]
    pub fn new(deserializer: T) -> Self {
        NewtypeDeserializer {
            deserializer: deserializer,
        }
    }
}

impl<'de, T> Deserializer<'de> for NewtypeDeserializer<T>
where
    T: Deserializer<'de>,
{
    type Error = T::Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self.deserializer)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A deserializer holding a [`SeqAccess`].
///
/// This deserializer will call [`Visitor::visit_seq`] for all requests.
#[derive(Clone, Copy, Debug)]
pub struct SeqAccessDeserializer<A> {
    seq: A,
}

impl<A> SeqAccessDeserializer<A> {
    #[allow(missing_docs)]
    pub fn new(seq: A) -> Self {
        SeqAccessDeserializer { seq }
    }
}

impl<'de, A> Deserializer<'de> for SeqAccessDeserializer<A>
where
    A: SeqAccess<'de>,
{
    type Error = A::Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(self.seq)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A deserializer holding a [`MapAccess`].
///
/// This deserializer will call [`Visitor::visit_map`] for all requests.
#[derive(Clone, Copy, Debug)]
pub struct MapAccessDeserializer<A> {
    map: A,
}

impl<A> MapAccessDeserializer<A> {
    #[allow(missing_docs)]
    pub fn new(map: A) -> Self {
        MapAccessDeserializer { map }
    }
}

impl<'de, A> Deserializer<'de> for MapAccessDeserializer<A>
where
    A: MapAccess<'de>,
{
    type Error = A::Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(self.map)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A deserializer holding an [`EnumAccess`].
///
/// This deserializer will call [`Visitor::visit_enum`] for all requests.
#[derive(Clone, Copy, Debug)]
pub struct EnumAccessDeserializer<A> {
    access: A,
}

impl<A> EnumAccessDeserializer<A> {
    #[allow(missing_docs)]
    pub fn new(access: A) -> Self {
        EnumAccessDeserializer { access: access }
    }
}

impl<'de, A> Deserializer<'de> for EnumAccessDeserializer<A>
where
    A: EnumAccess<'de>,
{
    type Error = A::Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(self.access)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Forwards specified method of [`Visitor`] to [`Deserialize::deserialize`]
/// implementation with specified deserializer
macro_rules! forward {
    // Unit, None
    ($visit:ident => $deserializer:ident) => {
        fn $visit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Deserialize::deserialize($deserializer::new())
        }
    };
    // iXX, uXX, fXX, bool, char, str, borrowed_str, bytes, borrowed_bytes
    ($visit:ident($ty:ty) => $deserializer:ident) => {
        fn $visit<E>(self, value: $ty) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Deserialize::deserialize($deserializer::new(value))
        }
    };
    // some, newtype, seq, map, enum
    ($visit:ident : $bound:path => $deserializer:ident) => {
        fn $visit<A>(self, accessor: A) -> Result<Self::Value, A::Error>
        where
            A: $bound,
        {
            Deserialize::deserialize($deserializer::new(accessor))
        }
    };
}

/// Converts any deserializable type to a [`Visitor`] that will deserialize that
/// type from the content, captured from another deserializer.
pub struct AsVisitor<'de, T, M> {
    expecting: M,
    marker: PhantomData<(&'de (), T)>,
}

impl<'de, T, M> AsVisitor<'de, T, M> {
    /// Creates a new instance of a visitor that will use the specified message
    /// as an "expecting" string in error messages.
    pub fn new(expecting: M) -> Self {
        AsVisitor {
            expecting,
            marker: PhantomData,
        }
    }
}

impl<'de, T, M> Visitor<'de> for AsVisitor<'de, T, M>
where
    T: Deserialize<'de>,
    M: Display,
{
    type Value = T;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.expecting)
    }

    forward!(visit_bool(bool) => BoolDeserializer);

    forward!(visit_u8(u8) => U8Deserializer);
    forward!(visit_u16(u16) => U16Deserializer);
    forward!(visit_u32(u32) => U32Deserializer);
    forward!(visit_u64(u64) => U64Deserializer);

    forward!(visit_i8(i8) => I8Deserializer);
    forward!(visit_i16(i16) => I16Deserializer);
    forward!(visit_i32(i32) => I32Deserializer);
    forward!(visit_i64(i64) => I64Deserializer);

    serde_if_integer128! {
        forward!(visit_u128(u128) => U128Deserializer);
        forward!(visit_i128(i128) => I128Deserializer);
    }

    forward!(visit_f32(f32) => F32Deserializer);
    forward!(visit_f64(f64) => F64Deserializer);

    forward!(visit_char(char) => CharDeserializer);

    forward!(visit_str(&str) => StrDeserializer);
    forward!(visit_borrowed_str(&'de str) => BorrowedStrDeserializer);

    #[cfg(any(feature = "std", feature = "alloc"))]
    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Deserialize::deserialize(StringDeserializer::new(value))
    }

    forward!(visit_bytes(&[u8]) => BytesDeserializer);
    forward!(visit_borrowed_bytes(&'de [u8]) => BorrowedBytesDeserializer);

    #[cfg(any(feature = "std", feature = "alloc"))]
    fn visit_byte_buf<E>(self, value: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Deserialize::deserialize(ByteBufDeserializer::new(value))
    }

    forward!(visit_none => NoneDeserializer);
    forward!(visit_some: Deserializer<'de> => SomeDeserializer);

    forward!(visit_unit => UnitDeserializer);
    forward!(visit_newtype_struct: Deserializer<'de> => NewtypeDeserializer);
    forward!(visit_seq: SeqAccess<'de> => SeqAccessDeserializer);
    forward!(visit_map: MapAccess<'de> => MapAccessDeserializer);
    forward!(visit_enum: EnumAccess<'de> => EnumAccessDeserializer);
}
