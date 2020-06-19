use serde::{Deserialize, Serialize};
pub struct GenericTupleStruct<T, U>(T, U);
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(rust_2018_idioms, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de, T, U> _serde::Deserialize<'de> for GenericTupleStruct<T, U>
    where
        T: _serde::Deserialize<'de>,
        U: _serde::Deserialize<'de>,
    {
        fn deserialize<__D>(__deserializer: __D) -> _serde::export::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            struct __Visitor<'de, T, U>
            where
                T: _serde::Deserialize<'de>,
                U: _serde::Deserialize<'de>,
            {
                marker: _serde::export::PhantomData<GenericTupleStruct<T, U>>,
                lifetime: _serde::export::PhantomData<&'de ()>,
            }
            impl<'de, T, U> _serde::de::Visitor<'de> for __Visitor<'de, T, U>
            where
                T: _serde::Deserialize<'de>,
                U: _serde::Deserialize<'de>,
            {
                type Value = GenericTupleStruct<T, U>;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::export::Formatter,
                ) -> _serde::export::fmt::Result {
                    _serde::export::Formatter::write_str(
                        __formatter,
                        "tuple struct GenericTupleStruct",
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
                    let __field0 = match match _serde::de::SeqAccess::next_element::<T>(&mut __seq)
                    {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    } {
                        _serde::export::Some(__value) => __value,
                        _serde::export::None => {
                            return _serde::export::Err(_serde::de::Error::invalid_length(
                                0usize,
                                &"tuple struct GenericTupleStruct with 2 elements",
                            ));
                        }
                    };
                    let __field1 = match match _serde::de::SeqAccess::next_element::<U>(&mut __seq)
                    {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    } {
                        _serde::export::Some(__value) => __value,
                        _serde::export::None => {
                            return _serde::export::Err(_serde::de::Error::invalid_length(
                                1usize,
                                &"tuple struct GenericTupleStruct with 2 elements",
                            ));
                        }
                    };
                    _serde::export::Ok(GenericTupleStruct(__field0, __field1))
                }
            }
            _serde::Deserializer::deserialize_tuple_struct(
                __deserializer,
                "GenericTupleStruct",
                2usize,
                __Visitor {
                    marker: _serde::export::PhantomData::<GenericTupleStruct<T, U>>,
                    lifetime: _serde::export::PhantomData,
                },
            )
        }
        fn deserialize_in_place<__D>(
            __deserializer: __D,
            __place: &mut Self,
        ) -> _serde::export::Result<(), __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            struct __Visitor<'de, 'place, T: 'place, U: 'place>
            where
                T: _serde::Deserialize<'de>,
                U: _serde::Deserialize<'de>,
            {
                place: &'place mut GenericTupleStruct<T, U>,
                lifetime: _serde::export::PhantomData<&'de ()>,
            }
            impl<'de, 'place, T: 'place, U: 'place> _serde::de::Visitor<'de> for __Visitor<'de, 'place, T, U>
            where
                T: _serde::Deserialize<'de>,
                U: _serde::Deserialize<'de>,
            {
                type Value = ();
                fn expecting(
                    &self,
                    __formatter: &mut _serde::export::Formatter,
                ) -> _serde::export::fmt::Result {
                    _serde::export::Formatter::write_str(
                        __formatter,
                        "tuple struct GenericTupleStruct",
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
                    if let _serde::export::None = match _serde::de::SeqAccess::next_element_seed(
                        &mut __seq,
                        _serde::private::de::InPlaceSeed(&mut self.place.0),
                    ) {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    } {
                        return _serde::export::Err(_serde::de::Error::invalid_length(
                            0usize,
                            &"tuple struct GenericTupleStruct with 2 elements",
                        ));
                    }
                    if let _serde::export::None = match _serde::de::SeqAccess::next_element_seed(
                        &mut __seq,
                        _serde::private::de::InPlaceSeed(&mut self.place.1),
                    ) {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    } {
                        return _serde::export::Err(_serde::de::Error::invalid_length(
                            1usize,
                            &"tuple struct GenericTupleStruct with 2 elements",
                        ));
                    }
                    _serde::export::Ok(())
                }
            }
            _serde::Deserializer::deserialize_tuple_struct(
                __deserializer,
                "GenericTupleStruct",
                2usize,
                __Visitor {
                    place: __place,
                    lifetime: _serde::export::PhantomData,
                },
            )
        }
    }
};
