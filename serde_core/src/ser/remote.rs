use crate::Serializer;

/// TODO
pub trait RemoteSerialize<Remote> {
    /// TODO
    fn serialize<S>(origin: &Remote, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}