use lib::*;

use de::value::{BorrowedBytesDeserializer, BytesDeserializer};
use de::{Deserialize, Deserializer, Error, IntoDeserializer, Visitor};

#[cfg(any(feature = "std", feature = "alloc"))]
use de::{buffer::EnumDeserializer, DeserializeSeed, MapAccess, Unexpected};

#[cfg(any(feature = "std", feature = "alloc"))]
pub use self::content::{
    Content, InternallyTaggedUnitVisitor, TagContentOtherField, TagContentOtherFieldVisitor,
    TagOrContentField, TagOrContentFieldVisitor, TaggedContentVisitor, UntaggedUnitVisitor,
};

#[cfg(any(feature = "std", feature = "alloc"))]
pub use de::buffer::{Buffer, ContentDeserializer, ContentRefDeserializer};

pub use seed::InPlaceSeed;

/// If the missing field is of type `Option<T>` then treat is as `None`,
/// otherwise it is an error.
pub fn missing_field<'de, V, E>(field: &'static str) -> Result<V, E>
where
    V: Deserialize<'de>,
    E: Error,
{
    struct MissingFieldDeserializer<E>(&'static str, PhantomData<E>);

    impl<'de, E> Deserializer<'de> for MissingFieldDeserializer<E>
    where
        E: Error,
    {
        type Error = E;

        fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, E>
        where
            V: Visitor<'de>,
        {
            Err(Error::missing_field(self.0))
        }

        fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, E>
        where
            V: Visitor<'de>,
        {
            visitor.visit_none()
        }

        forward_to_deserialize_any! {
            bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf unit unit_struct newtype_struct seq tuple
            tuple_struct map struct enum identifier ignored_any
        }
    }

    let deserializer = MissingFieldDeserializer(field, PhantomData);
    Deserialize::deserialize(deserializer)
}

#[cfg(any(feature = "std", feature = "alloc"))]
pub fn borrow_cow_str<'de: 'a, 'a, D, R>(deserializer: D) -> Result<R, D::Error>
where
    D: Deserializer<'de>,
    R: From<Cow<'a, str>>,
{
    struct CowStrVisitor;

    impl<'a> Visitor<'a> for CowStrVisitor {
        type Value = Cow<'a, str>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Cow::Owned(v.to_owned()))
        }

        fn visit_borrowed_str<E>(self, v: &'a str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Cow::Borrowed(v))
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Cow::Owned(v))
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: Error,
        {
            match str::from_utf8(v) {
                Ok(s) => Ok(Cow::Owned(s.to_owned())),
                Err(_) => Err(Error::invalid_value(Unexpected::Bytes(v), &self)),
            }
        }

        fn visit_borrowed_bytes<E>(self, v: &'a [u8]) -> Result<Self::Value, E>
        where
            E: Error,
        {
            match str::from_utf8(v) {
                Ok(s) => Ok(Cow::Borrowed(s)),
                Err(_) => Err(Error::invalid_value(Unexpected::Bytes(v), &self)),
            }
        }

        fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
        where
            E: Error,
        {
            match String::from_utf8(v) {
                Ok(s) => Ok(Cow::Owned(s)),
                Err(e) => Err(Error::invalid_value(
                    Unexpected::Bytes(&e.into_bytes()),
                    &self,
                )),
            }
        }
    }

    deserializer.deserialize_str(CowStrVisitor).map(From::from)
}

#[cfg(any(feature = "std", feature = "alloc"))]
pub fn borrow_cow_bytes<'de: 'a, 'a, D, R>(deserializer: D) -> Result<R, D::Error>
where
    D: Deserializer<'de>,
    R: From<Cow<'a, [u8]>>,
{
    struct CowBytesVisitor;

    impl<'a> Visitor<'a> for CowBytesVisitor {
        type Value = Cow<'a, [u8]>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a byte array")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Cow::Owned(v.as_bytes().to_vec()))
        }

        fn visit_borrowed_str<E>(self, v: &'a str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Cow::Borrowed(v.as_bytes()))
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Cow::Owned(v.into_bytes()))
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Cow::Owned(v.to_vec()))
        }

        fn visit_borrowed_bytes<E>(self, v: &'a [u8]) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Cow::Borrowed(v))
        }

        fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Cow::Owned(v))
        }
    }

    deserializer
        .deserialize_bytes(CowBytesVisitor)
        .map(From::from)
}

