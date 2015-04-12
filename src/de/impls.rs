use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;
use std::marker::PhantomData;
use std::path;
use std::rc::Rc;
use std::sync::Arc;

use num::FromPrimitive;

use de::{
    Deserialize,
    Deserializer,
    Error,
    MapVisitor,
    SeqVisitor,
    Visitor,
};

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
> Visitor for PrimitiveVisitor<T> {
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

// FIXME: `VecMap` is unstable.
/*
pub struct VecMapVisitor<V> {
    marker: PhantomData<VecMap<V>>,
}

impl<V> VecMapVisitor<V> {
    #[inline]
    pub fn new() -> Self {
        VecMapVisitor {
            marker: PhantomData,
        }
    }
}

impl<V> Visitor for VecMapVisitor<V>
    where V: Deserialize,
{
    type Value = VecMap<V>;

    #[inline]
    fn visit_unit<E>(&mut self) -> Result<VecMap<V>, E>
        where E: Error,
    {
        Ok(VecMap::new())
    }

    #[inline]
    fn visit_map<V_>(&mut self, mut visitor: V_) -> Result<VecMap<V>, V_::Error>
        where V_: MapVisitor,
    {
        let (len, _) = visitor.size_hint();
        let mut values = VecMap::with_capacity(len);

        while let Some((key, value)) = try!(visitor.visit()) {
            values.insert(key, value);
        }

        try!(visitor.end());

        Ok(values)
    }
}

impl<V> Deserialize for VecMap<V>
    where V: Deserialize,
{
    fn deserialize<D>(deserializer: &mut D) -> Result<VecMap<V>, D::Error>
        where D: Deserializer,
    {
        deserializer.visit(VecMapVisitor::new())
    }
}
*/

///////////////////////////////////////////////////////////////////////////////

struct PathBufVisitor;

impl Visitor for PathBufVisitor {
    type Value = path::PathBuf;

    fn visit_str<E>(&mut self, v: &str) -> Result<path::PathBuf, E>
        where E: Error,
    {
        Ok(From::from(v))
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

///////////////////////////////////////////////////////////////////////////////

impl<T: Deserialize> Deserialize for Box<T> {
    fn deserialize<D>(deserializer: &mut D) -> Result<Box<T>, D::Error>
        where D: Deserializer,
    {
        let val = try!(Deserialize::deserialize(deserializer));
        Ok(Box::new(val))
    }
}

impl<T: Deserialize> Deserialize for Arc<T> {
    fn deserialize<D>(deserializer: &mut D) -> Result<Arc<T>, D::Error>
        where D: Deserializer,
    {
        let val = try!(Deserialize::deserialize(deserializer));
        Ok(Arc::new(val))
    }
}

impl<T: Deserialize> Deserialize for Rc<T> {
    fn deserialize<D>(deserializer: &mut D) -> Result<Rc<T>, D::Error>
        where D: Deserializer,
    {
        let val = try!(Deserialize::deserialize(deserializer));
        Ok(Rc::new(val))
    }
}
