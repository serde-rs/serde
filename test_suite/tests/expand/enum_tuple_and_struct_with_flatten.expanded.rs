use serde::Deserialize;
struct Nested {}
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for Nested {
        fn deserialize<__D>(
            __deserializer: __D,
        ) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            enum __Field {
                __ignore,
            }
            struct __FieldVisitor;
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
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
                        _ => _serde::__private::Ok(__Field::__ignore),
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
            struct __Visitor<'de> {
                marker: _serde::__private::PhantomData<Nested>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = Nested;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "struct Nested")
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    _: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    _serde::__private::Ok(Nested {})
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    while let _serde::__private::Some(__key)
                        = match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        } {
                        match __key {
                            _ => {
                                let _ = match _serde::de::MapAccess::next_value::<
                                    _serde::de::IgnoredAny,
                                >(&mut __map) {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                };
                            }
                        }
                    }
                    _serde::__private::Ok(Nested {})
                }
            }
            const FIELDS: &'static [&'static str] = &[];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "Nested",
                FIELDS,
                __Visitor {
                    marker: _serde::__private::PhantomData::<Nested>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};
enum WithFlatten1 {
    Tuple(f64, String),
    Flatten { #[serde(flatten)] nested: Nested },
}
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for WithFlatten1 {
        fn deserialize<__D>(
            __deserializer: __D,
        ) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            enum __Field {
                __field0,
                __field1,
            }
            struct __FieldVisitor;
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(
                        __formatter,
                        "variant identifier",
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
                        _ => {
                            _serde::__private::Err(
                                _serde::de::Error::invalid_value(
                                    _serde::de::Unexpected::Unsigned(__value),
                                    &"variant index 0 <= i < 2",
                                ),
                            )
                        }
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
                        "Tuple" => _serde::__private::Ok(__Field::__field0),
                        "Flatten" => _serde::__private::Ok(__Field::__field1),
                        _ => {
                            _serde::__private::Err(
                                _serde::de::Error::unknown_variant(__value, VARIANTS),
                            )
                        }
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
                        b"Tuple" => _serde::__private::Ok(__Field::__field0),
                        b"Flatten" => _serde::__private::Ok(__Field::__field1),
                        _ => {
                            let __value = &_serde::__private::from_utf8_lossy(__value);
                            _serde::__private::Err(
                                _serde::de::Error::unknown_variant(__value, VARIANTS),
                            )
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
                    _serde::Deserializer::deserialize_identifier(
                        __deserializer,
                        __FieldVisitor,
                    )
                }
            }
            struct __Visitor<'de> {
                marker: _serde::__private::PhantomData<WithFlatten1>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = WithFlatten1;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(
                        __formatter,
                        "enum WithFlatten1",
                    )
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
                            struct __Visitor<'de> {
                                marker: _serde::__private::PhantomData<WithFlatten1>,
                                lifetime: _serde::__private::PhantomData<&'de ()>,
                            }
                            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                                type Value = WithFlatten1;
                                fn expecting(
                                    &self,
                                    __formatter: &mut _serde::__private::Formatter,
                                ) -> _serde::__private::fmt::Result {
                                    _serde::__private::Formatter::write_str(
                                        __formatter,
                                        "tuple variant WithFlatten1::Tuple",
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
                                        f64,
                                    >(&mut __seq) {
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
                                                    &"tuple variant WithFlatten1::Tuple with 2 elements",
                                                ),
                                            );
                                        }
                                    };
                                    let __field1 = match match _serde::de::SeqAccess::next_element::<
                                        String,
                                    >(&mut __seq) {
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
                                                    &"tuple variant WithFlatten1::Tuple with 2 elements",
                                                ),
                                            );
                                        }
                                    };
                                    _serde::__private::Ok(
                                        WithFlatten1::Tuple(__field0, __field1),
                                    )
                                }
                            }
                            _serde::de::VariantAccess::tuple_variant(
                                __variant,
                                2usize,
                                __Visitor {
                                    marker: _serde::__private::PhantomData::<WithFlatten1>,
                                    lifetime: _serde::__private::PhantomData,
                                },
                            )
                        }
                        (__Field::__field1, __variant) => {
                            #[allow(non_camel_case_types)]
                            enum __Field<'de> {
                                __other(_serde::__private::de::Content<'de>),
                            }
                            struct __FieldVisitor;
                            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                                type Value = __Field<'de>;
                                fn expecting(
                                    &self,
                                    __formatter: &mut _serde::__private::Formatter,
                                ) -> _serde::__private::fmt::Result {
                                    _serde::__private::Formatter::write_str(
                                        __formatter,
                                        "field identifier",
                                    )
                                }
                                fn visit_bool<__E>(
                                    self,
                                    __value: bool,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    _serde::__private::Ok(
                                        __Field::__other(
                                            _serde::__private::de::Content::Bool(__value),
                                        ),
                                    )
                                }
                                fn visit_i8<__E>(
                                    self,
                                    __value: i8,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    _serde::__private::Ok(
                                        __Field::__other(
                                            _serde::__private::de::Content::I8(__value),
                                        ),
                                    )
                                }
                                fn visit_i16<__E>(
                                    self,
                                    __value: i16,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    _serde::__private::Ok(
                                        __Field::__other(
                                            _serde::__private::de::Content::I16(__value),
                                        ),
                                    )
                                }
                                fn visit_i32<__E>(
                                    self,
                                    __value: i32,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    _serde::__private::Ok(
                                        __Field::__other(
                                            _serde::__private::de::Content::I32(__value),
                                        ),
                                    )
                                }
                                fn visit_i64<__E>(
                                    self,
                                    __value: i64,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    _serde::__private::Ok(
                                        __Field::__other(
                                            _serde::__private::de::Content::I64(__value),
                                        ),
                                    )
                                }
                                fn visit_u8<__E>(
                                    self,
                                    __value: u8,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    _serde::__private::Ok(
                                        __Field::__other(
                                            _serde::__private::de::Content::U8(__value),
                                        ),
                                    )
                                }
                                fn visit_u16<__E>(
                                    self,
                                    __value: u16,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    _serde::__private::Ok(
                                        __Field::__other(
                                            _serde::__private::de::Content::U16(__value),
                                        ),
                                    )
                                }
                                fn visit_u32<__E>(
                                    self,
                                    __value: u32,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    _serde::__private::Ok(
                                        __Field::__other(
                                            _serde::__private::de::Content::U32(__value),
                                        ),
                                    )
                                }
                                fn visit_u64<__E>(
                                    self,
                                    __value: u64,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    _serde::__private::Ok(
                                        __Field::__other(
                                            _serde::__private::de::Content::U64(__value),
                                        ),
                                    )
                                }
                                fn visit_f32<__E>(
                                    self,
                                    __value: f32,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    _serde::__private::Ok(
                                        __Field::__other(
                                            _serde::__private::de::Content::F32(__value),
                                        ),
                                    )
                                }
                                fn visit_f64<__E>(
                                    self,
                                    __value: f64,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    _serde::__private::Ok(
                                        __Field::__other(
                                            _serde::__private::de::Content::F64(__value),
                                        ),
                                    )
                                }
                                fn visit_char<__E>(
                                    self,
                                    __value: char,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    _serde::__private::Ok(
                                        __Field::__other(
                                            _serde::__private::de::Content::Char(__value),
                                        ),
                                    )
                                }
                                fn visit_unit<__E>(
                                    self,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    _serde::__private::Ok(
                                        __Field::__other(_serde::__private::de::Content::Unit),
                                    )
                                }
                                fn visit_str<__E>(
                                    self,
                                    __value: &str,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    match __value {
                                        _ => {
                                            let __value = _serde::__private::de::Content::String(
                                                _serde::__private::ToString::to_string(__value),
                                            );
                                            _serde::__private::Ok(__Field::__other(__value))
                                        }
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
                                        _ => {
                                            let __value = _serde::__private::de::Content::ByteBuf(
                                                __value.to_vec(),
                                            );
                                            _serde::__private::Ok(__Field::__other(__value))
                                        }
                                    }
                                }
                                fn visit_borrowed_str<__E>(
                                    self,
                                    __value: &'de str,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    match __value {
                                        _ => {
                                            let __value = _serde::__private::de::Content::Str(__value);
                                            _serde::__private::Ok(__Field::__other(__value))
                                        }
                                    }
                                }
                                fn visit_borrowed_bytes<__E>(
                                    self,
                                    __value: &'de [u8],
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    match __value {
                                        _ => {
                                            let __value = _serde::__private::de::Content::Bytes(
                                                __value,
                                            );
                                            _serde::__private::Ok(__Field::__other(__value))
                                        }
                                    }
                                }
                            }
                            impl<'de> _serde::Deserialize<'de> for __Field<'de> {
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
                            struct __Visitor<'de> {
                                marker: _serde::__private::PhantomData<WithFlatten1>,
                                lifetime: _serde::__private::PhantomData<&'de ()>,
                            }
                            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                                type Value = WithFlatten1;
                                fn expecting(
                                    &self,
                                    __formatter: &mut _serde::__private::Formatter,
                                ) -> _serde::__private::fmt::Result {
                                    _serde::__private::Formatter::write_str(
                                        __formatter,
                                        "struct variant WithFlatten1::Flatten",
                                    )
                                }
                                #[inline]
                                fn visit_map<__A>(
                                    self,
                                    mut __map: __A,
                                ) -> _serde::__private::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::MapAccess<'de>,
                                {
                                    let mut __collect = _serde::__private::Vec::<
                                        _serde::__private::Option<
                                            (
                                                _serde::__private::de::Content,
                                                _serde::__private::de::Content,
                                            ),
                                        >,
                                    >::new();
                                    while let _serde::__private::Some(__key)
                                        = match _serde::de::MapAccess::next_key::<
                                            __Field,
                                        >(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        } {
                                        match __key {
                                            __Field::__other(__name) => {
                                                __collect
                                                    .push(
                                                        _serde::__private::Some((
                                                            __name,
                                                            match _serde::de::MapAccess::next_value(&mut __map) {
                                                                _serde::__private::Ok(__val) => __val,
                                                                _serde::__private::Err(__err) => {
                                                                    return _serde::__private::Err(__err);
                                                                }
                                                            },
                                                        )),
                                                    );
                                            }
                                        }
                                    }
                                    let __field0: Nested = match _serde::de::Deserialize::deserialize(
                                        _serde::__private::de::FlatMapDeserializer(
                                            &mut __collect,
                                            _serde::__private::PhantomData,
                                        ),
                                    ) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    };
                                    _serde::__private::Ok(WithFlatten1::Flatten {
                                        nested: __field0,
                                    })
                                }
                            }
                            impl<'de> _serde::de::DeserializeSeed<'de>
                            for __Visitor<'de> {
                                type Value = WithFlatten1;
                                fn deserialize<__D>(
                                    self,
                                    __deserializer: __D,
                                ) -> _serde::__private::Result<Self::Value, __D::Error>
                                where
                                    __D: _serde::Deserializer<'de>,
                                {
                                    _serde::Deserializer::deserialize_map(__deserializer, self)
                                }
                            }
                            _serde::de::VariantAccess::newtype_variant_seed(
                                __variant,
                                __Visitor {
                                    marker: _serde::__private::PhantomData::<WithFlatten1>,
                                    lifetime: _serde::__private::PhantomData,
                                },
                            )
                        }
                    }
                }
            }
            const VARIANTS: &'static [&'static str] = &["Tuple", "Flatten"];
            _serde::Deserializer::deserialize_enum(
                __deserializer,
                "WithFlatten1",
                VARIANTS,
                __Visitor {
                    marker: _serde::__private::PhantomData::<WithFlatten1>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};
enum WithFlatten2 {
    Flatten { #[serde(flatten)] nested: Nested },
    Tuple(f64, String),
}
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for WithFlatten2 {
        fn deserialize<__D>(
            __deserializer: __D,
        ) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            enum __Field {
                __field0,
                __field1,
            }
            struct __FieldVisitor;
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(
                        __formatter,
                        "variant identifier",
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
                        _ => {
                            _serde::__private::Err(
                                _serde::de::Error::invalid_value(
                                    _serde::de::Unexpected::Unsigned(__value),
                                    &"variant index 0 <= i < 2",
                                ),
                            )
                        }
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
                        "Flatten" => _serde::__private::Ok(__Field::__field0),
                        "Tuple" => _serde::__private::Ok(__Field::__field1),
                        _ => {
                            _serde::__private::Err(
                                _serde::de::Error::unknown_variant(__value, VARIANTS),
                            )
                        }
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
                        b"Flatten" => _serde::__private::Ok(__Field::__field0),
                        b"Tuple" => _serde::__private::Ok(__Field::__field1),
                        _ => {
                            let __value = &_serde::__private::from_utf8_lossy(__value);
                            _serde::__private::Err(
                                _serde::de::Error::unknown_variant(__value, VARIANTS),
                            )
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
                    _serde::Deserializer::deserialize_identifier(
                        __deserializer,
                        __FieldVisitor,
                    )
                }
            }
            struct __Visitor<'de> {
                marker: _serde::__private::PhantomData<WithFlatten2>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = WithFlatten2;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(
                        __formatter,
                        "enum WithFlatten2",
                    )
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
                            #[allow(non_camel_case_types)]
                            enum __Field<'de> {
                                __other(_serde::__private::de::Content<'de>),
                            }
                            struct __FieldVisitor;
                            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                                type Value = __Field<'de>;
                                fn expecting(
                                    &self,
                                    __formatter: &mut _serde::__private::Formatter,
                                ) -> _serde::__private::fmt::Result {
                                    _serde::__private::Formatter::write_str(
                                        __formatter,
                                        "field identifier",
                                    )
                                }
                                fn visit_bool<__E>(
                                    self,
                                    __value: bool,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    _serde::__private::Ok(
                                        __Field::__other(
                                            _serde::__private::de::Content::Bool(__value),
                                        ),
                                    )
                                }
                                fn visit_i8<__E>(
                                    self,
                                    __value: i8,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    _serde::__private::Ok(
                                        __Field::__other(
                                            _serde::__private::de::Content::I8(__value),
                                        ),
                                    )
                                }
                                fn visit_i16<__E>(
                                    self,
                                    __value: i16,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    _serde::__private::Ok(
                                        __Field::__other(
                                            _serde::__private::de::Content::I16(__value),
                                        ),
                                    )
                                }
                                fn visit_i32<__E>(
                                    self,
                                    __value: i32,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    _serde::__private::Ok(
                                        __Field::__other(
                                            _serde::__private::de::Content::I32(__value),
                                        ),
                                    )
                                }
                                fn visit_i64<__E>(
                                    self,
                                    __value: i64,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    _serde::__private::Ok(
                                        __Field::__other(
                                            _serde::__private::de::Content::I64(__value),
                                        ),
                                    )
                                }
                                fn visit_u8<__E>(
                                    self,
                                    __value: u8,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    _serde::__private::Ok(
                                        __Field::__other(
                                            _serde::__private::de::Content::U8(__value),
                                        ),
                                    )
                                }
                                fn visit_u16<__E>(
                                    self,
                                    __value: u16,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    _serde::__private::Ok(
                                        __Field::__other(
                                            _serde::__private::de::Content::U16(__value),
                                        ),
                                    )
                                }
                                fn visit_u32<__E>(
                                    self,
                                    __value: u32,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    _serde::__private::Ok(
                                        __Field::__other(
                                            _serde::__private::de::Content::U32(__value),
                                        ),
                                    )
                                }
                                fn visit_u64<__E>(
                                    self,
                                    __value: u64,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    _serde::__private::Ok(
                                        __Field::__other(
                                            _serde::__private::de::Content::U64(__value),
                                        ),
                                    )
                                }
                                fn visit_f32<__E>(
                                    self,
                                    __value: f32,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    _serde::__private::Ok(
                                        __Field::__other(
                                            _serde::__private::de::Content::F32(__value),
                                        ),
                                    )
                                }
                                fn visit_f64<__E>(
                                    self,
                                    __value: f64,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    _serde::__private::Ok(
                                        __Field::__other(
                                            _serde::__private::de::Content::F64(__value),
                                        ),
                                    )
                                }
                                fn visit_char<__E>(
                                    self,
                                    __value: char,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    _serde::__private::Ok(
                                        __Field::__other(
                                            _serde::__private::de::Content::Char(__value),
                                        ),
                                    )
                                }
                                fn visit_unit<__E>(
                                    self,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    _serde::__private::Ok(
                                        __Field::__other(_serde::__private::de::Content::Unit),
                                    )
                                }
                                fn visit_str<__E>(
                                    self,
                                    __value: &str,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    match __value {
                                        _ => {
                                            let __value = _serde::__private::de::Content::String(
                                                _serde::__private::ToString::to_string(__value),
                                            );
                                            _serde::__private::Ok(__Field::__other(__value))
                                        }
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
                                        _ => {
                                            let __value = _serde::__private::de::Content::ByteBuf(
                                                __value.to_vec(),
                                            );
                                            _serde::__private::Ok(__Field::__other(__value))
                                        }
                                    }
                                }
                                fn visit_borrowed_str<__E>(
                                    self,
                                    __value: &'de str,
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    match __value {
                                        _ => {
                                            let __value = _serde::__private::de::Content::Str(__value);
                                            _serde::__private::Ok(__Field::__other(__value))
                                        }
                                    }
                                }
                                fn visit_borrowed_bytes<__E>(
                                    self,
                                    __value: &'de [u8],
                                ) -> _serde::__private::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    match __value {
                                        _ => {
                                            let __value = _serde::__private::de::Content::Bytes(
                                                __value,
                                            );
                                            _serde::__private::Ok(__Field::__other(__value))
                                        }
                                    }
                                }
                            }
                            impl<'de> _serde::Deserialize<'de> for __Field<'de> {
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
                            struct __Visitor<'de> {
                                marker: _serde::__private::PhantomData<WithFlatten2>,
                                lifetime: _serde::__private::PhantomData<&'de ()>,
                            }
                            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                                type Value = WithFlatten2;
                                fn expecting(
                                    &self,
                                    __formatter: &mut _serde::__private::Formatter,
                                ) -> _serde::__private::fmt::Result {
                                    _serde::__private::Formatter::write_str(
                                        __formatter,
                                        "struct variant WithFlatten2::Flatten",
                                    )
                                }
                                #[inline]
                                fn visit_map<__A>(
                                    self,
                                    mut __map: __A,
                                ) -> _serde::__private::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::MapAccess<'de>,
                                {
                                    let mut __collect = _serde::__private::Vec::<
                                        _serde::__private::Option<
                                            (
                                                _serde::__private::de::Content,
                                                _serde::__private::de::Content,
                                            ),
                                        >,
                                    >::new();
                                    while let _serde::__private::Some(__key)
                                        = match _serde::de::MapAccess::next_key::<
                                            __Field,
                                        >(&mut __map) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        } {
                                        match __key {
                                            __Field::__other(__name) => {
                                                __collect
                                                    .push(
                                                        _serde::__private::Some((
                                                            __name,
                                                            match _serde::de::MapAccess::next_value(&mut __map) {
                                                                _serde::__private::Ok(__val) => __val,
                                                                _serde::__private::Err(__err) => {
                                                                    return _serde::__private::Err(__err);
                                                                }
                                                            },
                                                        )),
                                                    );
                                            }
                                        }
                                    }
                                    let __field0: Nested = match _serde::de::Deserialize::deserialize(
                                        _serde::__private::de::FlatMapDeserializer(
                                            &mut __collect,
                                            _serde::__private::PhantomData,
                                        ),
                                    ) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    };
                                    _serde::__private::Ok(WithFlatten2::Flatten {
                                        nested: __field0,
                                    })
                                }
                            }
                            impl<'de> _serde::de::DeserializeSeed<'de>
                            for __Visitor<'de> {
                                type Value = WithFlatten2;
                                fn deserialize<__D>(
                                    self,
                                    __deserializer: __D,
                                ) -> _serde::__private::Result<Self::Value, __D::Error>
                                where
                                    __D: _serde::Deserializer<'de>,
                                {
                                    _serde::Deserializer::deserialize_map(__deserializer, self)
                                }
                            }
                            _serde::de::VariantAccess::newtype_variant_seed(
                                __variant,
                                __Visitor {
                                    marker: _serde::__private::PhantomData::<WithFlatten2>,
                                    lifetime: _serde::__private::PhantomData,
                                },
                            )
                        }
                        (__Field::__field1, __variant) => {
                            struct __Visitor<'de> {
                                marker: _serde::__private::PhantomData<WithFlatten2>,
                                lifetime: _serde::__private::PhantomData<&'de ()>,
                            }
                            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                                type Value = WithFlatten2;
                                fn expecting(
                                    &self,
                                    __formatter: &mut _serde::__private::Formatter,
                                ) -> _serde::__private::fmt::Result {
                                    _serde::__private::Formatter::write_str(
                                        __formatter,
                                        "tuple variant WithFlatten2::Tuple",
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
                                        f64,
                                    >(&mut __seq) {
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
                                                    &"tuple variant WithFlatten2::Tuple with 2 elements",
                                                ),
                                            );
                                        }
                                    };
                                    let __field1 = match match _serde::de::SeqAccess::next_element::<
                                        String,
                                    >(&mut __seq) {
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
                                                    &"tuple variant WithFlatten2::Tuple with 2 elements",
                                                ),
                                            );
                                        }
                                    };
                                    _serde::__private::Ok(
                                        WithFlatten2::Tuple(__field0, __field1),
                                    )
                                }
                            }
                            _serde::de::VariantAccess::tuple_variant(
                                __variant,
                                2usize,
                                __Visitor {
                                    marker: _serde::__private::PhantomData::<WithFlatten2>,
                                    lifetime: _serde::__private::PhantomData,
                                },
                            )
                        }
                    }
                }
            }
            const VARIANTS: &'static [&'static str] = &["Flatten", "Tuple"];
            _serde::Deserializer::deserialize_enum(
                __deserializer,
                "WithFlatten2",
                VARIANTS,
                __Visitor {
                    marker: _serde::__private::PhantomData::<WithFlatten2>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};
