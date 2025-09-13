//! Provides helpers for creating size hints for container deserialization.
#[cfg(any(feature = "std", feature = "alloc"))]
use crate::lib::*;

/// Extracts the exact size of an iterator if it has a known upper bound and it matches the lower bound.
pub fn from_bounds<I>(iter: &I) -> Option<usize>
where
    I: Iterator,
{
    helper(iter.size_hint())
}

/// Returns conservative size estimate for a container, clamping the result to a maximum size.
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn cautious<Element>(hint: Option<usize>) -> usize {
    const MAX_PREALLOC_BYTES: usize = 1024 * 1024;

    if mem::size_of::<Element>() == 0 {
        0
    } else {
        cmp::min(
            hint.unwrap_or(0),
            MAX_PREALLOC_BYTES / mem::size_of::<Element>(),
        )
    }
}

fn helper(bounds: (usize, Option<usize>)) -> Option<usize> {
    match bounds {
        (lower, Some(upper)) if lower == upper => Some(upper),
        _ => None,
    }
}
