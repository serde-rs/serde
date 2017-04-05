// Copyright 2013-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Extracted from https://github.com/rust-num/num.

// Rust 1.5 is unhappy that this private module is undocumented.
#![allow(missing_docs)]

use core::{usize, u8, u16, u32, u64};
use core::{isize, i8, i16, i32, i64};
use core::{f32, f64};
use core::mem::size_of;

/// Numbers which have upper and lower bounds
pub trait Bounded {
    // FIXME (#5527): These should be associated constants
    /// returns the smallest finite number this type can represent
    fn min_value() -> Self;
    /// returns the largest finite number this type can represent
    fn max_value() -> Self;
}

macro_rules! bounded_impl {
    ($t:ty, $min:expr, $max:expr) => {
        impl Bounded for $t {
            #[inline]
            fn min_value() -> $t { $min }

            #[inline]
            fn max_value() -> $t { $max }
        }
    }
}

bounded_impl!(usize, usize::MIN, usize::MAX);
bounded_impl!(u8, u8::MIN, u8::MAX);
bounded_impl!(u16, u16::MIN, u16::MAX);
bounded_impl!(u32, u32::MIN, u32::MAX);
bounded_impl!(u64, u64::MIN, u64::MAX);

bounded_impl!(isize, isize::MIN, isize::MAX);
bounded_impl!(i8, i8::MIN, i8::MAX);
bounded_impl!(i16, i16::MIN, i16::MAX);
bounded_impl!(i32, i32::MIN, i32::MAX);
bounded_impl!(i64, i64::MIN, i64::MAX);

bounded_impl!(f32, f32::MIN, f32::MAX);
bounded_impl!(f64, f64::MIN, f64::MAX);

/// A generic trait for converting a value to a number.
pub trait ToPrimitive {
    /// Converts the value of `self` to an `isize`.
    fn to_isize(&self) -> Option<isize>;

    /// Converts the value of `self` to an `i8`.
    fn to_i8(&self) -> Option<i8>;

    /// Converts the value of `self` to an `i16`.
    fn to_i16(&self) -> Option<i16>;

    /// Converts the value of `self` to an `i32`.
    fn to_i32(&self) -> Option<i32>;

    /// Converts the value of `self` to an `i64`.
    fn to_i64(&self) -> Option<i64>;

    /// Converts the value of `self` to a `usize`.
    fn to_usize(&self) -> Option<usize>;

    /// Converts the value of `self` to an `u8`.
    fn to_u8(&self) -> Option<u8>;

    /// Converts the value of `self` to an `u16`.
    fn to_u16(&self) -> Option<u16>;

    /// Converts the value of `self` to an `u32`.
    fn to_u32(&self) -> Option<u32>;

    /// Converts the value of `self` to an `u64`.
    fn to_u64(&self) -> Option<u64>;

    /// Converts the value of `self` to an `f32`.
    fn to_f32(&self) -> Option<f32>;

    /// Converts the value of `self` to an `f64`.
    fn to_f64(&self) -> Option<f64>;
}

macro_rules! impl_to_primitive_int_to_int {
    ($SrcT:ty, $DstT:ty, $slf:expr) => (
        {
            if size_of::<$SrcT>() <= size_of::<$DstT>() {
                Some($slf as $DstT)
            } else {
                let n = $slf as i64;
                let min_value: $DstT = Bounded::min_value();
                let max_value: $DstT = Bounded::max_value();
                if min_value as i64 <= n && n <= max_value as i64 {
                    Some($slf as $DstT)
                } else {
                    None
                }
            }
        }
    )
}

macro_rules! impl_to_primitive_int_to_uint {
    ($SrcT:ty, $DstT:ty, $slf:expr) => (
        {
            let zero: $SrcT = 0;
            let max_value: $DstT = Bounded::max_value();
            if zero <= $slf && $slf as u64 <= max_value as u64 {
                Some($slf as $DstT)
            } else {
                None
            }
        }
    )
}

macro_rules! impl_to_primitive_int {
    ($T:ty) => (
        impl ToPrimitive for $T {
            #[inline]
            fn to_isize(&self) -> Option<isize> { impl_to_primitive_int_to_int!($T, isize, *self) }
            #[inline]
            fn to_i8(&self) -> Option<i8> { impl_to_primitive_int_to_int!($T, i8, *self) }
            #[inline]
            fn to_i16(&self) -> Option<i16> { impl_to_primitive_int_to_int!($T, i16, *self) }
            #[inline]
            fn to_i32(&self) -> Option<i32> { impl_to_primitive_int_to_int!($T, i32, *self) }
            #[inline]
            fn to_i64(&self) -> Option<i64> { impl_to_primitive_int_to_int!($T, i64, *self) }

            #[inline]
            fn to_usize(&self) -> Option<usize> { impl_to_primitive_int_to_uint!($T, usize, *self) }
            #[inline]
            fn to_u8(&self) -> Option<u8> { impl_to_primitive_int_to_uint!($T, u8, *self) }
            #[inline]
            fn to_u16(&self) -> Option<u16> { impl_to_primitive_int_to_uint!($T, u16, *self) }
            #[inline]
            fn to_u32(&self) -> Option<u32> { impl_to_primitive_int_to_uint!($T, u32, *self) }
            #[inline]
            fn to_u64(&self) -> Option<u64> { impl_to_primitive_int_to_uint!($T, u64, *self) }

            #[inline]
            fn to_f32(&self) -> Option<f32> { Some(*self as f32) }
            #[inline]
            fn to_f64(&self) -> Option<f64> { Some(*self as f64) }
        }
    )
}

