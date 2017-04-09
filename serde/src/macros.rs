#[doc(hidden)]
#[macro_export]
macro_rules! forward_to_deserialize_method {
    ($func:ident<$l:tt, $v:ident>($($arg:ident : $ty:ty),*)) => {
        #[inline]
        fn $func<$v>(self, $($arg: $ty,)* visitor: $v) -> $crate::export::Result<$v::Value, Self::Error>
            where $v: $crate::de::Visitor<$l>
        {
            $(
                let _ = $arg;
            )*
            self.deserialize(visitor)
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! forward_to_deserialize_helper {
    (bool<$l:tt, $v:ident>) => {
        forward_to_deserialize_method!{deserialize_bool<$l, $v>()}
    };
    (u8<$l:tt, $v:ident>) => {
        forward_to_deserialize_method!{deserialize_u8<$l, $v>()}
    };
    (u16<$l:tt, $v:ident>) => {
        forward_to_deserialize_method!{deserialize_u16<$l, $v>()}
    };
    (u32<$l:tt, $v:ident>) => {
        forward_to_deserialize_method!{deserialize_u32<$l, $v>()}
    };
    (u64<$l:tt, $v:ident>) => {
        forward_to_deserialize_method!{deserialize_u64<$l, $v>()}
    };
    (i8<$l:tt, $v:ident>) => {
        forward_to_deserialize_method!{deserialize_i8<$l, $v>()}
    };
    (i16<$l:tt, $v:ident>) => {
        forward_to_deserialize_method!{deserialize_i16<$l, $v>()}
    };
    (i32<$l:tt, $v:ident>) => {
        forward_to_deserialize_method!{deserialize_i32<$l, $v>()}
    };
    (i64<$l:tt, $v:ident>) => {
        forward_to_deserialize_method!{deserialize_i64<$l, $v>()}
    };
    (f32<$l:tt, $v:ident>) => {
        forward_to_deserialize_method!{deserialize_f32<$l, $v>()}
    };
    (f64<$l:tt, $v:ident>) => {
        forward_to_deserialize_method!{deserialize_f64<$l, $v>()}
    };
    (char<$l:tt, $v:ident>) => {
        forward_to_deserialize_method!{deserialize_char<$l, $v>()}
    };
    (str<$l:tt, $v:ident>) => {
        forward_to_deserialize_method!{deserialize_str<$l, $v>()}
    };
    (string<$l:tt, $v:ident>) => {
        forward_to_deserialize_method!{deserialize_string<$l, $v>()}
    };
    (unit<$l:tt, $v:ident>) => {
        forward_to_deserialize_method!{deserialize_unit<$l, $v>()}
    };
    (option<$l:tt, $v:ident>) => {
        forward_to_deserialize_method!{deserialize_option<$l, $v>()}
    };
    (seq<$l:tt, $v:ident>) => {
        forward_to_deserialize_method!{deserialize_seq<$l, $v>()}
    };
    (seq_fixed_size<$l:tt, $v:ident>) => {
        forward_to_deserialize_method!{deserialize_seq_fixed_size<$l, $v>(len: usize)}
    };
    (bytes<$l:tt, $v:ident>) => {
        forward_to_deserialize_method!{deserialize_bytes<$l, $v>()}
    };
    (byte_buf<$l:tt, $v:ident>) => {
        forward_to_deserialize_method!{deserialize_byte_buf<$l, $v>()}
    };
    (map<$l:tt, $v:ident>) => {
        forward_to_deserialize_method!{deserialize_map<$l, $v>()}
    };
    (unit_struct<$l:tt, $v:ident>) => {
        forward_to_deserialize_method!{deserialize_unit_struct<$l, $v>(name: &'static str)}
    };
    (newtype_struct<$l:tt, $v:ident>) => {
        forward_to_deserialize_method!{deserialize_newtype_struct<$l, $v>(name: &'static str)}
    };
    (tuple_struct<$l:tt, $v:ident>) => {
        forward_to_deserialize_method!{deserialize_tuple_struct<$l, $v>(name: &'static str, len: usize)}
    };
    (struct<$l:tt, $v:ident>) => {
        forward_to_deserialize_method!{deserialize_struct<$l, $v>(name: &'static str, fields: &'static [&'static str])}
    };
    (struct_field<$l:tt, $v:ident>) => {
        forward_to_deserialize_method!{deserialize_struct_field<$l, $v>()}
    };
    (tuple<$l:tt, $v:ident>) => {
        forward_to_deserialize_method!{deserialize_tuple<$l, $v>(len: usize)}
    };
    (enum<$l:tt, $v:ident>) => {
        forward_to_deserialize_method!{deserialize_enum<$l, $v>(name: &'static str, variants: &'static [&'static str])}
    };
    (ignored_any<$l:tt, $v:ident>) => {
        forward_to_deserialize_method!{deserialize_ignored_any<$l, $v>()}
    };
}

// Super explicit first paragraph because this shows up at the top level and
// trips up people who are just looking for basic Serialize / Deserialize
// documentation.
//
/// Helper macro when implementing the `Deserializer` part of a new data format
/// for Serde.
///
/// Some `Deserializer` implementations for self-describing formats do not care
/// what hint the `Visitor` gives them, they just want to blindly call the
/// `Visitor` method corresponding to the data they can tell is in the input.
/// This requires repetitive implementations of all the `Deserializer` trait
/// methods.
///
/// ```rust
/// # #[macro_use]
/// # extern crate serde;
/// #
/// # use serde::de::{value, Deserializer, Visitor};
/// #
/// # struct MyDeserializer;
/// #
/// # impl<'de> Deserializer<'de> for MyDeserializer {
/// #     type Error = value::Error;
/// #
/// #     fn deserialize<V>(self, _: V) -> Result<V::Value, Self::Error>
/// #         where V: Visitor<'de>
/// #     {
/// #         unimplemented!()
/// #     }
/// #
/// #[inline]
/// fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
///     where V: Visitor<'de>
/// {
///     self.deserialize(visitor)
/// }
/// #
/// #     forward_to_deserialize! {
/// #         u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
/// #         seq seq_fixed_size bytes byte_buf map unit_struct newtype_struct
/// #         tuple_struct struct struct_field tuple enum ignored_any
/// #     }
/// # }
/// #
/// # fn main() {}
/// ```
///
/// The `forward_to_deserialize!` macro implements these simple forwarding
/// methods so that they forward directly to `Deserializer::deserialize`. You
/// can choose which methods to forward.
///
/// ```rust
/// # #[macro_use]
/// # extern crate serde;
/// #
/// # use serde::de::{value, Deserializer, Visitor};
/// #
/// # struct MyDeserializer;
/// #
/// impl<'de> Deserializer<'de> for MyDeserializer {
/// #   type Error = value::Error;
/// #
///     fn deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
///         where V: Visitor<'de>
///     {
///         /* ... */
/// #       let _ = visitor;
/// #       unimplemented!()
///     }
///
///     forward_to_deserialize! {
///         bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
///         seq seq_fixed_size bytes byte_buf map unit_struct newtype_struct
///         tuple_struct struct struct_field tuple enum ignored_any
///     }
/// }
/// #
/// # fn main() {}
/// ```
///
/// The macro assumes the convention that your `Deserializer` lifetime parameter
/// is called `'de` and that the `Visitor` type parameters on each method are
/// called `V`. A different type parameter and a different lifetime can be
/// specified explicitly if necessary.
///
/// ```rust
/// # #[macro_use]
/// # extern crate serde;
/// #
/// # use std::marker::PhantomData;
/// #
/// # use serde::de::{value, Deserializer, Visitor};
/// #
/// # struct MyDeserializer<V>(PhantomData<V>);
/// #
/// # impl<'q, V> Deserializer<'q> for MyDeserializer<V> {
/// #     type Error = value::Error;
/// #
/// #     fn deserialize<W>(self, visitor: W) -> Result<W::Value, Self::Error>
/// #         where W: Visitor<'q>
/// #     {
/// #         unimplemented!()
/// #     }
/// #
/// forward_to_deserialize! {
///     <W: Visitor<'q>>
///     bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
///     seq seq_fixed_size bytes byte_buf map unit_struct newtype_struct
///     tuple_struct struct struct_field tuple enum ignored_any
/// }
/// # }
/// #
/// # fn main() {}
/// ```
#[macro_export]
macro_rules! forward_to_deserialize {
    (<$visitor:ident: Visitor<$lifetime:tt>> $($func:ident)*) => {
        $(forward_to_deserialize_helper!{$func<$lifetime, $visitor>})*
    };
    ($($func:ident)*) => {
        $(forward_to_deserialize_helper!{$func<'de, V>})*
    };
}