#[cfg(any(feature = "std", feature = "alloc"))]
mod content {
    // This module is private and nothing here should be used outside of
    // generated code.
    //
    // We will iterate on the implementation for a few releases and only have to
    // worry about backward compatibility for the `untagged` and `tag` attributes
    // rather than for this entire mechanism.
    //
    // This issue is tracking making some of this stuff public:
    // https://github.com/serde-rs/serde/issues/741

    use lib::*;

    use __private::size_hint;
    use de::{
        self, Deserialize, DeserializeSeed, Deserializer, EnumAccess, IgnoredAny, MapAccess,
        SeqAccess, Unexpected, Visitor,
    };

    pub use de::buffer::{Buffer, BufferVisitor, ContentDeserializer, ContentRefDeserializer};

    pub type Content<'de> = Buffer<'de>;

    /// This is the type of the map keys in an internally tagged enum.
    ///
    /// Not public API.
    pub enum TagOrContent<'de> {
        Tag,
        Content(Buffer<'de>),
    }

    struct TagOrContentVisitor<'de> {
        name: &'static str,
        value: PhantomData<TagOrContent<'de>>,
    }

    impl<'de> TagOrContentVisitor<'de> {
        fn new(name: &'static str) -> Self {
            TagOrContentVisitor {
                name: name,
                value: PhantomData,
            }
        }
    }

    impl<'de> DeserializeSeed<'de> for TagOrContentVisitor<'de> {
        type Value = TagOrContent<'de>;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            // Internally tagged enums are only supported in self-describing
            // formats.
            deserializer.deserialize_any(self)
        }
    }

    impl<'de> Visitor<'de> for TagOrContentVisitor<'de> {
        type Value = TagOrContent<'de>;

        fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            write!(fmt, "a type tag `{}` or any other value", self.name)
        }

        fn visit_bool<F>(self, value: bool) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            BufferVisitor::new()
                .visit_bool(value)
                .map(TagOrContent::Content)
        }

        fn visit_i8<F>(self, value: i8) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            BufferVisitor::new()
                .visit_i8(value)
                .map(TagOrContent::Content)
        }

        fn visit_i16<F>(self, value: i16) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            BufferVisitor::new()
                .visit_i16(value)
                .map(TagOrContent::Content)
        }

        fn visit_i32<F>(self, value: i32) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            BufferVisitor::new()
                .visit_i32(value)
                .map(TagOrContent::Content)
        }

        fn visit_i64<F>(self, value: i64) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            BufferVisitor::new()
                .visit_i64(value)
                .map(TagOrContent::Content)
        }

        fn visit_u8<F>(self, value: u8) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            BufferVisitor::new()
                .visit_u8(value)
                .map(TagOrContent::Content)
        }

        fn visit_u16<F>(self, value: u16) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            BufferVisitor::new()
                .visit_u16(value)
                .map(TagOrContent::Content)
        }

        fn visit_u32<F>(self, value: u32) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            BufferVisitor::new()
                .visit_u32(value)
                .map(TagOrContent::Content)
        }

        fn visit_u64<F>(self, value: u64) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            BufferVisitor::new()
                .visit_u64(value)
                .map(TagOrContent::Content)
        }

        fn visit_f32<F>(self, value: f32) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            BufferVisitor::new()
                .visit_f32(value)
                .map(TagOrContent::Content)
        }

        fn visit_f64<F>(self, value: f64) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            BufferVisitor::new()
                .visit_f64(value)
                .map(TagOrContent::Content)
        }

        fn visit_char<F>(self, value: char) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            BufferVisitor::new()
                .visit_char(value)
                .map(TagOrContent::Content)
        }

        fn visit_str<F>(self, value: &str) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            if value == self.name {
                Ok(TagOrContent::Tag)
            } else {
                BufferVisitor::new()
                    .visit_str(value)
                    .map(TagOrContent::Content)
            }
        }

        fn visit_borrowed_str<F>(self, value: &'de str) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            if value == self.name {
                Ok(TagOrContent::Tag)
            } else {
                BufferVisitor::new()
                    .visit_borrowed_str(value)
                    .map(TagOrContent::Content)
            }
        }

        fn visit_string<F>(self, value: String) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            if value == self.name {
                Ok(TagOrContent::Tag)
            } else {
                BufferVisitor::new()
                    .visit_string(value)
                    .map(TagOrContent::Content)
            }
        }

        fn visit_bytes<F>(self, value: &[u8]) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            if value == self.name.as_bytes() {
                Ok(TagOrContent::Tag)
            } else {
                BufferVisitor::new()
                    .visit_bytes(value)
                    .map(TagOrContent::Content)
            }
        }

        fn visit_borrowed_bytes<F>(self, value: &'de [u8]) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            if value == self.name.as_bytes() {
                Ok(TagOrContent::Tag)
            } else {
                BufferVisitor::new()
                    .visit_borrowed_bytes(value)
                    .map(TagOrContent::Content)
            }
        }

        fn visit_byte_buf<F>(self, value: Vec<u8>) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            if value == self.name.as_bytes() {
                Ok(TagOrContent::Tag)
            } else {
                BufferVisitor::new()
                    .visit_byte_buf(value)
                    .map(TagOrContent::Content)
            }
        }

        fn visit_unit<F>(self) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            BufferVisitor::new().visit_unit().map(TagOrContent::Content)
        }

        fn visit_none<F>(self) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            BufferVisitor::new().visit_none().map(TagOrContent::Content)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            BufferVisitor::new()
                .visit_some(deserializer)
                .map(TagOrContent::Content)
        }

        fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            BufferVisitor::new()
                .visit_newtype_struct(deserializer)
                .map(TagOrContent::Content)
        }

        fn visit_seq<V>(self, visitor: V) -> Result<Self::Value, V::Error>
        where
            V: SeqAccess<'de>,
        {
            BufferVisitor::new()
                .visit_seq(visitor)
                .map(TagOrContent::Content)
        }

        fn visit_map<V>(self, visitor: V) -> Result<Self::Value, V::Error>
        where
            V: MapAccess<'de>,
        {
            BufferVisitor::new()
                .visit_map(visitor)
                .map(TagOrContent::Content)
        }

        fn visit_enum<V>(self, visitor: V) -> Result<Self::Value, V::Error>
        where
            V: EnumAccess<'de>,
        {
            BufferVisitor::new()
                .visit_enum(visitor)
                .map(TagOrContent::Content)
        }
    }

    /// Used by generated code to deserialize an internally tagged enum.
    ///
    /// Not public API.
    pub struct TaggedContent<'de, T> {
        pub tag: T,
        pub content: Buffer<'de>,
    }

    /// Not public API.
    pub struct TaggedContentVisitor<'de, T> {
        tag_name: &'static str,
        expecting: &'static str,
        value: PhantomData<TaggedContent<'de, T>>,
    }

    impl<'de, T> TaggedContentVisitor<'de, T> {
        /// Visitor for the content of an internally tagged enum with the given
        /// tag name.
        pub fn new(name: &'static str, expecting: &'static str) -> Self {
            TaggedContentVisitor {
                tag_name: name,
                expecting: expecting,
                value: PhantomData,
            }
        }
    }

    impl<'de, T> DeserializeSeed<'de> for TaggedContentVisitor<'de, T>
    where
        T: Deserialize<'de>,
    {
        type Value = TaggedContent<'de, T>;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            // Internally tagged enums are only supported in self-describing
            // formats.
            deserializer.deserialize_any(self)
        }
    }

    impl<'de, T> Visitor<'de> for TaggedContentVisitor<'de, T>
    where
        T: Deserialize<'de>,
    {
        type Value = TaggedContent<'de, T>;

        fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            fmt.write_str(self.expecting)
        }

        fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
        where
            S: SeqAccess<'de>,
        {
            let tag = match try!(seq.next_element()) {
                Some(tag) => tag,
                None => {
                    return Err(de::Error::missing_field(self.tag_name));
                }
            };
            let rest = de::value::SeqAccessDeserializer::new(seq);
            Ok(TaggedContent {
                tag: tag,
                content: try!(Buffer::deserialize(rest)),
            })
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut tag = None;
            let mut vec = Vec::with_capacity(size_hint::cautious(map.size_hint()));
            while let Some(k) = try!(map.next_key_seed(TagOrContentVisitor::new(self.tag_name))) {
                match k {
                    TagOrContent::Tag => {
                        if tag.is_some() {
                            return Err(de::Error::duplicate_field(self.tag_name));
                        }
                        tag = Some(try!(map.next_value()));
                    }
                    TagOrContent::Content(k) => {
                        let v = try!(map.next_value());
                        vec.push((k, v));
                    }
                }
            }
            match tag {
                None => Err(de::Error::missing_field(self.tag_name)),
                Some(tag) => Ok(TaggedContent {
                    tag: tag,
                    content: de::buffer::Buffer(de::buffer::BufferInner::Map(vec)),
                }),
            }
        }
    }

    /// Used by generated code to deserialize an adjacently tagged enum.
    ///
    /// Not public API.
    pub enum TagOrContentField {
        Tag,
        Content,
    }

    /// Not public API.
    pub struct TagOrContentFieldVisitor {
        pub tag: &'static str,
        pub content: &'static str,
    }

    impl<'de> DeserializeSeed<'de> for TagOrContentFieldVisitor {
        type Value = TagOrContentField;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_str(self)
        }
    }

    impl<'de> Visitor<'de> for TagOrContentFieldVisitor {
        type Value = TagOrContentField;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "{:?} or {:?}", self.tag, self.content)
        }

        fn visit_str<E>(self, field: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if field == self.tag {
                Ok(TagOrContentField::Tag)
            } else if field == self.content {
                Ok(TagOrContentField::Content)
            } else {
                Err(de::Error::invalid_value(Unexpected::Str(field), &self))
            }
        }
    }

    /// Used by generated code to deserialize an adjacently tagged enum when
    /// ignoring unrelated fields is allowed.
    ///
    /// Not public API.
    pub enum TagContentOtherField {
        Tag,
        Content,
        Other,
    }

    /// Not public API.
    pub struct TagContentOtherFieldVisitor {
        pub tag: &'static str,
        pub content: &'static str,
    }

    impl<'de> DeserializeSeed<'de> for TagContentOtherFieldVisitor {
        type Value = TagContentOtherField;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_str(self)
        }
    }

    impl<'de> Visitor<'de> for TagContentOtherFieldVisitor {
        type Value = TagContentOtherField;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(
                formatter,
                "{:?}, {:?}, or other ignored fields",
                self.tag, self.content
            )
        }

        fn visit_str<E>(self, field: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if field == self.tag {
                Ok(TagContentOtherField::Tag)
            } else if field == self.content {
                Ok(TagContentOtherField::Content)
            } else {
                Ok(TagContentOtherField::Other)
            }
        }
    }

    /// Visitor for deserializing an internally tagged unit variant.
    ///
    /// Not public API.
    pub struct InternallyTaggedUnitVisitor<'a> {
        type_name: &'a str,
        variant_name: &'a str,
    }

    impl<'a> InternallyTaggedUnitVisitor<'a> {
        /// Not public API.
        pub fn new(type_name: &'a str, variant_name: &'a str) -> Self {
            InternallyTaggedUnitVisitor {
                type_name: type_name,
                variant_name: variant_name,
            }
        }
    }

    impl<'de, 'a> Visitor<'de> for InternallyTaggedUnitVisitor<'a> {
        type Value = ();

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(
                formatter,
                "unit variant {}::{}",
                self.type_name, self.variant_name
            )
        }

        fn visit_seq<S>(self, _: S) -> Result<(), S::Error>
        where
            S: SeqAccess<'de>,
        {
            Ok(())
        }

        fn visit_map<M>(self, mut access: M) -> Result<(), M::Error>
        where
            M: MapAccess<'de>,
        {
            while try!(access.next_entry::<IgnoredAny, IgnoredAny>()).is_some() {}
            Ok(())
        }
    }

    /// Visitor for deserializing an untagged unit variant.
    ///
    /// Not public API.
    pub struct UntaggedUnitVisitor<'a> {
        type_name: &'a str,
        variant_name: &'a str,
    }

    impl<'a> UntaggedUnitVisitor<'a> {
        /// Not public API.
        pub fn new(type_name: &'a str, variant_name: &'a str) -> Self {
            UntaggedUnitVisitor {
                type_name: type_name,
                variant_name: variant_name,
            }
        }
    }

    impl<'de, 'a> Visitor<'de> for UntaggedUnitVisitor<'a> {
        type Value = ();

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(
                formatter,
                "unit variant {}::{}",
                self.type_name, self.variant_name
            )
        }

        fn visit_unit<E>(self) -> Result<(), E>
        where
            E: de::Error,
        {
            Ok(())
        }

        fn visit_none<E>(self) -> Result<(), E>
        where
            E: de::Error,
        {
            Ok(())
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

// Like `IntoDeserializer` but also implemented for `&[u8]`. This is used for
// the newtype fallthrough case of `field_identifier`.
//
//    #[derive(Deserialize)]
//    #[serde(field_identifier)]
//    enum F {
//        A,
//        B,
//        Other(String), // deserialized using IdentifierDeserializer
//    }
pub trait IdentifierDeserializer<'de, E: Error> {
    type Deserializer: Deserializer<'de, Error = E>;

    fn from(self) -> Self::Deserializer;
}

