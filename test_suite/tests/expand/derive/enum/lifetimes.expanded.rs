use serde::{Deserialize, Serialize};
enum Lifetimes<'a> {
    LifetimeSeq(&'a i32),
    NoLifetimeSeq(i32),
    LifetimeMap { a: &'a i32 },
    NoLifetimeMap { a: i32 },
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_SERIALIZE_FOR_Lifetimes: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'a> _serde::Serialize for Lifetimes<'a> {
        fn serialize<__S>(&self, __serializer: __S) -> _serde::export::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            match *self {
                Lifetimes::LifetimeSeq(ref __field0) => {
                    _serde::Serializer::serialize_newtype_variant(
                        __serializer,
                        "Lifetimes",
                        0u32,
                        "LifetimeSeq",
                        __field0,
                    )
                }
                Lifetimes::NoLifetimeSeq(ref __field0) => {
                    _serde::Serializer::serialize_newtype_variant(
                        __serializer,
                        "Lifetimes",
                        1u32,
                        "NoLifetimeSeq",
                        __field0,
                    )
                }
                Lifetimes::LifetimeMap { ref a } => {
                    let mut __serde_state = match _serde::Serializer::serialize_struct_variant(
                        __serializer,
                        "Lifetimes",
                        2u32,
                        "LifetimeMap",
                        0 + 1,
                    ) {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStructVariant::serialize_field(
                        &mut __serde_state,
                        "a",
                        a,
                    ) {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    };
                    _serde::ser::SerializeStructVariant::end(__serde_state)
                }
                Lifetimes::NoLifetimeMap { ref a } => {
                    let mut __serde_state = match _serde::Serializer::serialize_struct_variant(
                        __serializer,
                        "Lifetimes",
                        3u32,
                        "NoLifetimeMap",
                        0 + 1,
                    ) {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    };
                    match _serde::ser::SerializeStructVariant::serialize_field(
                        &mut __serde_state,
                        "a",
                        a,
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
const _IMPL_DESERIALIZE_FOR_Lifetimes: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de, 'a> _serde::Deserialize<'de> for Lifetimes<'a> {
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
                        "LifetimeSeq" => _serde::export::Ok(__Field::__field0),
                        "NoLifetimeSeq" => _serde::export::Ok(__Field::__field1),
                        "LifetimeMap" => _serde::export::Ok(__Field::__field2),
                        "NoLifetimeMap" => _serde::export::Ok(__Field::__field3),
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
                        b"LifetimeSeq" => _serde::export::Ok(__Field::__field0),
                        b"NoLifetimeSeq" => _serde::export::Ok(__Field::__field1),
                        b"LifetimeMap" => _serde::export::Ok(__Field::__field2),
                        b"NoLifetimeMap" => _serde::export::Ok(__Field::__field3),
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
            struct __Visitor<'de, 'a> {
                marker: _serde::export::PhantomData<Lifetimes<'a>>,
                lifetime: _serde::export::PhantomData<&'de ()>,
            }
            impl<'de, 'a> _serde::de::Visitor<'de> for __Visitor<'de, 'a> {
                type Value = Lifetimes<'a>;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::export::Formatter,
                ) -> _serde::export::fmt::Result {
                    _serde::export::Formatter::write_str(__formatter, "enum Lifetimes")
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
                        (__Field::__field0, __variant) => _serde::export::Result::map(
                            _serde::de::VariantAccess::newtype_variant::<&'a i32>(__variant),
                            Lifetimes::LifetimeSeq,
                        ),
                        (__Field::__field1, __variant) => _serde::export::Result::map(
                            _serde::de::VariantAccess::newtype_variant::<i32>(__variant),
                            Lifetimes::NoLifetimeSeq,
                        ),
                        (__Field::__field2, __variant) => {
                            #[allow(non_camel_case_types)]
                            enum __Field {
                                __field0,
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
                                        _ => _serde::export::Err(_serde::de::Error::invalid_value(
                                            _serde::de::Unexpected::Unsigned(__value),
                                            &"field index 0 <= i < 1",
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
                                        "a" => _serde::export::Ok(__Field::__field0),
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
                                        b"a" => _serde::export::Ok(__Field::__field0),
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
                            struct __Visitor<'de, 'a> {
                                marker: _serde::export::PhantomData<Lifetimes<'a>>,
                                lifetime: _serde::export::PhantomData<&'de ()>,
                            }
                            impl<'de, 'a> _serde::de::Visitor<'de> for __Visitor<'de, 'a> {
                                type Value = Lifetimes<'a>;
                                fn expecting(
                                    &self,
                                    __formatter: &mut _serde::export::Formatter,
                                ) -> _serde::export::fmt::Result {
                                    _serde::export::Formatter::write_str(
                                        __formatter,
                                        "struct variant Lifetimes::LifetimeMap",
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
                                        &'a i32,
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
                                            return _serde :: export :: Err ( _serde :: de :: Error :: invalid_length ( 0usize , & "struct variant Lifetimes::LifetimeMap with 1 element" ) ) ;
                                        }
                                    };
                                    _serde::export::Ok(Lifetimes::LifetimeMap { a: __field0 })
                                }
                                #[inline]
                                fn visit_map<__A>(
                                    self,
                                    mut __map: __A,
                                ) -> _serde::export::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::MapAccess<'de>,
                                {
                                    let mut __field0: _serde::export::Option<&'a i32> =
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
                                                    return _serde :: export :: Err ( < __A :: Error as _serde :: de :: Error > :: duplicate_field ( "a" ) ) ;
                                                }
                                                __field0 = _serde::export::Some(
                                                    match _serde::de::MapAccess::next_value::<&'a i32>(
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
                                            match _serde::private::de::missing_field("a") {
                                                _serde::export::Ok(__val) => __val,
                                                _serde::export::Err(__err) => {
                                                    return _serde::export::Err(__err);
                                                }
                                            }
                                        }
                                    };
                                    _serde::export::Ok(Lifetimes::LifetimeMap { a: __field0 })
                                }
                            }
                            const FIELDS: &'static [&'static str] = &["a"];
                            _serde::de::VariantAccess::struct_variant(
                                __variant,
                                FIELDS,
                                __Visitor {
                                    marker: _serde::export::PhantomData::<Lifetimes<'a>>,
                                    lifetime: _serde::export::PhantomData,
                                },
                            )
                        }
                        (__Field::__field3, __variant) => {
                            #[allow(non_camel_case_types)]
                            enum __Field {
                                __field0,
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
                                        _ => _serde::export::Err(_serde::de::Error::invalid_value(
                                            _serde::de::Unexpected::Unsigned(__value),
                                            &"field index 0 <= i < 1",
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
                                        "a" => _serde::export::Ok(__Field::__field0),
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
                                        b"a" => _serde::export::Ok(__Field::__field0),
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
                            struct __Visitor<'de, 'a> {
                                marker: _serde::export::PhantomData<Lifetimes<'a>>,
                                lifetime: _serde::export::PhantomData<&'de ()>,
                            }
                            impl<'de, 'a> _serde::de::Visitor<'de> for __Visitor<'de, 'a> {
                                type Value = Lifetimes<'a>;
                                fn expecting(
                                    &self,
                                    __formatter: &mut _serde::export::Formatter,
                                ) -> _serde::export::fmt::Result {
                                    _serde::export::Formatter::write_str(
                                        __formatter,
                                        "struct variant Lifetimes::NoLifetimeMap",
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
                                        i32,
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
                                            return _serde :: export :: Err ( _serde :: de :: Error :: invalid_length ( 0usize , & "struct variant Lifetimes::NoLifetimeMap with 1 element" ) ) ;
                                        }
                                    };
                                    _serde::export::Ok(Lifetimes::NoLifetimeMap { a: __field0 })
                                }
                                #[inline]
                                fn visit_map<__A>(
                                    self,
                                    mut __map: __A,
                                ) -> _serde::export::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::MapAccess<'de>,
                                {
                                    let mut __field0: _serde::export::Option<i32> =
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
                                                    return _serde :: export :: Err ( < __A :: Error as _serde :: de :: Error > :: duplicate_field ( "a" ) ) ;
                                                }
                                                __field0 = _serde::export::Some(
                                                    match _serde::de::MapAccess::next_value::<i32>(
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
                                            match _serde::private::de::missing_field("a") {
                                                _serde::export::Ok(__val) => __val,
                                                _serde::export::Err(__err) => {
                                                    return _serde::export::Err(__err);
                                                }
                                            }
                                        }
                                    };
                                    _serde::export::Ok(Lifetimes::NoLifetimeMap { a: __field0 })
                                }
                            }
                            const FIELDS: &'static [&'static str] = &["a"];
                            _serde::de::VariantAccess::struct_variant(
                                __variant,
                                FIELDS,
                                __Visitor {
                                    marker: _serde::export::PhantomData::<Lifetimes<'a>>,
                                    lifetime: _serde::export::PhantomData,
                                },
                            )
                        }
                    }
                }
            }
            const VARIANTS: &'static [&'static str] = &[
                "LifetimeSeq",
                "NoLifetimeSeq",
                "LifetimeMap",
                "NoLifetimeMap",
            ];
            _serde::Deserializer::deserialize_enum(
                __deserializer,
                "Lifetimes",
                VARIANTS,
                __Visitor {
                    marker: _serde::export::PhantomData::<Lifetimes<'a>>,
                    lifetime: _serde::export::PhantomData,
                },
            )
        }
    }
};
