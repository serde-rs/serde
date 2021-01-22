use serde::{Deserialize, Serialize};
enum DeEnum<B, C, D> {
    Unit,
    Seq(i8, B, C, D),
    Map { a: i8, b: B, c: C, d: D },
    _Unit2,
    _Seq2(i8, B, C, D),
    _Map2 { a: i8, b: B, c: C, d: D },
}
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(rust_2018_idioms, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<B, C, D> _serde::Serialize for DeEnum<B, C, D>
    where
        B: _serde::Serialize,
        C: _serde::Serialize,
        D: _serde::Serialize,
    {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            match *self {
                DeEnum::Unit => {
                    _serde::Serializer::serialize_unit_variant(__serializer, "DeEnum", 0u32, "Unit")
                }
                DeEnum::Seq(ref __field0, ref __field1, ref __field2, ref __field3) => {
                    let mut __serde_state = match _serde::Serializer::serialize_tuple_variant(
                        __serializer,
                        "DeEnum",
                        1u32,
                        "Seq",
                        0 + 1 + 1 + 1 + 1,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeTupleVariant::serialize_field(
                        &mut __serde_state,
                        __field0,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeTupleVariant::serialize_field(
                        &mut __serde_state,
                        __field1,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeTupleVariant::serialize_field(
                        &mut __serde_state,
                        __field2,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeTupleVariant::serialize_field(
                        &mut __serde_state,
                        __field3,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    _serde::ser::SerializeTupleVariant::end(__serde_state)
                }
                DeEnum::Map {
                    ref a,
                    ref b,
                    ref c,
                    ref d,
                } => {
                    let mut __serde_state = match _serde::Serializer::serialize_struct_variant(
                        __serializer,
                        "DeEnum",
                        2u32,
                        "Map",
                        0 + 1 + 1 + 1 + 1,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStructVariant::serialize_field(
                        &mut __serde_state,
                        "a",
                        a,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStructVariant::serialize_field(
                        &mut __serde_state,
                        "b",
                        b,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStructVariant::serialize_field(
                        &mut __serde_state,
                        "c",
                        c,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStructVariant::serialize_field(
                        &mut __serde_state,
                        "d",
                        d,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    _serde::ser::SerializeStructVariant::end(__serde_state)
                }
                DeEnum::_Unit2 => _serde::Serializer::serialize_unit_variant(
                    __serializer,
                    "DeEnum",
                    3u32,
                    "_Unit2",
                ),
                DeEnum::_Seq2(ref __field0, ref __field1, ref __field2, ref __field3) => {
                    let mut __serde_state = match _serde::Serializer::serialize_tuple_variant(
                        __serializer,
                        "DeEnum",
                        4u32,
                        "_Seq2",
                        0 + 1 + 1 + 1 + 1,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeTupleVariant::serialize_field(
                        &mut __serde_state,
                        __field0,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeTupleVariant::serialize_field(
                        &mut __serde_state,
                        __field1,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeTupleVariant::serialize_field(
                        &mut __serde_state,
                        __field2,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeTupleVariant::serialize_field(
                        &mut __serde_state,
                        __field3,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    _serde::ser::SerializeTupleVariant::end(__serde_state)
                }
                DeEnum::_Map2 {
                    ref a,
                    ref b,
                    ref c,
                    ref d,
                } => {
                    let mut __serde_state = match _serde::Serializer::serialize_struct_variant(
                        __serializer,
                        "DeEnum",
                        5u32,
                        "_Map2",
                        0 + 1 + 1 + 1 + 1,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStructVariant::serialize_field(
                        &mut __serde_state,
                        "a",
                        a,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStructVariant::serialize_field(
                        &mut __serde_state,
                        "b",
                        b,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStructVariant::serialize_field(
                        &mut __serde_state,
                        "c",
                        c,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStructVariant::serialize_field(
                        &mut __serde_state,
                        "d",
                        d,
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    };
                    _serde::ser::SerializeStructVariant::end(__serde_state)
                }
            }
        }
    }
};
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(rust_2018_idioms, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de, B, C, D> _serde::Deserialize<'de> for DeEnum<B, C, D>
    where
        B: _serde::Deserialize<'de>,
        C: _serde::Deserialize<'de>,
        D: _serde::Deserialize<'de>,
    {
        fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            enum __Field {
                __field0,
                __field1,
                __field2,
                __field3,
                __field4,
                __field5,
            }
            struct __FieldVisitor;
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "variant identifier")
                }
                fn visit_u64<__E>(self, __value: u64) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::__private::Ok(__Field::__field0),
                        1u64 => _serde::__private::Ok(__Field::__field1),
                        2u64 => _serde::__private::Ok(__Field::__field2),
                        3u64 => _serde::__private::Ok(__Field::__field3),
                        4u64 => _serde::__private::Ok(__Field::__field4),
                        5u64 => _serde::__private::Ok(__Field::__field5),
                        _ => _serde::__private::Err(_serde::de::Error::invalid_value(
                            _serde::de::Unexpected::Unsigned(__value),
                            &"variant index 0 <= i < 6",
                        )),
                    }
                }
                fn visit_str<__E>(
                    self,
                    __value: &str,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "Unit" => _serde::__private::Ok(__Field::__field0),
                        "Seq" => _serde::__private::Ok(__Field::__field1),
                        "Map" => _serde::__private::Ok(__Field::__field2),
                        "_Unit2" => _serde::__private::Ok(__Field::__field3),
                        "_Seq2" => _serde::__private::Ok(__Field::__field4),
                        "_Map2" => _serde::__private::Ok(__Field::__field5),
                        _ => _serde::__private::Err(_serde::de::Error::unknown_variant(
                            __value, VARIANTS,
                        )),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"Unit" => _serde::__private::Ok(__Field::__field0),
                        b"Seq" => _serde::__private::Ok(__Field::__field1),
                        b"Map" => _serde::__private::Ok(__Field::__field2),
                        b"_Unit2" => _serde::__private::Ok(__Field::__field3),
                        b"_Seq2" => _serde::__private::Ok(__Field::__field4),
                        b"_Map2" => _serde::__private::Ok(__Field::__field5),
                        _ => {
                            let __value = &_serde::__private::from_utf8_lossy(__value);
                            _serde::__private::Err(_serde::de::Error::unknown_variant(
                                __value, VARIANTS,
                            ))
                        }
                    }
                }
            }
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                }
            }
            struct __Visitor<'de, B, C, D>
            where
                B: _serde::Deserialize<'de>,
                C: _serde::Deserialize<'de>,
                D: _serde::Deserialize<'de>,
            {
                marker: _serde::__private::PhantomData<DeEnum<B, C, D>>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            impl<'de, B, C, D> _serde::de::Visitor<'de> for __Visitor<'de, B, C, D>
            where
                B: _serde::Deserialize<'de>,
                C: _serde::Deserialize<'de>,
                D: _serde::Deserialize<'de>,
            {
                type Value = DeEnum<B, C, D>;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "enum DeEnum")
                }
                fn visit_enum<__A>(
                    self,
                    __data: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::EnumAccess<'de>,
                {
                    match match _serde::de::EnumAccess::variant(__data) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    } {
                        (__Field::__field0, __variant) => {
                            match _serde::de::VariantAccess::unit_variant(__variant) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            };
                            _serde::__private::Ok(DeEnum::Unit)
                        }
                        (__Field::__field1, __variant) => {
                            struct __Visitor<'de, B, C, D>
                            where
                                B: _serde::Deserialize<'de>,
                                C: _serde::Deserialize<'de>,
                                D: _serde::Deserialize<'de>,
                            {
                                marker: _serde::__private::PhantomData<DeEnum<B, C, D>>,
                                lifetime: _serde::__private::PhantomData<&'de ()>,
                            }
                            impl<'de, B, C, D> _serde::de::Visitor<'de> for __Visitor<'de, B, C, D>
                            where
                                B: _serde::Deserialize<'de>,
                                C: _serde::Deserialize<'de>,
                                D: _serde::Deserialize<'de>,
                            {
                                type Value = DeEnum<B, C, D>;
                                fn expecting(
                                    &self,
                                    __formatter: &mut _serde::__private::Formatter,
                                ) -> _serde::__private::fmt::Result
                                {
                                    _serde::__private::Formatter::write_str(
                                        __formatter,
                                        "tuple variant DeEnum::Seq",
                                    )
                                }
                                #[inline]
                                fn visit_seq<__A>(
                                    self,
                                    mut __seq: __A,
                                ) -> _serde::__private::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::SeqAccess<'de>,
                                {
                                    let __field0 = match match _serde::de::SeqAccess::next_element::<
                                        i8,
                                    >(
                                        &mut __seq
                                    ) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    } {
                                        _serde::__private::Some(__value) => __value,
                                        _serde::__private::None => {
                                            return _serde::__private::Err(
                                                _serde::de::Error::invalid_length(
                                                    0usize,
                                                    &"tuple variant DeEnum::Seq with 4 elements",
                                                ),
                                            );
                                        }
                                    };
                                    let __field1 = match match _serde::de::SeqAccess::next_element::<
                                        B,
                                    >(
                                        &mut __seq
                                    ) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    } {
                                        _serde::__private::Some(__value) => __value,
                                        _serde::__private::None => {
                                            return _serde::__private::Err(
                                                _serde::de::Error::invalid_length(
                                                    1usize,
                                                    &"tuple variant DeEnum::Seq with 4 elements",
                                                ),
                                            );
                                        }
                                    };
                                    let __field2 = match match _serde::de::SeqAccess::next_element::<
                                        C,
                                    >(
                                        &mut __seq
                                    ) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    } {
                                        _serde::__private::Some(__value) => __value,
                                        _serde::__private::None => {
                                            return _serde::__private::Err(
                                                _serde::de::Error::invalid_length(
                                                    2usize,
                                                    &"tuple variant DeEnum::Seq with 4 elements",
                                                ),
                                            );
                                        }
                                    };
                                    let __field3 = match match _serde::de::SeqAccess::next_element::<
                                        D,
                                    >(
                                        &mut __seq
                                    ) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    } {
                                        _serde::__private::Some(__value) => __value,
                                        _serde::__private::None => {
                                            return _serde::__private::Err(
                                                _serde::de::Error::invalid_length(
                                                    3usize,
                                                    &"tuple variant DeEnum::Seq with 4 elements",
                                                ),
                                            );
                                        }
                                    };
                                    _serde::__private::Ok(DeEnum::Seq(
                                        __field0, __field1, __field2, __field3,
                                    ))
                                }
                            }
                            _serde::de::VariantAccess::tuple_variant(
                                __variant,
                                4usize,
                                __Visitor {
                                    marker: _serde::__private::PhantomData::<DeEnum<B, C, D>>,
                                    lifetime: _serde::__private::PhantomData,
                                },
                            )
                        }
                        (__Field::__field2, __variant) => {
                            #[allow(non_camel_case_types)]
                            enum __Field {
                                __field0,
                                __field1,
                                __field2,
                                __field3,
                                __ignore,
                            }
                            struct __FieldVisitor;
                            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                                type Value = __Field;
                                fn expecting(
                                    &self,
                                    __formatter: &mut _serde::__private::Formatter,
                                ) -> _serde::__private::fmt::Result
                                {
                                    _serde::__private::Formatter::write_str(
                                        __formatter,
                                        "field identifier",
                                    )
                                }
                                fn visit_u64<__E>(
                                    self,
                                    __value: u64,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    match __value {
                                        0u64 => _serde::__private::Ok(__Field::__field0),
                                        1u64 => _serde::__private::Ok(__Field::__field1),
                                        2u64 => _serde::__private::Ok(__Field::__field2),
                                        3u64 => _serde::__private::Ok(__Field::__field3),
                                        _ => _serde::__private::Err(
                                            _serde::de::Error::invalid_value(
                                                _serde::de::Unexpected::Unsigned(__value),
                                                &"field index 0 <= i < 4",
                                            ),
                                        ),
                                    }
                                }
                                fn visit_str<__E>(
                                    self,
                                    __value: &str,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    match __value {
                                        "a" => _serde::__private::Ok(__Field::__field0),
                                        "b" => _serde::__private::Ok(__Field::__field1),
                                        "c" => _serde::__private::Ok(__Field::__field2),
                                        "d" => _serde::__private::Ok(__Field::__field3),
                                        _ => _serde::__private::Ok(__Field::__ignore),
                                    }
                                }
                                fn visit_bytes<__E>(
                                    self,
                                    __value: &[u8],
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    match __value {
                                        b"a" => _serde::__private::Ok(__Field::__field0),
                                        b"b" => _serde::__private::Ok(__Field::__field1),
                                        b"c" => _serde::__private::Ok(__Field::__field2),
                                        b"d" => _serde::__private::Ok(__Field::__field3),
                                        _ => _serde::__private::Ok(__Field::__ignore),
                                    }
                                }
                            }
                            impl<'de> _serde::Deserialize<'de> for __Field {
                                #[inline]
                                fn deserialize<__D>(
                                    __deserializer: __D,
                                ) -> _serde::__private::Result<Self, __D::Error>
                                where
                                    __D: _serde::Deserializer<'de>,
                                {
                                    _serde::Deserializer::deserialize_identifier(
                                        __deserializer,
                                        __FieldVisitor,
                                    )
                                }
                            }
                            struct __Visitor<'de, B, C, D>
                            where
                                B: _serde::Deserialize<'de>,
                                C: _serde::Deserialize<'de>,
                                D: _serde::Deserialize<'de>,
                            {
                                marker: _serde::__private::PhantomData<DeEnum<B, C, D>>,
                                lifetime: _serde::__private::PhantomData<&'de ()>,
                            }
                            impl<'de, B, C, D> _serde::de::Visitor<'de> for __Visitor<'de, B, C, D>
                            where
                                B: _serde::Deserialize<'de>,
                                C: _serde::Deserialize<'de>,
                                D: _serde::Deserialize<'de>,
                            {
                                type Value = DeEnum<B, C, D>;
                                fn expecting(
                                    &self,
                                    __formatter: &mut _serde::__private::Formatter,
                                ) -> _serde::__private::fmt::Result
                                {
                                    _serde::__private::Formatter::write_str(
                                        __formatter,
                                        "struct variant DeEnum::Map",
                                    )
                                }
                                #[inline]
                                fn visit_seq<__A>(
                                    self,
                                    mut __seq: __A,
                                ) -> _serde::__private::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::SeqAccess<'de>,
                                {
                                    let __field0 = match match _serde::de::SeqAccess::next_element::<
                                        i8,
                                    >(
                                        &mut __seq
                                    ) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    } {
                                        _serde::__private::Some(__value) => __value,
                                        _serde::__private::None => {
                                            return _serde::__private::Err(
                                                _serde::de::Error::invalid_length(
                                                    0usize,
                                                    &"struct variant DeEnum::Map with 4 elements",
                                                ),
                                            );
                                        }
                                    };
                                    let __field1 = match match _serde::de::SeqAccess::next_element::<
                                        B,
                                    >(
                                        &mut __seq
                                    ) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    } {
                                        _serde::__private::Some(__value) => __value,
                                        _serde::__private::None => {
                                            return _serde::__private::Err(
                                                _serde::de::Error::invalid_length(
                                                    1usize,
                                                    &"struct variant DeEnum::Map with 4 elements",
                                                ),
                                            );
                                        }
                                    };
                                    let __field2 = match match _serde::de::SeqAccess::next_element::<
                                        C,
                                    >(
                                        &mut __seq
                                    ) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    } {
                                        _serde::__private::Some(__value) => __value,
                                        _serde::__private::None => {
                                            return _serde::__private::Err(
                                                _serde::de::Error::invalid_length(
                                                    2usize,
                                                    &"struct variant DeEnum::Map with 4 elements",
                                                ),
                                            );
                                        }
                                    };
                                    let __field3 = match match _serde::de::SeqAccess::next_element::<
                                        D,
                                    >(
                                        &mut __seq
                                    ) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    } {
                                        _serde::__private::Some(__value) => __value,
                                        _serde::__private::None => {
                                            return _serde::__private::Err(
                                                _serde::de::Error::invalid_length(
                                                    3usize,
                                                    &"struct variant DeEnum::Map with 4 elements",
                                                ),
                                            );
                                        }
                                    };
                                    _serde::__private::Ok(DeEnum::Map {
                                        a: __field0,
                                        b: __field1,
                                        c: __field2,
                                        d: __field3,
                                    })
                                }
                                #[inline]
                                fn visit_map<__A>(
                                    self,
                                    mut __map: __A,
                                ) -> _serde::__private::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::MapAccess<'de>,
                                {
                                    let mut __field0: _serde::__private::Option<i8> =
                                        _serde::__private::None;
                                    let mut __field1: _serde::__private::Option<B> =
                                        _serde::__private::None;
                                    let mut __field2: _serde::__private::Option<C> =
                                        _serde::__private::None;
                                    let mut __field3: _serde::__private::Option<D> =
                                        _serde::__private::None;
                                    while let _serde::__private::Some(__key) =
                                        match _serde::de::MapAccess::next_key::<__Field>(&mut __map)
                                        {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        }
                                    {
                                        match __key {
                                            __Field::__field0 => {
                                                if _serde::__private::Option::is_some(&__field0) {
                                                    return _serde :: __private :: Err (< __A :: Error as _serde :: de :: Error > :: duplicate_field ("a")) ;
                                                }
                                                __field0 = _serde::__private::Some(
                                                    match _serde::de::MapAccess::next_value::<i8>(
                                                        &mut __map,
                                                    ) {
                                                        _serde::__private::Ok(__val) => __val,
                                                        _serde::__private::Err(__err) => {
                                                            return _serde::__private::Err(__err);
                                                        }
                                                    },
                                                );
                                            }
                                            __Field::__field1 => {
                                                if _serde::__private::Option::is_some(&__field1) {
                                                    return _serde :: __private :: Err (< __A :: Error as _serde :: de :: Error > :: duplicate_field ("b")) ;
                                                }
                                                __field1 = _serde::__private::Some(
                                                    match _serde::de::MapAccess::next_value::<B>(
                                                        &mut __map,
                                                    ) {
                                                        _serde::__private::Ok(__val) => __val,
                                                        _serde::__private::Err(__err) => {
                                                            return _serde::__private::Err(__err);
                                                        }
                                                    },
                                                );
                                            }
                                            __Field::__field2 => {
                                                if _serde::__private::Option::is_some(&__field2) {
                                                    return _serde :: __private :: Err (< __A :: Error as _serde :: de :: Error > :: duplicate_field ("c")) ;
                                                }
                                                __field2 = _serde::__private::Some(
                                                    match _serde::de::MapAccess::next_value::<C>(
                                                        &mut __map,
                                                    ) {
                                                        _serde::__private::Ok(__val) => __val,
                                                        _serde::__private::Err(__err) => {
                                                            return _serde::__private::Err(__err);
                                                        }
                                                    },
                                                );
                                            }
                                            __Field::__field3 => {
                                                if _serde::__private::Option::is_some(&__field3) {
                                                    return _serde :: __private :: Err (< __A :: Error as _serde :: de :: Error > :: duplicate_field ("d")) ;
                                                }
                                                __field3 = _serde::__private::Some(
                                                    match _serde::de::MapAccess::next_value::<D>(
                                                        &mut __map,
                                                    ) {
                                                        _serde::__private::Ok(__val) => __val,
                                                        _serde::__private::Err(__err) => {
                                                            return _serde::__private::Err(__err);
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
                                                    _serde::__private::Ok(__val) => __val,
                                                    _serde::__private::Err(__err) => {
                                                        return _serde::__private::Err(__err);
                                                    }
                                                };
                                            }
                                        }
                                    }
                                    let __field0 = match __field0 {
                                        _serde::__private::Some(__field0) => __field0,
                                        _serde::__private::None => {
                                            match _serde::__private::de::missing_field("a") {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            }
                                        }
                                    };
                                    let __field1 = match __field1 {
                                        _serde::__private::Some(__field1) => __field1,
                                        _serde::__private::None => {
                                            match _serde::__private::de::missing_field("b") {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            }
                                        }
                                    };
                                    let __field2 = match __field2 {
                                        _serde::__private::Some(__field2) => __field2,
                                        _serde::__private::None => {
                                            match _serde::__private::de::missing_field("c") {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            }
                                        }
                                    };
                                    let __field3 = match __field3 {
                                        _serde::__private::Some(__field3) => __field3,
                                        _serde::__private::None => {
                                            match _serde::__private::de::missing_field("d") {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            }
                                        }
                                    };
                                    _serde::__private::Ok(DeEnum::Map {
                                        a: __field0,
                                        b: __field1,
                                        c: __field2,
                                        d: __field3,
                                    })
                                }
                            }
                            const FIELDS: &'static [&'static str] = &["a", "b", "c", "d"];
                            _serde::de::VariantAccess::struct_variant(
                                __variant,
                                FIELDS,
                                __Visitor {
                                    marker: _serde::__private::PhantomData::<DeEnum<B, C, D>>,
                                    lifetime: _serde::__private::PhantomData,
                                },
                            )
                        }
                        (__Field::__field3, __variant) => {
                            match _serde::de::VariantAccess::unit_variant(__variant) {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            };
                            _serde::__private::Ok(DeEnum::_Unit2)
                        }
                        (__Field::__field4, __variant) => {
                            struct __Visitor<'de, B, C, D>
                            where
                                B: _serde::Deserialize<'de>,
                                C: _serde::Deserialize<'de>,
                                D: _serde::Deserialize<'de>,
                            {
                                marker: _serde::__private::PhantomData<DeEnum<B, C, D>>,
                                lifetime: _serde::__private::PhantomData<&'de ()>,
                            }
                            impl<'de, B, C, D> _serde::de::Visitor<'de> for __Visitor<'de, B, C, D>
                            where
                                B: _serde::Deserialize<'de>,
                                C: _serde::Deserialize<'de>,
                                D: _serde::Deserialize<'de>,
                            {
                                type Value = DeEnum<B, C, D>;
                                fn expecting(
                                    &self,
                                    __formatter: &mut _serde::__private::Formatter,
                                ) -> _serde::__private::fmt::Result
                                {
                                    _serde::__private::Formatter::write_str(
                                        __formatter,
                                        "tuple variant DeEnum::_Seq2",
                                    )
                                }
                                #[inline]
                                fn visit_seq<__A>(
                                    self,
                                    mut __seq: __A,
                                ) -> _serde::__private::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::SeqAccess<'de>,
                                {
                                    let __field0 = match match _serde::de::SeqAccess::next_element::<
                                        i8,
                                    >(
                                        &mut __seq
                                    ) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    } {
                                        _serde::__private::Some(__value) => __value,
                                        _serde::__private::None => {
                                            return _serde::__private::Err(
                                                _serde::de::Error::invalid_length(
                                                    0usize,
                                                    &"tuple variant DeEnum::_Seq2 with 4 elements",
                                                ),
                                            );
                                        }
                                    };
                                    let __field1 = match match _serde::de::SeqAccess::next_element::<
                                        B,
                                    >(
                                        &mut __seq
                                    ) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    } {
                                        _serde::__private::Some(__value) => __value,
                                        _serde::__private::None => {
                                            return _serde::__private::Err(
                                                _serde::de::Error::invalid_length(
                                                    1usize,
                                                    &"tuple variant DeEnum::_Seq2 with 4 elements",
                                                ),
                                            );
                                        }
                                    };
                                    let __field2 = match match _serde::de::SeqAccess::next_element::<
                                        C,
                                    >(
                                        &mut __seq
                                    ) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    } {
                                        _serde::__private::Some(__value) => __value,
                                        _serde::__private::None => {
                                            return _serde::__private::Err(
                                                _serde::de::Error::invalid_length(
                                                    2usize,
                                                    &"tuple variant DeEnum::_Seq2 with 4 elements",
                                                ),
                                            );
                                        }
                                    };
                                    let __field3 = match match _serde::de::SeqAccess::next_element::<
                                        D,
                                    >(
                                        &mut __seq
                                    ) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    } {
                                        _serde::__private::Some(__value) => __value,
                                        _serde::__private::None => {
                                            return _serde::__private::Err(
                                                _serde::de::Error::invalid_length(
                                                    3usize,
                                                    &"tuple variant DeEnum::_Seq2 with 4 elements",
                                                ),
                                            );
                                        }
                                    };
                                    _serde::__private::Ok(DeEnum::_Seq2(
                                        __field0, __field1, __field2, __field3,
                                    ))
                                }
                            }
                            _serde::de::VariantAccess::tuple_variant(
                                __variant,
                                4usize,
                                __Visitor {
                                    marker: _serde::__private::PhantomData::<DeEnum<B, C, D>>,
                                    lifetime: _serde::__private::PhantomData,
                                },
                            )
                        }
                        (__Field::__field5, __variant) => {
                            #[allow(non_camel_case_types)]
                            enum __Field {
                                __field0,
                                __field1,
                                __field2,
                                __field3,
                                __ignore,
                            }
                            struct __FieldVisitor;
                            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                                type Value = __Field;
                                fn expecting(
                                    &self,
                                    __formatter: &mut _serde::__private::Formatter,
                                ) -> _serde::__private::fmt::Result
                                {
                                    _serde::__private::Formatter::write_str(
                                        __formatter,
                                        "field identifier",
                                    )
                                }
                                fn visit_u64<__E>(
                                    self,
                                    __value: u64,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    match __value {
                                        0u64 => _serde::__private::Ok(__Field::__field0),
                                        1u64 => _serde::__private::Ok(__Field::__field1),
                                        2u64 => _serde::__private::Ok(__Field::__field2),
                                        3u64 => _serde::__private::Ok(__Field::__field3),
                                        _ => _serde::__private::Err(
                                            _serde::de::Error::invalid_value(
                                                _serde::de::Unexpected::Unsigned(__value),
                                                &"field index 0 <= i < 4",
                                            ),
                                        ),
                                    }
                                }
                                fn visit_str<__E>(
                                    self,
                                    __value: &str,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    match __value {
                                        "a" => _serde::__private::Ok(__Field::__field0),
                                        "b" => _serde::__private::Ok(__Field::__field1),
                                        "c" => _serde::__private::Ok(__Field::__field2),
                                        "d" => _serde::__private::Ok(__Field::__field3),
                                        _ => _serde::__private::Ok(__Field::__ignore),
                                    }
                                }
                                fn visit_bytes<__E>(
                                    self,
                                    __value: &[u8],
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    match __value {
                                        b"a" => _serde::__private::Ok(__Field::__field0),
                                        b"b" => _serde::__private::Ok(__Field::__field1),
                                        b"c" => _serde::__private::Ok(__Field::__field2),
                                        b"d" => _serde::__private::Ok(__Field::__field3),
                                        _ => _serde::__private::Ok(__Field::__ignore),
                                    }
                                }
                            }
                            impl<'de> _serde::Deserialize<'de> for __Field {
                                #[inline]
                                fn deserialize<__D>(
                                    __deserializer: __D,
                                ) -> _serde::__private::Result<Self, __D::Error>
                                where
                                    __D: _serde::Deserializer<'de>,
                                {
                                    _serde::Deserializer::deserialize_identifier(
                                        __deserializer,
                                        __FieldVisitor,
                                    )
                                }
                            }
                            struct __Visitor<'de, B, C, D>
                            where
                                B: _serde::Deserialize<'de>,
                                C: _serde::Deserialize<'de>,
                                D: _serde::Deserialize<'de>,
                            {
                                marker: _serde::__private::PhantomData<DeEnum<B, C, D>>,
                                lifetime: _serde::__private::PhantomData<&'de ()>,
                            }
                            impl<'de, B, C, D> _serde::de::Visitor<'de> for __Visitor<'de, B, C, D>
                            where
                                B: _serde::Deserialize<'de>,
                                C: _serde::Deserialize<'de>,
                                D: _serde::Deserialize<'de>,
                            {
                                type Value = DeEnum<B, C, D>;
                                fn expecting(
                                    &self,
                                    __formatter: &mut _serde::__private::Formatter,
                                ) -> _serde::__private::fmt::Result
                                {
                                    _serde::__private::Formatter::write_str(
                                        __formatter,
                                        "struct variant DeEnum::_Map2",
                                    )
                                }
                                #[inline]
                                fn visit_seq<__A>(
                                    self,
                                    mut __seq: __A,
                                ) -> _serde::__private::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::SeqAccess<'de>,
                                {
                                    let __field0 = match match _serde::de::SeqAccess::next_element::<
                                        i8,
                                    >(
                                        &mut __seq
                                    ) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    } {
                                        _serde::__private::Some(__value) => __value,
                                        _serde::__private::None => {
                                            return _serde::__private::Err(
                                                _serde::de::Error::invalid_length(
                                                    0usize,
                                                    &"struct variant DeEnum::_Map2 with 4 elements",
                                                ),
                                            );
                                        }
                                    };
                                    let __field1 = match match _serde::de::SeqAccess::next_element::<
                                        B,
                                    >(
                                        &mut __seq
                                    ) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    } {
                                        _serde::__private::Some(__value) => __value,
                                        _serde::__private::None => {
                                            return _serde::__private::Err(
                                                _serde::de::Error::invalid_length(
                                                    1usize,
                                                    &"struct variant DeEnum::_Map2 with 4 elements",
                                                ),
                                            );
                                        }
                                    };
                                    let __field2 = match match _serde::de::SeqAccess::next_element::<
                                        C,
                                    >(
                                        &mut __seq
                                    ) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    } {
                                        _serde::__private::Some(__value) => __value,
                                        _serde::__private::None => {
                                            return _serde::__private::Err(
                                                _serde::de::Error::invalid_length(
                                                    2usize,
                                                    &"struct variant DeEnum::_Map2 with 4 elements",
                                                ),
                                            );
                                        }
                                    };
                                    let __field3 = match match _serde::de::SeqAccess::next_element::<
                                        D,
                                    >(
                                        &mut __seq
                                    ) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    } {
                                        _serde::__private::Some(__value) => __value,
                                        _serde::__private::None => {
                                            return _serde::__private::Err(
                                                _serde::de::Error::invalid_length(
                                                    3usize,
                                                    &"struct variant DeEnum::_Map2 with 4 elements",
                                                ),
                                            );
                                        }
                                    };
                                    _serde::__private::Ok(DeEnum::_Map2 {
                                        a: __field0,
                                        b: __field1,
                                        c: __field2,
                                        d: __field3,
                                    })
                                }
                                #[inline]
                                fn visit_map<__A>(
                                    self,
                                    mut __map: __A,
                                ) -> _serde::__private::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::MapAccess<'de>,
                                {
                                    let mut __field0: _serde::__private::Option<i8> =
                                        _serde::__private::None;
                                    let mut __field1: _serde::__private::Option<B> =
                                        _serde::__private::None;
                                    let mut __field2: _serde::__private::Option<C> =
                                        _serde::__private::None;
                                    let mut __field3: _serde::__private::Option<D> =
                                        _serde::__private::None;
                                    while let _serde::__private::Some(__key) =
                                        match _serde::de::MapAccess::next_key::<__Field>(&mut __map)
                                        {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        }
                                    {
                                        match __key {
                                            __Field::__field0 => {
                                                if _serde::__private::Option::is_some(&__field0) {
                                                    return _serde :: __private :: Err (< __A :: Error as _serde :: de :: Error > :: duplicate_field ("a")) ;
                                                }
                                                __field0 = _serde::__private::Some(
                                                    match _serde::de::MapAccess::next_value::<i8>(
                                                        &mut __map,
                                                    ) {
                                                        _serde::__private::Ok(__val) => __val,
                                                        _serde::__private::Err(__err) => {
                                                            return _serde::__private::Err(__err);
                                                        }
                                                    },
                                                );
                                            }
                                            __Field::__field1 => {
                                                if _serde::__private::Option::is_some(&__field1) {
                                                    return _serde :: __private :: Err (< __A :: Error as _serde :: de :: Error > :: duplicate_field ("b")) ;
                                                }
                                                __field1 = _serde::__private::Some(
                                                    match _serde::de::MapAccess::next_value::<B>(
                                                        &mut __map,
                                                    ) {
                                                        _serde::__private::Ok(__val) => __val,
                                                        _serde::__private::Err(__err) => {
                                                            return _serde::__private::Err(__err);
                                                        }
                                                    },
                                                );
                                            }
                                            __Field::__field2 => {
                                                if _serde::__private::Option::is_some(&__field2) {
                                                    return _serde :: __private :: Err (< __A :: Error as _serde :: de :: Error > :: duplicate_field ("c")) ;
                                                }
                                                __field2 = _serde::__private::Some(
                                                    match _serde::de::MapAccess::next_value::<C>(
                                                        &mut __map,
                                                    ) {
                                                        _serde::__private::Ok(__val) => __val,
                                                        _serde::__private::Err(__err) => {
                                                            return _serde::__private::Err(__err);
                                                        }
                                                    },
                                                );
                                            }
                                            __Field::__field3 => {
                                                if _serde::__private::Option::is_some(&__field3) {
                                                    return _serde :: __private :: Err (< __A :: Error as _serde :: de :: Error > :: duplicate_field ("d")) ;
                                                }
                                                __field3 = _serde::__private::Some(
                                                    match _serde::de::MapAccess::next_value::<D>(
                                                        &mut __map,
                                                    ) {
                                                        _serde::__private::Ok(__val) => __val,
                                                        _serde::__private::Err(__err) => {
                                                            return _serde::__private::Err(__err);
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
                                                    _serde::__private::Ok(__val) => __val,
                                                    _serde::__private::Err(__err) => {
                                                        return _serde::__private::Err(__err);
                                                    }
                                                };
                                            }
                                        }
                                    }
                                    let __field0 = match __field0 {
                                        _serde::__private::Some(__field0) => __field0,
                                        _serde::__private::None => {
                                            match _serde::__private::de::missing_field("a") {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            }
                                        }
                                    };
                                    let __field1 = match __field1 {
                                        _serde::__private::Some(__field1) => __field1,
                                        _serde::__private::None => {
                                            match _serde::__private::de::missing_field("b") {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            }
                                        }
                                    };
                                    let __field2 = match __field2 {
                                        _serde::__private::Some(__field2) => __field2,
                                        _serde::__private::None => {
                                            match _serde::__private::de::missing_field("c") {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            }
                                        }
                                    };
                                    let __field3 = match __field3 {
                                        _serde::__private::Some(__field3) => __field3,
                                        _serde::__private::None => {
                                            match _serde::__private::de::missing_field("d") {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            }
                                        }
                                    };
                                    _serde::__private::Ok(DeEnum::_Map2 {
                                        a: __field0,
                                        b: __field1,
                                        c: __field2,
                                        d: __field3,
                                    })
                                }
                            }
                            const FIELDS: &'static [&'static str] = &["a", "b", "c", "d"];
                            _serde::de::VariantAccess::struct_variant(
                                __variant,
                                FIELDS,
                                __Visitor {
                                    marker: _serde::__private::PhantomData::<DeEnum<B, C, D>>,
                                    lifetime: _serde::__private::PhantomData,
                                },
                            )
                        }
                    }
                }
            }
            const VARIANTS: &'static [&'static str] =
                &["Unit", "Seq", "Map", "_Unit2", "_Seq2", "_Map2"];
            _serde::Deserializer::deserialize_enum(
                __deserializer,
                "DeEnum",
                VARIANTS,
                __Visitor {
                    marker: _serde::__private::PhantomData::<DeEnum<B, C, D>>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};