impl_to_primitive_int! { isize }
impl_to_primitive_int! { i8 }
impl_to_primitive_int! { i16 }
impl_to_primitive_int! { i32 }
impl_to_primitive_int! { i64 }

macro_rules! impl_to_primitive_uint_to_int {
    ($DstT:ty, $slf:expr) => (
        {
            let max_value: $DstT = Bounded::max_value();
            if $slf as u64 <= max_value as u64 {
                Some($slf as $DstT)
            } else {
                None
            }
        }
    )
}

macro_rules! impl_to_primitive_uint_to_uint {
    ($SrcT:ty, $DstT:ty, $slf:expr) => (
        {
            if size_of::<$SrcT>() <= size_of::<$DstT>() {
                Some($slf as $DstT)
            } else {
                let zero: $SrcT = 0;
                let max_value: $DstT = Bounded::max_value();
                if zero <= $slf && $slf as u64 <= max_value as u64 {
                    Some($slf as $DstT)
                } else {
                    None
                }
            }
        }
    )
}

macro_rules! impl_to_primitive_uint {
    ($T:ty) => (
        impl ToPrimitive for $T {
            #[inline]
            fn to_isize(&self) -> Option<isize> { impl_to_primitive_uint_to_int!(isize, *self) }
            #[inline]
            fn to_i8(&self) -> Option<i8> { impl_to_primitive_uint_to_int!(i8, *self) }
            #[inline]
            fn to_i16(&self) -> Option<i16> { impl_to_primitive_uint_to_int!(i16, *self) }
            #[inline]
            fn to_i32(&self) -> Option<i32> { impl_to_primitive_uint_to_int!(i32, *self) }
            #[inline]
            fn to_i64(&self) -> Option<i64> { impl_to_primitive_uint_to_int!(i64, *self) }

            #[inline]
            fn to_usize(&self) -> Option<usize> {
                impl_to_primitive_uint_to_uint!($T, usize, *self)
            }
            #[inline]
            fn to_u8(&self) -> Option<u8> { impl_to_primitive_uint_to_uint!($T, u8, *self) }
            #[inline]
            fn to_u16(&self) -> Option<u16> { impl_to_primitive_uint_to_uint!($T, u16, *self) }
            #[inline]
            fn to_u32(&self) -> Option<u32> { impl_to_primitive_uint_to_uint!($T, u32, *self) }
            #[inline]
            fn to_u64(&self) -> Option<u64> { impl_to_primitive_uint_to_uint!($T, u64, *self) }

            #[inline]
            fn to_f32(&self) -> Option<f32> { Some(*self as f32) }
            #[inline]
            fn to_f64(&self) -> Option<f64> { Some(*self as f64) }
        }
    )
}

impl_to_primitive_uint! { usize }
impl_to_primitive_uint! { u8 }
impl_to_primitive_uint! { u16 }
impl_to_primitive_uint! { u32 }
impl_to_primitive_uint! { u64 }

pub trait FromPrimitive: Sized {
    fn from_isize(n: isize) -> Option<Self>;
    fn from_i8(n: i8) -> Option<Self>;
    fn from_i16(n: i16) -> Option<Self>;
    fn from_i32(n: i32) -> Option<Self>;
    fn from_i64(n: i64) -> Option<Self>;
    fn from_usize(n: usize) -> Option<Self>;
    fn from_u8(n: u8) -> Option<Self>;
    fn from_u16(n: u16) -> Option<Self>;
    fn from_u32(n: u32) -> Option<Self>;
    fn from_u64(n: u64) -> Option<Self>;
}

macro_rules! impl_from_primitive {
    ($T:ty, $to_ty:ident) => (
        impl FromPrimitive for $T {
            #[inline] fn from_isize(n: isize) -> Option<$T> { n.$to_ty() }
            #[inline] fn from_i8(n: i8) -> Option<$T> { n.$to_ty() }
            #[inline] fn from_i16(n: i16) -> Option<$T> { n.$to_ty() }
            #[inline] fn from_i32(n: i32) -> Option<$T> { n.$to_ty() }
            #[inline] fn from_i64(n: i64) -> Option<$T> { n.$to_ty() }
            #[inline] fn from_usize(n: usize) -> Option<$T> { n.$to_ty() }
            #[inline] fn from_u8(n: u8) -> Option<$T> { n.$to_ty() }
            #[inline] fn from_u16(n: u16) -> Option<$T> { n.$to_ty() }
            #[inline] fn from_u32(n: u32) -> Option<$T> { n.$to_ty() }
            #[inline] fn from_u64(n: u64) -> Option<$T> { n.$to_ty() }
        }
    )
}

impl_from_primitive! { isize, to_isize }
impl_from_primitive! { i8, to_i8 }
impl_from_primitive! { i16, to_i16 }
impl_from_primitive! { i32, to_i32 }
impl_from_primitive! { i64, to_i64 }
impl_from_primitive! { usize, to_usize }
impl_from_primitive! { u8, to_u8 }
impl_from_primitive! { u16, to_u16 }
impl_from_primitive! { u32, to_u32 }
impl_from_primitive! { u64, to_u64 }
impl_from_primitive! { f32, to_f32 }
impl_from_primitive! { f64, to_f64 }
