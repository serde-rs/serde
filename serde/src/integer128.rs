/// TODO: rustdoc
#[cfg(integer128)]
#[macro_export]
macro_rules! serde_if_integer128 {
    ($($tt:tt)*) => {
        $($tt)*
    };
}

#[cfg(not(integer128))]
#[macro_export]
#[doc(hidden)]
macro_rules! serde_if_integer128 {
    ($($tt:tt)*) => {};
}
