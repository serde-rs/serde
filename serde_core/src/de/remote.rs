use crate::Deserializer;

/// TODO
pub trait RemoteDeserialize<'de, Remote> where Remote: Sized {
    /// TODO
    fn deserialize<D>(deserializer: D) -> Result<Remote, D::Error>
    where
        D: Deserializer<'de>;
}