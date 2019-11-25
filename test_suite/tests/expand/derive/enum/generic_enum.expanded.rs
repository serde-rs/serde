use serde::{Deserialize, Serialize};
pub enum GenericEnum<T, U> {
    Unit,
    NewType(T),
    Seq(T, U),
    Map { x: T, y: U },
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_SERIALIZE_FOR_GenericEnum: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<T, U> _serde::Serialize for GenericEnum<T, U>
    where
        T: _serde::Serialize,
        U: _serde::Serialize,
    {
        fn serialize<__S>(&self, __serializer: __S) -> _serde::export::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            match *self {
                GenericEnum::Unit => _serde::Serializer::serialize_unit_variant(
                    __serializer,
                    "GenericEnum",
                    0u32,
                    "Unit",
                ),
                GenericEnum::NewType(ref __field0) => {
                    _serde::Serializer::serialize_newtype_variant(
                        __serializer,
                        "GenericEnum",
                        1u32,
                        "NewType",
                        __field0,
                    )
                }
                GenericEnum::Seq(ref __field0, ref __field1) => {
                    let mut __serde_state = match _serde::Serializer::serialize_tuple_variant(
                        __serializer,
                        "GenericEnum",
                        2u32,
                        "Seq",
                        0 + 1 + 1,
                    ) {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeTupleVariant::serialize_field(
                        &mut __serde_state,
                        __field0,
                    ) {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeTupleVariant::serialize_field(
                        &mut __serde_state,
                        __field1,
                    ) {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    };
                    _serde::ser::SerializeTupleVariant::end(__serde_state)
                }
                GenericEnum::Map { ref x, ref y } => {
                    let mut __serde_state = match _serde::Serializer::serialize_struct_variant(
                        __serializer,
                        "GenericEnum",
                        3u32,
                        "Map",
                        0 + 1 + 1,
                    ) {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStructVariant::serialize_field(
                        &mut __serde_state,
                        "x",
                        x,
                    ) {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStructVariant::serialize_field(
                        &mut __serde_state,
                        "y",
                        y,
                    ) {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    };
                    _serde::ser::SerializeStructVariant::end(__serde_state)
                }
            }
        }
    }
};
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DESERIALIZE_FOR_GenericEnum: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de, T, U> _serde::Deserialize<'de> for GenericEnum<T, U>
    where
        T: _serde::Deserialize<'de>,
        U: _serde::Deserialize<'de>,
    {
        fn deserialize<__D>(__deserializer: __D) -> _serde::export::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            enum __Field {
                __field0,
                __field1,
                __field2,
                __field3,
            }
            struct __FieldVisitor;
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::export::Formatter,
                ) -> _serde::export::fmt::Result {
                    _serde::export::Formatter::write_str(__formatter, "variant identifier")
                }
                fn visit_u64<__E>(self, __value: u64) -> _serde::export::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::export::Ok(__Field::__field0),
                        1u64 => _serde::export::Ok(__Field::__field1),
                        2u64 => _serde::export::Ok(__Field::__field2),
                        3u64 => _serde::export::Ok(__Field::__field3),
                        _ => _serde::export::Err(_serde::de::Error::invalid_value(
                            _serde::de::Unexpected::Unsigned(__value),
                            &"variant index 0 <= i < 4",
                        )),
                    }
                }
                fn visit_str<__E>(self, __value: &str) -> _serde::export::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "Unit" => _serde::export::Ok(__Field::__field0),
                        "NewType" => _serde::export::Ok(__Field::__field1),
                        "Seq" => _serde::export::Ok(__Field::__field2),
                        "Map" => _serde::export::Ok(__Field::__field3),
                        _ => _serde::export::Err(_serde::de::Error::unknown_variant(
                            __value, VARIANTS,
                        )),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::export::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"Unit" => _serde::export::Ok(__Field::__field0),
                        b"NewType" => _serde::export::Ok(__Field::__field1),
                        b"Seq" => _serde::export::Ok(__Field::__field2),
                        b"Map" => _serde::export::Ok(__Field::__field3),
                        _ => {
                            let __value = &_serde::export::from_utf8_lossy(__value);
                            _serde::export::Err(_serde::de::Error::unknown_variant(
                                __value, VARIANTS,
                            ))
                        }
                    }
                }
            }
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(__deserializer: __D) -> _serde::export::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                }
            }
            struct __Visitor<'de, T, U>
            where
                T: _serde::Deserialize<'de>,
                U: _serde::Deserialize<'de>,
            {
                marker: _serde::export::PhantomData<GenericEnum<T, U>>,
                lifetime: _serde::export::PhantomData<&'de ()>,
            }
            impl<'de, T, U> _serde::de::Visitor<'de> for __Visitor<'de, T, U>
            where
                T: _serde::Deserialize<'de>,
                U: _serde::Deserialize<'de>,
            {
                type Value = GenericEnum<T, U>;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::export::Formatter,
                ) -> _serde::export::fmt::Result {
                    _serde::export::Formatter::write_str(__formatter, "enum GenericEnum")
                }
                fn visit_enum<__A>(
                    self,
                    __data: __A,
                ) -> _serde::export::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::EnumAccess<'de>,
                {
                    match match _serde::de::EnumAccess::variant(__data) {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    } {
                        (__Field::__field0, __variant) => {
                            match _serde::de::VariantAccess::unit_variant(__variant) {
                                _serde::export::Ok(__val) => __val,
                                _serde::export::Err(__err) => {
                                    return _serde::export::Err(__err);
                                }
                            };
                            _serde::export::Ok(GenericEnum::Unit)
                        }
                        (__Field::__field1, __variant) => _serde::export::Result::map(
                            _serde::de::VariantAccess::newtype_variant::<T>(__variant),
                            GenericEnum::NewType,
                        ),
                        (__Field::__field2, __variant) => {
                            struct __Visitor<'de, T, U>
                            where
                                T: _serde::Deserialize<'de>,
                                U: _serde::Deserialize<'de>,
                            {
                                marker: _serde::export::PhantomData<GenericEnum<T, U>>,
                                lifetime: _serde::export::PhantomData<&'de ()>,
                            }
                            impl<'de, T, U> _serde::de::Visitor<'de> for __Visitor<'de, T, U>
                            where
                                T: _serde::Deserialize<'de>,
                                U: _serde::Deserialize<'de>,
                            {
                                type Value = GenericEnum<T, U>;
                                fn expecting(
                                    &self,
                                    __formatter: &mut _serde::export::Formatter,
                                ) -> _serde::export::fmt::Result {
                                    _serde::export::Formatter::write_str(
                                        __formatter,
                                        "tuple variant GenericEnum::Seq",
                                    )
                                }
                                #[inline]
                                fn visit_seq<__A>(
                                    self,
                                    mut __seq: __A,
                                ) -> _serde::export::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::SeqAccess<'de>,
                                {
                                    let __field0 = match match _serde::de::SeqAccess::next_element::<
                                        T,
                                    >(
                                        &mut __seq
                                    ) {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    } {
                                        _serde::export::Some(__value) => __value,
                                        _serde::export::None => {
                                            return _serde :: export :: Err ( _serde :: de :: Error :: invalid_length ( 0usize , & "tuple variant GenericEnum::Seq with 2 elements" ) ) ;
                                        }
                                    };
                                    let __field1 = match match _serde::de::SeqAccess::next_element::<
                                        U,
                                    >(
                                        &mut __seq
                                    ) {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    } {
                                        _serde::export::Some(__value) => __value,
                                        _serde::export::None => {
                                            return _serde :: export :: Err ( _serde :: de :: Error :: invalid_length ( 1usize , & "tuple variant GenericEnum::Seq with 2 elements" ) ) ;
                                        }
                                    };
                                    _serde::export::Ok(GenericEnum::Seq(__field0, __field1))
                                }
                            }
                            _serde::de::VariantAccess::tuple_variant(
                                __variant,
                                2usize,
                                __Visitor {
                                    marker: _serde::export::PhantomData::<GenericEnum<T, U>>,
                                    lifetime: _serde::export::PhantomData,
                                },
                            )
                        }
                        (__Field::__field3, __variant) => {
                            #[allow(non_camel_case_types)]
                            enum __Field {
                                __field0,
                                __field1,
                                __ignore,
                            }
                            struct __FieldVisitor;
                            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                                type Value = __Field;
                                fn expecting(
                                    &self,
                                    __formatter: &mut _serde::export::Formatter,
                                ) -> _serde::export::fmt::Result {
                                    _serde::export::Formatter::write_str(
                                        __formatter,
                                        "field identifier",
                                    )
                                }
                                fn visit_u64<__E>(
                                    self,
                                    __value: u64,
                                ) -> _serde::export::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    match __value {
                                        0u64 => _serde::export::Ok(__Field::__field0),
                                        1u64 => _serde::export::Ok(__Field::__field1),
                                        _ => _serde::export::Err(_serde::de::Error::invalid_value(
                                            _serde::de::Unexpected::Unsigned(__value),
                                            &"field index 0 <= i < 2",
                                        )),
                                    }
                                }
                                fn visit_str<__E>(
                                    self,
                                    __value: &str,
                                ) -> _serde::export::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    match __value {
                                        "x" => _serde::export::Ok(__Field::__field0),
                                        "y" => _serde::export::Ok(__Field::__field1),
                                        _ => _serde::export::Ok(__Field::__ignore),
                                    }
                                }
                                fn visit_bytes<__E>(
                                    self,
                                    __value: &[u8],
                                ) -> _serde::export::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    match __value {
                                        b"x" => _serde::export::Ok(__Field::__field0),
                                        b"y" => _serde::export::Ok(__Field::__field1),
                                        _ => _serde::export::Ok(__Field::__ignore),
                                    }
                                }
                            }
                            impl<'de> _serde::Deserialize<'de> for __Field {
                                #[inline]
                                fn deserialize<__D>(
                                    __deserializer: __D,
                                ) -> _serde::export::Result<Self, __D::Error>
                                where
                                    __D: _serde::Deserializer<'de>,
                                {
                                    _serde::Deserializer::deserialize_identifier(
                                        __deserializer,
                                        __FieldVisitor,
                                    )
                                }
                            }
                            struct __Visitor<'de, T, U>
                            where
                                T: _serde::Deserialize<'de>,
                                U: _serde::Deserialize<'de>,
                            {
                                marker: _serde::export::PhantomData<GenericEnum<T, U>>,
                                lifetime: _serde::export::PhantomData<&'de ()>,
                            }
                            impl<'de, T, U> _serde::de::Visitor<'de> for __Visitor<'de, T, U>
                            where
                                T: _serde::Deserialize<'de>,
                                U: _serde::Deserialize<'de>,
                            {
                                type Value = GenericEnum<T, U>;
                                fn expecting(
                                    &self,
                                    __formatter: &mut _serde::export::Formatter,
                                ) -> _serde::export::fmt::Result {
                                    _serde::export::Formatter::write_str(
                                        __formatter,
                                        "struct variant GenericEnum::Map",
                                    )
                                }
                                #[inline]
                                fn visit_seq<__A>(
                                    self,
                                    mut __seq: __A,
                                ) -> _serde::export::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::SeqAccess<'de>,
                                {
                                    let __field0 = match match _serde::de::SeqAccess::next_element::<
                                        T,
                                    >(
                                        &mut __seq
                                    ) {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    } {
                                        _serde::export::Some(__value) => __value,
                                        _serde::export::None => {
                                            return _serde :: export :: Err ( _serde :: de :: Error :: invalid_length ( 0usize , & "struct variant GenericEnum::Map with 2 elements" ) ) ;
                                        }
                                    };
                                    let __field1 = match match _serde::de::SeqAccess::next_element::<
                                        U,
                                    >(
                                        &mut __seq
                                    ) {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    } {
                                        _serde::export::Some(__value) => __value,
                                        _serde::export::None => {
                                            return _serde :: export :: Err ( _serde :: de :: Error :: invalid_length ( 1usize , & "struct variant GenericEnum::Map with 2 elements" ) ) ;
                                        }
                                    };
                                    _serde::export::Ok(GenericEnum::Map {
                                        x: __field0,
                                        y: __field1,
                                    })
                                }
                                #[inline]
                                fn visit_map<__A>(
                                    self,
                                    mut __map: __A,
                                ) -> _serde::export::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::MapAccess<'de>,
                                {
                                    let mut __field0: _serde::export::Option<T> =
                                        _serde::export::None;
                                    let mut __field1: _serde::export::Option<U> =
                                        _serde::export::None;
                                    while let _serde::export::Some(__key) =
                                        match _serde::de::MapAccess::next_key::<__Field>(&mut __map)
                                        {
                                            _serde::export::Ok(__val) => __val,
                                            _serde::export::Err(__err) => {
                                                return _serde::export::Err(__err);
                                            }
                                        }
                                    {
                                        match __key {
                                            __Field::__field0 => {
                                                if _serde::export::Option::is_some(&__field0) {
                                                    return _serde :: export :: Err ( < __A :: Error as _serde :: de :: Error > :: duplicate_field ( "x" ) ) ;
                                                }
                                                __field0 = _serde::export::Some(
                                                    match _serde::de::MapAccess::next_value::<T>(
                                                        &mut __map,
                                                    ) {
                                                        _serde::export::Ok(__val) => __val,
                                                        _serde::export::Err(__err) => {
                                                            return _serde::export::Err(__err);
                                                        }
                                                    },
                                                );
                                            }
                                            __Field::__field1 => {
                                                if _serde::export::Option::is_some(&__field1) {
                                                    return _serde :: export :: Err ( < __A :: Error as _serde :: de :: Error > :: duplicate_field ( "y" ) ) ;
                                                }
                                                __field1 = _serde::export::Some(
                                                    match _serde::de::MapAccess::next_value::<U>(
                                                        &mut __map,
                                                    ) {
                                                        _serde::export::Ok(__val) => __val,
                                                        _serde::export::Err(__err) => {
                                                            return _serde::export::Err(__err);
                                                        }
                                                    },
                                                );
                                            }
                                            _ => {
                                                let _ = match _serde::de::MapAccess::next_value::<
                                                    _serde::de::IgnoredAny,
                                                >(
                                                    &mut __map
                                                ) {
                                                    _serde::export::Ok(__val) => __val,
                                                    _serde::export::Err(__err) => {
                                                        return _serde::export::Err(__err);
                                                    }
                                                };
                                            }
                                        }
                                    }
                                    let __field0 = match __field0 {
                                        _serde::export::Some(__field0) => __field0,
                                        _serde::export::None => {
                                            match _serde::private::de::missing_field("x") {
                                                _serde::export::Ok(__val) => __val,
                                                _serde::export::Err(__err) => {
                                                    return _serde::export::Err(__err);
                                                }
                                            }
                                        }
                                    };
                                    let __field1 = match __field1 {
                                        _serde::export::Some(__field1) => __field1,
                                        _serde::export::None => {
                                            match _serde::private::de::missing_field("y") {
                                                _serde::export::Ok(__val) => __val,
                                                _serde::export::Err(__err) => {
                                                    return _serde::export::Err(__err);
                                                }
                                            }
                                        }
                                    };
                                    _serde::export::Ok(GenericEnum::Map {
                                        x: __field0,
                                        y: __field1,
                                    })
                                }
                            }
                            const FIELDS: &'static [&'static str] = &["x", "y"];
                            _serde::de::VariantAccess::struct_variant(
                                __variant,
                                FIELDS,
                                __Visitor {
                                    marker: _serde::export::PhantomData::<GenericEnum<T, U>>,
                                    lifetime: _serde::export::PhantomData,
                                },
                            )
                        }
                    }
                }
            }
            const VARIANTS: &'static [&'static str] = &["Unit", "NewType", "Seq", "Map"];
            _serde::Deserializer::deserialize_enum(
                __deserializer,
                "GenericEnum",
                VARIANTS,
                __Visitor {
                    marker: _serde::export::PhantomData::<GenericEnum<T, U>>,
                    lifetime: _serde::export::PhantomData,
                },
            )
        }
    }
};
