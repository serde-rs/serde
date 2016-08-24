/// Helper to forward `Deserializer` methods to `Deserializer::deserialize`.
/// Every given method ignores all arguments and forwards to `deserialize`.
/// Note that `deserialize_enum` simply returns an `Error::invalid_type`; a
/// better approach is tracked in [serde-rs/serde#521][1].
///
/// ```rust,ignore
/// impl Deserializer for MyDeserializer {
///     fn deserialize<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
///         where V: Visitor
///     {
///         /* ... */
///     }
///
///     forward_to_deserialize! {
///         bool usize u8 u16 u32 u64 isize i8 i16 i32 i64 f32 f64 char str string
///         unit option seq seq_fixed_size bytes map unit_struct newtype_struct
///         tuple_struct struct struct_field tuple enum ignored_any
///     }
/// }
/// ```
///
/// [1]: https://github.com/serde-rs/serde/issues/521
#[macro_export]
macro_rules! forward_to_deserialize {
    (@func bool) => {
        forward_to_deserialize!{@forward deserialize_bool()}
    };
    (@func usize) => {
        forward_to_deserialize!{@forward deserialize_usize()}
    };
    (@func u8) => {
        forward_to_deserialize!{@forward deserialize_u8()}
    };
    (@func u16) => {
        forward_to_deserialize!{@forward deserialize_u16()}
    };
    (@func u32) => {
        forward_to_deserialize!{@forward deserialize_u32()}
    };
    (@func u64) => {
        forward_to_deserialize!{@forward deserialize_u64()}
    };
    (@func isize) => {
        forward_to_deserialize!{@forward deserialize_isize()}
    };
    (@func i8) => {
        forward_to_deserialize!{@forward deserialize_i8()}
    };
    (@func i16) => {
        forward_to_deserialize!{@forward deserialize_i16()}
    };
    (@func i32) => {
        forward_to_deserialize!{@forward deserialize_i32()}
    };
    (@func i64) => {
        forward_to_deserialize!{@forward deserialize_i64()}
    };
    (@func f32) => {
        forward_to_deserialize!{@forward deserialize_f32()}
    };
    (@func f64) => {
        forward_to_deserialize!{@forward deserialize_f64()}
    };
    (@func char) => {
        forward_to_deserialize!{@forward deserialize_char()}
    };
    (@func str) => {
        forward_to_deserialize!{@forward deserialize_str()}
    };
    (@func string) => {
        forward_to_deserialize!{@forward deserialize_string()}
    };
    (@func unit) => {
        forward_to_deserialize!{@forward deserialize_unit()}
    };
    (@func option) => {
        forward_to_deserialize!{@forward deserialize_option()}
    };
    (@func seq) => {
        forward_to_deserialize!{@forward deserialize_seq()}
    };
    (@func seq_fixed_size) => {
        forward_to_deserialize!{@forward deserialize_seq_fixed_size(usize)}
    };
    (@func bytes) => {
        forward_to_deserialize!{@forward deserialize_bytes()}
    };
    (@func map) => {
        forward_to_deserialize!{@forward deserialize_map()}
    };
    (@func unit_struct) => {
        forward_to_deserialize!{@forward deserialize_unit_struct(&'static str)}
    };
    (@func newtype_struct) => {
        forward_to_deserialize!{@forward deserialize_newtype_struct(&'static str)}
    };
    (@func tuple_struct) => {
        forward_to_deserialize!{@forward deserialize_tuple_struct(&'static str, usize)}
    };
    (@func struct) => {
        forward_to_deserialize!{@forward deserialize_struct(&'static str, &'static [&'static str])}
    };
    (@func struct_field) => {
        forward_to_deserialize!{@forward deserialize_struct_field()}
    };
    (@func tuple) => {
        forward_to_deserialize!{@forward deserialize_tuple(usize)}
    };
    (@func enum) => {
        #[inline]
        fn deserialize_enum<__V>(&mut self, _: &str, _: &[&str], _: __V) -> ::std::result::Result<__V::Value, Self::Error>
            where __V: $crate::de::EnumVisitor
        {
            Err($crate::de::Error::invalid_type($crate::de::Type::Enum))
        }
    };
    (@func ignored_any) => {
        forward_to_deserialize!{@forward deserialize_ignored_any()}
    };
    (@forward $func:ident($($arg:ty),*)) => {
        #[inline]
        fn $func<__V>(&mut self, $(_: $arg,)* visitor: __V) -> ::std::result::Result<__V::Value, Self::Error>
            where __V: $crate::de::Visitor
        {
            self.deserialize(visitor)
        }
    };

    ($($func:ident)*) => {
        $(forward_to_deserialize!{@func $func})*
    };
}
