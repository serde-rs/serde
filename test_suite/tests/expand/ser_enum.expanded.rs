use serde::Serialize;
enum SerEnum<'a, B: 'a, C: 'a, D>
where
    D: 'a,
{
    Unit,
    Seq(i8, B, &'a C, &'a mut D),
    Map { a: i8, b: B, c: &'a C, d: &'a mut D },
    _Unit2,
    _Seq2(i8, B, &'a C, &'a mut D),
    _Map2 { a: i8, b: B, c: &'a C, d: &'a mut D },
}
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(rust_2018_idioms, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'a, B: 'a, C: 'a, D> _serde::Serialize for SerEnum<'a, B, C, D>
    where
        D: 'a,
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
                SerEnum::Unit => _serde::Serializer::serialize_unit_variant(
                    __serializer,
                    "SerEnum",
                    0u32,
                    "Unit",
                ),
                SerEnum::Seq(ref __field0, ref __field1, ref __field2, ref __field3) => {
                    let mut __serde_state = match _serde::Serializer::serialize_tuple_variant(
                        __serializer,
                        "SerEnum",
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
                SerEnum::Map {
                    ref a,
                    ref b,
                    ref c,
                    ref d,
                } => {
                    let mut __serde_state = match _serde::Serializer::serialize_struct_variant(
                        __serializer,
                        "SerEnum",
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
                SerEnum::_Unit2 => _serde::Serializer::serialize_unit_variant(
                    __serializer,
                    "SerEnum",
                    3u32,
                    "_Unit2",
                ),
                SerEnum::_Seq2(ref __field0, ref __field1, ref __field2, ref __field3) => {
                    let mut __serde_state = match _serde::Serializer::serialize_tuple_variant(
                        __serializer,
                        "SerEnum",
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
                SerEnum::_Map2 {
                    ref a,
                    ref b,
                    ref c,
                    ref d,
                } => {
                    let mut __serde_state = match _serde::Serializer::serialize_struct_variant(
                        __serializer,
                        "SerEnum",
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
