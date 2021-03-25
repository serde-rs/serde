use ser::{Serialize, SerializeTuple, Serializer};

#[cfg(feature = "const-generics")]
impl<T, const N: usize> Serialize for [T; N]
where
    T: Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_tuple(N)?;
        for e in self.iter() {
            seq.serialize_element(e)?;
        }
        seq.end()
    }
}
