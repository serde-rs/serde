use crate::lib::*;

use crate::de::{DeserializeSeed, Deserializer, Error, Unexpected, Visitor};
use crate::private::from_utf8_lossy;
#[cfg(any(feature = "std", feature = "alloc"))]
use crate::serde_core_private::Content;

// FIXME: this is a copy of serde_core::format::Buf because it is private
struct Buf<'a> {
    bytes: &'a mut [u8],
    offset: usize,
}

impl<'a> Buf<'a> {
    pub fn new(bytes: &'a mut [u8]) -> Self {
        Buf { bytes, offset: 0 }
    }

    pub fn as_str(&self) -> &str {
        let slice = &self.bytes[..self.offset];
        unsafe { str::from_utf8_unchecked(slice) }
    }
}

impl<'a> fmt::Write for Buf<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.offset + s.len() > self.bytes.len() {
            Err(fmt::Error)
        } else {
            self.bytes[self.offset..self.offset + s.len()].copy_from_slice(s.as_bytes());
            self.offset += s.len();
            Ok(())
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Represents field of a struct
#[derive(Debug)]
pub enum Field {
    /// Represents non-skipped field with specified index assigned to the field
    /// after skipping all non-deserializable fields. For example, for a struct
    ///
    /// ```ignore
    /// #[derive(Deserialize)]
    /// struct StructWithSkippedFields {
    ///     #[serde(skip_deserializing)]
    ///     skipped: (),
    ///
    ///     field: (),
    /// }
    /// ```
    /// field `field` will have an index `0`, because it is the first non-skipped
    /// field of the struct (and counting starts with zero).
    Field(usize),
    /// Field that is not present in struct
    Unknown,
}

/// Creates a field deserialization seed that wrapped array with all possible
/// field names and their aliases.
///
/// # Example
///
/// ```ignore
/// let seed = FieldSeed::new(&[
///     // First field with two aliases
///     &["a", "alias 1", "alias 2"],
///     // Second field with one alias
///     &["b", "alias 3"],
///     // Third field without aliases
///     &["c"],
/// ]);
/// ```
#[derive(Debug)]
pub struct FieldSeed<'a> {
    /// List of all field aliases in order of their presence in the struct
    aliases: &'a [&'a [&'a str]],
}

impl<'a> FieldSeed<'a> {
    #[allow(missing_docs)]
    pub const fn new(aliases: &'a [&'a [&'a str]]) -> Self {
        Self { aliases }
    }

    fn matches(&self, value: &[u8]) -> Option<usize> {
        self.aliases
            .iter()
            .position(|aliases| aliases.iter().any(|a| a.as_bytes() == value))
    }

    fn index(&self, value: u64) -> Field {
        if value < self.aliases.len() as u64 {
            Field::Field(value as usize)
        } else {
            Field::Unknown
        }
    }
}

impl<'de, 'a, 'b> Visitor<'de> for &'b mut FieldSeed<'a> {
    type Value = Field;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("field identifier")
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(self.index(value))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visit_bytes(value.as_bytes())
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match self.matches(value) {
            Some(i) => Ok(Field::Field(i)),
            None => Ok(Field::Unknown),
        }
    }
}

impl<'de, 'a, 'b> DeserializeSeed<'de> for &'b mut FieldSeed<'a> {
    type Value = Field;

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_identifier(self)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Represents field of a struct that denies unknown fields
#[derive(Debug)]
pub enum FieldStrong {
    /// Represents non-skipped field with specified index assigned to the field
    /// after skipping all non-deserializable fields. For example, for a struct
    ///
    /// ```ignore
    /// #[derive(Deserialize)]
    /// struct StructWithSkippedFields {
    ///     #[serde(skip_deserializing)]
    ///     skipped: (),
    ///
    ///     field: (),
    /// }
    /// ```
    /// field `field` will have an index `0`, because it is the first non-skipped
    /// field of the struct (and counting starts with zero).
    Field(usize),
}

/// Creates a field deserialization seed that wrapped array with all possible
/// field names and their aliases.
///
/// # Example
///
/// ```ignore
/// let seed = FieldStrongSeed::new(
///     &[
///         // First field with two aliases
///         &["a", "alias 1", "alias 2"],
///         // Second field with one alias
///         &["b", "alias 3"],
///         // Third field without aliases
///         &["c"],
///     ],
///     &[
///         // First field with two aliases
///         "a", "alias 1", "alias 2",
///         // Second field with one alias
///         "b", "alias 3",
///         // Third field without aliases
///         "c",
///     ],
/// );
/// ```
#[derive(Debug)]
pub struct FieldStrongSeed<'a> {
    seed: FieldSeed<'a>,
    fields: &'static [&'static str],
}

