use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;
use std::marker::PhantomData;
use std::num::FromPrimitive;
use std::path;
use std::str;

///////////////////////////////////////////////////////////////////////////////

pub trait Error {
    fn syntax_error() -> Self;

    fn end_of_stream_error() -> Self;

    fn missing_field_error(&'static str) -> Self;
}

///////////////////////////////////////////////////////////////////////////////

pub trait Deserialize {
    /// Deserialize this value given this `Deserializer`.
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer;
}

///////////////////////////////////////////////////////////////////////////////

/// `Deserializer` is an abstract trait that can deserialize values into a `Visitor`.
pub trait Deserializer {
    type Error: Error;

    /// The `visit` method walks a visitor through a value as it is being deserialized.
    fn visit<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// The `visit_option` method allows a `Deserialize` type to inform the `Deserializer` that
    /// it's expecting an optional value. This allows deserializers that encode an optional value
    /// as a nullable value to convert the null value into a `None`, and a regular value as
    /// `Some(value)`.
    #[inline]
    fn visit_option<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit(visitor)
    }

    /// The `visit_seq` method allows a `Deserialize` type to inform the `Deserializer` that it's
    /// expecting a sequence of values. This allows deserializers to parse sequences that aren't
    /// tagged as sequences.
    #[inline]
    fn visit_seq<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit(visitor)
    }

    /// The `visit_map` method allows a `Deserialize` type to inform the `Deserializer` that it's
    /// expecting a map of values. This allows deserializers to parse sequences that aren't tagged
    /// as maps.
    #[inline]
    fn visit_map<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit(visitor)
    }

    /// The `visit_named_unit` method allows a `Deserialize` type to inform the `Deserializer` that
    /// it's expecting a named unit. This allows deserializers to a named unit that aren't tagged
    /// as a named unit.
    #[inline]
    fn visit_named_unit<V>(&mut self, _name: &str, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit(visitor)
    }

    /// The `visit_named_seq` method allows a `Deserialize` type to inform the `Deserializer` that
    /// it's expecting a named sequence of values. This allows deserializers to parse sequences
    /// that aren't tagged as sequences.
    #[inline]
    fn visit_named_seq<V>(&mut self, _name: &str, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit_seq(visitor)
    }

    /// The `visit_named_map` method allows a `Deserialize` type to inform the `Deserializer` that
    /// it's expecting a map of values. This allows deserializers to parse sequences that aren't
    /// tagged as maps.
    #[inline]
    fn visit_named_map<V>(&mut self, _name: &str, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor,
    {
        self.visit_map(visitor)
    }

    /// The `visit_enum` method allows a `Deserialize` type to inform the `Deserializer` that it's
    /// expecting an enum value. This allows deserializers that provide a custom enumeration
    /// serialization to properly deserialize the type.
    #[inline]
    fn visit_enum<V>(&mut self, _enum: &str, _visitor: V) -> Result<V::Value, Self::Error>
        where V: EnumVisitor,
    {
        Err(Error::syntax_error())
    }
}

///////////////////////////////////////////////////////////////////////////////

pub trait Visitor {
    type Value;

    fn visit_bool<E>(&mut self, _v: bool) -> Result<Self::Value, E>
        where E: Error,
    {
        Err(Error::syntax_error())
    }

