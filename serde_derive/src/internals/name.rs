use crate::internals::attr::{Attr, VecAttr};
use std::collections::BTreeSet;

pub struct MultiName {
    pub(crate) serialize: String,
    pub(crate) serialize_renamed: bool,
    pub(crate) deserialize: String,
    pub(crate) deserialize_renamed: bool,
    pub(crate) deserialize_aliases: BTreeSet<String>,
}

impl MultiName {
    pub(crate) fn from_attrs(
        source_name: String,
        ser_name: Attr<String>,
        de_name: Attr<String>,
        de_aliases: Option<VecAttr<String>>,
    ) -> Self {
        let mut alias_set = BTreeSet::new();
        if let Some(de_aliases) = de_aliases {
            for alias_name in de_aliases.get() {
                alias_set.insert(alias_name);
            }
        }

        let ser_name = ser_name.get();
        let ser_renamed = ser_name.is_some();
        let de_name = de_name.get();
        let de_renamed = de_name.is_some();
        MultiName {
            serialize: ser_name.unwrap_or_else(|| source_name.clone()),
            serialize_renamed: ser_renamed,
            deserialize: de_name.unwrap_or(source_name),
            deserialize_renamed: de_renamed,
            deserialize_aliases: alias_set,
        }
    }

    /// Return the container name for the container when serializing.
    pub fn serialize_name(&self) -> &str {
        &self.serialize
    }

    /// Return the container name for the container when deserializing.
    pub fn deserialize_name(&self) -> &str {
        &self.deserialize
    }

    pub(crate) fn deserialize_aliases(&self) -> &BTreeSet<String> {
        &self.deserialize_aliases
    }
}