impl<'a> FieldStrongSeed<'a> {
    #[allow(missing_docs)]
    pub const fn new(aliases: &'a [&'a [&'a str]], fields: &'static [&'static str]) -> Self {
        Self {
            seed: FieldSeed::new(aliases),
            fields,
        }
    }

    fn matches(&self, value: &[u8]) -> Option<FieldStrong> {
        match self.seed.matches(value) {
            Some(i) => Some(FieldStrong::Field(i)),
            None => None,
        }
    }
}

impl<'de, 'a, 'b> Visitor<'de> for &'b mut FieldStrongSeed<'a> {
    type Value = FieldStrong;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("field identifier")
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match self.seed.index(value) {
            Field::Field(i) => Ok(FieldStrong::Field(i)),
            Field::Unknown => {
                // length of string "field index 0 <= i < " and u64::MAX.to_string()
                let mut buf = [0u8; 21 + 20];
                let mut writer = Buf::new(&mut buf);
                fmt::Write::write_fmt(&mut writer, format_args!("field index 0 <= i < {}", value))
                    .unwrap();
                Err(Error::invalid_value(
                    Unexpected::Unsigned(value),
                    &writer.as_str(),
                ))
            }
        }
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match self.matches(value.as_bytes()) {
            Some(field) => Ok(field),
            None => Err(Error::unknown_field(value, self.fields)),
        }
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match self.matches(value) {
            Some(field) => Ok(field),
            None => Err(Error::unknown_field(&from_utf8_lossy(value), self.fields)),
        }
    }
}

impl<'de, 'a, 'b> DeserializeSeed<'de> for &'b mut FieldStrongSeed<'a> {
    type Value = FieldStrong;

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_identifier(self)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Represents field of a struct with flatten fields
#[cfg(any(feature = "std", feature = "alloc"))]
pub enum FieldFlatten<'de> {
    /// Represents non-skipped field with specified index assigned to the field
    /// after skipping all non-deserializable fields. For example, for a struct
    ///
    /// ```ignore
    /// #[derive(Deserialize)]
    /// struct StructWithSkippedFields {
    ///     #[serde(skip_deserializing)]
    ///     skipped: (),
    ///
    ///     field: (),
    /// }
    /// ```
    /// field `field` will have an index `0`, because it is the first non-skipped
    /// field of the struct (and counting starts with zero).
    Field(usize),
    /// Field that is not present in the struct and will be forwarded to the
    /// flatten fields
    Other(Content<'de>),
}

/// Creates a field deserialization seed that wrapped array with all possible
/// field names and their aliases.
///
/// # Example
///
/// ```ignore
/// let seed = FieldFlattenSeed::new(&[
///     // First field with two aliases
///     &["a", "alias 1", "alias 2"],
///     // Second field with one alias
///     &["b", "alias 3"],
///     // Third field without aliases
///     &["c"],
/// ]);
/// ```
#[cfg(any(feature = "std", feature = "alloc"))]
#[derive(Debug)]
pub struct FieldFlattenSeed<'a> {
    seed: FieldSeed<'a>,
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'a> FieldFlattenSeed<'a> {
    #[allow(missing_docs)]
    pub const fn new(aliases: &'a [&'a [&'a str]]) -> Self {
        Self {
            seed: FieldSeed::new(aliases),
        }
    }

    fn matches(&self, value: &[u8]) -> Option<FieldFlatten<'static>> {
        match self.seed.matches(value) {
            Some(i) => Some(FieldFlatten::Field(i)),
            None => None,
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'de, 'a, 'b> Visitor<'de> for &'b mut FieldFlattenSeed<'a> {
    type Value = FieldFlatten<'de>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("field identifier")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(FieldFlatten::Other(Content::Bool(value)))
    }

    fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(FieldFlatten::Other(Content::I8(value)))
    }

    fn visit_i16<E>(self, value: i16) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(FieldFlatten::Other(Content::I16(value)))
    }

    fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(FieldFlatten::Other(Content::I32(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(FieldFlatten::Other(Content::I64(value)))
    }

    fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(FieldFlatten::Other(Content::U8(value)))
    }

    fn visit_u16<E>(self, value: u16) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(FieldFlatten::Other(Content::U16(value)))
    }

    fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(FieldFlatten::Other(Content::U32(value)))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(FieldFlatten::Other(Content::U64(value)))
    }

    fn visit_f32<E>(self, value: f32) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(FieldFlatten::Other(Content::F32(value)))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(FieldFlatten::Other(Content::F64(value)))
    }

    fn visit_char<E>(self, value: char) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(FieldFlatten::Other(Content::Char(value)))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(FieldFlatten::Other(Content::Unit))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match self.matches(value.as_bytes()) {
            Some(field) => Ok(field),
            None => Ok(FieldFlatten::Other(Content::String(value))),
        }
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match self.matches(value.as_bytes()) {
            Some(field) => Ok(field),
            None => Ok(FieldFlatten::Other(Content::String(value.to_string()))),
        }
    }

    fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match self.matches(value.as_bytes()) {
            Some(field) => Ok(field),
            None => Ok(FieldFlatten::Other(Content::Str(value))),
        }
    }

    fn visit_byte_buf<E>(self, value: Vec<u8>) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match self.matches(&value) {
            Some(field) => Ok(field),
            None => Ok(FieldFlatten::Other(Content::ByteBuf(value))),
        }
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match self.matches(value) {
            Some(field) => Ok(field),
            None => Ok(FieldFlatten::Other(Content::ByteBuf(value.to_vec()))),
        }
    }

    fn visit_borrowed_bytes<E>(self, value: &'de [u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match self.matches(value) {
            Some(field) => Ok(field),
            None => Ok(FieldFlatten::Other(Content::Bytes(value))),
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'de, 'a, 'b> DeserializeSeed<'de> for &'b mut FieldFlattenSeed<'a> {
    type Value = FieldFlatten<'de>;

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_identifier(self)
    }
}