pub struct Borrowed<'de, T: 'de + ?Sized>(pub &'de T);

impl<'de, E> IdentifierDeserializer<'de, E> for u64
where
    E: Error,
{
    type Deserializer = <u64 as IntoDeserializer<'de, E>>::Deserializer;

    fn from(self) -> Self::Deserializer {
        self.into_deserializer()
    }
}

pub struct StrDeserializer<'a, E> {
    value: &'a str,
    marker: PhantomData<E>,
}

impl<'de, 'a, E> Deserializer<'de> for StrDeserializer<'a, E>
where
    E: Error,
{
    type Error = E;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(self.value)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

pub struct BorrowedStrDeserializer<'de, E> {
    value: &'de str,
    marker: PhantomData<E>,
}

impl<'de, E> Deserializer<'de> for BorrowedStrDeserializer<'de, E>
where
    E: Error,
{
    type Error = E;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.value)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'a, E> IdentifierDeserializer<'a, E> for &'a str
where
    E: Error,
{
    type Deserializer = StrDeserializer<'a, E>;

    fn from(self) -> Self::Deserializer {
        StrDeserializer {
            value: self,
            marker: PhantomData,
        }
    }
}

impl<'de, E> IdentifierDeserializer<'de, E> for Borrowed<'de, str>
where
    E: Error,
{
    type Deserializer = BorrowedStrDeserializer<'de, E>;

    fn from(self) -> Self::Deserializer {
        BorrowedStrDeserializer {
            value: self.0,
            marker: PhantomData,
        }
    }
}

