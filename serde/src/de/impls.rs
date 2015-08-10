use std::borrow::Cow;
use std::collections::{
    BinaryHeap,
    BTreeMap,
    BTreeSet,
    LinkedList,
    HashMap,
    HashSet,
    VecDeque,
};
#[cfg(feature = "nightly")]
use collections::enum_set::{CLike, EnumSet};
#[cfg(feature = "nightly")]
use collections::vec_map::VecMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::path;
use std::rc::Rc;
use std::str;
use std::sync::Arc;

use num::FromPrimitive;

#[cfg(feature = "nightly")]
use core::nonzero::{NonZero, Zeroable};

#[cfg(feature = "nightly")]
use std::num::Zero;

use de::{
    Deserialize,
    Deserializer,
    EnumVisitor,
    Error,
    MapVisitor,
    SeqVisitor,
    Type,
    VariantVisitor,
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
        deserializer.visit_unit(UnitVisitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

pub struct BoolVisitor;

impl Visitor for BoolVisitor {
    type Value = bool;

    fn visit_bool<E>(&mut self, v: bool) -> Result<bool, E>
        where E: Error,
    {
        Ok(v)
    }

    fn visit_str<E>(&mut self, s: &str) -> Result<bool, E>
        where E: Error,
    {
        match s.trim() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(Error::type_mismatch(Type::Bool)),
        }
    }
}

impl Deserialize for bool {
    fn deserialize<D>(deserializer: &mut D) -> Result<bool, D::Error>
        where D: Deserializer,
    {
        deserializer.visit_bool(BoolVisitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

macro_rules! impl_deserialize_num_method {
    ($src_ty:ty, $method:ident, $from_method:ident, $ty:expr) => {
        #[inline]
        fn $method<E>(&mut self, v: $src_ty) -> Result<T, E>
            where E: Error,
        {
            match FromPrimitive::$from_method(v) {
                Some(v) => Ok(v),
                None => Err(Error::type_mismatch($ty)),
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
    T: Deserialize + FromPrimitive + str::FromStr
> Visitor for PrimitiveVisitor<T> {
    type Value = T;

    impl_deserialize_num_method!(isize, visit_isize, from_isize, Type::Isize);
    impl_deserialize_num_method!(i8, visit_i8, from_i8, Type::I8);
    impl_deserialize_num_method!(i16, visit_i16, from_i16, Type::I16);
    impl_deserialize_num_method!(i32, visit_i32, from_i32, Type::I32);
    impl_deserialize_num_method!(i64, visit_i64, from_i64, Type::I64);
    impl_deserialize_num_method!(usize, visit_usize, from_usize, Type::Usize);
    impl_deserialize_num_method!(u8, visit_u8, from_u8, Type::U8);
    impl_deserialize_num_method!(u16, visit_u16, from_u16, Type::U16);
    impl_deserialize_num_method!(u32, visit_u32, from_u32, Type::U32);
    impl_deserialize_num_method!(u64, visit_u64, from_u64, Type::U64);
    impl_deserialize_num_method!(f32, visit_f32, from_f32, Type::F32);
    impl_deserialize_num_method!(f64, visit_f64, from_f64, Type::F64);

    #[inline]
    fn visit_str<E>(&mut self, v: &str) -> Result<T, E>
        where E: Error,
    {
        str::FromStr::from_str(v.trim()).or(Err(Error::type_mismatch(Type::Str)))
    }
}

macro_rules! impl_deserialize_num {
    ($ty:ty, $method:ident) => {
        impl Deserialize for $ty {
            #[inline]
            fn deserialize<D>(deserializer: &mut D) -> Result<$ty, D::Error>
                where D: Deserializer,
            {
                deserializer.$method(PrimitiveVisitor::new())
            }
        }
    }
}

impl_deserialize_num!(isize, visit_isize);
impl_deserialize_num!(i8, visit_i8);
impl_deserialize_num!(i16, visit_i16);
impl_deserialize_num!(i32, visit_i32);
impl_deserialize_num!(i64, visit_i64);
impl_deserialize_num!(usize, visit_usize);
impl_deserialize_num!(u8, visit_u8);
impl_deserialize_num!(u16, visit_u16);
impl_deserialize_num!(u32, visit_u32);
impl_deserialize_num!(u64, visit_u64);
impl_deserialize_num!(f32, visit_f32);
impl_deserialize_num!(f64, visit_f64);

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
                Err(Error::type_mismatch(Type::Char))
            } else {
                Ok(v)
            }
        } else {
            Err(Error::end_of_stream())
        }
    }
}

impl Deserialize for char {
    #[inline]
    fn deserialize<D>(deserializer: &mut D) -> Result<char, D::Error>
        where D: Deserializer,
    {
        deserializer.visit_char(CharVisitor)
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

    fn visit_bytes<E>(&mut self, v: &[u8]) -> Result<String, E>
        where E: Error,
    {
        match str::from_utf8(v) {
            Ok(s) => Ok(s.to_string()),
            Err(_) => Err(Error::type_mismatch(Type::String)),
        }
    }

    fn visit_byte_buf<'a, E>(&mut self, v: Vec<u8>) -> Result<String, E>
        where E: Error,
    {
        match String::from_utf8(v) {
            Ok(s) => Ok(s),
            Err(_) => Err(Error::type_mismatch(Type::String)),
        }
    }
}

impl Deserialize for String {
    fn deserialize<D>(deserializer: &mut D) -> Result<String, D::Error>
        where D: Deserializer,
    {
        deserializer.visit_string(StringVisitor)
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

macro_rules! set_impl {
    (
        $ty:ty,
        < $($constraints:ident),* >,
        $visitor_name:ident,
        $visitor:ident,
        $ctor:expr,
        $with_capacity:expr,
        $insert:expr
    ) => {
        pub struct $visitor_name<T> {
            marker: PhantomData<T>,
        }

        impl<T> $visitor_name<T> {
            pub fn new() -> Self {
                $visitor_name {
                    marker: PhantomData,
                }
            }
        }

        impl<T> Visitor for $visitor_name<T>
            where T: $($constraints +)*,
        {
            type Value = $ty;

            #[inline]
            fn visit_unit<E>(&mut self) -> Result<$ty, E>
                where E: Error,
            {
                Ok($ctor)
            }

            #[inline]
            fn visit_seq<V>(&mut self, mut $visitor: V) -> Result<$ty, V::Error>
                where V: SeqVisitor,
            {
                let mut values = $with_capacity;

                while let Some(value) = try!($visitor.visit()) {
                    $insert(&mut values, value);
                }

                try!($visitor.end());

                Ok(values)
            }
        }

        impl<T> Deserialize for $ty
            where T: $($constraints +)*,
        {
            fn deserialize<D>(deserializer: &mut D) -> Result<$ty, D::Error>
                where D: Deserializer,
            {
                deserializer.visit_seq($visitor_name::new())
            }
        }
    }
}

set_impl!(
    BinaryHeap<T>,
    <Deserialize, Ord>,
    BinaryHeapVisitor,
    visitor,
    BinaryHeap::new(),
    BinaryHeap::with_capacity(visitor.size_hint().0),
    BinaryHeap::push);

set_impl!(
    BTreeSet<T>,
    <Deserialize, Eq, Ord>,
    BTreeSetVisitor,
    visitor,
    BTreeSet::new(),
    BTreeSet::new(),
    BTreeSet::insert);

#[cfg(feature = "nightly")]
set_impl!(
    EnumSet<T>,
    <Deserialize, CLike>,
    EnumSetVisitor,
    visitor,
    EnumSet::new(),
    EnumSet::new(),
    EnumSet::insert);

set_impl!(
    LinkedList<T>,
    <Deserialize>,
    LinkedListVisitor,
    visitor,
    LinkedList::new(),
    LinkedList::new(),
    LinkedList::push_back);

set_impl!(
    HashSet<T>,
    <Deserialize, Eq, Hash>,
    HashSetVisitor,
    visitor,
    HashSet::new(),
    HashSet::with_capacity(visitor.size_hint().0),
    HashSet::insert);

set_impl!(
    Vec<T>,
    <Deserialize>,
    VecVisitor,
    visitor,
    Vec::new(),
    Vec::with_capacity(visitor.size_hint().0),
    Vec::push);

set_impl!(
    VecDeque<T>,
    <Deserialize>,
    VecDequeVisitor,
    visitor,
    VecDeque::new(),
    VecDeque::with_capacity(visitor.size_hint().0),
    VecDeque::push_back);

///////////////////////////////////////////////////////////////////////////////

struct ArrayVisitor0<T> {
    marker: PhantomData<T>,
}

impl<T> ArrayVisitor0<T> {
    pub fn new() -> Self {
        ArrayVisitor0 {
            marker: PhantomData,
        }
    }
}

impl<T> Visitor for ArrayVisitor0<T> where T: Deserialize + Default {
    type Value = [T; 0];

    #[inline]
    fn visit_unit<E>(&mut self) -> Result<[T; 0], E>
        where E: Error,
    {
        Ok([T::default(); 0])
    }

    #[inline]
    fn visit_seq<V>(&mut self, mut visitor: V) -> Result<[T; 0], V::Error>
        where V: SeqVisitor,
    {
        try!(visitor.end());
        Ok([T::default(); 0])
    }
}

impl<T> Deserialize for [T; 0]
    where T: Deserialize + Default
{
    fn deserialize<D>(deserializer: &mut D) -> Result<[T; 0], D::Error>
        where D: Deserializer,
    {
        deserializer.visit_seq(ArrayVisitor0::new())
    }
}

macro_rules! array_impls {
    ($($visitor:ident, $len:expr => ($($name:ident),+),)+) => {
        $(
            struct $visitor<T> {
                marker: PhantomData<T>,
            }

            impl<T> $visitor<T> {
                pub fn new() -> Self {
                    $visitor {
                        marker: PhantomData
                    }
                }
            }

            impl<T> Visitor for $visitor<T> where T: Deserialize {
                type Value = [T; $len];

                #[inline]
                fn visit_seq<V>(&mut self, mut visitor: V) -> Result<[T; $len], V::Error>
                    where V: SeqVisitor,
                {
                    $(
                        let $name = match try!(visitor.visit()) {
                            Some(val) => val,
                            None => { return Err(Error::end_of_stream()); }
                        };
                    )+;

                    try!(visitor.end());

                    Ok([$($name,)+])
                }
            }

            impl<T> Deserialize for [T; $len]
                where T: Deserialize,
            {
                fn deserialize<D>(deserializer: &mut D) -> Result<[T; $len], D::Error>
                    where D: Deserializer,
                {
                    deserializer.visit_seq($visitor::new())
                }
            }
        )+
    }
}

array_impls! {
    ArrayVisitor1, 1 => (a),
    ArrayVisitor2, 2 => (a, b),
    ArrayVisitor3, 3 => (a, b, c),
    ArrayVisitor4, 4 => (a, b, c, d),
    ArrayVisitor5, 5 => (a, b, c, d, e),
    ArrayVisitor6, 6 => (a, b, c, d, e, f),
    ArrayVisitor7, 7 => (a, b, c, d, e, f, g),
    ArrayVisitor8, 8 => (a, b, c, d, e, f, g, h),
    ArrayVisitor9, 9 => (a, b, c, d, e, f, g, h, i),
    ArrayVisitor10, 10 => (a, b, c, d, e, f, g, h, i, j),
    ArrayVisitor11, 11 => (a, b, c, d, e, f, g, h, i, j, k),
    ArrayVisitor12, 12 => (a, b, c, d, e, f, g, h, i, j, k, l),
    ArrayVisitor13, 13 => (a, b, c, d, e, f, g, h, i, j, k, l, m),
    ArrayVisitor14, 14 => (a, b, c, d, e, f, g, h, i, j, k, l, m, n),
    ArrayVisitor15, 15 => (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o),
    ArrayVisitor16, 16 => (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p),
    ArrayVisitor17, 17 => (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q),
    ArrayVisitor18, 18 => (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r),
    ArrayVisitor19, 19 => (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s),
    ArrayVisitor20, 20 => (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s ,t),
    ArrayVisitor21, 21 => (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u),
    ArrayVisitor22, 22 => (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v),
    ArrayVisitor23, 23 => (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w),
    ArrayVisitor24, 24 => (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x),
    ArrayVisitor25, 25 => (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x,
                           y),
    ArrayVisitor26, 26 => (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x,
                           y, z),
    ArrayVisitor27, 27 => (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x,
                           y, z, aa),
    ArrayVisitor28, 28 => (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x,
                           y, z, aa, ab),
    ArrayVisitor29, 29 => (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x,
                           y, z, aa, ab, ac),
    ArrayVisitor30, 30 => (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x,
                           y, z, aa, ab, ac, ad),
    ArrayVisitor31, 31 => (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x,
                           y, z, aa, ab, ac, ad, ae),
    ArrayVisitor32, 32 => (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x,
                           y, z, aa, ab, ac, ad, ae, af),
}

///////////////////////////////////////////////////////////////////////////////

macro_rules! tuple_impls {
    () => {};
    ($($len:expr => $visitor:ident => ($($name:ident),+),)+) => {
        $(
            pub struct $visitor<$($name,)+> {
                marker: PhantomData<($($name,)+)>,
            }

            impl<
                $($name: Deserialize,)+
            > $visitor<$($name,)+> {
                pub fn new() -> Self {
                    $visitor { marker: PhantomData }
                }
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
                            None => { return Err(Error::end_of_stream()); }
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
                    deserializer.visit_tuple($len, $visitor::new())
                }
            }
        )+
    }
}

tuple_impls! {
    1 => TupleVisitor1 => (T0),
    2 => TupleVisitor2 => (T0, T1),
    3 => TupleVisitor3 => (T0, T1, T2),
    4 => TupleVisitor4 => (T0, T1, T2, T3),
    5 => TupleVisitor5 => (T0, T1, T2, T3, T4),
    6 => TupleVisitor6 => (T0, T1, T2, T3, T4, T5),
    7 => TupleVisitor7 => (T0, T1, T2, T3, T4, T5, T6),
    8 => TupleVisitor8 => (T0, T1, T2, T3, T4, T5, T6, T7),
    9 => TupleVisitor9 => (T0, T1, T2, T3, T4, T5, T6, T7, T8),
    10 => TupleVisitor10 => (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9),
    11 => TupleVisitor11 => (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10),
    12 => TupleVisitor12 => (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11),
}

///////////////////////////////////////////////////////////////////////////////

macro_rules! map_impl {
    (
        $ty:ty,
        < $($constraints:ident),* >,
        $visitor_name:ident,
        $visitor:ident,
        $ctor:expr,
        $with_capacity:expr,
        $insert:expr
    ) => {
        pub struct $visitor_name<K, V> {
            marker: PhantomData<$ty>,
        }

        impl<K, V> $visitor_name<K, V> {
            pub fn new() -> Self {
                $visitor_name {
                    marker: PhantomData,
                }
            }
        }

        impl<K, V> Visitor for $visitor_name<K, V>
            where K: $($constraints +)*,
                  V: Deserialize,
        {
            type Value = $ty;

            #[inline]
            fn visit_unit<E>(&mut self) -> Result<$ty, E>
                where E: Error,
            {
                Ok($ctor)
            }

            #[inline]
            fn visit_map<Visitor>(&mut self, mut $visitor: Visitor) -> Result<$ty, Visitor::Error>
                where Visitor: MapVisitor,
            {
                let mut values = $with_capacity;

                while let Some((key, value)) = try!($visitor.visit()) {
                    $insert(&mut values, key, value);
                }

                try!($visitor.end());

                Ok(values)
            }
        }

        impl<K, V> Deserialize for $ty
            where K: $($constraints +)*,
                  V: Deserialize,
        {
            fn deserialize<D>(deserializer: &mut D) -> Result<$ty, D::Error>
                where D: Deserializer,
            {
                deserializer.visit_map($visitor_name::new())
            }
        }
    }
}

map_impl!(
    BTreeMap<K, V>,
    <Deserialize, Eq, Ord>,
    BTreeMapVisitor,
    visitor,
    BTreeMap::new(),
    BTreeMap::new(),
    BTreeMap::insert);

map_impl!(
    HashMap<K, V>,
    <Deserialize, Eq, Hash>,
    HashMapVisitor,
    visitor,
    HashMap::new(),
    HashMap::with_capacity(visitor.size_hint().0),
    HashMap::insert);

///////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "nightly")]
pub struct VecMapVisitor<V> {
    marker: PhantomData<VecMap<V>>,
}

#[cfg(feature = "nightly")]
impl<V> VecMapVisitor<V> {
    #[inline]
    pub fn new() -> Self {
        VecMapVisitor {
            marker: PhantomData,
        }
    }
}

#[cfg(feature = "nightly")]
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

#[cfg(feature = "nightly")]
impl<V> Deserialize for VecMap<V>
    where V: Deserialize,
{
    fn deserialize<D>(deserializer: &mut D) -> Result<VecMap<V>, D::Error>
        where D: Deserializer,
    {
        deserializer.visit_map(VecMapVisitor::new())
    }
}

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

impl<'a, T: ?Sized> Deserialize for Cow<'a, T> where T: ToOwned, T::Owned: Deserialize, {
    #[inline]
    fn deserialize<D>(deserializer: &mut D) -> Result<Cow<'a, T>, D::Error>
        where D: Deserializer,
    {
        let val = try!(Deserialize::deserialize(deserializer));
        Ok(Cow::Owned(val))
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "nightly")]
impl<T> Deserialize for NonZero<T> where T: Deserialize + PartialEq + Zeroable + Zero {
    fn deserialize<D>(deserializer: &mut D) -> Result<NonZero<T>, D::Error> where D: Deserializer {
        let value = try!(Deserialize::deserialize(deserializer));
        if value == Zero::zero() {
            return Err(Error::syntax("expected a non-zero value"))
        }
        unsafe {
            Ok(NonZero::new(value))
        }
    }
}

///////////////////////////////////////////////////////////////////////////////


impl<T, E> Deserialize for Result<T, E> where T: Deserialize, E: Deserialize {
    fn deserialize<D>(deserializer: &mut D) -> Result<Result<T, E>, D::Error>
                      where D: Deserializer {
        enum Field {
            Ok,
            Err,
        }

        impl Deserialize for Field {
            #[inline]
            fn deserialize<D>(deserializer: &mut D) -> Result<Field, D::Error>
                where D: Deserializer
            {
                struct FieldVisitor;

                impl ::de::Visitor for FieldVisitor {
                    type Value = Field;

                    fn visit_usize<E>(&mut self, value: usize) -> Result<Field, E> where E: Error {
                        match value {
                            0 => Ok(Field::Ok),
                            1 => Ok(Field::Err),
                            _ => Err(Error::unknown_field(&value.to_string())),
                        }
                    }

                    fn visit_str<E>(&mut self, value: &str) -> Result<Field, E> where E: Error {
                        match value {
                            "Ok" => Ok(Field::Ok),
                            "Err" => Ok(Field::Err),
                            _ => Err(Error::unknown_field(value)),
                        }
                    }

                    fn visit_bytes<E>(&mut self, value: &[u8]) -> Result<Field, E> where E: Error {
                        match value {
                            b"Ok" => Ok(Field::Ok),
                            b"Err" => Ok(Field::Err),
                            _ => {
                                match str::from_utf8(value) {
                                    Ok(value) => Err(Error::unknown_field(value)),
                                    Err(_) => Err(Error::type_mismatch(Type::String)),
                                }
                            }
                        }
                    }
                }

                deserializer.visit(FieldVisitor)
            }
        }

        struct Visitor<T, E>(PhantomData<Result<T, E>>);

        impl<T, E> EnumVisitor for Visitor<T, E>
            where T: Deserialize,
                  E: Deserialize
        {
            type Value = Result<T, E>;

            fn visit<V>(&mut self, mut visitor: V) -> Result<Result<T, E>, V::Error>
                where V: VariantVisitor
            {
                match try!(visitor.visit_variant()) {
                    Field::Ok => {
                        let value = try!(visitor.visit_newtype());
                        Ok(Ok(value))
                    }
                    Field::Err => {
                        let value = try!(visitor.visit_newtype());
                        Ok(Err(value))
                    }
                }
            }
        }

        const VARIANTS: &'static [&'static str] = &["Ok", "Err"];

        deserializer.visit_enum("Result", VARIANTS, Visitor(PhantomData))
    }
}
