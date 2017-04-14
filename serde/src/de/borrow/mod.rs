#![allow(missing_docs)]

mod impls;

use core::marker::PhantomData;

use de::{Deserializer, Visitor, SeqVisitor, MapVisitor, EnumVisitor, VariantVisitor, Error};

pub trait DeserializeBorrow<'a>: Sized {
    fn deserialize_borrow<D>(deserializer: D) -> Result<Self, D::Error>
        where D: DeserializerBorrow<'a>;
}

pub trait DeserializeBorrowSeed<'a>: Sized {
    type Value;

    fn deserialize_borrow<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: DeserializerBorrow<'a>;
}

impl<'a, T> DeserializeBorrowSeed<'a> for PhantomData<T>
    where T: DeserializeBorrow<'a>
{
    type Value = T;

    #[inline]
    fn deserialize_borrow<D>(self, deserializer: D) -> Result<T, D::Error>
        where D: DeserializerBorrow<'a>
    {
        T::deserialize_borrow(deserializer)
    }
}

pub trait DeserializerBorrow<'a>: Deserializer {
    fn deserialize_borrow_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: VisitorBorrow<'a>;

    fn deserialize_borrow_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: VisitorBorrow<'a>;

    fn deserialize_borrow_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: VisitorBorrow<'a>;

    fn deserialize_borrow_newtype_struct<V>(self,
                                     name: &'static str,
                                     visitor: V)
                                     -> Result<V::Value, Self::Error>
        where V: VisitorBorrow<'a>;

    fn deserialize_borrow_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: VisitorBorrow<'a>;

    fn deserialize_borrow_seq_fixed_size<V>(self,
                                     len: usize,
                                     visitor: V)
                                     -> Result<V::Value, Self::Error>
        where V: VisitorBorrow<'a>;

    fn deserialize_borrow_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
        where V: VisitorBorrow<'a>;

    fn deserialize_borrow_tuple_struct<V>(self,
                                   name: &'static str,
                                   len: usize,
                                   visitor: V)
                                   -> Result<V::Value, Self::Error>
        where V: VisitorBorrow<'a>;

    fn deserialize_borrow_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: VisitorBorrow<'a>;

    fn deserialize_borrow_struct<V>(self,
                             name: &'static str,
                             fields: &'static [&'static str],
                             visitor: V)
                             -> Result<V::Value, Self::Error>
        where V: VisitorBorrow<'a>;

    fn deserialize_borrow_enum<V>(self,
                           name: &'static str,
                           variants: &'static [&'static str],
                           visitor: V)
                           -> Result<V::Value, Self::Error>
        where V: VisitorBorrow<'a>;
}

pub trait VisitorBorrow<'a>: Visitor {
    fn visit_borrow_str<E>(self, v: &'a str) -> Result<Self::Value, E>
        where E: Error
    {
        self.visit_str(v)
    }

    fn visit_borrow_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: DeserializerBorrow<'a>
    {
        self.visit_some(deserializer)
    }

    fn visit_borrow_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: DeserializerBorrow<'a>
    {
        self.visit_newtype_struct(deserializer)
    }

    fn visit_borrow_seq<V>(self, visitor: V) -> Result<Self::Value, V::Error>
        where V: SeqVisitorBorrow<'a>
    {
        self.visit_seq(visitor)
    }

    fn visit_borrow_map<V>(self, visitor: V) -> Result<Self::Value, V::Error>
        where V: MapVisitorBorrow<'a>
    {
        self.visit_map(visitor)
    }

    fn visit_borrow_enum<V>(self, visitor: V) -> Result<Self::Value, V::Error>
        where V: EnumVisitorBorrow<'a>
    {
        self.visit_enum(visitor)
    }

    fn visit_borrow_bytes<E>(self, v: &'a [u8]) -> Result<Self::Value, E>
        where E: Error
    {
        self.visit_bytes(v)
    }
}

pub trait SeqVisitorBorrow<'a>: SeqVisitor {
    fn visit_borrow_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
        where T: DeserializeBorrowSeed<'a>;

    #[inline]
    fn visit_borrow<T>(&mut self) -> Result<Option<T>, Self::Error>
        where T: DeserializeBorrow<'a>
    {
        self.visit_borrow_seed(PhantomData)
    }
}

impl<'a, 'r, V> SeqVisitorBorrow<'a> for &'r mut V
    where V: SeqVisitorBorrow<'a>
{
    #[inline]
    fn visit_borrow_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, V::Error>
        where T: DeserializeBorrowSeed<'a>
    {
        (**self).visit_borrow_seed(seed)
    }

    #[inline]
    fn visit_borrow<T>(&mut self) -> Result<Option<T>, V::Error>
        where T: DeserializeBorrow<'a>
    {
        (**self).visit_borrow()
    }
}

