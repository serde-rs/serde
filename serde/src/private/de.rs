// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use lib::*;

use de::{Deserialize, Deserializer, Error, Visitor};

#[cfg(any(feature = "std", feature = "collections"))]
use de::Unexpected;

#[cfg(any(feature = "std", feature = "collections"))]
pub use self::content::{Content, ContentRefDeserializer, ContentDeserializer,
                        TaggedContentVisitor, TagOrContentField, TagOrContentFieldVisitor,
                        InternallyTaggedUnitVisitor, UntaggedUnitVisitor};

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
            bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit seq
            seq_fixed_size bytes byte_buf map unit_struct newtype_struct
            tuple_struct struct identifier tuple enum ignored_any
        }
    }

    let deserializer = MissingFieldDeserializer(field, PhantomData);
    Deserialize::deserialize(deserializer)
}

#[cfg(any(feature = "std", feature = "collections"))]
pub fn borrow_cow_str<'de: 'a, 'a, D>(deserializer: D) -> Result<Cow<'a, str>, D::Error>
where
    D: Deserializer<'de>,
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
                Err(e) => Err(Error::invalid_value(Unexpected::Bytes(&e.into_bytes()), &self),),
            }
        }
    }

    deserializer.deserialize_str(CowStrVisitor)
}

#[cfg(any(feature = "std", feature = "collections"))]
pub fn borrow_cow_bytes<'de: 'a, 'a, D>(deserializer: D) -> Result<Cow<'a, [u8]>, D::Error>
where
    D: Deserializer<'de>,
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

    deserializer.deserialize_str(CowBytesVisitor)
}

pub mod size_hint {
    use lib::*;

    pub fn from_bounds<I>(iter: &I) -> Option<usize>
        where I: Iterator
    {
        helper(iter.size_hint())
    }

    pub fn cautious(hint: Option<usize>) -> usize {
        cmp::min(hint.unwrap_or(0), 4096)
    }

    fn helper(bounds: (usize, Option<usize>)) -> Option<usize> {
        match bounds {
            (lower, Some(upper)) if lower == upper => {
                Some(upper)
            }
            _ => None,
        }
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
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

    use de::{self, Deserialize, DeserializeSeed, Deserializer, Visitor, SeqAccess, MapAccess,
             EnumAccess, Unexpected};
    use super::size_hint;

    /// Used from generated code to buffer the contents of the Deserializer when
    /// deserializing untagged enums and internally tagged enums.
    ///
    /// Not public API. Use serde-value instead.
    #[derive(Debug)]
    pub enum Content {
        Bool(bool),

        U8(u8),
        U16(u16),
        U32(u32),
        U64(u64),

        I8(i8),
        I16(i16),
        I32(i32),
        I64(i64),

        F32(f32),
        F64(f64),

        Char(char),
        String(String),
        Bytes(Vec<u8>),

        None,
        Some(Box<Content>),

        Unit,
        Newtype(Box<Content>),
        Seq(Vec<Content>),
        Map(Vec<(Content, Content)>),
    }

    impl Content {
        fn unexpected(&self) -> Unexpected {
            match *self {
                Content::Bool(b) => Unexpected::Bool(b),
                Content::U8(n) => Unexpected::Unsigned(n as u64),
                Content::U16(n) => Unexpected::Unsigned(n as u64),
                Content::U32(n) => Unexpected::Unsigned(n as u64),
                Content::U64(n) => Unexpected::Unsigned(n),
                Content::I8(n) => Unexpected::Signed(n as i64),
                Content::I16(n) => Unexpected::Signed(n as i64),
                Content::I32(n) => Unexpected::Signed(n as i64),
                Content::I64(n) => Unexpected::Signed(n),
                Content::F32(f) => Unexpected::Float(f as f64),
                Content::F64(f) => Unexpected::Float(f),
                Content::Char(c) => Unexpected::Char(c),
                Content::String(ref s) => Unexpected::Str(s),
                Content::Bytes(ref b) => Unexpected::Bytes(b),
                Content::None | Content::Some(_) => Unexpected::Option,
                Content::Unit => Unexpected::Unit,
                Content::Newtype(_) => Unexpected::NewtypeStruct,
                Content::Seq(_) => Unexpected::Seq,
                Content::Map(_) => Unexpected::Map,
            }
        }
    }

    impl<'de> Deserialize<'de> for Content {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            // Untagged and internally tagged enums are only supported in
            // self-describing formats.
            deserializer.deserialize_any(ContentVisitor)
        }
    }

