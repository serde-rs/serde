use crate::de::impls::{ArrayInPlaceVisitor, ArrayVisitor, InPlaceSeed};
use de::{Deserialize, Deserializer, Error, SeqAccess, Visitor};
use lib::fmt;

struct ArrayGuard<T, const N: usize> {
    dst: *mut T,
    initialized: usize,
}

impl<T, const N: usize> Drop for ArrayGuard<T, N> {
    fn drop(&mut self) {
        debug_assert!(self.initialized <= N);
        let initialized_part = core::ptr::slice_from_raw_parts_mut(self.dst, self.initialized);
        #[allow(unsafe_code)]
        unsafe {
            core::ptr::drop_in_place(initialized_part);
        }
    }
}

fn try_create_array<E, F, T, const N: usize>(mut cb: F) -> Result<[T; N], E>
where
    F: FnMut(usize) -> Result<T, E>,
{
    let mut array: core::mem::MaybeUninit<[T; N]> = core::mem::MaybeUninit::uninit();
    let mut guard: ArrayGuard<T, N> = ArrayGuard {
        dst: array.as_mut_ptr() as _,
        initialized: 0,
    };
    #[allow(unsafe_code)]
    unsafe {
        for (idx, value_ptr) in (&mut *array.as_mut_ptr()).iter_mut().enumerate() {
            core::ptr::write(value_ptr, cb(idx)?);
            guard.initialized += 1;
        }
        core::mem::forget(guard);
        Ok(array.assume_init())
    }
}

impl<'de, T, const N: usize> Visitor<'de> for ArrayVisitor<[T; N]>
where
    T: Deserialize<'de>,
{
    type Value = [T; N];

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("array")
    }

    #[inline]
    fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
    where
        S: SeqAccess<'de>,
    {
        try_create_array(|idx| seq.next_element()?.ok_or(Error::invalid_length(idx, &self)))
    }
}

impl<'a, 'de, T, const N: usize> Visitor<'de> for ArrayInPlaceVisitor<'a, [T; N]>
where
    T: Deserialize<'de>,
{
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("array")
    }

    #[inline]
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut fail_idx = None;
        for (idx, dest) in self.0[..].iter_mut().enumerate() {
            if seq.next_element_seed(InPlaceSeed(dest))?.is_none() {
                fail_idx = Some(idx);
                break;
            }
        }
        if let Some(idx) = fail_idx {
            return Err(Error::invalid_length(idx, &self));
        }
        Ok(())
    }
}

impl<'de, T, const N: usize> Deserialize<'de> for [T; N]
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(N, ArrayVisitor::<[T; N]>::new())
    }

    fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(N, ArrayInPlaceVisitor(place))
    }
}
