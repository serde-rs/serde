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

macro_rules! int_to_int {
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

macro_rules! int_to_uint {
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

macro_rules! uint_to_int {
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

macro_rules! uint_to_uint {
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

macro_rules! impl_from_primitive_for_int {
    ($T:ty) => (
        impl FromPrimitive for $T {
            #[inline] fn from_isize(n: isize) -> Option<Self> { int_to_int!(isize, $T, n) }
            #[inline] fn from_i8(n: i8) -> Option<Self> { int_to_int!(i8, $T, n) }
            #[inline] fn from_i16(n: i16) -> Option<Self> { int_to_int!(i16, $T, n) }
            #[inline] fn from_i32(n: i32) -> Option<Self> { int_to_int!(i32, $T, n) }
            #[inline] fn from_i64(n: i64) -> Option<Self> { int_to_int!(i64, $T, n) }
            #[inline] fn from_usize(n: usize) -> Option<Self> { uint_to_int!($T, n) }
            #[inline] fn from_u8(n: u8) -> Option<Self> { uint_to_int!($T, n) }
            #[inline] fn from_u16(n: u16) -> Option<Self> { uint_to_int!($T, n) }
            #[inline] fn from_u32(n: u32) -> Option<Self> { uint_to_int!($T, n) }
            #[inline] fn from_u64(n: u64) -> Option<Self> { uint_to_int!($T, n) }
        }
    )
}

macro_rules! impl_from_primitive_for_uint {
    ($T:ty) => (
        impl FromPrimitive for $T {
            #[inline] fn from_isize(n: isize) -> Option<Self> { int_to_uint!(isize, $T, n) }
            #[inline] fn from_i8(n: i8) -> Option<Self> { int_to_uint!(i8, $T, n) }
            #[inline] fn from_i16(n: i16) -> Option<Self> { int_to_uint!(i16, $T, n) }
            #[inline] fn from_i32(n: i32) -> Option<Self> { int_to_uint!(i32, $T, n) }
            #[inline] fn from_i64(n: i64) -> Option<Self> { int_to_uint!(i64, $T, n) }
            #[inline] fn from_usize(n: usize) -> Option<Self> { uint_to_uint!(usize, $T, n) }
            #[inline] fn from_u8(n: u8) -> Option<Self> { uint_to_uint!(u8, $T, n) }
            #[inline] fn from_u16(n: u16) -> Option<Self> { uint_to_uint!(u16, $T, n) }
            #[inline] fn from_u32(n: u32) -> Option<Self> { uint_to_uint!(u32, $T, n) }
            #[inline] fn from_u64(n: u64) -> Option<Self> { uint_to_uint!(u64, $T, n) }
        }
    )
}

macro_rules! impl_from_primitive_for_float {
    ($T:ty) => (
        impl FromPrimitive for $T {
            #[inline] fn from_isize(n: isize) -> Option<Self> { Some(n as Self) }
            #[inline] fn from_i8(n: i8) -> Option<Self> { Some(n as Self) }
            #[inline] fn from_i16(n: i16) -> Option<Self> { Some(n as Self) }
            #[inline] fn from_i32(n: i32) -> Option<Self> { Some(n as Self) }
            #[inline] fn from_i64(n: i64) -> Option<Self> { Some(n as Self) }
            #[inline] fn from_usize(n: usize) -> Option<Self> { Some(n as Self) }
            #[inline] fn from_u8(n: u8) -> Option<Self> { Some(n as Self) }
            #[inline] fn from_u16(n: u16) -> Option<Self> { Some(n as Self) }
            #[inline] fn from_u32(n: u32) -> Option<Self> { Some(n as Self) }
            #[inline] fn from_u64(n: u64) -> Option<Self> { Some(n as Self) }
        }
    )
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
