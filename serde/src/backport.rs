use lib::*;

// An opaque chunk of bytes that is at least as big as T.
//
// We can't use core::mem::MaybeUninit because it is not stable yet.
// https://github.com/rust-lang/rust/issues/53491
//
// We can't use [u8; core::mem::size_of::<T>()] because type parameters are not
// supported in an array length expression.
// https://github.com/rust-lang/rust/issues/43408
pub struct MaybeUninit<T> {
    #[allow(dead_code)]
    mem: ManuallyDrop<MemAtLeast<T>>,
}

enum MemAtLeast<T> {
    Uninit,
    #[allow(dead_code)]
    Value(T),
}

impl<T> MaybeUninit<T> {
    pub fn uninitialized() -> Self {
        debug_assert!(mem::size_of::<Self>() >= mem::size_of::<T>());

        MaybeUninit {
            mem: ManuallyDrop::new(MemAtLeast::Uninit),
        }
    }
}
