#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2018::*;
#[macro_use]
extern crate std;
mod expand {
    mod enum_adjacently_tagged {
        use serde_derive::{Deserialize, Serialize};
        #[serde(tag = "tag", content = "content")]
        pub enum AdjacentlyTaggedEnum<T, U> {
            Unit,
            NewType(T),
            Seq(T, U),
            Map { x: T, y: U },
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<T, U> _serde::Serialize for AdjacentlyTaggedEnum<T, U>
            where
                T: _serde::Serialize,
                U: _serde::Serialize,
            {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private::Result<__S::Ok, __S::Error>
                where
                    __S: _serde::Serializer,
                {
                    match *self {
                        AdjacentlyTaggedEnum::Unit => {
                            let mut __struct = _serde::Serializer::serialize_struct(
                                __serializer,
                                "AdjacentlyTaggedEnum",
                                1,
                            )?;
                            _serde::ser::SerializeStruct::serialize_field(
                                &mut __struct,
                                "tag",
                                "Unit",
                            )?;
                            _serde::ser::SerializeStruct::end(__struct)
                        }
                        AdjacentlyTaggedEnum::NewType(ref __field0) => {
                            let mut __struct = _serde::Serializer::serialize_struct(
                                __serializer,
                                "AdjacentlyTaggedEnum",
                                2,
                            )?;
                            _serde::ser::SerializeStruct::serialize_field(
                                &mut __struct,
                                "tag",
                                "NewType",
                            )?;
                            _serde::ser::SerializeStruct::serialize_field(
                                &mut __struct,
                                "content",
                                __field0,
                            )?;
                            _serde::ser::SerializeStruct::end(__struct)
                        }
                        AdjacentlyTaggedEnum::Seq(ref __field0, ref __field1) => {
                            #[doc(hidden)]
                            struct __AdjacentlyTagged<'__a, T: '__a, U: '__a>
                            where
                                T: _serde::Serialize,
                                U: _serde::Serialize,
                            {
                                data: (&'__a T, &'__a U),
                                phantom: _serde::__private::PhantomData<
                                    AdjacentlyTaggedEnum<T, U>,
                                >,
                            }
                            impl<'__a, T: '__a, U: '__a> _serde::Serialize
                            for __AdjacentlyTagged<'__a, T, U>
                            where
                                T: _serde::Serialize,
                                U: _serde::Serialize,
                            {
                                fn serialize<__S>(
                                    &self,
                                    __serializer: __S,
                                ) -> _serde::__private::Result<__S::Ok, __S::Error>
                                where
                                    __S: _serde::Serializer,
                                {
                                    #[allow(unused_variables)]
                                    let (__field0, __field1) = self.data;
                                    let mut __serde_state = _serde::Serializer::serialize_tuple(
                                        __serializer,
                                        0 + 1 + 1,
                                    )?;
                                    _serde::ser::SerializeTuple::serialize_element(
                                        &mut __serde_state,
                                        __field0,
                                    )?;
                                    _serde::ser::SerializeTuple::serialize_element(
                                        &mut __serde_state,
                                        __field1,
                                    )?;
                                    _serde::ser::SerializeTuple::end(__serde_state)
                                }
                            }
                            let mut __struct = _serde::Serializer::serialize_struct(
                                __serializer,
                                "AdjacentlyTaggedEnum",
                                2,
                            )?;
                            _serde::ser::SerializeStruct::serialize_field(
                                &mut __struct,
                                "tag",
                                "Seq",
                            )?;
                            _serde::ser::SerializeStruct::serialize_field(
                                &mut __struct,
                                "content",
                                &__AdjacentlyTagged {
                                    data: (__field0, __field1),
                                    phantom: _serde::__private::PhantomData::<
                                        AdjacentlyTaggedEnum<T, U>,
                                    >,
                                },
                            )?;
                            _serde::ser::SerializeStruct::end(__struct)
                        }
                        AdjacentlyTaggedEnum::Map { ref x, ref y } => {
                            #[doc(hidden)]
                            struct __AdjacentlyTagged<'__a, T: '__a, U: '__a>
                            where
                                T: _serde::Serialize,
                                U: _serde::Serialize,
                            {
                                data: (&'__a T, &'__a U),
                                phantom: _serde::__private::PhantomData<
                                    AdjacentlyTaggedEnum<T, U>,
                                >,
                            }
                            impl<'__a, T: '__a, U: '__a> _serde::Serialize
                            for __AdjacentlyTagged<'__a, T, U>
                            where
                                T: _serde::Serialize,
                                U: _serde::Serialize,
                            {
                                fn serialize<__S>(
                                    &self,
                                    __serializer: __S,
                                ) -> _serde::__private::Result<__S::Ok, __S::Error>
                                where
                                    __S: _serde::Serializer,
                                {
                                    #[allow(unused_variables)]
                                    let (x, y) = self.data;
                                    let mut __serde_state = _serde::Serializer::serialize_struct(
                                        __serializer,
                                        "Map",
                                        0 + 1 + 1,
                                    )?;
                                    _serde::ser::SerializeStruct::serialize_field(
                                        &mut __serde_state,
                                        "x",
                                        x,
                                    )?;
                                    _serde::ser::SerializeStruct::serialize_field(
                                        &mut __serde_state,
                                        "y",
                                        y,
                                    )?;
                                    _serde::ser::SerializeStruct::end(__serde_state)
                                }
                            }
                            let mut __struct = _serde::Serializer::serialize_struct(
                                __serializer,
                                "AdjacentlyTaggedEnum",
                                2,
                            )?;
                            _serde::ser::SerializeStruct::serialize_field(
                                &mut __struct,
                                "tag",
                                "Map",
                            )?;
                            _serde::ser::SerializeStruct::serialize_field(
                                &mut __struct,
                                "content",
                                &__AdjacentlyTagged {
                                    data: (x, y),
                                    phantom: _serde::__private::PhantomData::<
                                        AdjacentlyTaggedEnum<T, U>,
                                    >,
                                },
                            )?;
                            _serde::ser::SerializeStruct::end(__struct)
                        }
                    }
                }
            }
        };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de, T, U> _serde::Deserialize<'de> for AdjacentlyTaggedEnum<T, U>
            where
                T: _serde::Deserialize<'de>,
                U: _serde::Deserialize<'de>,
            {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    #[doc(hidden)]
                    enum __Field {
                        __field0,
                        __field1,
                        __field2,
                        __field3,
                    }
                    #[doc(hidden)]
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
                                2u64 => _serde::__private::Ok(__Field::__field2),
                                3u64 => _serde::__private::Ok(__Field::__field3),
                                _ => {
                                    _serde::__private::Err(
                                        _serde::de::Error::invalid_value(
                                            _serde::de::Unexpected::Unsigned(__value),
                                            &"variant index 0 <= i < 4",
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
                                "Unit" => _serde::__private::Ok(__Field::__field0),
                                "NewType" => _serde::__private::Ok(__Field::__field1),
                                "Seq" => _serde::__private::Ok(__Field::__field2),
                                "Map" => _serde::__private::Ok(__Field::__field3),
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
                                b"Unit" => _serde::__private::Ok(__Field::__field0),
                                b"NewType" => _serde::__private::Ok(__Field::__field1),
                                b"Seq" => _serde::__private::Ok(__Field::__field2),
                                b"Map" => _serde::__private::Ok(__Field::__field3),
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
                    #[doc(hidden)]
                    const VARIANTS: &'static [&'static str] = &[
                        "Unit",
                        "NewType",
                        "Seq",
                        "Map",
                    ];
                    #[doc(hidden)]
                    struct __Seed<'de, T, U>
                    where
                        T: _serde::Deserialize<'de>,
                        U: _serde::Deserialize<'de>,
                    {
                        field: __Field,
                        marker: _serde::__private::PhantomData<
                            AdjacentlyTaggedEnum<T, U>,
                        >,
                        lifetime: _serde::__private::PhantomData<&'de ()>,
                    }
                    impl<'de, T, U> _serde::de::DeserializeSeed<'de>
                    for __Seed<'de, T, U>
                    where
                        T: _serde::Deserialize<'de>,
                        U: _serde::Deserialize<'de>,
                    {
                        type Value = AdjacentlyTaggedEnum<T, U>;
                        fn deserialize<__D>(
                            self,
                            __deserializer: __D,
                        ) -> _serde::__private::Result<Self::Value, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                        {
                            match self.field {
                                __Field::__field0 => {
                                    match _serde::Deserializer::deserialize_any(
                                        __deserializer,
                                        _serde::__private::de::UntaggedUnitVisitor::new(
                                            "AdjacentlyTaggedEnum",
                                            "Unit",
                                        ),
                                    ) {
                                        _serde::__private::Ok(()) => {
                                            _serde::__private::Ok(AdjacentlyTaggedEnum::Unit)
                                        }
                                        _serde::__private::Err(__err) => {
                                            _serde::__private::Err(__err)
                                        }
                                    }
                                }
                                __Field::__field1 => {
                                    _serde::__private::Result::map(
                                        <T as _serde::Deserialize>::deserialize(__deserializer),
                                        AdjacentlyTaggedEnum::NewType,
                                    )
                                }
                                __Field::__field2 => {
                                    #[doc(hidden)]
                                    struct __Visitor<'de, T, U>
                                    where
                                        T: _serde::Deserialize<'de>,
                                        U: _serde::Deserialize<'de>,
                                    {
                                        marker: _serde::__private::PhantomData<
                                            AdjacentlyTaggedEnum<T, U>,
                                        >,
                                        lifetime: _serde::__private::PhantomData<&'de ()>,
                                    }
                                    impl<'de, T, U> _serde::de::Visitor<'de>
                                    for __Visitor<'de, T, U>
                                    where
                                        T: _serde::Deserialize<'de>,
                                        U: _serde::Deserialize<'de>,
                                    {
                                        type Value = AdjacentlyTaggedEnum<T, U>;
                                        fn expecting(
                                            &self,
                                            __formatter: &mut _serde::__private::Formatter,
                                        ) -> _serde::__private::fmt::Result {
                                            _serde::__private::Formatter::write_str(
                                                __formatter,
                                                "tuple variant AdjacentlyTaggedEnum::Seq",
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
                                            let __field0 = match _serde::de::SeqAccess::next_element::<
                                                T,
                                            >(&mut __seq)? {
                                                _serde::__private::Some(__value) => __value,
                                                _serde::__private::None => {
                                                    return _serde::__private::Err(
                                                        _serde::de::Error::invalid_length(
                                                            0usize,
                                                            &"tuple variant AdjacentlyTaggedEnum::Seq with 2 elements",
                                                        ),
                                                    );
                                                }
                                            };
                                            let __field1 = match _serde::de::SeqAccess::next_element::<
                                                U,
                                            >(&mut __seq)? {
                                                _serde::__private::Some(__value) => __value,
                                                _serde::__private::None => {
                                                    return _serde::__private::Err(
                                                        _serde::de::Error::invalid_length(
                                                            1usize,
                                                            &"tuple variant AdjacentlyTaggedEnum::Seq with 2 elements",
                                                        ),
                                                    );
                                                }
                                            };
                                            _serde::__private::Ok(
                                                AdjacentlyTaggedEnum::Seq(__field0, __field1),
                                            )
                                        }
                                    }
                                    _serde::Deserializer::deserialize_tuple(
                                        __deserializer,
                                        2usize,
                                        __Visitor {
                                            marker: _serde::__private::PhantomData::<
                                                AdjacentlyTaggedEnum<T, U>,
                                            >,
                                            lifetime: _serde::__private::PhantomData,
                                        },
                                    )
                                }
                                __Field::__field3 => {
                                    #[allow(non_camel_case_types)]
                                    #[doc(hidden)]
                                    enum __Field {
                                        __field0,
                                        __field1,
                                        __ignore,
                                    }
                                    #[doc(hidden)]
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
                                                0u64 => _serde::__private::Ok(__Field::__field0),
                                                1u64 => _serde::__private::Ok(__Field::__field1),
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
                                                "x" => _serde::__private::Ok(__Field::__field0),
                                                "y" => _serde::__private::Ok(__Field::__field1),
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
                                                b"x" => _serde::__private::Ok(__Field::__field0),
                                                b"y" => _serde::__private::Ok(__Field::__field1),
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
                                    #[doc(hidden)]
                                    struct __Visitor<'de, T, U>
                                    where
                                        T: _serde::Deserialize<'de>,
                                        U: _serde::Deserialize<'de>,
                                    {
                                        marker: _serde::__private::PhantomData<
                                            AdjacentlyTaggedEnum<T, U>,
                                        >,
                                        lifetime: _serde::__private::PhantomData<&'de ()>,
                                    }
                                    impl<'de, T, U> _serde::de::Visitor<'de>
                                    for __Visitor<'de, T, U>
                                    where
                                        T: _serde::Deserialize<'de>,
                                        U: _serde::Deserialize<'de>,
                                    {
                                        type Value = AdjacentlyTaggedEnum<T, U>;
                                        fn expecting(
                                            &self,
                                            __formatter: &mut _serde::__private::Formatter,
                                        ) -> _serde::__private::fmt::Result {
                                            _serde::__private::Formatter::write_str(
                                                __formatter,
                                                "struct variant AdjacentlyTaggedEnum::Map",
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
                                            let mut __field0: _serde::__private::Option<T> = _serde::__private::None;
                                            let mut __field1: _serde::__private::Option<U> = _serde::__private::None;
                                            while let _serde::__private::Some(__key)
                                                = _serde::de::MapAccess::next_key::<__Field>(&mut __map)? {
                                                match __key {
                                                    __Field::__field0 => {
                                                        if _serde::__private::Option::is_some(&__field0) {
                                                            return _serde::__private::Err(
                                                                <__A::Error as _serde::de::Error>::duplicate_field("x"),
                                                            );
                                                        }
                                                        __field0 = _serde::__private::Some(
                                                            _serde::de::MapAccess::next_value::<T>(&mut __map)?,
                                                        );
                                                    }
                                                    __Field::__field1 => {
                                                        if _serde::__private::Option::is_some(&__field1) {
                                                            return _serde::__private::Err(
                                                                <__A::Error as _serde::de::Error>::duplicate_field("y"),
                                                            );
                                                        }
                                                        __field1 = _serde::__private::Some(
                                                            _serde::de::MapAccess::next_value::<U>(&mut __map)?,
                                                        );
                                                    }
                                                    _ => {
                                                        let _ = _serde::de::MapAccess::next_value::<
                                                            _serde::de::IgnoredAny,
                                                        >(&mut __map)?;
                                                    }
                                                }
                                            }
                                            let __field0 = match __field0 {
                                                _serde::__private::Some(__field0) => __field0,
                                                _serde::__private::None => {
                                                    _serde::__private::de::missing_field("x")?
                                                }
                                            };
                                            let __field1 = match __field1 {
                                                _serde::__private::Some(__field1) => __field1,
                                                _serde::__private::None => {
                                                    _serde::__private::de::missing_field("y")?
                                                }
                                            };
                                            _serde::__private::Ok(AdjacentlyTaggedEnum::Map {
                                                x: __field0,
                                                y: __field1,
                                            })
                                        }
                                    }
                                    #[doc(hidden)]
                                    const FIELDS: &'static [&'static str] = &["x", "y"];
                                    _serde::Deserializer::deserialize_any(
                                        __deserializer,
                                        __Visitor {
                                            marker: _serde::__private::PhantomData::<
                                                AdjacentlyTaggedEnum<T, U>,
                                            >,
                                            lifetime: _serde::__private::PhantomData,
                                        },
                                    )
                                }
                            }
                        }
                    }
                    #[doc(hidden)]
                    struct __Visitor<'de, T, U>
                    where
                        T: _serde::Deserialize<'de>,
                        U: _serde::Deserialize<'de>,
                    {
                        marker: _serde::__private::PhantomData<
                            AdjacentlyTaggedEnum<T, U>,
                        >,
                        lifetime: _serde::__private::PhantomData<&'de ()>,
                    }
                    impl<'de, T, U> _serde::de::Visitor<'de> for __Visitor<'de, T, U>
                    where
                        T: _serde::Deserialize<'de>,
                        U: _serde::Deserialize<'de>,
                    {
                        type Value = AdjacentlyTaggedEnum<T, U>;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(
                                __formatter,
                                "adjacently tagged enum AdjacentlyTaggedEnum",
                            )
                        }
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::MapAccess<'de>,
                        {
                            match {
                                let mut __rk: _serde::__private::Option<
                                    _serde::__private::de::TagOrContentField,
                                > = _serde::__private::None;
                                while let _serde::__private::Some(__k)
                                    = _serde::de::MapAccess::next_key_seed(
                                        &mut __map,
                                        _serde::__private::de::TagContentOtherFieldVisitor {
                                            tag: "tag",
                                            content: "content",
                                        },
                                    )? {
                                    match __k {
                                        _serde::__private::de::TagContentOtherField::Other => {
                                            let _ = _serde::de::MapAccess::next_value::<
                                                _serde::de::IgnoredAny,
                                            >(&mut __map)?;
                                            continue;
                                        }
                                        _serde::__private::de::TagContentOtherField::Tag => {
                                            __rk = _serde::__private::Some(
                                                _serde::__private::de::TagOrContentField::Tag,
                                            );
                                            break;
                                        }
                                        _serde::__private::de::TagContentOtherField::Content => {
                                            __rk = _serde::__private::Some(
                                                _serde::__private::de::TagOrContentField::Content,
                                            );
                                            break;
                                        }
                                    }
                                }
                                __rk
                            } {
                                _serde::__private::Some(
                                    _serde::__private::de::TagOrContentField::Tag,
                                ) => {
                                    let __field = _serde::de::MapAccess::next_value(
                                        &mut __map,
                                    )?;
                                    match {
                                        let mut __rk: _serde::__private::Option<
                                            _serde::__private::de::TagOrContentField,
                                        > = _serde::__private::None;
                                        while let _serde::__private::Some(__k)
                                            = _serde::de::MapAccess::next_key_seed(
                                                &mut __map,
                                                _serde::__private::de::TagContentOtherFieldVisitor {
                                                    tag: "tag",
                                                    content: "content",
                                                },
                                            )? {
                                            match __k {
                                                _serde::__private::de::TagContentOtherField::Other => {
                                                    let _ = _serde::de::MapAccess::next_value::<
                                                        _serde::de::IgnoredAny,
                                                    >(&mut __map)?;
                                                    continue;
                                                }
                                                _serde::__private::de::TagContentOtherField::Tag => {
                                                    __rk = _serde::__private::Some(
                                                        _serde::__private::de::TagOrContentField::Tag,
                                                    );
                                                    break;
                                                }
                                                _serde::__private::de::TagContentOtherField::Content => {
                                                    __rk = _serde::__private::Some(
                                                        _serde::__private::de::TagOrContentField::Content,
                                                    );
                                                    break;
                                                }
                                            }
                                        }
                                        __rk
                                    } {
                                        _serde::__private::Some(
                                            _serde::__private::de::TagOrContentField::Tag,
                                        ) => {
                                            _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("tag"),
                                            )
                                        }
                                        _serde::__private::Some(
                                            _serde::__private::de::TagOrContentField::Content,
                                        ) => {
                                            let __ret = _serde::de::MapAccess::next_value_seed(
                                                &mut __map,
                                                __Seed {
                                                    field: __field,
                                                    marker: _serde::__private::PhantomData,
                                                    lifetime: _serde::__private::PhantomData,
                                                },
                                            )?;
                                            match {
                                                let mut __rk: _serde::__private::Option<
                                                    _serde::__private::de::TagOrContentField,
                                                > = _serde::__private::None;
                                                while let _serde::__private::Some(__k)
                                                    = _serde::de::MapAccess::next_key_seed(
                                                        &mut __map,
                                                        _serde::__private::de::TagContentOtherFieldVisitor {
                                                            tag: "tag",
                                                            content: "content",
                                                        },
                                                    )? {
                                                    match __k {
                                                        _serde::__private::de::TagContentOtherField::Other => {
                                                            let _ = _serde::de::MapAccess::next_value::<
                                                                _serde::de::IgnoredAny,
                                                            >(&mut __map)?;
                                                            continue;
                                                        }
                                                        _serde::__private::de::TagContentOtherField::Tag => {
                                                            __rk = _serde::__private::Some(
                                                                _serde::__private::de::TagOrContentField::Tag,
                                                            );
                                                            break;
                                                        }
                                                        _serde::__private::de::TagContentOtherField::Content => {
                                                            __rk = _serde::__private::Some(
                                                                _serde::__private::de::TagOrContentField::Content,
                                                            );
                                                            break;
                                                        }
                                                    }
                                                }
                                                __rk
                                            } {
                                                _serde::__private::Some(
                                                    _serde::__private::de::TagOrContentField::Tag,
                                                ) => {
                                                    _serde::__private::Err(
                                                        <__A::Error as _serde::de::Error>::duplicate_field("tag"),
                                                    )
                                                }
                                                _serde::__private::Some(
                                                    _serde::__private::de::TagOrContentField::Content,
                                                ) => {
                                                    _serde::__private::Err(
                                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                                            "content",
                                                        ),
                                                    )
                                                }
                                                _serde::__private::None => _serde::__private::Ok(__ret),
                                            }
                                        }
                                        _serde::__private::None => {
                                            match __field {
                                                __Field::__field0 => {
                                                    _serde::__private::Ok(AdjacentlyTaggedEnum::Unit)
                                                }
                                                __Field::__field1 => {
                                                    _serde::__private::de::missing_field("content")
                                                        .map(AdjacentlyTaggedEnum::NewType)
                                                }
                                                _ => {
                                                    _serde::__private::Err(
                                                        <__A::Error as _serde::de::Error>::missing_field("content"),
                                                    )
                                                }
                                            }
                                        }
                                    }
                                }
                                _serde::__private::Some(
                                    _serde::__private::de::TagOrContentField::Content,
                                ) => {
                                    let __content = _serde::de::MapAccess::next_value::<
                                        _serde::__private::de::Content,
                                    >(&mut __map)?;
                                    match {
                                        let mut __rk: _serde::__private::Option<
                                            _serde::__private::de::TagOrContentField,
                                        > = _serde::__private::None;
                                        while let _serde::__private::Some(__k)
                                            = _serde::de::MapAccess::next_key_seed(
                                                &mut __map,
                                                _serde::__private::de::TagContentOtherFieldVisitor {
                                                    tag: "tag",
                                                    content: "content",
                                                },
                                            )? {
                                            match __k {
                                                _serde::__private::de::TagContentOtherField::Other => {
                                                    let _ = _serde::de::MapAccess::next_value::<
                                                        _serde::de::IgnoredAny,
                                                    >(&mut __map)?;
                                                    continue;
                                                }
                                                _serde::__private::de::TagContentOtherField::Tag => {
                                                    __rk = _serde::__private::Some(
                                                        _serde::__private::de::TagOrContentField::Tag,
                                                    );
                                                    break;
                                                }
                                                _serde::__private::de::TagContentOtherField::Content => {
                                                    __rk = _serde::__private::Some(
                                                        _serde::__private::de::TagOrContentField::Content,
                                                    );
                                                    break;
                                                }
                                            }
                                        }
                                        __rk
                                    } {
                                        _serde::__private::Some(
                                            _serde::__private::de::TagOrContentField::Tag,
                                        ) => {
                                            let __deserializer = _serde::__private::de::ContentDeserializer::<
                                                __A::Error,
                                            >::new(__content);
                                            let __ret = match _serde::de::MapAccess::next_value(
                                                &mut __map,
                                            )? {
                                                __Field::__field0 => {
                                                    match _serde::Deserializer::deserialize_any(
                                                        __deserializer,
                                                        _serde::__private::de::UntaggedUnitVisitor::new(
                                                            "AdjacentlyTaggedEnum",
                                                            "Unit",
                                                        ),
                                                    ) {
                                                        _serde::__private::Ok(()) => {
                                                            _serde::__private::Ok(AdjacentlyTaggedEnum::Unit)
                                                        }
                                                        _serde::__private::Err(__err) => {
                                                            _serde::__private::Err(__err)
                                                        }
                                                    }
                                                }
                                                __Field::__field1 => {
                                                    _serde::__private::Result::map(
                                                        <T as _serde::Deserialize>::deserialize(__deserializer),
                                                        AdjacentlyTaggedEnum::NewType,
                                                    )
                                                }
                                                __Field::__field2 => {
                                                    #[doc(hidden)]
                                                    struct __Visitor<'de, T, U>
                                                    where
                                                        T: _serde::Deserialize<'de>,
                                                        U: _serde::Deserialize<'de>,
                                                    {
                                                        marker: _serde::__private::PhantomData<
                                                            AdjacentlyTaggedEnum<T, U>,
                                                        >,
                                                        lifetime: _serde::__private::PhantomData<&'de ()>,
                                                    }
                                                    impl<'de, T, U> _serde::de::Visitor<'de>
                                                    for __Visitor<'de, T, U>
                                                    where
                                                        T: _serde::Deserialize<'de>,
                                                        U: _serde::Deserialize<'de>,
                                                    {
                                                        type Value = AdjacentlyTaggedEnum<T, U>;
                                                        fn expecting(
                                                            &self,
                                                            __formatter: &mut _serde::__private::Formatter,
                                                        ) -> _serde::__private::fmt::Result {
                                                            _serde::__private::Formatter::write_str(
                                                                __formatter,
                                                                "tuple variant AdjacentlyTaggedEnum::Seq",
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
                                                            let __field0 = match _serde::de::SeqAccess::next_element::<
                                                                T,
                                                            >(&mut __seq)? {
                                                                _serde::__private::Some(__value) => __value,
                                                                _serde::__private::None => {
                                                                    return _serde::__private::Err(
                                                                        _serde::de::Error::invalid_length(
                                                                            0usize,
                                                                            &"tuple variant AdjacentlyTaggedEnum::Seq with 2 elements",
                                                                        ),
                                                                    );
                                                                }
                                                            };
                                                            let __field1 = match _serde::de::SeqAccess::next_element::<
                                                                U,
                                                            >(&mut __seq)? {
                                                                _serde::__private::Some(__value) => __value,
                                                                _serde::__private::None => {
                                                                    return _serde::__private::Err(
                                                                        _serde::de::Error::invalid_length(
                                                                            1usize,
                                                                            &"tuple variant AdjacentlyTaggedEnum::Seq with 2 elements",
                                                                        ),
                                                                    );
                                                                }
                                                            };
                                                            _serde::__private::Ok(
                                                                AdjacentlyTaggedEnum::Seq(__field0, __field1),
                                                            )
                                                        }
                                                    }
                                                    _serde::Deserializer::deserialize_tuple(
                                                        __deserializer,
                                                        2usize,
                                                        __Visitor {
                                                            marker: _serde::__private::PhantomData::<
                                                                AdjacentlyTaggedEnum<T, U>,
                                                            >,
                                                            lifetime: _serde::__private::PhantomData,
                                                        },
                                                    )
                                                }
                                                __Field::__field3 => {
                                                    #[allow(non_camel_case_types)]
                                                    #[doc(hidden)]
                                                    enum __Field {
                                                        __field0,
                                                        __field1,
                                                        __ignore,
                                                    }
                                                    #[doc(hidden)]
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
                                                                0u64 => _serde::__private::Ok(__Field::__field0),
                                                                1u64 => _serde::__private::Ok(__Field::__field1),
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
                                                                "x" => _serde::__private::Ok(__Field::__field0),
                                                                "y" => _serde::__private::Ok(__Field::__field1),
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
                                                                b"x" => _serde::__private::Ok(__Field::__field0),
                                                                b"y" => _serde::__private::Ok(__Field::__field1),
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
                                                    #[doc(hidden)]
                                                    struct __Visitor<'de, T, U>
                                                    where
                                                        T: _serde::Deserialize<'de>,
                                                        U: _serde::Deserialize<'de>,
                                                    {
                                                        marker: _serde::__private::PhantomData<
                                                            AdjacentlyTaggedEnum<T, U>,
                                                        >,
                                                        lifetime: _serde::__private::PhantomData<&'de ()>,
                                                    }
                                                    impl<'de, T, U> _serde::de::Visitor<'de>
                                                    for __Visitor<'de, T, U>
                                                    where
                                                        T: _serde::Deserialize<'de>,
                                                        U: _serde::Deserialize<'de>,
                                                    {
                                                        type Value = AdjacentlyTaggedEnum<T, U>;
                                                        fn expecting(
                                                            &self,
                                                            __formatter: &mut _serde::__private::Formatter,
                                                        ) -> _serde::__private::fmt::Result {
                                                            _serde::__private::Formatter::write_str(
                                                                __formatter,
                                                                "struct variant AdjacentlyTaggedEnum::Map",
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
                                                            let mut __field0: _serde::__private::Option<T> = _serde::__private::None;
                                                            let mut __field1: _serde::__private::Option<U> = _serde::__private::None;
                                                            while let _serde::__private::Some(__key)
                                                                = _serde::de::MapAccess::next_key::<__Field>(&mut __map)? {
                                                                match __key {
                                                                    __Field::__field0 => {
                                                                        if _serde::__private::Option::is_some(&__field0) {
                                                                            return _serde::__private::Err(
                                                                                <__A::Error as _serde::de::Error>::duplicate_field("x"),
                                                                            );
                                                                        }
                                                                        __field0 = _serde::__private::Some(
                                                                            _serde::de::MapAccess::next_value::<T>(&mut __map)?,
                                                                        );
                                                                    }
                                                                    __Field::__field1 => {
                                                                        if _serde::__private::Option::is_some(&__field1) {
                                                                            return _serde::__private::Err(
                                                                                <__A::Error as _serde::de::Error>::duplicate_field("y"),
                                                                            );
                                                                        }
                                                                        __field1 = _serde::__private::Some(
                                                                            _serde::de::MapAccess::next_value::<U>(&mut __map)?,
                                                                        );
                                                                    }
                                                                    _ => {
                                                                        let _ = _serde::de::MapAccess::next_value::<
                                                                            _serde::de::IgnoredAny,
                                                                        >(&mut __map)?;
                                                                    }
                                                                }
                                                            }
                                                            let __field0 = match __field0 {
                                                                _serde::__private::Some(__field0) => __field0,
                                                                _serde::__private::None => {
                                                                    _serde::__private::de::missing_field("x")?
                                                                }
                                                            };
                                                            let __field1 = match __field1 {
                                                                _serde::__private::Some(__field1) => __field1,
                                                                _serde::__private::None => {
                                                                    _serde::__private::de::missing_field("y")?
                                                                }
                                                            };
                                                            _serde::__private::Ok(AdjacentlyTaggedEnum::Map {
                                                                x: __field0,
                                                                y: __field1,
                                                            })
                                                        }
                                                    }
                                                    #[doc(hidden)]
                                                    const FIELDS: &'static [&'static str] = &["x", "y"];
                                                    _serde::Deserializer::deserialize_any(
                                                        __deserializer,
                                                        __Visitor {
                                                            marker: _serde::__private::PhantomData::<
                                                                AdjacentlyTaggedEnum<T, U>,
                                                            >,
                                                            lifetime: _serde::__private::PhantomData,
                                                        },
                                                    )
                                                }
                                            }?;
                                            match {
                                                let mut __rk: _serde::__private::Option<
                                                    _serde::__private::de::TagOrContentField,
                                                > = _serde::__private::None;
                                                while let _serde::__private::Some(__k)
                                                    = _serde::de::MapAccess::next_key_seed(
                                                        &mut __map,
                                                        _serde::__private::de::TagContentOtherFieldVisitor {
                                                            tag: "tag",
                                                            content: "content",
                                                        },
                                                    )? {
                                                    match __k {
                                                        _serde::__private::de::TagContentOtherField::Other => {
                                                            let _ = _serde::de::MapAccess::next_value::<
                                                                _serde::de::IgnoredAny,
                                                            >(&mut __map)?;
                                                            continue;
                                                        }
                                                        _serde::__private::de::TagContentOtherField::Tag => {
                                                            __rk = _serde::__private::Some(
                                                                _serde::__private::de::TagOrContentField::Tag,
                                                            );
                                                            break;
                                                        }
                                                        _serde::__private::de::TagContentOtherField::Content => {
                                                            __rk = _serde::__private::Some(
                                                                _serde::__private::de::TagOrContentField::Content,
                                                            );
                                                            break;
                                                        }
                                                    }
                                                }
                                                __rk
                                            } {
                                                _serde::__private::Some(
                                                    _serde::__private::de::TagOrContentField::Tag,
                                                ) => {
                                                    _serde::__private::Err(
                                                        <__A::Error as _serde::de::Error>::duplicate_field("tag"),
                                                    )
                                                }
                                                _serde::__private::Some(
                                                    _serde::__private::de::TagOrContentField::Content,
                                                ) => {
                                                    _serde::__private::Err(
                                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                                            "content",
                                                        ),
                                                    )
                                                }
                                                _serde::__private::None => _serde::__private::Ok(__ret),
                                            }
                                        }
                                        _serde::__private::Some(
                                            _serde::__private::de::TagOrContentField::Content,
                                        ) => {
                                            _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "content",
                                                ),
                                            )
                                        }
                                        _serde::__private::None => {
                                            _serde::__private::Err(
                                                <__A::Error as _serde::de::Error>::missing_field("tag"),
                                            )
                                        }
                                    }
                                }
                                _serde::__private::None => {
                                    _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::missing_field("tag"),
                                    )
                                }
                            }
                        }
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::SeqAccess<'de>,
                        {
                            match _serde::de::SeqAccess::next_element(&mut __seq)? {
                                _serde::__private::Some(__field) => {
                                    match _serde::de::SeqAccess::next_element_seed(
                                        &mut __seq,
                                        __Seed {
                                            field: __field,
                                            marker: _serde::__private::PhantomData,
                                            lifetime: _serde::__private::PhantomData,
                                        },
                                    )? {
                                        _serde::__private::Some(__ret) => {
                                            _serde::__private::Ok(__ret)
                                        }
                                        _serde::__private::None => {
                                            _serde::__private::Err(
                                                _serde::de::Error::invalid_length(1, &self),
                                            )
                                        }
                                    }
                                }
                                _serde::__private::None => {
                                    _serde::__private::Err(
                                        _serde::de::Error::invalid_length(0, &self),
                                    )
                                }
                            }
                        }
                    }
                    #[doc(hidden)]
                    const FIELDS: &'static [&'static str] = &["tag", "content"];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "AdjacentlyTaggedEnum",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private::PhantomData::<
                                AdjacentlyTaggedEnum<T, U>,
                            >,
                            lifetime: _serde::__private::PhantomData,
                        },
                    )
                }
            }
        };
    }
    mod enum_internally_tagged {
        use serde_derive::{Deserialize, Serialize};
        #[serde(tag = "tag")]
        pub enum InternallyTaggedEnum<T, U> {
            Unit,
            NewType(T),
            Map { x: T, y: U },
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<T, U> _serde::Serialize for InternallyTaggedEnum<T, U>
            where
                T: _serde::Serialize,
                U: _serde::Serialize,
            {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private::Result<__S::Ok, __S::Error>
                where
                    __S: _serde::Serializer,
                {
                    match *self {
                        InternallyTaggedEnum::Unit => {
                            let mut __struct = _serde::Serializer::serialize_struct(
                                __serializer,
                                "InternallyTaggedEnum",
                                1,
                            )?;
                            _serde::ser::SerializeStruct::serialize_field(
                                &mut __struct,
                                "tag",
                                "Unit",
                            )?;
                            _serde::ser::SerializeStruct::end(__struct)
                        }
                        InternallyTaggedEnum::NewType(ref __field0) => {
                            _serde::__private::ser::serialize_tagged_newtype(
                                __serializer,
                                "InternallyTaggedEnum",
                                "NewType",
                                "tag",
                                "NewType",
                                __field0,
                            )
                        }
                        InternallyTaggedEnum::Map { ref x, ref y } => {
                            let mut __serde_state = _serde::Serializer::serialize_struct(
                                __serializer,
                                "InternallyTaggedEnum",
                                0 + 1 + 1 + 1,
                            )?;
                            _serde::ser::SerializeStruct::serialize_field(
                                &mut __serde_state,
                                "tag",
                                "Map",
                            )?;
                            _serde::ser::SerializeStruct::serialize_field(
                                &mut __serde_state,
                                "x",
                                x,
                            )?;
                            _serde::ser::SerializeStruct::serialize_field(
                                &mut __serde_state,
                                "y",
                                y,
                            )?;
                            _serde::ser::SerializeStruct::end(__serde_state)
                        }
                    }
                }
            }
        };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de, T, U> _serde::Deserialize<'de> for InternallyTaggedEnum<T, U>
            where
                T: _serde::Deserialize<'de>,
                U: _serde::Deserialize<'de>,
            {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    #[doc(hidden)]
                    enum __Field {
                        __field0,
                        __field1,
                        __field2,
                    }
                    #[doc(hidden)]
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
                                2u64 => _serde::__private::Ok(__Field::__field2),
                                _ => {
                                    _serde::__private::Err(
                                        _serde::de::Error::invalid_value(
                                            _serde::de::Unexpected::Unsigned(__value),
                                            &"variant index 0 <= i < 3",
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
                                "Unit" => _serde::__private::Ok(__Field::__field0),
                                "NewType" => _serde::__private::Ok(__Field::__field1),
                                "Map" => _serde::__private::Ok(__Field::__field2),
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
                                b"Unit" => _serde::__private::Ok(__Field::__field0),
                                b"NewType" => _serde::__private::Ok(__Field::__field1),
                                b"Map" => _serde::__private::Ok(__Field::__field2),
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
                    #[doc(hidden)]
                    const VARIANTS: &'static [&'static str] = &[
                        "Unit",
                        "NewType",
                        "Map",
                    ];
                    struct __Visitor<'de, T, U>
                    where
                        T: _serde::Deserialize<'de>,
                        U: _serde::Deserialize<'de>,
                    {
                        marker: _serde::__private::PhantomData<
                            InternallyTaggedEnum<T, U>,
                        >,
                        lifetime: _serde::__private::PhantomData<&'de ()>,
                    }
                    impl<'de, T, U> __Visitor<'de, T, U>
                    where
                        T: _serde::Deserialize<'de>,
                        U: _serde::Deserialize<'de>,
                    {
                        fn visit<__D>(
                            __tag: __Field,
                            __deserializer: __D,
                        ) -> _serde::__private::Result<
                            InternallyTaggedEnum<T, U>,
                            __D::Error,
                        >
                        where
                            __D: _serde::de::Deserializer<'de>,
                        {
                            match __tag {
                                __Field::__field0 => {
                                    _serde::Deserializer::deserialize_any(
                                        __deserializer,
                                        _serde::__private::de::InternallyTaggedUnitVisitor::new(
                                            "InternallyTaggedEnum",
                                            "Unit",
                                        ),
                                    )?;
                                    _serde::__private::Ok(InternallyTaggedEnum::Unit)
                                }
                                __Field::__field1 => {
                                    _serde::__private::Result::map(
                                        <T as _serde::Deserialize>::deserialize(__deserializer),
                                        InternallyTaggedEnum::NewType,
                                    )
                                }
                                __Field::__field2 => {
                                    #[allow(non_camel_case_types)]
                                    #[doc(hidden)]
                                    enum __Field {
                                        __field0,
                                        __field1,
                                        __ignore,
                                    }
                                    #[doc(hidden)]
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
                                                0u64 => _serde::__private::Ok(__Field::__field0),
                                                1u64 => _serde::__private::Ok(__Field::__field1),
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
                                                "x" => _serde::__private::Ok(__Field::__field0),
                                                "y" => _serde::__private::Ok(__Field::__field1),
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
                                                b"x" => _serde::__private::Ok(__Field::__field0),
                                                b"y" => _serde::__private::Ok(__Field::__field1),
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
                                    #[doc(hidden)]
                                    struct __Visitor<'de, T, U>
                                    where
                                        T: _serde::Deserialize<'de>,
                                        U: _serde::Deserialize<'de>,
                                    {
                                        marker: _serde::__private::PhantomData<
                                            InternallyTaggedEnum<T, U>,
                                        >,
                                        lifetime: _serde::__private::PhantomData<&'de ()>,
                                    }
                                    impl<'de, T, U> _serde::de::Visitor<'de>
                                    for __Visitor<'de, T, U>
                                    where
                                        T: _serde::Deserialize<'de>,
                                        U: _serde::Deserialize<'de>,
                                    {
                                        type Value = InternallyTaggedEnum<T, U>;
                                        fn expecting(
                                            &self,
                                            __formatter: &mut _serde::__private::Formatter,
                                        ) -> _serde::__private::fmt::Result {
                                            _serde::__private::Formatter::write_str(
                                                __formatter,
                                                "struct variant InternallyTaggedEnum::Map",
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
                                            let __field0 = match _serde::de::SeqAccess::next_element::<
                                                T,
                                            >(&mut __seq)? {
                                                _serde::__private::Some(__value) => __value,
                                                _serde::__private::None => {
                                                    return _serde::__private::Err(
                                                        _serde::de::Error::invalid_length(
                                                            0usize,
                                                            &"struct variant InternallyTaggedEnum::Map with 2 elements",
                                                        ),
                                                    );
                                                }
                                            };
                                            let __field1 = match _serde::de::SeqAccess::next_element::<
                                                U,
                                            >(&mut __seq)? {
                                                _serde::__private::Some(__value) => __value,
                                                _serde::__private::None => {
                                                    return _serde::__private::Err(
                                                        _serde::de::Error::invalid_length(
                                                            1usize,
                                                            &"struct variant InternallyTaggedEnum::Map with 2 elements",
                                                        ),
                                                    );
                                                }
                                            };
                                            _serde::__private::Ok(InternallyTaggedEnum::Map {
                                                x: __field0,
                                                y: __field1,
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
                                            let mut __field0: _serde::__private::Option<T> = _serde::__private::None;
                                            let mut __field1: _serde::__private::Option<U> = _serde::__private::None;
                                            while let _serde::__private::Some(__key)
                                                = _serde::de::MapAccess::next_key::<__Field>(&mut __map)? {
                                                match __key {
                                                    __Field::__field0 => {
                                                        if _serde::__private::Option::is_some(&__field0) {
                                                            return _serde::__private::Err(
                                                                <__A::Error as _serde::de::Error>::duplicate_field("x"),
                                                            );
                                                        }
                                                        __field0 = _serde::__private::Some(
                                                            _serde::de::MapAccess::next_value::<T>(&mut __map)?,
                                                        );
                                                    }
                                                    __Field::__field1 => {
                                                        if _serde::__private::Option::is_some(&__field1) {
                                                            return _serde::__private::Err(
                                                                <__A::Error as _serde::de::Error>::duplicate_field("y"),
                                                            );
                                                        }
                                                        __field1 = _serde::__private::Some(
                                                            _serde::de::MapAccess::next_value::<U>(&mut __map)?,
                                                        );
                                                    }
                                                    _ => {
                                                        let _ = _serde::de::MapAccess::next_value::<
                                                            _serde::de::IgnoredAny,
                                                        >(&mut __map)?;
                                                    }
                                                }
                                            }
                                            let __field0 = match __field0 {
                                                _serde::__private::Some(__field0) => __field0,
                                                _serde::__private::None => {
                                                    _serde::__private::de::missing_field("x")?
                                                }
                                            };
                                            let __field1 = match __field1 {
                                                _serde::__private::Some(__field1) => __field1,
                                                _serde::__private::None => {
                                                    _serde::__private::de::missing_field("y")?
                                                }
                                            };
                                            _serde::__private::Ok(InternallyTaggedEnum::Map {
                                                x: __field0,
                                                y: __field1,
                                            })
                                        }
                                    }
                                    #[doc(hidden)]
                                    const FIELDS: &'static [&'static str] = &["x", "y"];
                                    _serde::Deserializer::deserialize_any(
                                        __deserializer,
                                        __Visitor {
                                            marker: _serde::__private::PhantomData::<
                                                InternallyTaggedEnum<T, U>,
                                            >,
                                            lifetime: _serde::__private::PhantomData,
                                        },
                                    )
                                }
                            }
                        }
                    }
                    impl<'de, T, U> _serde::de::Visitor<'de> for __Visitor<'de, T, U>
                    where
                        T: _serde::Deserialize<'de>,
                        U: _serde::Deserialize<'de>,
                    {
                        type Value = InternallyTaggedEnum<T, U>;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private::Formatter,
                        ) -> _serde::__private::fmt::Result {
                            _serde::__private::Formatter::write_str(
                                __formatter,
                                "internally tagged enum InternallyTaggedEnum",
                            )
                        }
                        fn visit_seq<__S>(
                            self,
                            mut __seq: __S,
                        ) -> _serde::__private::Result<Self::Value, __S::Error>
                        where
                            __S: _serde::de::SeqAccess<'de>,
                        {
                            match _serde::de::SeqAccess::next_element(&mut __seq)? {
                                _serde::__private::Some(__tag) => {
                                    let __rest = _serde::de::value::SeqAccessDeserializer::new(
                                        __seq,
                                    );
                                    let __content = <_serde::__private::de::Content as _serde::Deserialize>::deserialize(
                                        __rest,
                                    )?;
                                    let __deserializer = _serde::__private::de::ContentDeserializer::<
                                        __S::Error,
                                    >::new(__content);
                                    Self::visit(__tag, __deserializer)
                                }
                                _serde::__private::None => {
                                    _serde::__private::Err(
                                        _serde::de::Error::missing_field("tag"),
                                    )
                                }
                            }
                        }
                        fn visit_map<__M>(
                            self,
                            mut __map: __M,
                        ) -> _serde::__private::Result<Self::Value, __M::Error>
                        where
                            __M: _serde::de::MapAccess<'de>,
                        {
                            match _serde::de::MapAccess::next_key_seed(
                                &mut __map,
                                _serde::__private::de::TagOrContentVisitor::new("tag"),
                            )? {
                                _serde::__private::Some(
                                    _serde::__private::de::TagOrContent::Tag,
                                ) => {
                                    let __tag = _serde::de::MapAccess::next_value(&mut __map)?;
                                    Self::visit(
                                        __tag,
                                        _serde::de::value::MapAccessDeserializer::new(__map),
                                    )
                                }
                                _serde::__private::Some(
                                    _serde::__private::de::TagOrContent::Content(__key),
                                ) => {
                                    let (__tag, __deserializer) = _serde::__private::de::drain_map(
                                        __map,
                                        "tag",
                                        __key,
                                    )?;
                                    Self::visit(__tag, __deserializer)
                                }
                                _serde::__private::None => {
                                    _serde::__private::Err(
                                        _serde::de::Error::missing_field("tag"),
                                    )
                                }
                            }
                        }
                    }
                    _serde::Deserializer::deserialize_any(
                        __deserializer,
                        __Visitor {
                            marker: _serde::__private::PhantomData::<
                                InternallyTaggedEnum<T, U>,
                            >,
                            lifetime: _serde::__private::PhantomData,
                        },
                    )
                }
            }
        };
    }
    mod enum_untagged {
        use serde_derive::{Deserialize, Serialize};
        #[serde(untagged)]
        pub enum UntaggedEnum<T, U> {
            Unit,
            NewType(T),
            Seq(T, U),
            Map { x: T, y: U },
        }
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<T, U> _serde::Serialize for UntaggedEnum<T, U>
            where
                T: _serde::Serialize,
                U: _serde::Serialize,
            {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private::Result<__S::Ok, __S::Error>
                where
                    __S: _serde::Serializer,
                {
                    match *self {
                        UntaggedEnum::Unit => {
                            _serde::Serializer::serialize_unit(__serializer)
                        }
                        UntaggedEnum::NewType(ref __field0) => {
                            _serde::Serialize::serialize(__field0, __serializer)
                        }
                        UntaggedEnum::Seq(ref __field0, ref __field1) => {
                            let mut __serde_state = _serde::Serializer::serialize_tuple(
                                __serializer,
                                0 + 1 + 1,
                            )?;
                            _serde::ser::SerializeTuple::serialize_element(
                                &mut __serde_state,
                                __field0,
                            )?;
                            _serde::ser::SerializeTuple::serialize_element(
                                &mut __serde_state,
                                __field1,
                            )?;
                            _serde::ser::SerializeTuple::end(__serde_state)
                        }
                        UntaggedEnum::Map { ref x, ref y } => {
                            let mut __serde_state = _serde::Serializer::serialize_struct(
                                __serializer,
                                "UntaggedEnum",
                                0 + 1 + 1,
                            )?;
                            _serde::ser::SerializeStruct::serialize_field(
                                &mut __serde_state,
                                "x",
                                x,
                            )?;
                            _serde::ser::SerializeStruct::serialize_field(
                                &mut __serde_state,
                                "y",
                                y,
                            )?;
                            _serde::ser::SerializeStruct::end(__serde_state)
                        }
                    }
                }
            }
        };
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de, T, U> _serde::Deserialize<'de> for UntaggedEnum<T, U>
            where
                T: _serde::Deserialize<'de>,
                U: _serde::Deserialize<'de>,
            {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    let __content = <_serde::__private::de::Content as _serde::Deserialize>::deserialize(
                        __deserializer,
                    )?;
                    let __deserializer = _serde::__private::de::ContentRefDeserializer::<
                        __D::Error,
                    >::new(&__content);
                    if let _serde::__private::Ok(__ok)
                        = match _serde::Deserializer::deserialize_any(
                            __deserializer,
                            _serde::__private::de::UntaggedUnitVisitor::new(
                                "UntaggedEnum",
                                "Unit",
                            ),
                        ) {
                            _serde::__private::Ok(()) => {
                                _serde::__private::Ok(UntaggedEnum::Unit)
                            }
                            _serde::__private::Err(__err) => {
                                _serde::__private::Err(__err)
                            }
                        } {
                        return _serde::__private::Ok(__ok);
                    }
                    if let _serde::__private::Ok(__ok)
                        = _serde::__private::Result::map(
                            <T as _serde::Deserialize>::deserialize(__deserializer),
                            UntaggedEnum::NewType,
                        ) {
                        return _serde::__private::Ok(__ok);
                    }
                    if let _serde::__private::Ok(__ok)
                        = {
                            #[doc(hidden)]
                            struct __Visitor<'de, T, U>
                            where
                                T: _serde::Deserialize<'de>,
                                U: _serde::Deserialize<'de>,
                            {
                                marker: _serde::__private::PhantomData<UntaggedEnum<T, U>>,
                                lifetime: _serde::__private::PhantomData<&'de ()>,
                            }
                            impl<'de, T, U> _serde::de::Visitor<'de>
                            for __Visitor<'de, T, U>
                            where
                                T: _serde::Deserialize<'de>,
                                U: _serde::Deserialize<'de>,
                            {
                                type Value = UntaggedEnum<T, U>;
                                fn expecting(
                                    &self,
                                    __formatter: &mut _serde::__private::Formatter,
                                ) -> _serde::__private::fmt::Result {
                                    _serde::__private::Formatter::write_str(
                                        __formatter,
                                        "tuple variant UntaggedEnum::Seq",
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
                                    let __field0 = match _serde::de::SeqAccess::next_element::<
                                        T,
                                    >(&mut __seq)? {
                                        _serde::__private::Some(__value) => __value,
                                        _serde::__private::None => {
                                            return _serde::__private::Err(
                                                _serde::de::Error::invalid_length(
                                                    0usize,
                                                    &"tuple variant UntaggedEnum::Seq with 2 elements",
                                                ),
                                            );
                                        }
                                    };
                                    let __field1 = match _serde::de::SeqAccess::next_element::<
                                        U,
                                    >(&mut __seq)? {
                                        _serde::__private::Some(__value) => __value,
                                        _serde::__private::None => {
                                            return _serde::__private::Err(
                                                _serde::de::Error::invalid_length(
                                                    1usize,
                                                    &"tuple variant UntaggedEnum::Seq with 2 elements",
                                                ),
                                            );
                                        }
                                    };
                                    _serde::__private::Ok(UntaggedEnum::Seq(__field0, __field1))
                                }
                            }
                            _serde::Deserializer::deserialize_tuple(
                                __deserializer,
                                2usize,
                                __Visitor {
                                    marker: _serde::__private::PhantomData::<
                                        UntaggedEnum<T, U>,
                                    >,
                                    lifetime: _serde::__private::PhantomData,
                                },
                            )
                        } {
                        return _serde::__private::Ok(__ok);
                    }
                    if let _serde::__private::Ok(__ok)
                        = {
                            #[allow(non_camel_case_types)]
                            #[doc(hidden)]
                            enum __Field {
                                __field0,
                                __field1,
                                __ignore,
                            }
                            #[doc(hidden)]
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
                                        0u64 => _serde::__private::Ok(__Field::__field0),
                                        1u64 => _serde::__private::Ok(__Field::__field1),
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
                                        "x" => _serde::__private::Ok(__Field::__field0),
                                        "y" => _serde::__private::Ok(__Field::__field1),
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
                                        b"x" => _serde::__private::Ok(__Field::__field0),
                                        b"y" => _serde::__private::Ok(__Field::__field1),
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
                            #[doc(hidden)]
                            struct __Visitor<'de, T, U>
                            where
                                T: _serde::Deserialize<'de>,
                                U: _serde::Deserialize<'de>,
                            {
                                marker: _serde::__private::PhantomData<UntaggedEnum<T, U>>,
                                lifetime: _serde::__private::PhantomData<&'de ()>,
                            }
                            impl<'de, T, U> _serde::de::Visitor<'de>
                            for __Visitor<'de, T, U>
                            where
                                T: _serde::Deserialize<'de>,
                                U: _serde::Deserialize<'de>,
                            {
                                type Value = UntaggedEnum<T, U>;
                                fn expecting(
                                    &self,
                                    __formatter: &mut _serde::__private::Formatter,
                                ) -> _serde::__private::fmt::Result {
                                    _serde::__private::Formatter::write_str(
                                        __formatter,
                                        "struct variant UntaggedEnum::Map",
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
                                    let mut __field0: _serde::__private::Option<T> = _serde::__private::None;
                                    let mut __field1: _serde::__private::Option<U> = _serde::__private::None;
                                    while let _serde::__private::Some(__key)
                                        = _serde::de::MapAccess::next_key::<__Field>(&mut __map)? {
                                        match __key {
                                            __Field::__field0 => {
                                                if _serde::__private::Option::is_some(&__field0) {
                                                    return _serde::__private::Err(
                                                        <__A::Error as _serde::de::Error>::duplicate_field("x"),
                                                    );
                                                }
                                                __field0 = _serde::__private::Some(
                                                    _serde::de::MapAccess::next_value::<T>(&mut __map)?,
                                                );
                                            }
                                            __Field::__field1 => {
                                                if _serde::__private::Option::is_some(&__field1) {
                                                    return _serde::__private::Err(
                                                        <__A::Error as _serde::de::Error>::duplicate_field("y"),
                                                    );
                                                }
                                                __field1 = _serde::__private::Some(
                                                    _serde::de::MapAccess::next_value::<U>(&mut __map)?,
                                                );
                                            }
                                            _ => {
                                                let _ = _serde::de::MapAccess::next_value::<
                                                    _serde::de::IgnoredAny,
                                                >(&mut __map)?;
                                            }
                                        }
                                    }
                                    let __field0 = match __field0 {
                                        _serde::__private::Some(__field0) => __field0,
                                        _serde::__private::None => {
                                            _serde::__private::de::missing_field("x")?
                                        }
                                    };
                                    let __field1 = match __field1 {
                                        _serde::__private::Some(__field1) => __field1,
                                        _serde::__private::None => {
                                            _serde::__private::de::missing_field("y")?
                                        }
                                    };
                                    _serde::__private::Ok(UntaggedEnum::Map {
                                        x: __field0,
                                        y: __field1,
                                    })
                                }
                            }
                            #[doc(hidden)]
                            const FIELDS: &'static [&'static str] = &["x", "y"];
                            _serde::Deserializer::deserialize_any(
                                __deserializer,
                                __Visitor {
                                    marker: _serde::__private::PhantomData::<
                                        UntaggedEnum<T, U>,
                                    >,
                                    lifetime: _serde::__private::PhantomData,
                                },
                            )
                        } {
                        return _serde::__private::Ok(__ok);
                    }
                    _serde::__private::Err(
                        _serde::de::Error::custom(
                            "data did not match any variant of untagged enum UntaggedEnum",
                        ),
                    )
                }
            }
        };
    }
}
#[rustc_main]
#[no_coverage]
pub fn main() -> () {
    extern crate test;
    test::test_main_static(&[])
}
