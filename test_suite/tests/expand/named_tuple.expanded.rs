use serde::{Deserialize, Serialize};
struct SerNamedTuple<'a, 'b, A: 'a, B: 'b, C>(&'a A, &'b mut B, C);
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(rust_2018_idioms, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'a, 'b, A: 'a, B: 'b, C> _serde::Serialize for SerNamedTuple<'a, 'b, A, B, C>
    where
        A: _serde::Serialize,
        B: _serde::Serialize,
        C: _serde::Serialize,
    {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = match _serde::Serializer::serialize_tuple_struct(
                __serializer,
                "SerNamedTuple",
                0 + 1 + 1 + 1,
            ) {
                _serde::__private::Ok(__val) => __val,
                _serde::__private::Err(__err) => {
                    return _serde::__private::Err(__err);
                }
            };
            match _serde::ser::SerializeTupleStruct::serialize_field(&mut __serde_state, &self.0) {
                _serde::__private::Ok(__val) => __val,
                _serde::__private::Err(__err) => {
                    return _serde::__private::Err(__err);
                }
            };
            match _serde::ser::SerializeTupleStruct::serialize_field(&mut __serde_state, &self.1) {
                _serde::__private::Ok(__val) => __val,
                _serde::__private::Err(__err) => {
                    return _serde::__private::Err(__err);
                }
            };
            match _serde::ser::SerializeTupleStruct::serialize_field(&mut __serde_state, &self.2) {
                _serde::__private::Ok(__val) => __val,
                _serde::__private::Err(__err) => {
                    return _serde::__private::Err(__err);
                }
            };
            _serde::ser::SerializeTupleStruct::end(__serde_state)
        }
    }
};
struct DeNamedTuple<A, B, C>(A, B, C);
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(rust_2018_idioms, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de, A, B, C> _serde::Deserialize<'de> for DeNamedTuple<A, B, C>
    where
        A: _serde::Deserialize<'de>,
        B: _serde::Deserialize<'de>,
        C: _serde::Deserialize<'de>,
    {
        fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            struct __Visitor<'de, A, B, C>
            where
                A: _serde::Deserialize<'de>,
                B: _serde::Deserialize<'de>,
                C: _serde::Deserialize<'de>,
            {
                marker: _serde::__private::PhantomData<DeNamedTuple<A, B, C>>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            impl<'de, A, B, C> _serde::de::Visitor<'de> for __Visitor<'de, A, B, C>
            where
                A: _serde::Deserialize<'de>,
                B: _serde::Deserialize<'de>,
                C: _serde::Deserialize<'de>,
            {
                type Value = DeNamedTuple<A, B, C>;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(
                        __formatter,
                        "tuple struct DeNamedTuple",
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
                    let __field0 = match match _serde::de::SeqAccess::next_element::<A>(&mut __seq)
                    {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    } {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                0usize,
                                &"tuple struct DeNamedTuple with 3 elements",
                            ));
                        }
                    };
                    let __field1 = match match _serde::de::SeqAccess::next_element::<B>(&mut __seq)
                    {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    } {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                1usize,
                                &"tuple struct DeNamedTuple with 3 elements",
                            ));
                        }
                    };
                    let __field2 = match match _serde::de::SeqAccess::next_element::<C>(&mut __seq)
                    {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    } {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                2usize,
                                &"tuple struct DeNamedTuple with 3 elements",
                            ));
                        }
                    };
                    _serde::__private::Ok(DeNamedTuple(__field0, __field1, __field2))
                }
            }
            _serde::Deserializer::deserialize_tuple_struct(
                __deserializer,
                "DeNamedTuple",
                3usize,
                __Visitor {
                    marker: _serde::__private::PhantomData::<DeNamedTuple<A, B, C>>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
        fn deserialize_in_place<__D>(
            __deserializer: __D,
            __place: &mut Self,
        ) -> _serde::__private::Result<(), __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            struct __Visitor<'de, 'place, A: 'place, B: 'place, C: 'place>
            where
                A: _serde::Deserialize<'de>,
                B: _serde::Deserialize<'de>,
                C: _serde::Deserialize<'de>,
            {
                place: &'place mut DeNamedTuple<A, B, C>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            impl<'de, 'place, A: 'place, B: 'place, C: 'place> _serde::de::Visitor<'de>
                for __Visitor<'de, 'place, A, B, C>
            where
                A: _serde::Deserialize<'de>,
                B: _serde::Deserialize<'de>,
                C: _serde::Deserialize<'de>,
            {
                type Value = ();
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(
                        __formatter,
                        "tuple struct DeNamedTuple",
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
                    if let _serde::__private::None = match _serde::de::SeqAccess::next_element_seed(
                        &mut __seq,
                        _serde::__private::de::InPlaceSeed(&mut self.place.0),
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    } {
                        return _serde::__private::Err(_serde::de::Error::invalid_length(
                            0usize,
                            &"tuple struct DeNamedTuple with 3 elements",
                        ));
                    }
                    if let _serde::__private::None = match _serde::de::SeqAccess::next_element_seed(
                        &mut __seq,
                        _serde::__private::de::InPlaceSeed(&mut self.place.1),
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    } {
                        return _serde::__private::Err(_serde::de::Error::invalid_length(
                            1usize,
                            &"tuple struct DeNamedTuple with 3 elements",
                        ));
                    }
                    if let _serde::__private::None = match _serde::de::SeqAccess::next_element_seed(
                        &mut __seq,
                        _serde::__private::de::InPlaceSeed(&mut self.place.2),
                    ) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    } {
                        return _serde::__private::Err(_serde::de::Error::invalid_length(
                            2usize,
                            &"tuple struct DeNamedTuple with 3 elements",
                        ));
                    }
                    _serde::__private::Ok(())
                }
            }
            _serde::Deserializer::deserialize_tuple_struct(
                __deserializer,
                "DeNamedTuple",
                3usize,
                __Visitor {
                    place: __place,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};