    struct ContentVisitor;

    impl<'de> Visitor<'de> for ContentVisitor {
        type Value = Content;

        fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            fmt.write_str("any value")
        }

        fn visit_bool<F>(self, value: bool) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            Ok(Content::Bool(value))
        }

        fn visit_i8<F>(self, value: i8) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            Ok(Content::I8(value))
        }

        fn visit_i16<F>(self, value: i16) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            Ok(Content::I16(value))
        }

        fn visit_i32<F>(self, value: i32) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            Ok(Content::I32(value))
        }

        fn visit_i64<F>(self, value: i64) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            Ok(Content::I64(value))
        }

        fn visit_u8<F>(self, value: u8) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            Ok(Content::U8(value))
        }

        fn visit_u16<F>(self, value: u16) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            Ok(Content::U16(value))
        }

        fn visit_u32<F>(self, value: u32) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            Ok(Content::U32(value))
        }

        fn visit_u64<F>(self, value: u64) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            Ok(Content::U64(value))
        }

        fn visit_f32<F>(self, value: f32) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            Ok(Content::F32(value))
        }

        fn visit_f64<F>(self, value: f64) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            Ok(Content::F64(value))
        }

        fn visit_char<F>(self, value: char) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            Ok(Content::Char(value))
        }

        fn visit_str<F>(self, value: &str) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            Ok(Content::String(value.into()))
        }

        fn visit_string<F>(self, value: String) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            Ok(Content::String(value))
        }

        fn visit_bytes<F>(self, value: &[u8]) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            Ok(Content::Bytes(value.into()))
        }

        fn visit_byte_buf<F>(self, value: Vec<u8>) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            Ok(Content::Bytes(value))
        }

        fn visit_unit<F>(self) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            Ok(Content::Unit)
        }

        fn visit_none<F>(self) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            Ok(Content::None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            Deserialize::deserialize(deserializer).map(|v| Content::Some(Box::new(v)))
        }

        fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            Deserialize::deserialize(deserializer).map(|v| Content::Newtype(Box::new(v)))
        }

        fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
        where
            V: SeqAccess<'de>,
        {
            let mut vec = Vec::with_capacity(size_hint::cautious(visitor.size_hint()));
            while let Some(e) = try!(visitor.next_element()) {
                vec.push(e);
            }
            Ok(Content::Seq(vec))
        }

        fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
        where
            V: MapAccess<'de>,
        {
            let mut vec = Vec::with_capacity(size_hint::cautious(visitor.size_hint()));
            while let Some(kv) = try!(visitor.next_entry()) {
                vec.push(kv);
            }
            Ok(Content::Map(vec))
        }

        fn visit_enum<V>(self, _visitor: V) -> Result<Self::Value, V::Error>
        where
            V: EnumAccess<'de>,
        {
            Err(de::Error::custom("untagged and internally tagged enums do not support enum input",),)
        }
    }

    /// This is the type of the map keys in an internally tagged enum.
    ///
    /// Not public API.
    pub enum TagOrContent {
        Tag,
        Content(Content),
    }

    struct TagOrContentVisitor {
        name: &'static str,
    }

    impl TagOrContentVisitor {
        fn new(name: &'static str) -> Self {
            TagOrContentVisitor { name: name }
        }
    }

    impl<'de> DeserializeSeed<'de> for TagOrContentVisitor {
        type Value = TagOrContent;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            // Internally tagged enums are only supported in self-describing
            // formats.
            deserializer.deserialize_any(self)
        }
    }

    impl<'de> Visitor<'de> for TagOrContentVisitor {
        type Value = TagOrContent;

        fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            write!(fmt, "a type tag `{}` or any other value", self.name)
        }

        fn visit_bool<F>(self, value: bool) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            ContentVisitor
                .visit_bool(value)
                .map(TagOrContent::Content)
        }

        fn visit_i8<F>(self, value: i8) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            ContentVisitor.visit_i8(value).map(TagOrContent::Content)
        }

        fn visit_i16<F>(self, value: i16) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            ContentVisitor
                .visit_i16(value)
                .map(TagOrContent::Content)
        }

        fn visit_i32<F>(self, value: i32) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            ContentVisitor
                .visit_i32(value)
                .map(TagOrContent::Content)
        }

        fn visit_i64<F>(self, value: i64) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            ContentVisitor
                .visit_i64(value)
                .map(TagOrContent::Content)
        }

        fn visit_u8<F>(self, value: u8) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            ContentVisitor.visit_u8(value).map(TagOrContent::Content)
        }

        fn visit_u16<F>(self, value: u16) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            ContentVisitor
                .visit_u16(value)
                .map(TagOrContent::Content)
        }

        fn visit_u32<F>(self, value: u32) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            ContentVisitor
                .visit_u32(value)
                .map(TagOrContent::Content)
        }

        fn visit_u64<F>(self, value: u64) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            ContentVisitor
                .visit_u64(value)
                .map(TagOrContent::Content)
        }

        fn visit_f32<F>(self, value: f32) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            ContentVisitor
                .visit_f32(value)
                .map(TagOrContent::Content)
        }

        fn visit_f64<F>(self, value: f64) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            ContentVisitor
                .visit_f64(value)
                .map(TagOrContent::Content)
        }

        fn visit_char<F>(self, value: char) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            ContentVisitor
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
                ContentVisitor
                    .visit_str(value)
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
                ContentVisitor
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
                ContentVisitor
                    .visit_bytes(value)
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
                ContentVisitor
                    .visit_byte_buf(value)
                    .map(TagOrContent::Content)
            }
        }

        fn visit_unit<F>(self) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            ContentVisitor.visit_unit().map(TagOrContent::Content)
        }

        fn visit_none<F>(self) -> Result<Self::Value, F>
        where
            F: de::Error,
        {
            ContentVisitor.visit_none().map(TagOrContent::Content)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            ContentVisitor
                .visit_some(deserializer)
                .map(TagOrContent::Content)
        }

        fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            ContentVisitor
                .visit_newtype_struct(deserializer)
                .map(TagOrContent::Content)
        }

        fn visit_seq<V>(self, visitor: V) -> Result<Self::Value, V::Error>
        where
            V: SeqAccess<'de>,
        {
            ContentVisitor
                .visit_seq(visitor)
                .map(TagOrContent::Content)
        }

        fn visit_map<V>(self, visitor: V) -> Result<Self::Value, V::Error>
        where
            V: MapAccess<'de>,
        {
            ContentVisitor
                .visit_map(visitor)
                .map(TagOrContent::Content)
        }

        fn visit_enum<V>(self, visitor: V) -> Result<Self::Value, V::Error>
        where
            V: EnumAccess<'de>,
        {
            ContentVisitor
                .visit_enum(visitor)
                .map(TagOrContent::Content)
        }
    }

    /// Used by generated code to deserialize an internally tagged enum.
    ///
    /// Not public API.
    pub struct TaggedContent<T> {
        pub tag: T,
        pub content: Content,
    }

    /// Not public API.
    pub struct TaggedContentVisitor<T> {
        tag_name: &'static str,
        tag: PhantomData<T>,
    }

    impl<T> TaggedContentVisitor<T> {
        /// Visitor for the content of an internally tagged enum with the given tag
        /// name.
        pub fn new(name: &'static str) -> Self {
            TaggedContentVisitor {
                tag_name: name,
                tag: PhantomData,
            }
        }
    }

    impl<'de, T> DeserializeSeed<'de> for TaggedContentVisitor<T>
    where
        T: Deserialize<'de>,
    {
        type Value = TaggedContent<T>;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            // Internally tagged enums are only supported in self-describing
            // formats.
            deserializer.deserialize_any(self)
        }
    }

    impl<'de, T> Visitor<'de> for TaggedContentVisitor<T>
    where
        T: Deserialize<'de>,
    {
        type Value = TaggedContent<T>;

        fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            fmt.write_str("any value")
        }

        fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
        where
            V: MapAccess<'de>,
        {
            let mut tag = None;
            let mut vec = Vec::with_capacity(size_hint::cautious(visitor.size_hint()));
            while let Some(k) =
                try!(visitor.next_key_seed(TagOrContentVisitor::new(self.tag_name))) {
                match k {
                    TagOrContent::Tag => {
                        if tag.is_some() {
                            return Err(de::Error::duplicate_field(self.tag_name));
                        }
                        tag = Some(try!(visitor.next_value()));
                    }
                    TagOrContent::Content(k) => {
                        let v = try!(visitor.next_value());
                        vec.push((k, v));
                    }
                }
            }
            match tag {
                None => Err(de::Error::missing_field(self.tag_name)),
                Some(tag) => {
                    Ok(
                        TaggedContent {
                            tag: tag,
                            content: Content::Map(vec),
                        },
                    )
                }
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

    /// Not public API
    pub struct ContentDeserializer<E> {
        content: Content,
        err: PhantomData<E>,
    }

    /// Used when deserializing an internally tagged enum because the content will
    /// be used exactly once.
    impl<'de, E> Deserializer<'de> for ContentDeserializer<E>
    where
        E: de::Error,
    {
        type Error = E;

        fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            match self.content {
                Content::Bool(v) => visitor.visit_bool(v),
                Content::U8(v) => visitor.visit_u8(v),
                Content::U16(v) => visitor.visit_u16(v),
                Content::U32(v) => visitor.visit_u32(v),
                Content::U64(v) => visitor.visit_u64(v),
                Content::I8(v) => visitor.visit_i8(v),
                Content::I16(v) => visitor.visit_i16(v),
                Content::I32(v) => visitor.visit_i32(v),
                Content::I64(v) => visitor.visit_i64(v),
                Content::F32(v) => visitor.visit_f32(v),
                Content::F64(v) => visitor.visit_f64(v),
                Content::Char(v) => visitor.visit_char(v),
                Content::String(v) => visitor.visit_string(v),
                Content::Unit => visitor.visit_unit(),
                Content::None => visitor.visit_none(),
                Content::Some(v) => visitor.visit_some(ContentDeserializer::new(*v)),
                Content::Newtype(v) => visitor.visit_newtype_struct(ContentDeserializer::new(*v)),
                Content::Seq(v) => {
                    let seq = v.into_iter().map(ContentDeserializer::new);
                    let mut seq_visitor = de::value::SeqDeserializer::new(seq);
                    let value = try!(visitor.visit_seq(&mut seq_visitor));
                    try!(seq_visitor.end());
                    Ok(value)
                }
                Content::Map(v) => {
                    let map = v.into_iter().map(|(k, v)| {
                                                    (ContentDeserializer::new(k),
                                                    ContentDeserializer::new(v))
                                                });
                    let mut map_visitor = de::value::MapDeserializer::new(map);
                    let value = try!(visitor.visit_map(&mut map_visitor));
                    try!(map_visitor.end());
                    Ok(value)
                }
                Content::Bytes(v) => visitor.visit_byte_buf(v),
            }
        }

        fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            match self.content {
                Content::None => visitor.visit_none(),
                Content::Some(v) => visitor.visit_some(ContentDeserializer::new(*v)),
                Content::Unit => visitor.visit_unit(),
                _ => visitor.visit_some(self),
            }
        }

        fn deserialize_newtype_struct<V>(
            self,
            _name: &str,
            visitor: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            visitor.visit_newtype_struct(self)
        }

        fn deserialize_enum<V>(
            self,
            _name: &str,
            _variants: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            let (variant, value) = match self.content {
                Content::Map(value) => {
                    let mut iter = value.into_iter();
                    let (variant, value) = match iter.next() {
                        Some(v) => v,
                        None => {
                            return Err(
                                de::Error::invalid_value(
                                    de::Unexpected::Map,
                                    &"map with a single key",
                                ),
                            );
                        }
                    };
                    // enums are encoded in json as maps with a single key:value pair
                    if iter.next().is_some() {
                        return Err(
                            de::Error::invalid_value(
                                de::Unexpected::Map,
                                &"map with a single key",
                            ),
                        );
                    }
                    (variant, Some(value))
                }
                Content::String(variant) => (Content::String(variant), None),
                other => {
                    return Err(de::Error::invalid_type(other.unexpected(), &"string or map"),);
                }
            };

            visitor.visit_enum(
                EnumDeserializer {
                    variant: variant,
                    value: value,
                    err: PhantomData,
                },
            )
        }

        forward_to_deserialize_any! {
            bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit seq
            seq_fixed_size bytes byte_buf map unit_struct tuple_struct struct
            identifier tuple ignored_any
        }
    }

    impl<E> ContentDeserializer<E> {
        /// private API, don't use
        pub fn new(content: Content) -> Self {
            ContentDeserializer {
                content: content,
                err: PhantomData,
            }
        }
    }

    struct EnumDeserializer<E>
    where
        E: de::Error,
    {
        variant: Content,
        value: Option<Content>,
        err: PhantomData<E>,
    }

    impl<'de, E> de::EnumAccess<'de> for EnumDeserializer<E>
    where
        E: de::Error,
    {
        type Error = E;
        type Variant = VariantDeserializer<Self::Error>;

        fn variant_seed<V>(
            self,
            seed: V,
        ) -> Result<(V::Value, VariantDeserializer<E>), Self::Error>
        where
            V: de::DeserializeSeed<'de>,
        {
            let visitor = VariantDeserializer {
                value: self.value,
                err: PhantomData,
            };
            seed.deserialize(ContentDeserializer::new(self.variant))
                .map(|v| (v, visitor))
        }
    }

    struct VariantDeserializer<E>
    where
        E: de::Error,
    {
        value: Option<Content>,
        err: PhantomData<E>,
    }

    impl<'de, E> de::VariantAccess<'de> for VariantDeserializer<E>
    where
        E: de::Error,
    {
        type Error = E;

        fn deserialize_unit(self) -> Result<(), E> {
            match self.value {
                Some(value) => de::Deserialize::deserialize(ContentDeserializer::new(value)),
                None => Ok(()),
            }
        }

        fn deserialize_newtype_seed<T>(self, seed: T) -> Result<T::Value, E>
        where
            T: de::DeserializeSeed<'de>,
        {
            match self.value {
                Some(value) => seed.deserialize(ContentDeserializer::new(value)),
                None => {
                    Err(de::Error::invalid_type(de::Unexpected::UnitVariant, &"newtype variant"),)
                }
            }
        }

        fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            match self.value {
                Some(Content::Seq(v)) => {
                    de::Deserializer::deserialize_any(SeqDeserializer::new(v), visitor)
                }
                Some(other) => Err(de::Error::invalid_type(other.unexpected(), &"tuple variant"),),
                None => Err(de::Error::invalid_type(de::Unexpected::UnitVariant, &"tuple variant"),),
            }
        }

        fn deserialize_struct<V>(
            self,
            _fields: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            match self.value {
                Some(Content::Map(v)) => {
                    de::Deserializer::deserialize_any(MapDeserializer::new(v), visitor)
                }
                Some(other) => Err(de::Error::invalid_type(other.unexpected(), &"struct variant"),),
                _ => Err(de::Error::invalid_type(de::Unexpected::UnitVariant, &"struct variant"),),
            }
        }
    }

    struct SeqDeserializer<E>
    where
        E: de::Error,
    {
        iter: <Vec<Content> as IntoIterator>::IntoIter,
        err: PhantomData<E>,
    }

    impl<E> SeqDeserializer<E>
    where
        E: de::Error,
    {
        fn new(vec: Vec<Content>) -> Self {
            SeqDeserializer {
                iter: vec.into_iter(),
                err: PhantomData,
            }
        }
    }

    impl<'de, E> de::Deserializer<'de> for SeqDeserializer<E>
    where
        E: de::Error,
    {
        type Error = E;

        #[inline]
        fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            let len = self.iter.len();
            if len == 0 {
                visitor.visit_unit()
            } else {
                let ret = try!(visitor.visit_seq(&mut self));
                let remaining = self.iter.len();
                if remaining == 0 {
                    Ok(ret)
                } else {
                    Err(de::Error::invalid_length(len, &"fewer elements in array"))
                }
            }
        }

        forward_to_deserialize_any! {
            bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
            seq seq_fixed_size bytes byte_buf map unit_struct newtype_struct
            tuple_struct struct identifier tuple enum ignored_any
        }
    }

    impl<'de, E> de::SeqAccess<'de> for SeqDeserializer<E>
    where
        E: de::Error,
    {
        type Error = E;

        fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
        where
            T: de::DeserializeSeed<'de>,
        {
            match self.iter.next() {
                Some(value) => {
                    seed.deserialize(ContentDeserializer::new(value))
                        .map(Some)
                }
                None => Ok(None),
            }
        }

        fn size_hint(&self) -> Option<usize> {
            size_hint::from_bounds(&self.iter)
        }
    }

    struct MapDeserializer<E>
    where
        E: de::Error,
    {
        iter: <Vec<(Content, Content)> as IntoIterator>::IntoIter,
        value: Option<Content>,
        err: PhantomData<E>,
    }

    impl<E> MapDeserializer<E>
    where
        E: de::Error,
    {
        fn new(map: Vec<(Content, Content)>) -> Self {
            MapDeserializer {
                iter: map.into_iter(),
                value: None,
                err: PhantomData,
            }
        }
    }

    impl<'de, E> de::MapAccess<'de> for MapDeserializer<E>
    where
        E: de::Error,
    {
        type Error = E;

        fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
        where
            T: de::DeserializeSeed<'de>,
        {
            match self.iter.next() {
                Some((key, value)) => {
                    self.value = Some(value);
                    seed.deserialize(ContentDeserializer::new(key)).map(Some)
                }
                None => Ok(None),
            }
        }

        fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Self::Error>
        where
            T: de::DeserializeSeed<'de>,
        {
            match self.value.take() {
                Some(value) => seed.deserialize(ContentDeserializer::new(value)),
                None => Err(de::Error::custom("value is missing")),
            }
        }

        fn size_hint(&self) -> Option<usize> {
            size_hint::from_bounds(&self.iter)
        }
    }

    impl<'de, E> de::Deserializer<'de> for MapDeserializer<E>
    where
        E: de::Error,
    {
        type Error = E;

        #[inline]
        fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            visitor.visit_map(self)
        }

        forward_to_deserialize_any! {
            bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
            seq seq_fixed_size bytes byte_buf map unit_struct newtype_struct
            tuple_struct struct identifier tuple enum ignored_any
        }
    }


    /// Not public API.
    pub struct ContentRefDeserializer<'a, E> {
        content: &'a Content,
        err: PhantomData<E>,
    }

    /// Used when deserializing an untagged enum because the content may need to be
    /// used more than once.
    impl<'de, 'a, E> Deserializer<'de> for ContentRefDeserializer<'a, E>
    where
        E: de::Error,
    {
        type Error = E;

        fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, E>
        where
            V: Visitor<'de>,
        {
            match *self.content {
                Content::Bool(v) => visitor.visit_bool(v),
                Content::U8(v) => visitor.visit_u8(v),
                Content::U16(v) => visitor.visit_u16(v),
                Content::U32(v) => visitor.visit_u32(v),
                Content::U64(v) => visitor.visit_u64(v),
                Content::I8(v) => visitor.visit_i8(v),
                Content::I16(v) => visitor.visit_i16(v),
                Content::I32(v) => visitor.visit_i32(v),
                Content::I64(v) => visitor.visit_i64(v),
                Content::F32(v) => visitor.visit_f32(v),
                Content::F64(v) => visitor.visit_f64(v),
                Content::Char(v) => visitor.visit_char(v),
                Content::String(ref v) => visitor.visit_str(v),
                Content::Unit => visitor.visit_unit(),
                Content::None => visitor.visit_none(),
                Content::Some(ref v) => visitor.visit_some(ContentRefDeserializer::new(v)),
                Content::Newtype(ref v) => {
                    visitor.visit_newtype_struct(ContentRefDeserializer::new(v))
                }
                Content::Seq(ref v) => {
                    let seq = v.into_iter().map(ContentRefDeserializer::new);
                    let mut seq_visitor = de::value::SeqDeserializer::new(seq);
                    let value = try!(visitor.visit_seq(&mut seq_visitor));
                    try!(seq_visitor.end());
                    Ok(value)
                }
                Content::Map(ref v) => {
                    let map = v.into_iter()
                        .map(
                            |&(ref k, ref v)| {
                                (ContentRefDeserializer::new(k), ContentRefDeserializer::new(v))
                            },
                        );
                    let mut map_visitor = de::value::MapDeserializer::new(map);
                    let value = try!(visitor.visit_map(&mut map_visitor));
                    try!(map_visitor.end());
                    Ok(value)
                }
                Content::Bytes(ref v) => visitor.visit_bytes(v),
            }
        }

        fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, E>
        where
            V: Visitor<'de>,
        {
            match *self.content {
                Content::None => visitor.visit_none(),
                Content::Some(ref v) => visitor.visit_some(ContentRefDeserializer::new(v)),
                Content::Unit => visitor.visit_unit(),
                _ => visitor.visit_some(self),
            }
        }

        fn deserialize_newtype_struct<V>(self, _name: &str, visitor: V) -> Result<V::Value, E>
        where
            V: Visitor<'de>,
        {
            visitor.visit_newtype_struct(self)
        }

        fn deserialize_enum<V>(
            self,
            _name: &str,
            _variants: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            let (variant, value) = match *self.content {
                Content::Map(ref value) => {
                    let mut iter = value.into_iter();
                    let &(ref variant, ref value) = match iter.next() {
                        Some(v) => v,
                        None => {
                            return Err(
                                de::Error::invalid_value(
                                    de::Unexpected::Map,
                                    &"map with a single key",
                                ),
                            );
                        }
                    };
                    // enums are encoded in json as maps with a single key:value pair
                    if iter.next().is_some() {
                        return Err(
                            de::Error::invalid_value(
                                de::Unexpected::Map,
                                &"map with a single key",
                            ),
                        );
                    }
                    (variant, Some(value))
                }
                ref s @ Content::String(_) => (s, None),
                ref other => {
                    return Err(de::Error::invalid_type(other.unexpected(), &"string or map"),);
                }
            };

            visitor.visit_enum(
                EnumRefDeserializer {
                    variant: variant,
                    value: value,
                    err: PhantomData,
                },
            )
        }

        forward_to_deserialize_any! {
            bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit seq
            seq_fixed_size bytes byte_buf map unit_struct tuple_struct struct
            identifier tuple ignored_any
        }
    }

    impl<'a, E> ContentRefDeserializer<'a, E> {
        /// private API, don't use
        pub fn new(content: &'a Content) -> Self {
            ContentRefDeserializer {
                content: content,
                err: PhantomData,
            }
        }
    }

    struct EnumRefDeserializer<'a, E>
    where
        E: de::Error,
    {
        variant: &'a Content,
        value: Option<&'a Content>,
        err: PhantomData<E>,
    }

    impl<'de, 'a, E> de::EnumAccess<'de> for EnumRefDeserializer<'a, E>
    where
        E: de::Error,
    {
        type Error = E;
        type Variant = VariantRefDeserializer<'a, Self::Error>;

        fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
        where
            V: de::DeserializeSeed<'de>,
        {
            let visitor = VariantRefDeserializer {
                value: self.value,
                err: PhantomData,
            };
            seed.deserialize(ContentRefDeserializer::new(self.variant))
                .map(|v| (v, visitor))
        }
    }

    struct VariantRefDeserializer<'a, E>
    where
        E: de::Error,
    {
        value: Option<&'a Content>,
        err: PhantomData<E>,
    }

    impl<'de, 'a, E> de::VariantAccess<'de> for VariantRefDeserializer<'a, E>
    where
        E: de::Error,
    {
        type Error = E;

        fn deserialize_unit(self) -> Result<(), E> {
            match self.value {
                Some(value) => de::Deserialize::deserialize(ContentRefDeserializer::new(value)),
                None => Ok(()),
            }
        }

        fn deserialize_newtype_seed<T>(self, seed: T) -> Result<T::Value, E>
        where
            T: de::DeserializeSeed<'de>,
        {
            match self.value {
                Some(value) => seed.deserialize(ContentRefDeserializer::new(value)),
                None => {
                    Err(de::Error::invalid_type(de::Unexpected::UnitVariant, &"newtype variant"),)
                }
            }
        }

        fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            match self.value {
                Some(&Content::Seq(ref v)) => {
                    de::Deserializer::deserialize_any(SeqRefDeserializer::new(v), visitor)
                }
                Some(other) => Err(de::Error::invalid_type(other.unexpected(), &"tuple variant"),),
                None => Err(de::Error::invalid_type(de::Unexpected::UnitVariant, &"tuple variant"),),
            }
        }

        fn deserialize_struct<V>(
            self,
            _fields: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            match self.value {
                Some(&Content::Map(ref v)) => {
                    de::Deserializer::deserialize_any(MapRefDeserializer::new(v), visitor)
                }
                Some(other) => Err(de::Error::invalid_type(other.unexpected(), &"struct variant"),),
                _ => Err(de::Error::invalid_type(de::Unexpected::UnitVariant, &"struct variant"),),
            }
        }
    }

    struct SeqRefDeserializer<'a, E>
    where
        E: de::Error,
    {
        iter: <&'a [Content] as IntoIterator>::IntoIter,
        err: PhantomData<E>,
    }

    impl<'a, E> SeqRefDeserializer<'a, E>
    where
        E: de::Error,
    {
        fn new(vec: &'a [Content]) -> Self {
            SeqRefDeserializer {
                iter: vec.into_iter(),
                err: PhantomData,
            }
        }
    }

    impl<'de, 'a, E> de::Deserializer<'de> for SeqRefDeserializer<'a, E>
    where
        E: de::Error,
    {
        type Error = E;

        #[inline]
        fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            let len = self.iter.len();
            if len == 0 {
                visitor.visit_unit()
            } else {
                let ret = try!(visitor.visit_seq(&mut self));
                let remaining = self.iter.len();
                if remaining == 0 {
                    Ok(ret)
                } else {
                    Err(de::Error::invalid_length(len, &"fewer elements in array"))
                }
            }
        }

        forward_to_deserialize_any! {
            bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
            seq seq_fixed_size bytes byte_buf map unit_struct newtype_struct
            tuple_struct struct identifier tuple enum ignored_any
        }
    }

    impl<'de, 'a, E> de::SeqAccess<'de> for SeqRefDeserializer<'a, E>
    where
        E: de::Error,
    {
        type Error = E;

        fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
        where
            T: de::DeserializeSeed<'de>,
        {
            match self.iter.next() {
                Some(value) => {
                    seed.deserialize(ContentRefDeserializer::new(value))
                        .map(Some)
                }
                None => Ok(None),
            }
        }

        fn size_hint(&self) -> Option<usize> {
            size_hint::from_bounds(&self.iter)
        }
    }

    struct MapRefDeserializer<'a, E>
    where
        E: de::Error,
    {
        iter: <&'a [(Content, Content)] as IntoIterator>::IntoIter,
        value: Option<&'a Content>,
        err: PhantomData<E>,
    }

    impl<'a, E> MapRefDeserializer<'a, E>
    where
        E: de::Error,
    {
        fn new(map: &'a [(Content, Content)]) -> Self {
            MapRefDeserializer {
                iter: map.into_iter(),
                value: None,
                err: PhantomData,
            }
        }
    }

    impl<'de, 'a, E> de::MapAccess<'de> for MapRefDeserializer<'a, E>
    where
        E: de::Error,
    {
        type Error = E;

        fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
        where
            T: de::DeserializeSeed<'de>,
        {
            match self.iter.next() {
                Some(&(ref key, ref value)) => {
                    self.value = Some(value);
                    seed.deserialize(ContentRefDeserializer::new(key))
                        .map(Some)
                }
                None => Ok(None),
            }
        }

        fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Self::Error>
        where
            T: de::DeserializeSeed<'de>,
        {
            match self.value.take() {
                Some(value) => seed.deserialize(ContentRefDeserializer::new(value)),
                None => Err(de::Error::custom("value is missing")),
            }
        }

        fn size_hint(&self) -> Option<usize> {
            size_hint::from_bounds(&self.iter)
        }
    }

    impl<'de, 'a, E> de::Deserializer<'de> for MapRefDeserializer<'a, E>
    where
        E: de::Error,
    {
        type Error = E;

        #[inline]
        fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            visitor.visit_map(self)
        }

        forward_to_deserialize_any! {
            bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
            seq seq_fixed_size bytes byte_buf map unit_struct newtype_struct
            tuple_struct struct identifier tuple enum ignored_any
        }
    }

    impl<'de, E> de::IntoDeserializer<'de, E> for ContentDeserializer<E>
    where
        E: de::Error,
    {
        type Deserializer = Self;

        fn into_deserializer(self) -> Self {
            self
        }
    }

    impl<'de, 'a, E> de::IntoDeserializer<'de, E> for ContentRefDeserializer<'a, E>
    where
        E: de::Error,
    {
        type Deserializer = Self;

        fn into_deserializer(self) -> Self {
            self
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
            write!(formatter, "unit variant {}::{}", self.type_name, self.variant_name)
        }

        fn visit_map<V>(self, _: V) -> Result<(), V::Error>
        where
            V: MapAccess<'de>,
        {
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
            write!(formatter, "unit variant {}::{}", self.type_name, self.variant_name)
        }

        fn visit_unit<E>(self) -> Result<(), E>
        where
            E: de::Error,
        {
            Ok(())
        }
    }
}