impl<'a, E> IdentifierDeserializer<'a, E> for &'a [u8]
where
    E: Error,
{
    type Deserializer = BytesDeserializer<'a, E>;

    fn from(self) -> Self::Deserializer {
        BytesDeserializer::new(self)
    }
}

impl<'de, E> IdentifierDeserializer<'de, E> for Borrowed<'de, [u8]>
where
    E: Error,
{
    type Deserializer = BorrowedBytesDeserializer<'de, E>;

    fn from(self) -> Self::Deserializer {
        BorrowedBytesDeserializer::new(self.0)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
pub struct FlatMapDeserializer<'a, 'de: 'a, E>(
    pub &'a mut Vec<Option<(Buffer<'de>, Buffer<'de>)>>,
    pub PhantomData<E>,
);

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'a, 'de, E> FlatMapDeserializer<'a, 'de, E>
where
    E: Error,
{
    fn deserialize_other<V>() -> Result<V, E> {
        Err(Error::custom("can only flatten structs and maps"))
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
macro_rules! forward_to_deserialize_other {
    ($($func:ident ($($arg:ty),*))*) => {
        $(
            fn $func<V>(self, $(_: $arg,)* _visitor: V) -> Result<V::Value, Self::Error>
            where
                V: Visitor<'de>,
            {
                Self::deserialize_other()
            }
        )*
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'a, 'de, E> Deserializer<'de> for FlatMapDeserializer<'a, 'de, E>
where
    E: Error,
{
    type Error = E;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(FlatInternallyTaggedAccess {
            iter: self.0.iter_mut(),
            pending: None,
            _marker: PhantomData,
        })
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        for item in self.0.iter_mut() {
            // items in the vector are nulled out when used.  So we can only use
            // an item if it's still filled in and if the field is one we care
            // about.
            let use_item = match *item {
                None => false,
                Some((ref c, _)) => c.as_str().map_or(false, |x| variants.contains(&x)),
            };

            if use_item {
                let (key, value) = item.take().unwrap();
                return visitor.visit_enum(EnumDeserializer::new(key, Some(value)));
            }
        }

        Err(Error::custom(format_args!(
            "no variant of enum {} found in flattened data",
            name
        )))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(FlatMapAccess::new(self.0.iter()))
    }

    fn deserialize_struct<V>(
        self,
        _: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(FlatStructAccess::new(self.0.iter_mut(), fields))
    }

    fn deserialize_newtype_struct<V>(self, _name: &str, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match visitor.__private_visit_untagged_option(self) {
            Ok(value) => Ok(value),
            Err(()) => Self::deserialize_other(),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    forward_to_deserialize_other! {
        deserialize_bool()
        deserialize_i8()
        deserialize_i16()
        deserialize_i32()
        deserialize_i64()
        deserialize_u8()
        deserialize_u16()
        deserialize_u32()
        deserialize_u64()
        deserialize_f32()
        deserialize_f64()
        deserialize_char()
        deserialize_str()
        deserialize_string()
        deserialize_bytes()
        deserialize_byte_buf()
        deserialize_unit_struct(&'static str)
        deserialize_seq()
        deserialize_tuple(usize)
        deserialize_tuple_struct(&'static str, usize)
        deserialize_identifier()
        deserialize_ignored_any()
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
pub struct FlatMapAccess<'a, 'de: 'a, E> {
    iter: slice::Iter<'a, Option<(Buffer<'de>, Buffer<'de>)>>,
    pending_content: Option<&'a Buffer<'de>>,
    _marker: PhantomData<E>,
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'a, 'de, E> FlatMapAccess<'a, 'de, E> {
    fn new(iter: slice::Iter<'a, Option<(Buffer<'de>, Buffer<'de>)>>) -> FlatMapAccess<'a, 'de, E> {
        FlatMapAccess {
            iter: iter,
            pending_content: None,
            _marker: PhantomData,
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'a, 'de, E> MapAccess<'de> for FlatMapAccess<'a, 'de, E>
where
    E: Error,
{
    type Error = E;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        for item in &mut self.iter {
            // Items in the vector are nulled out when used by a struct.
            if let Some((ref key, ref content)) = *item {
                self.pending_content = Some(content);
                return seed
                    .deserialize(ContentRefDeserializer::new_inner(&key.0))
                    .map(Some);
            }
        }
        Ok(None)
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.pending_content.take() {
            Some(value) => seed.deserialize(ContentRefDeserializer::new_inner(&value.0)),
            None => Err(Error::custom("value is missing")),
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
pub struct FlatStructAccess<'a, 'de: 'a, E> {
    iter: slice::IterMut<'a, Option<(Buffer<'de>, Buffer<'de>)>>,
    pending_content: Option<Buffer<'de>>,
    fields: &'static [&'static str],
    _marker: PhantomData<E>,
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'a, 'de, E> FlatStructAccess<'a, 'de, E> {
    fn new(
        iter: slice::IterMut<'a, Option<(Buffer<'de>, Buffer<'de>)>>,
        fields: &'static [&'static str],
    ) -> FlatStructAccess<'a, 'de, E> {
        FlatStructAccess {
            iter: iter,
            pending_content: None,
            fields: fields,
            _marker: PhantomData,
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'a, 'de, E> MapAccess<'de> for FlatStructAccess<'a, 'de, E>
where
    E: Error,
{
    type Error = E;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        while let Some(item) = self.iter.next() {
            // items in the vector are nulled out when used.  So we can only use
            // an item if it's still filled in and if the field is one we care
            // about.  In case we do not know which fields we want, we take them all.
            let use_item = match *item {
                None => false,
                Some((ref c, _)) => c.as_str().map_or(false, |key| self.fields.contains(&key)),
            };

            if use_item {
                let (key, content) = item.take().unwrap();
                self.pending_content = Some(content);
                return seed.deserialize(ContentDeserializer::new(key)).map(Some);
            }
        }
        Ok(None)
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.pending_content.take() {
            Some(value) => seed.deserialize(ContentDeserializer::new(value)),
            None => Err(Error::custom("value is missing")),
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
pub struct FlatInternallyTaggedAccess<'a, 'de: 'a, E> {
    iter: slice::IterMut<'a, Option<(Buffer<'de>, Buffer<'de>)>>,
    pending: Option<&'a Buffer<'de>>,
    _marker: PhantomData<E>,
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'a, 'de, E> MapAccess<'de> for FlatInternallyTaggedAccess<'a, 'de, E>
where
    E: Error,
{
    type Error = E;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        for item in &mut self.iter {
            if let Some((ref key, ref content)) = *item {
                // Do not take(), instead borrow this entry. The internally tagged
                // enum does its own buffering so we can't tell whether this entry
                // is going to be consumed. Borrowing here leaves the entry
                // available for later flattened fields.
                self.pending = Some(content);
                return seed
                    .deserialize(ContentRefDeserializer::new_inner(&key.0))
                    .map(Some);
            }
        }
        Ok(None)
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.pending.take() {
            Some(value) => seed.deserialize(ContentRefDeserializer::new_inner(&value.0)),
            None => panic!("value is missing"),
        }
    }
}
