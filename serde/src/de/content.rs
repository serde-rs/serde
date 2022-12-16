use crate::{de, private, Deserialize, Deserializer};

/// An efficient buffer for arbitrary deserialized *content*.
///
/// ℹ️ Note that [`Content`] can only be constructed by deserialization.
#[derive(Clone)]
#[repr(transparent)]
pub struct Content<'de>(private::de::Content<'de>);

impl<'de> Content<'de> {
    /// Turns the content into a deserializer.
    pub fn into_deserializer<E: de::Error>(self) -> impl Deserializer<'de, Error = E> {
        private::de::ContentDeserializer::new(self.0)
    }
}

impl<'de> Deserialize<'de> for Content<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: crate::Deserializer<'de>,
    {
        Ok(Self(private::de::Content::deserialize(deserializer)?))
    }
}