    fn visit_isize<E>(&mut self, v: isize) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_i64(v as i64)
    }

    fn visit_i8<E>(&mut self, v: i8) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_i64(v as i64)
    }

    fn visit_i16<E>(&mut self, v: i16) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_i64(v as i64)
    }

    fn visit_i32<E>(&mut self, v: i32) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_i64(v as i64)
    }

    fn visit_i64<E>(&mut self, _v: i64) -> Result<Self::Value, E>
        where E: Error,
    {
        Err(Error::syntax_error())
    }

    fn visit_usize<E>(&mut self, v: usize) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_u64(v as u64)
    }

    fn visit_u8<E>(&mut self, v: u8) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_u64(v as u64)
    }

    fn visit_u16<E>(&mut self, v: u16) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_u64(v as u64)
    }

    fn visit_u32<E>(&mut self, v: u32) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_u64(v as u64)
    }

    fn visit_u64<E>(&mut self, _v: u64) -> Result<Self::Value, E>
        where E: Error,
    {
        Err(Error::syntax_error())
    }

    fn visit_f32<E>(&mut self, v: f32) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_f64(v as f64)
    }

    fn visit_f64<E>(&mut self, _v: f64) -> Result<Self::Value, E>
        where E: Error,
    {
        Err(Error::syntax_error())
    }

    #[inline]
    fn visit_char<E>(&mut self, v: char) -> Result<Self::Value, E>
        where E: Error,
    {
        // The unwraps in here should be safe.
        let mut s = &mut [0; 4];
        let len = v.encode_utf8(s).unwrap();
        self.visit_str(str::from_utf8(&s[..len]).unwrap())
    }

    fn visit_str<E>(&mut self, _v: &str) -> Result<Self::Value, E>
        where E: Error,
    {
        Err(Error::syntax_error())
    }

    #[inline]
    fn visit_string<E>(&mut self, v: String) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_str(&v)
    }

    fn visit_unit<E>(&mut self) -> Result<Self::Value, E>
        where E: Error,
    {
        Err(Error::syntax_error())
    }

    #[inline]
    fn visit_named_unit<E>(&mut self, _name: &str) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_unit()
    }

    fn visit_none<E>(&mut self) -> Result<Self::Value, E>
        where E: Error,
    {
        Err(Error::syntax_error())
    }

    fn visit_some<D>(&mut self, _deserializer: &mut D) -> Result<Self::Value, D::Error>
        where D: Deserializer,
    {
        Err(Error::syntax_error())
    }

    fn visit_seq<V>(&mut self, _visitor: V) -> Result<Self::Value, V::Error>
        where V: SeqVisitor,
    {
        Err(Error::syntax_error())
    }

    fn visit_map<V>(&mut self, _visitor: V) -> Result<Self::Value, V::Error>
        where V: MapVisitor,
    {
        Err(Error::syntax_error())
    }
}

///////////////////////////////////////////////////////////////////////////////

pub trait SeqVisitor {
    type Error: Error;

    fn visit<T>(&mut self) -> Result<Option<T>, Self::Error>
        where T: Deserialize;

    fn end(&mut self) -> Result<(), Self::Error>;

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl<'a, V> SeqVisitor for &'a mut V where V: SeqVisitor {
    type Error = V::Error;

    #[inline]
    fn visit<T>(&mut self) -> Result<Option<T>, V::Error>
        where T: Deserialize
    {
        (**self).visit()
    }

    #[inline]
    fn end(&mut self) -> Result<(), V::Error> {
        (**self).end()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (**self).size_hint()
    }
}

///////////////////////////////////////////////////////////////////////////////

pub trait MapVisitor {
    type Error: Error;

    #[inline]
    fn visit<K, V>(&mut self) -> Result<Option<(K, V)>, Self::Error>
        where K: Deserialize,
              V: Deserialize,
    {
        match try!(self.visit_key()) {
            Some(key) => {
                let value = try!(self.visit_value());
                Ok(Some((key, value)))
            }
            None => Ok(None)
        }
    }

    fn visit_key<K>(&mut self) -> Result<Option<K>, Self::Error>
        where K: Deserialize;

    fn visit_value<V>(&mut self) -> Result<V, Self::Error>
        where V: Deserialize;

    fn end(&mut self) -> Result<(), Self::Error>;

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }

    fn missing_field<V>(&mut self, field: &'static str) -> Result<V, Self::Error>
        where V: Deserialize,
    {
        Err(Error::missing_field_error(field))
    }
}

impl<'a, V_> MapVisitor for &'a mut V_ where V_: MapVisitor {
    type Error = V_::Error;

    #[inline]
    fn visit<K, V>(&mut self) -> Result<Option<(K, V)>, V_::Error>
        where K: Deserialize,
              V: Deserialize,
    {
        (**self).visit()
    }

    #[inline]
    fn visit_key<K>(&mut self) -> Result<Option<K>, V_::Error>
        where K: Deserialize
    {
        (**self).visit_key()
    }

    #[inline]
    fn visit_value<V>(&mut self) -> Result<V, V_::Error>
        where V: Deserialize
    {
        (**self).visit_value()
    }