pub trait MapVisitorBorrow<'a>: MapVisitor {
    fn visit_key_borrow_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
        where K: DeserializeBorrowSeed<'a>;

    fn visit_value_borrow_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
        where V: DeserializeBorrowSeed<'a>;

    #[inline]
    fn visit_borrow_seed<K, V>(&mut self,
                        kseed: K,
                        vseed: V)
                        -> Result<Option<(K::Value, V::Value)>, Self::Error>
        where K: DeserializeBorrowSeed<'a>,
              V: DeserializeBorrowSeed<'a>
    {
        match try!(self.visit_key_borrow_seed(kseed)) {
            Some(key) => {
                let value = try!(self.visit_value_borrow_seed(vseed));
                Ok(Some((key, value)))
            }
            None => Ok(None),
        }
    }

    #[inline]
    fn visit_key_borrow<K>(&mut self) -> Result<Option<K>, Self::Error>
        where K: DeserializeBorrow<'a>
    {
        self.visit_key_borrow_seed(PhantomData)
    }

    #[inline]
    fn visit_value_borrow<V>(&mut self) -> Result<V, Self::Error>
        where V: DeserializeBorrow<'a>
    {
        self.visit_value_borrow_seed(PhantomData)
    }

    #[inline]
    fn visit_borrow<K, V>(&mut self) -> Result<Option<(K, V)>, Self::Error>
        where K: DeserializeBorrow<'a>,
              V: DeserializeBorrow<'a>
    {
        self.visit_borrow_seed(PhantomData, PhantomData)
    }
}

impl<'a, 'r, V_> MapVisitorBorrow<'a> for &'r mut V_
    where V_: MapVisitorBorrow<'a>
{
    #[inline]
    fn visit_key_borrow_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
        where K: DeserializeBorrowSeed<'a>
    {
        (**self).visit_key_borrow_seed(seed)
    }

    #[inline]
    fn visit_value_borrow_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
        where V: DeserializeBorrowSeed<'a>
    {
        (**self).visit_value_borrow_seed(seed)
    }

    #[inline]
    fn visit_borrow_seed<K, V>(&mut self,
                        kseed: K,
                        vseed: V)
                        -> Result<Option<(K::Value, V::Value)>, Self::Error>
        where K: DeserializeBorrowSeed<'a>,
              V: DeserializeBorrowSeed<'a>
    {
        (**self).visit_borrow_seed(kseed, vseed)
    }

    #[inline]
    fn visit_borrow<K, V>(&mut self) -> Result<Option<(K, V)>, V_::Error>
        where K: DeserializeBorrow<'a>,
              V: DeserializeBorrow<'a>
    {
        (**self).visit_borrow()
    }

    #[inline]
    fn visit_key_borrow<K>(&mut self) -> Result<Option<K>, V_::Error>
        where K: DeserializeBorrow<'a>
    {
        (**self).visit_key_borrow()
    }

    #[inline]
    fn visit_value_borrow<V>(&mut self) -> Result<V, V_::Error>
        where V: DeserializeBorrow<'a>
    {
        (**self).visit_value_borrow()
    }
}

pub trait EnumVisitorBorrow<'a>: EnumVisitor {
    type VariantBorrow: VariantVisitorBorrow<'a, Error = Self::Error>;

    fn visit_variant_borrow_seed<V>(self, seed: V) -> Result<(V::Value, Self::VariantBorrow), Self::Error>
        where V: DeserializeBorrowSeed<'a>;

    #[inline]
    fn visit_variant_borrow<V>(self) -> Result<(V, Self::VariantBorrow), Self::Error>
        where V: DeserializeBorrow<'a>
    {
        self.visit_variant_borrow_seed(PhantomData)
    }
}

pub trait VariantVisitorBorrow<'a>: VariantVisitor {
    fn visit_newtype_borrow_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
        where T: DeserializeBorrowSeed<'a>;

    #[inline]
    fn visit_newtype_borrow<T>(self) -> Result<T, Self::Error>
        where T: DeserializeBorrow<'a>
    {
        self.visit_newtype_borrow_seed(PhantomData)
    }

    fn visit_tuple_borrow<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
        where V: VisitorBorrow<'a>;

    fn visit_struct_borrow<V>(self,
                       fields: &'static [&'static str],
                       visitor: V)
                       -> Result<V::Value, Self::Error>
        where V: VisitorBorrow<'a>;
}
