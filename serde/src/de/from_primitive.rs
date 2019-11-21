use lib::*;

#[cfg(integer128)]
type SignedInt = i128;

#[cfg(not(integer128))]
type SignedInt = i64;

#[cfg(integer128)]
type UnsignedInt = u128;

#[cfg(not(integer128))]
type UnsignedInt = u64;

macro_rules! int_to_int {
    ($dst:ident, $n:ident) => {
        if $dst::min_value() as SignedInt <= $n as SignedInt &&
            $n as SignedInt <= $dst::max_value() as SignedInt
        {
            Some($n as $dst)
        } else {
            None
        }
    };
}

macro_rules! int_to_uint {
    ($dst:ident, $n:ident) => {
        if 0 <= $n && $n as UnsignedInt <= $dst::max_value() as UnsignedInt {
            Some($n as $dst)
        } else {
            None
        }
    };
}

macro_rules! uint_to {
    ($dst:ident, $n:ident) => {
        if $n as UnsignedInt <= $dst::max_value() as UnsignedInt {
            Some($n as $dst)
        } else {
            None
        }
    };
}

pub trait FromPrimitive: Sized {
    fn from_i8(n: i8) -> Option<Self>;
    fn from_i16(n: i16) -> Option<Self>;
    fn from_i32(n: i32) -> Option<Self>;
    fn from_i64(n: i64) -> Option<Self>;
    #[cfg(integer128)]
    fn from_i128(n: i128) -> Option<Self>;
    fn from_u8(n: u8) -> Option<Self>;
    fn from_u16(n: u16) -> Option<Self>;
    fn from_u32(n: u32) -> Option<Self>;
    fn from_u64(n: u64) -> Option<Self>;
    #[cfg(integer128)]
    fn from_u128(n: u128) -> Option<Self>;
}

macro_rules! impl_from_primitive_for_int {
    ($t:ident) => {
        impl FromPrimitive for $t {
            #[inline]
            fn from_i8(n: i8) -> Option<Self> {
                int_to_int!($t, n)
            }
            #[inline]
            fn from_i16(n: i16) -> Option<Self> {
                int_to_int!($t, n)
            }
            #[inline]
            fn from_i32(n: i32) -> Option<Self> {
                int_to_int!($t, n)
            }
            #[inline]
            fn from_i64(n: i64) -> Option<Self> {
                int_to_int!($t, n)
            }
            #[cfg(integer128)]
            #[inline]
            fn from_i128(n: i128) -> Option<Self> {
                int_to_int!($t, n)
            }
            #[inline]
            fn from_u8(n: u8) -> Option<Self> {
                uint_to!($t, n)
            }
            #[inline]
            fn from_u16(n: u16) -> Option<Self> {
                uint_to!($t, n)
            }
            #[inline]
            fn from_u32(n: u32) -> Option<Self> {
                uint_to!($t, n)
            }
            #[inline]
            fn from_u64(n: u64) -> Option<Self> {
                uint_to!($t, n)
            }
            #[cfg(integer128)]
            #[inline]
            fn from_u128(n: u128) -> Option<Self> {
                uint_to!($t, n)
            }
        }
    };
}

macro_rules! impl_from_primitive_for_uint {
    ($t:ident) => {
        impl FromPrimitive for $t {
            #[inline]
            fn from_i8(n: i8) -> Option<Self> {
                int_to_uint!($t, n)
            }
            #[inline]
            fn from_i16(n: i16) -> Option<Self> {
                int_to_uint!($t, n)
            }
            #[inline]
            fn from_i32(n: i32) -> Option<Self> {
                int_to_uint!($t, n)
            }
            #[inline]
            fn from_i64(n: i64) -> Option<Self> {
                int_to_uint!($t, n)
            }
            #[cfg(integer128)]
            #[inline]
            fn from_i128(n: i128) -> Option<Self> {
                int_to_uint!($t, n)
            }
            #[inline]
            fn from_u8(n: u8) -> Option<Self> {
                uint_to!($t, n)
            }
            #[inline]
            fn from_u16(n: u16) -> Option<Self> {
                uint_to!($t, n)
            }
            #[inline]
            fn from_u32(n: u32) -> Option<Self> {
                uint_to!($t, n)
            }
            #[inline]
            fn from_u64(n: u64) -> Option<Self> {
                uint_to!($t, n)
            }
            #[cfg(integer128)]
            #[inline]
            fn from_u128(n: u128) -> Option<Self> {
                uint_to!($t, n)
            }
        }
    };
}

macro_rules! impl_from_primitive_for_float {
    ($t:ident) => {
        impl FromPrimitive for $t {
            #[inline]
            fn from_i8(n: i8) -> Option<Self> {
                Some(n as Self)
            }
            #[inline]
            fn from_i16(n: i16) -> Option<Self> {
                Some(n as Self)
            }
            #[inline]
            fn from_i32(n: i32) -> Option<Self> {
                Some(n as Self)
            }
            #[inline]
            fn from_i64(n: i64) -> Option<Self> {
                Some(n as Self)
            }
            #[cfg(integer128)]
            #[inline]
            fn from_i128(n: i128) -> Option<Self> {
                Some(n as Self)
            }
            #[inline]
            fn from_u8(n: u8) -> Option<Self> {
                Some(n as Self)
            }
            #[inline]
            fn from_u16(n: u16) -> Option<Self> {
                Some(n as Self)
            }
            #[inline]
            fn from_u32(n: u32) -> Option<Self> {
                Some(n as Self)
            }
            #[inline]
            fn from_u64(n: u64) -> Option<Self> {
                Some(n as Self)
            }
            #[cfg(integer128)]
            #[inline]
            fn from_u128(n: u128) -> Option<Self> {
                Some(n as Self)
            }
        }
    };
}

impl_from_primitive_for_int!(isize);
impl_from_primitive_for_int!(i8);
impl_from_primitive_for_int!(i16);
impl_from_primitive_for_int!(i32);
impl_from_primitive_for_int!(i64);
impl_from_primitive_for_uint!(usize);
impl_from_primitive_for_uint!(u8);
impl_from_primitive_for_uint!(u16);
impl_from_primitive_for_uint!(u32);
impl_from_primitive_for_uint!(u64);
impl_from_primitive_for_float!(f32);
impl_from_primitive_for_float!(f64);

serde_if_integer128! {
    impl_from_primitive_for_int!(i128);
    impl_from_primitive_for_uint!(u128);
}