    #[inline]
    fn end(&mut self) -> Result<(), V_::Error> {
        (**self).end()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (**self).size_hint()
    }
}

///////////////////////////////////////////////////////////////////////////////

pub trait EnumVisitor {
    type Value;

    fn visit<V>(&mut self, visitor: V) -> Result<Self::Value, V::Error>
        where V: VariantVisitor;
}

///////////////////////////////////////////////////////////////////////////////

pub trait VariantVisitor {
    type Error: Error;

    fn visit_variant<V>(&mut self) -> Result<V, Self::Error>
        where V: Deserialize;

    fn visit_value<V>(&mut self, _visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;
}

impl<'a, T> VariantVisitor for &'a mut T where T: VariantVisitor {
    type Error = T::Error;

    fn visit_variant<V>(&mut self) -> Result<V, T::Error>
        where V: Deserialize
    {
        (**self).visit_variant()
    }

    fn visit_value<V>(&mut self, visitor: V) -> Result<V::Value, T::Error>
        where V: Visitor,
    {
        (**self).visit_value(visitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

pub trait EnumSeqVisitor {
    type Value;

    fn visit<V>(&mut self, visitor: V) -> Result<Self::Value, V::Error>
        where V: SeqVisitor;
}

///////////////////////////////////////////////////////////////////////////////

pub trait EnumMapVisitor {
    type Value;

    fn visit<V>(&mut self, visitor: V) -> Result<Self::Value, V::Error>
        where V: MapVisitor;
}

///////////////////////////////////////////////////////////////////////////////

pub struct UnitVisitor;

impl Visitor for UnitVisitor {
    type Value = ();

    fn visit_unit<E>(&mut self) -> Result<(), E>
        where E: Error,
    {
        Ok(())
    }

    fn visit_seq<V>(&mut self, mut visitor: V) -> Result<(), V::Error>
        where V: SeqVisitor,
    {
        visitor.end()
    }
}

impl Deserialize for () {
    fn deserialize<D>(deserializer: &mut D) -> Result<(), D::Error>
        where D: Deserializer,
    {
        deserializer.visit(UnitVisitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

struct BoolVisitor;

impl Visitor for BoolVisitor {
    type Value = bool;

    fn visit_bool<E>(&mut self, v: bool) -> Result<bool, E>
        where E: Error,
    {
        Ok(v)
    }
}

impl Deserialize for bool {
    fn deserialize<D>(deserializer: &mut D) -> Result<bool, D::Error>
        where D: Deserializer,
    {
        deserializer.visit(BoolVisitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

macro_rules! impl_deserialize_num_method {
    ($src_ty:ty, $method:ident, $from_method:ident) => {
        #[inline]
        fn $method<E>(&mut self, v: $src_ty) -> Result<T, E>
            where E: Error,
        {
            match FromPrimitive::$from_method(v) {
                Some(v) => Ok(v),
                None => Err(Error::syntax_error()),
            }
        }
    }
}

pub struct PrimitiveVisitor<T> {
    marker: PhantomData<T>,
}

impl<T> PrimitiveVisitor<T> {
    #[inline]
    pub fn new() -> Self {
        PrimitiveVisitor {
            marker: PhantomData,
        }
    }
}

impl<
    T: Deserialize + FromPrimitive
> self::Visitor for PrimitiveVisitor<T> {
    type Value = T;

    impl_deserialize_num_method!(isize, visit_isize, from_isize);
    impl_deserialize_num_method!(i8, visit_i8, from_i8);
    impl_deserialize_num_method!(i16, visit_i16, from_i16);
    impl_deserialize_num_method!(i32, visit_i32, from_i32);
    impl_deserialize_num_method!(i64, visit_i64, from_i64);
    impl_deserialize_num_method!(usize, visit_usize, from_usize);
    impl_deserialize_num_method!(u8, visit_u8, from_u8);
    impl_deserialize_num_method!(u16, visit_u16, from_u16);
    impl_deserialize_num_method!(u32, visit_u32, from_u32);
    impl_deserialize_num_method!(u64, visit_u64, from_u64);
    impl_deserialize_num_method!(f32, visit_f32, from_f32);
    impl_deserialize_num_method!(f64, visit_f64, from_f64);
}

macro_rules! impl_deserialize_num {
    ($ty:ty) => {
        impl Deserialize for $ty {
            #[inline]
            fn deserialize<D>(deserializer: &mut D) -> Result<$ty, D::Error>
                where D: Deserializer,
            {
                deserializer.visit(PrimitiveVisitor::new())
            }
        }
    }
}

impl_deserialize_num!(isize);
impl_deserialize_num!(i8);
impl_deserialize_num!(i16);
impl_deserialize_num!(i32);
impl_deserialize_num!(i64);
impl_deserialize_num!(usize);
impl_deserialize_num!(u8);
impl_deserialize_num!(u16);
impl_deserialize_num!(u32);
impl_deserialize_num!(u64);
impl_deserialize_num!(f32);
impl_deserialize_num!(f64);

///////////////////////////////////////////////////////////////////////////////

struct CharVisitor;

impl Visitor for CharVisitor {
    type Value = char;

    #[inline]
    fn visit_char<E>(&mut self, v: char) -> Result<char, E>
        where E: Error,
    {
        Ok(v)
    }

    #[inline]
    fn visit_str<E>(&mut self, v: &str) -> Result<char, E>
        where E: Error,
    {
        let mut iter = v.chars();
        if let Some(v) = iter.next() {
            if iter.next().is_some() {
                Err(Error::syntax_error())
            } else {
                Ok(v)
            }
        } else {
            Err(Error::end_of_stream_error())
        }
    }
}

impl Deserialize for char {
    #[inline]
    fn deserialize<D>(deserializer: &mut D) -> Result<char, D::Error>
        where D: Deserializer,
    {
        deserializer.visit(CharVisitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

struct StringVisitor;

impl Visitor for StringVisitor {
    type Value = String;

    fn visit_str<E>(&mut self, v: &str) -> Result<String, E>
        where E: Error,
    {
        Ok(v.to_string())
    }

    fn visit_string<E>(&mut self, v: String) -> Result<String, E>
        where E: Error,
    {
        Ok(v)
    }
}

impl Deserialize for String {
    fn deserialize<D>(deserializer: &mut D) -> Result<String, D::Error>
        where D: Deserializer,
    {
        deserializer.visit(StringVisitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

struct OptionVisitor<T> {
    marker: PhantomData<T>,
}

impl<
    T: Deserialize,
> Visitor for OptionVisitor<T> {
    type Value = Option<T>;

    #[inline]
    fn visit_none<E>(&mut self) -> Result<Option<T>, E>
        where E: Error,
    {
        Ok(None)
    }

    #[inline]
    fn visit_some<D>(&mut self, deserializer: &mut D) -> Result<Option<T>, D::Error>
        where D: Deserializer,
    {
        Ok(Some(try!(Deserialize::deserialize(deserializer))))
    }
}

impl<T> Deserialize for Option<T> where T: Deserialize {
    fn deserialize<D>(deserializer: &mut D) -> Result<Option<T>, D::Error>
        where D: Deserializer,
    {
        deserializer.visit_option(OptionVisitor { marker: PhantomData })
    }
}

///////////////////////////////////////////////////////////////////////////////

pub struct BTreeSetVisitor<T> {
    marker: PhantomData<T>,
}

impl<T> BTreeSetVisitor<T> {
    pub fn new() -> Self {
        BTreeSetVisitor {
            marker: PhantomData,
        }
    }
}

impl<T> Visitor for BTreeSetVisitor<T>
    where T: Deserialize + Eq + Ord,
{
    type Value = BTreeSet<T>;

    #[inline]
    fn visit_unit<E>(&mut self) -> Result<BTreeSet<T>, E>
        where E: Error,
    {
        Ok(BTreeSet::new())
    }

    #[inline]
    fn visit_seq<V>(&mut self, mut visitor: V) -> Result<BTreeSet<T>, V::Error>
        where V: SeqVisitor,
    {
        let mut values = BTreeSet::new();

        while let Some(value) = try!(visitor.visit()) {
            values.insert(value);
        }

        try!(visitor.end());

        Ok(values)
    }
}

impl<T> Deserialize for BTreeSet<T>
    where T: Deserialize + Eq + Ord,
{
    fn deserialize<D>(deserializer: &mut D) -> Result<BTreeSet<T>, D::Error>
        where D: Deserializer,
    {
        deserializer.visit(BTreeSetVisitor::new())
    }
}

///////////////////////////////////////////////////////////////////////////////

pub struct HashSetVisitor<T> {
    marker: PhantomData<T>,
}

impl<T> HashSetVisitor<T> {
    pub fn new() -> Self {
        HashSetVisitor {
            marker: PhantomData,
        }
    }
}

impl<T> Visitor for HashSetVisitor<T>
    where T: Deserialize + Eq + Hash,
{
    type Value = HashSet<T>;

    #[inline]
    fn visit_unit<E>(&mut self) -> Result<HashSet<T>, E>
        where E: Error,
    {
        Ok(HashSet::new())
    }

    #[inline]
    fn visit_seq<V>(&mut self, mut visitor: V) -> Result<HashSet<T>, V::Error>
        where V: SeqVisitor,
    {
        let (len, _) = visitor.size_hint();
        let mut values = HashSet::with_capacity(len);

        while let Some(value) = try!(visitor.visit()) {
            values.insert(value);
        }

        try!(visitor.end());

        Ok(values)
    }
}

impl<T> Deserialize for HashSet<T>
    where T: Deserialize + Eq + Hash,
{
    fn deserialize<D>(deserializer: &mut D) -> Result<HashSet<T>, D::Error>
        where D: Deserializer,
    {
        deserializer.visit(HashSetVisitor::new())
    }
}

///////////////////////////////////////////////////////////////////////////////

pub struct VecVisitor<T> {
    marker: PhantomData<T>,
}

impl<T> VecVisitor<T> {
    pub fn new() -> Self {
        VecVisitor {
            marker: PhantomData,
        }
    }
}

impl<T> Visitor for VecVisitor<T> where T: Deserialize {
    type Value = Vec<T>;

    #[inline]
    fn visit_unit<E>(&mut self) -> Result<Vec<T>, E>
        where E: Error,
    {
        Ok(Vec::new())
    }

    #[inline]
    fn visit_seq<V>(&mut self, mut visitor: V) -> Result<Vec<T>, V::Error>
        where V: SeqVisitor,
    {
        let (len, _) = visitor.size_hint();
        let mut values = Vec::with_capacity(len);

        while let Some(value) = try!(visitor.visit()) {
            values.push(value);
        }

        try!(visitor.end());

        Ok(values)
    }
}

impl<T> Deserialize for Vec<T>
    where T: Deserialize,
{
    fn deserialize<D>(deserializer: &mut D) -> Result<Vec<T>, D::Error>
        where D: Deserializer,
    {
        deserializer.visit(VecVisitor::new())
    }
}

///////////////////////////////////////////////////////////////////////////////

macro_rules! tuple_impls {
    () => {};
    ($($visitor:ident => ($($name:ident),+),)+) => {
        $(
            struct $visitor<$($name,)+> {
                marker: PhantomData<($($name,)+)>,
            }

            impl<
                $($name: Deserialize,)+
            > Visitor for $visitor<$($name,)+> {
                type Value = ($($name,)+);

                #[inline]
                #[allow(non_snake_case)]
                fn visit_seq<V>(&mut self, mut visitor: V) -> Result<($($name,)+), V::Error>
                    where V: SeqVisitor,
                {
                    $(
                        let $name = match try!(visitor.visit()) {
                            Some(value) => value,
                            None => { return Err(Error::end_of_stream_error()); }
                        };
                     )+;

                    try!(visitor.end());

                    Ok(($($name,)+))
                }
            }

            impl<
                $($name: Deserialize),+
            > Deserialize for ($($name,)+) {
                #[inline]
                fn deserialize<D>(deserializer: &mut D) -> Result<($($name,)+), D::Error>
                    where D: Deserializer,
                {
                    deserializer.visit($visitor { marker: PhantomData })
                }
            }
        )+
    }
}

tuple_impls! {
    TupleVisitor1 => (T0),
    TupleVisitor2 => (T0, T1),
    TupleVisitor3 => (T0, T1, T2),
    TupleVisitor4 => (T0, T1, T2, T3),
    TupleVisitor5 => (T0, T1, T2, T3, T4),
    TupleVisitor6 => (T0, T1, T2, T3, T4, T5),
    TupleVisitor7 => (T0, T1, T2, T3, T4, T5, T6),
    TupleVisitor8 => (T0, T1, T2, T3, T4, T5, T6, T7),
    TupleVisitor9 => (T0, T1, T2, T3, T4, T5, T6, T7, T8),
    TupleVisitor10 => (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9),
    TupleVisitor11 => (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10),
    TupleVisitor12 => (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11),
}

///////////////////////////////////////////////////////////////////////////////

pub struct BTreeMapVisitor<K, V> {
    marker: PhantomData<BTreeMap<K, V>>,
}

impl<K, V> BTreeMapVisitor<K, V> {
    #[inline]
    pub fn new() -> Self {
        BTreeMapVisitor {
            marker: PhantomData,
        }
    }
}

impl<K, V> Visitor for BTreeMapVisitor<K, V>
    where K: Deserialize + Ord,
          V: Deserialize
{
    type Value = BTreeMap<K, V>;

    #[inline]
    fn visit_unit<E>(&mut self) -> Result<BTreeMap<K, V>, E>
        where E: Error,
    {
        Ok(BTreeMap::new())
    }

    #[inline]
    fn visit_map<Visitor>(&mut self, mut visitor: Visitor) -> Result<BTreeMap<K, V>, Visitor::Error>
        where Visitor: MapVisitor,
    {
        let mut values = BTreeMap::new();

        while let Some((key, value)) = try!(visitor.visit()) {
            values.insert(key, value);
        }

        try!(visitor.end());

        Ok(values)
    }
}

impl<K, V> Deserialize for BTreeMap<K, V>
    where K: Deserialize + Eq + Ord,
          V: Deserialize,
{
    fn deserialize<D>(deserializer: &mut D) -> Result<BTreeMap<K, V>, D::Error>
        where D: Deserializer,
    {
        deserializer.visit(BTreeMapVisitor::new())
    }
}

///////////////////////////////////////////////////////////////////////////////

pub struct HashMapVisitor<K, V> {
    marker: PhantomData<HashMap<K, V>>,
}

impl<K, V> HashMapVisitor<K, V> {
    #[inline]
    pub fn new() -> Self {
        HashMapVisitor {
            marker: PhantomData,
        }
    }
}

impl<K, V> Visitor for HashMapVisitor<K, V>
    where K: Deserialize + Eq + Hash,
          V: Deserialize,
{
    type Value = HashMap<K, V>;

    #[inline]
    fn visit_unit<E>(&mut self) -> Result<HashMap<K, V>, E>
        where E: Error,
    {
        Ok(HashMap::new())
    }

    #[inline]
    fn visit_map<V_>(&mut self, mut visitor: V_) -> Result<HashMap<K, V>, V_::Error>
        where V_: MapVisitor,
    {
        let (len, _) = visitor.size_hint();
        let mut values = HashMap::with_capacity(len);

        while let Some((key, value)) = try!(visitor.visit()) {
            values.insert(key, value);
        }

        try!(visitor.end());

        Ok(values)
    }
}

impl<K, V> Deserialize for HashMap<K, V>
    where K: Deserialize + Eq + Hash,
          V: Deserialize,
{
    fn deserialize<D>(deserializer: &mut D) -> Result<HashMap<K, V>, D::Error>
        where D: Deserializer,
    {
        deserializer.visit(HashMapVisitor::new())
    }
}

///////////////////////////////////////////////////////////////////////////////

struct PathBufVisitor;

impl Visitor for PathBufVisitor {
    type Value = path::PathBuf;

    fn visit_str<E>(&mut self, v: &str) -> Result<path::PathBuf, E>
        where E: Error,
    {
        Ok(path::PathBuf::new(&v))
    }

    fn visit_string<E>(&mut self, v: String) -> Result<path::PathBuf, E>
        where E: Error,
    {
        self.visit_str(&v)
    }
}

impl Deserialize for path::PathBuf {
    fn deserialize<D>(deserializer: &mut D) -> Result<path::PathBuf, D::Error>
        where D: Deserializer,
    {
        deserializer.visit(PathBufVisitor)
    }
}
