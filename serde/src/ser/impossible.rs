//! This module contains `Impossible` serializer and its implementations.

use core::marker::PhantomData;

use ser::{
    self,
    Serialize,
    SerializeSeq,
    SerializeTuple,
    SerializeTupleStruct,
    SerializeTupleVariant,
    SerializeMap,
    SerializeStruct,
    SerializeStructVariant,
};

/// The impossible serializer.
///
/// This serializer cannot be instantiated, to be used in the associated types
/// of the `Serializer` trait, when implementing serializers that don't support
/// some constructs, like sequences or tuples.
///
/// ```rust,ignore
/// impl Serializer for MySerializer {
///     type SerializeSeq = Impossible<(), Error>;
///
///     fn serialize_seq(self,
///                      len: Option<usize>)
///                      -> Result<Self::SerializeSeq, Error> {
///         // Given Impossible cannot be instantiated, the only
///         // thing we can do here is to return an error.
///         Err(...)
///     }
///
///     ...
/// }
/// ```
pub struct Impossible<Ok, E> {
    void: Void,
    _marker: PhantomData<(Ok, E)>,
}

enum Void {}

impl<Ok, E> SerializeSeq for Impossible<Ok, E>
    where E: ser::Error,
{
    type Ok = Ok;
    type Error = E;

    fn serialize_element<T: ?Sized + Serialize>(&mut self,
                                                _value: &T)
                                                -> Result<(), E> {
        match self.void {}
    }

    fn end(self) -> Result<Ok, E> {
        match self.void {}
    }
}

impl<Ok, E> SerializeTuple for Impossible<Ok, E>
    where E: ser::Error,
{
    type Ok = Ok;
    type Error = E;

    fn serialize_element<T: ?Sized + Serialize>(&mut self,
                                                _value: &T)
                                                -> Result<(), E> {
        match self.void {}
    }

    fn end(self) -> Result<Ok, E> {
        match self.void {}
    }
}

impl<Ok, E> SerializeTupleStruct for Impossible<Ok, E>
    where E: ser::Error,
{
    type Ok = Ok;
    type Error = E;

    fn serialize_field<T: ?Sized + Serialize>(&mut self,
                                              _value: &T)
                                              -> Result<(), E> {
        match self.void {}
    }

    fn end(self) -> Result<Ok, E> {
        match self.void {}
    }
}

impl<Ok, E> SerializeTupleVariant for Impossible<Ok, E>
    where E: ser::Error,
{
    type Ok = Ok;
    type Error = E;

    fn serialize_field<T: ?Sized + Serialize>(&mut self,
                                              _value: &T)
                                              -> Result<(), E> {
        match self.void {}
    }

    fn end(self) -> Result<Ok, E> {
        match self.void {}
    }
}

impl<Ok, E> SerializeMap for Impossible<Ok, E>
    where E: ser::Error,
{
    type Ok = Ok;
    type Error = E;

    fn serialize_key<T: ?Sized + Serialize>(&mut self,
                                                 _key: &T)
                                            -> Result<(), E> {
        match self.void {}
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self,
                                              _value: &T)
                                              -> Result<(), E> {
        match self.void {}
    }

    fn end(self) -> Result<Ok, E> {
        match self.void {}
    }
}

impl<Ok, E> SerializeStruct for Impossible<Ok, E>
    where E: ser::Error,
{
    type Ok = Ok;
    type Error = E;

    fn serialize_field<T: ?Sized + Serialize>(&mut self,
                                                   _key: &'static str,
                                              _value: &T)
                                              -> Result<(), E> {
        match self.void {}
    }

    fn end(self) -> Result<Ok, E> {
        match self.void {}
    }
}

impl<Ok, E> SerializeStructVariant for Impossible<Ok, E>
    where E: ser::Error,
{
    type Ok = Ok;
    type Error = E;

    fn serialize_field<T: ?Sized + Serialize>(&mut self,
                                                   _key: &'static str,
                                              _value: &T)
                                              -> Result<(), E> {
        match self.void {}
    }

    fn end(self) -> Result<Ok, E> {
        match self.void {}
    }
}
