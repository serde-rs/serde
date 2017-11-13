// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use lib::*;

use de::{Deserialize, Deserializer, EnumAccess, Error, SeqAccess, Unexpected, VariantAccess,
         Visitor};

#[cfg(any(feature = "std", feature = "alloc"))]
use de::MapAccess;

use de::from_primitive::FromPrimitive;
use private::de::DeserializeFromSeed;

#[cfg(any(feature = "std", feature = "alloc"))]
use private::de::size_hint;

////////////////////////////////////////////////////////////////////////////////

struct UnitVisitor;

impl<'de> Visitor<'de> for UnitVisitor {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("unit")
    }

    fn visit_unit<E>(self) -> Result<(), E>
    where
        E: Error,
    {
        Ok(())
    }
}

impl<'de> Deserialize<'de> for () {
    fn deserialize<D>(deserializer: D) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_unit(UnitVisitor)
    }
}

////////////////////////////////////////////////////////////////////////////////

struct BoolVisitor;


impl<'de> Visitor<'de> for BoolVisitor {
    type Value = bool;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a boolean")
    }

    fn visit_bool<E>(self, v: bool) -> Result<bool, E>
    where
        E: Error,
    {
        Ok(v)
    }
}

impl<'de> Deserialize<'de> for bool {
    fn deserialize<D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bool(BoolVisitor)
    }
}

////////////////////////////////////////////////////////////////////////////////

macro_rules! visit_integer_method {
    ($src_ty:ident, $method:ident, $from_method:ident, $group:ident, $group_ty:ident) => {
        #[inline]
        fn $method<E>(self, v: $src_ty) -> Result<Self::Value, E>
        where
            E: Error,
        {
            match FromPrimitive::$from_method(v) {
                Some(v) => Ok(v),
                None => Err(Error::invalid_value(Unexpected::$group(v as $group_ty), &self)),
            }
        }
    }
}

macro_rules! visit_float_method {
    ($src_ty:ident, $method:ident) => {
        #[inline]
        fn $method<E>(self, v: $src_ty) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(v as Self::Value)
        }
    }
}

macro_rules! impl_deserialize_num {
    ($ty:ident, $method:ident, $($visit:ident),*) => {
        impl<'de> Deserialize<'de> for $ty {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<$ty, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct PrimitiveVisitor;

                impl<'de> Visitor<'de> for PrimitiveVisitor {
                    type Value = $ty;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str(stringify!($ty))
                    }

                    $(
                        impl_deserialize_num!($visit $ty);
                    )*
                }

                deserializer.$method(PrimitiveVisitor)
            }
        }
    };

    (integer $ty:ident) => {
        visit_integer_method!(i8, visit_i8, from_i8, Signed, i64);
        visit_integer_method!(i16, visit_i16, from_i16, Signed, i64);
        visit_integer_method!(i32, visit_i32, from_i32, Signed, i64);
        visit_integer_method!(i64, visit_i64, from_i64, Signed, i64);

        visit_integer_method!(u8, visit_u8, from_u8, Unsigned, u64);
        visit_integer_method!(u16, visit_u16, from_u16, Unsigned, u64);
        visit_integer_method!(u32, visit_u32, from_u32, Unsigned, u64);
        visit_integer_method!(u64, visit_u64, from_u64, Unsigned, u64);
    };

    (float $ty:ident) => {
        visit_float_method!(f32, visit_f32);
        visit_float_method!(f64, visit_f64);
    };
}

impl_deserialize_num!(i8, deserialize_i8, integer);
impl_deserialize_num!(i16, deserialize_i16, integer);
impl_deserialize_num!(i32, deserialize_i32, integer);
impl_deserialize_num!(i64, deserialize_i64, integer);
impl_deserialize_num!(isize, deserialize_i64, integer);

impl_deserialize_num!(u8, deserialize_u8, integer);
impl_deserialize_num!(u16, deserialize_u16, integer);
impl_deserialize_num!(u32, deserialize_u32, integer);
impl_deserialize_num!(u64, deserialize_u64, integer);
impl_deserialize_num!(usize, deserialize_u64, integer);

impl_deserialize_num!(f32, deserialize_f32, integer, float);
impl_deserialize_num!(f64, deserialize_f64, integer, float);

////////////////////////////////////////////////////////////////////////////////

struct CharVisitor;

impl<'de> Visitor<'de> for CharVisitor {
    type Value = char;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a character")
    }

    #[inline]
    fn visit_char<E>(self, v: char) -> Result<char, E>
    where
        E: Error,
    {
        Ok(v)
    }

    #[inline]
    fn visit_str<E>(self, v: &str) -> Result<char, E>
    where
        E: Error,
    {
        let mut iter = v.chars();
        match (iter.next(), iter.next()) {
            (Some(c), None) => Ok(c),
            _ => Err(Error::invalid_value(Unexpected::Str(v), &self)),
        }
    }
}

impl<'de> Deserialize<'de> for char {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<char, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_char(CharVisitor)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(any(feature = "std", feature = "alloc"))]
struct StringVisitor;
#[cfg(any(feature = "std", feature = "alloc"))]
struct StringFromVisitor<'a>(&'a mut String);

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'de> Visitor<'de> for StringVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<String, E>
    where
        E: Error,
    {
        Ok(v.to_owned())
    }

    fn visit_string<E>(self, v: String) -> Result<String, E>
    where
        E: Error,
    {
        Ok(v)
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<String, E>
    where
        E: Error,
    {
        match str::from_utf8(v) {
            Ok(s) => Ok(s.to_owned()),
            Err(_) => Err(Error::invalid_value(Unexpected::Bytes(v), &self)),
        }
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<String, E>
    where
        E: Error,
    {
        match String::from_utf8(v) {
            Ok(s) => Ok(s),
            Err(e) => Err(Error::invalid_value(Unexpected::Bytes(&e.into_bytes()), &self),),
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'a, 'de> Visitor<'de> for StringFromVisitor<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<(), E>
    where
        E: Error,
    {
        self.0.clear();
        self.0.push_str(v);
        Ok(())
    }

    fn visit_string<E>(self, v: String) -> Result<(), E>
    where
        E: Error,
    {
        *self.0 = v;
        Ok(())
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<(), E>
    where
        E: Error,
    {
        match str::from_utf8(v) {
            Ok(s) => {
                self.0.clear();
                self.0.push_str(s);
                Ok(())
            }
            Err(_) => Err(Error::invalid_value(Unexpected::Bytes(v), &self)),
        }
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<(), E>
    where
        E: Error,
    {
        match String::from_utf8(v) {
            Ok(s) => {
                *self.0 = s;
                Ok(())
            }
            Err(e) => Err(Error::invalid_value(Unexpected::Bytes(&e.into_bytes()), &self),),
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'de> Deserialize<'de> for String {
    fn deserialize<D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(StringVisitor)
    }

    fn deserialize_from<D>(&mut self, deserializer: D) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(StringFromVisitor(self))
    }
}

////////////////////////////////////////////////////////////////////////////////

struct StrVisitor;

impl<'a> Visitor<'a> for StrVisitor {
    type Value = &'a str;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a borrowed string")
    }

    fn visit_borrowed_str<E>(self, v: &'a str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(v) // so easy
    }

    fn visit_borrowed_bytes<E>(self, v: &'a [u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        str::from_utf8(v).map_err(|_| Error::invalid_value(Unexpected::Bytes(v), &self))
    }
}

impl<'de: 'a, 'a> Deserialize<'de> for &'a str {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(StrVisitor)
    }
}

////////////////////////////////////////////////////////////////////////////////

struct BytesVisitor;

impl<'a> Visitor<'a> for BytesVisitor {
    type Value = &'a [u8];

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a borrowed byte array")
    }

    fn visit_borrowed_bytes<E>(self, v: &'a [u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(v)
    }

    fn visit_borrowed_str<E>(self, v: &'a str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(v.as_bytes())
    }
}

impl<'de: 'a, 'a> Deserialize<'de> for &'a [u8] {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(BytesVisitor)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "std")]
struct CStringVisitor;

#[cfg(feature = "std")]
impl<'de> Visitor<'de> for CStringVisitor {
    type Value = CString;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("byte array")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<CString, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let len = size_hint::cautious(seq.size_hint());
        let mut values = Vec::with_capacity(len);

        while let Some(value) = try!(seq.next_element()) {
            values.push(value);
        }

        CString::new(values).map_err(Error::custom)
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<CString, E>
    where
        E: Error,
    {
        CString::new(v).map_err(Error::custom)
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<CString, E>
    where
        E: Error,
    {
        CString::new(v).map_err(Error::custom)
    }

    fn visit_str<E>(self, v: &str) -> Result<CString, E>
    where
        E: Error,
    {
        CString::new(v).map_err(Error::custom)
    }

    fn visit_string<E>(self, v: String) -> Result<CString, E>
    where
        E: Error,
    {
        CString::new(v).map_err(Error::custom)
    }
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for CString {
    fn deserialize<D>(deserializer: D) -> Result<CString, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_byte_buf(CStringVisitor)
    }
}

macro_rules! forwarded_impl {
    (( $($id: ident),* ), $ty: ty, $func: expr) => {
        impl<'de $(, $id : Deserialize<'de>,)*> Deserialize<'de> for $ty {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer).map($func)
            }
        }
    }
}

#[cfg(all(feature = "std", feature = "unstable"))]
forwarded_impl!((), Box<CStr>, CString::into_boxed_c_str);

////////////////////////////////////////////////////////////////////////////////

struct OptionVisitor<T> {
    marker: PhantomData<T>,
}
struct OptionFromVisitor<'a, T: 'a>(&'a mut Option<T>);

impl<'de, T> Visitor<'de> for OptionVisitor<T>
where
    T: Deserialize<'de>,
{
    type Value = Option<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("option")
    }

    #[inline]
    fn visit_unit<E>(self) -> Result<Option<T>, E>
    where
        E: Error,
    {
        Ok(None)
    }

    #[inline]
    fn visit_none<E>(self) -> Result<Option<T>, E>
    where
        E: Error,
    {
        Ok(None)
    }

    #[inline]
    fn visit_some<D>(self, deserializer: D) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(deserializer).map(Some)
    }
}

impl<'a, 'de, T> Visitor<'de> for OptionFromVisitor<'a, T>
where
    T: Deserialize<'de>,
{
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("option")
    }

    #[inline]
    fn visit_unit<E>(self) -> Result<(), E>
    where
        E: Error,
    {
        *self.0 = None;
        Ok(())
    }

    #[inline]
    fn visit_none<E>(self) -> Result<(), E>
    where
        E: Error,
    {
        *self.0 = None;
        Ok(())
    }

    #[inline]
    fn visit_some<D>(self, deserializer: D) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        // The some enum's repr is opaque, so we can't play cute tricks with
        // its tag to build this in place unconditionally.
        //
        // FIXME: investigate whether branching on the old value being Some to
        // deserialize_from the value is profitable (probably data-dependent?)
        *self.0 = try!(T::deserialize(deserializer).map(Some));
        Ok(())
    }
}

impl<'de, T> Deserialize<'de> for Option<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_option(OptionVisitor { marker: PhantomData })
    }

    fn deserialize_from<D>(&mut self, deserializer: D) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_option(OptionFromVisitor(self))
    }
}

////////////////////////////////////////////////////////////////////////////////

struct PhantomDataVisitor<T> {
    marker: PhantomData<T>,
}

impl<'de, T> Visitor<'de> for PhantomDataVisitor<T> {
    type Value = PhantomData<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("unit")
    }

    #[inline]
    fn visit_unit<E>(self) -> Result<PhantomData<T>, E>
    where
        E: Error,
    {
        Ok(PhantomData)
    }
}

impl<'de, T> Deserialize<'de> for PhantomData<T> {
    fn deserialize<D>(deserializer: D) -> Result<PhantomData<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let visitor = PhantomDataVisitor { marker: PhantomData };
        deserializer.deserialize_unit_struct("PhantomData", visitor)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(any(feature = "std", feature = "alloc"))]
macro_rules! seq_impl {
    (
        $ty:ident < T $(: $tbound1:ident $(+ $tbound2:ident)*)* $(, $typaram:ident : $bound1:ident $(+ $bound2:ident)*)* >,
        $access:ident,
        $ctor:expr,
        $clear:expr,
        $with_capacity:expr,
        $reserve:expr,
        $insert:expr
    ) => {
        impl<'de, T $(, $typaram)*> Deserialize<'de> for $ty<T $(, $typaram)*>
        where
            T: Deserialize<'de> $(+ $tbound1 $(+ $tbound2)*)*,
            $($typaram: $bound1 $(+ $bound2)*,)*
        {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct SeqVisitor<T $(, $typaram)*> {
                    marker: PhantomData<$ty<T $(, $typaram)*>>,
                }

                impl<'de, T $(, $typaram)*> Visitor<'de> for SeqVisitor<T $(, $typaram)*>
                where
                    T: Deserialize<'de> $(+ $tbound1 $(+ $tbound2)*)*,
                    $($typaram: $bound1 $(+ $bound2)*,)*
                {
                    type Value = $ty<T $(, $typaram)*>;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("a sequence")
                    }

                    #[inline]
                    fn visit_seq<A>(self, mut $access: A) -> Result<Self::Value, A::Error>
                    where
                        A: SeqAccess<'de>,
                    {
                        let mut values = $with_capacity;

                        while let Some(value) = try!($access.next_element()) {
                            $insert(&mut values, value);
                        }

                        Ok(values)
                    }
                }

                let visitor = SeqVisitor { marker: PhantomData };
                deserializer.deserialize_seq(visitor)
            }

            fn deserialize_from<D>(&mut self, deserializer: D) -> Result<(), D::Error>
            where
                D: Deserializer<'de>,
            {
                struct SeqVisitor<'a, T: 'a $(, $typaram: 'a)*>(&'a mut $ty<T $(, $typaram)*>);

                impl<'a, 'de, T $(, $typaram)*> Visitor<'de> for SeqVisitor<'a, T $(, $typaram)*>
                where
                    T: Deserialize<'de> $(+ $tbound1 $(+ $tbound2)*)*,
                    $($typaram: $bound1 $(+ $bound2)*,)*
                {
                    type Value = ();

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("a sequence")
                    }

                    #[inline]
                    fn visit_seq<A>(mut self, mut $access: A) -> Result<(), A::Error>
                    where
                        A: SeqAccess<'de>,
                    {
                        $clear(&mut self.0);
                        $reserve(&mut self.0, size_hint::cautious($access.size_hint()));

                        // FIXME: try to overwrite old values here? (Vec, VecDeque, LinkedList)
                        while let Some(value) = try!($access.next_element()) {
                            $insert(&mut self.0, value);
                        }

                        Ok(())
                    }
                }

                deserializer.deserialize_seq(SeqVisitor(self))
            }
        }
    }
}

// Dummy impl of reserve
#[cfg(any(feature = "std", feature = "alloc"))]
fn nop_reserve<T, U>(_x: T, _y: U) { }

#[cfg(any(feature = "std", feature = "alloc"))]
seq_impl!(
    BinaryHeap<T: Ord>,
    seq,
    BinaryHeap::new(),
    BinaryHeap::clear,
    BinaryHeap::with_capacity(size_hint::cautious(seq.size_hint())),
    BinaryHeap::reserve,
    BinaryHeap::push);

#[cfg(any(feature = "std", feature = "alloc"))]
seq_impl!(
    BTreeSet<T: Eq + Ord>,
    seq,
    BTreeSet::new(),
    BTreeSet::clear,
    BTreeSet::new(),
    nop_reserve,
    BTreeSet::insert);

#[cfg(any(feature = "std", feature = "alloc"))]
seq_impl!(
    LinkedList<T>,
    seq,
    LinkedList::new(),
    LinkedList::clear,
    LinkedList::new(),
    nop_reserve,
    LinkedList::push_back);

#[cfg(feature = "std")]
seq_impl!(
    HashSet<T: Eq + Hash, S: BuildHasher + Default>,
    seq,
    HashSet::with_hasher(S::default()),
    HashSet::clear,
    HashSet::with_capacity_and_hasher(size_hint::cautious(seq.size_hint()), S::default()),
    HashSet::reserve,
    HashSet::insert);

#[cfg(any(feature = "std", feature = "alloc"))]
seq_impl!(
    Vec<T>,
    seq,
    Vec::new(),
    Vec::clear,
    Vec::with_capacity(size_hint::cautious(seq.size_hint())),
    Vec::reserve,
    Vec::push);

#[cfg(any(feature = "std", feature = "alloc"))]
seq_impl!(
    VecDeque<T>,
    seq,
    VecDeque::new(),
    VecDeque::clear,
    VecDeque::with_capacity(size_hint::cautious(seq.size_hint())),
    VecDeque::reserve,
    VecDeque::push_back);

////////////////////////////////////////////////////////////////////////////////

struct ArrayVisitor<A> {
    marker: PhantomData<A>,
}
struct ArrayFromVisitor<'a, A: 'a>(&'a mut A);

impl<A> ArrayVisitor<A> {
    fn new() -> Self {
        ArrayVisitor { marker: PhantomData }
    }
}

impl<'de, T> Visitor<'de> for ArrayVisitor<[T; 0]> {
    type Value = [T; 0];

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an empty array")
    }

    #[inline]
    fn visit_seq<A>(self, _: A) -> Result<[T; 0], A::Error>
    where
        A: SeqAccess<'de>,
    {
        Ok([])
    }
}

// Does not require T: Deserialize<'de>.
impl<'de, T> Deserialize<'de> for [T; 0] {
    fn deserialize<D>(deserializer: D) -> Result<[T; 0], D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(0, ArrayVisitor::<[T; 0]>::new())
    }
}

macro_rules! array_impls {
    ($($len:expr => ($($n:tt $name:ident)+))+) => {
        $(
            impl<'de, T> Visitor<'de> for ArrayVisitor<[T; $len]>
            where
                T: Deserialize<'de>,
            {
                type Value = [T; $len];

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str(concat!("an array of length ", $len))
                }

                #[inline]
                fn visit_seq<A>(self, mut seq: A) -> Result<[T; $len], A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    $(
                        let $name = match try!(seq.next_element()) {
                            Some(val) => val,
                            None => return Err(Error::invalid_length($n, &self)),
                        };
                    )+

                    Ok([$($name),+])
                }
            }

            impl<'a, 'de, T> Visitor<'de> for ArrayFromVisitor<'a, [T; $len]>
            where
                T: Deserialize<'de>,
            {
                type Value = ();

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str(concat!("an array of length ", $len))
                }

                #[inline]
                fn visit_seq<A>(self, mut seq: A) -> Result<(), A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    let mut fail_idx = None;
                    for (idx, dest) in self.0[..].iter_mut().enumerate() {
                        if try!(seq.next_element_seed(DeserializeFromSeed(dest))).is_none() {
                            fail_idx = Some(idx);
                            break;
                        }
                    }
                    if let Some(idx) = fail_idx {
                        return Err(Error::invalid_length(idx, &self));
                    }
                    Ok(())
                }
            }

            impl<'de, T> Deserialize<'de> for [T; $len]
            where
                T: Deserialize<'de>,
            {
                fn deserialize<D>(deserializer: D) -> Result<[T; $len], D::Error>
                where
                    D: Deserializer<'de>,
                {
                    deserializer.deserialize_tuple($len, ArrayVisitor::<[T; $len]>::new())
                }

                fn deserialize_from<D>(&mut self, deserializer: D) -> Result<(), D::Error>
                where
                    D: Deserializer<'de>,
                {
                    deserializer.deserialize_tuple($len, ArrayFromVisitor(self))
                }
            }
        )+
    }
}

array_impls! {
    1 => (0 a)
    2 => (0 a 1 b)
    3 => (0 a 1 b 2 c)
    4 => (0 a 1 b 2 c 3 d)
    5 => (0 a 1 b 2 c 3 d 4 e)
    6 => (0 a 1 b 2 c 3 d 4 e 5 f)
    7 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g)
    8 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h)
    9 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i)
    10 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j)
    11 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k)
    12 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l)
    13 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m)
    14 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n)
    15 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o)
    16 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p)
    17 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q)
    18 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r)
    19 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r 18 s)
    20 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r 18 s 19 t)
    21 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r 18 s 19 t 20 u)
    22 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r 18 s 19 t 20 u 21 v)
    23 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r 18 s 19 t 20 u 21 v 22 w)
    24 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r 18 s 19 t 20 u 21 v 22 w 23 x)
    25 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r 18 s 19 t 20 u 21 v 22 w 23 x 24 y)
    26 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r 18 s 19 t 20 u 21 v 22 w 23 x 24 y 25 z)
    27 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r 18 s 19 t 20 u 21 v 22 w 23 x 24 y 25 z 26 aa)
    28 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r 18 s 19 t 20 u 21 v 22 w 23 x 24 y 25 z 26 aa 27 ab)
    29 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r 18 s 19 t 20 u 21 v 22 w 23 x 24 y 25 z 26 aa 27 ab 28 ac)
    30 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r 18 s 19 t 20 u 21 v 22 w 23 x 24 y 25 z 26 aa 27 ab 28 ac 29 ad)
    31 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r 18 s 19 t 20 u 21 v 22 w 23 x 24 y 25 z 26 aa 27 ab 28 ac 29 ad 30 ae)
    32 => (0 a 1 b 2 c 3 d 4 e 5 f 6 g 7 h 8 i 9 j 10 k 11 l 12 m 13 n 14 o 15 p 16 q 17 r 18 s 19 t 20 u 21 v 22 w 23 x 24 y 25 z 26 aa 27 ab 28 ac 29 ad 30 ae 31 af)
}

////////////////////////////////////////////////////////////////////////////////

macro_rules! tuple_impls {
    ($($len:tt => ($($n:tt $name:ident)+))+) => {
        $(
            impl<'de, $($name: Deserialize<'de>),+> Deserialize<'de> for ($($name,)+) {
                #[inline]
                fn deserialize<D>(deserializer: D) -> Result<($($name,)+), D::Error>
                where
                    D: Deserializer<'de>,
                {
                    struct TupleVisitor<$($name,)+> {
                        marker: PhantomData<($($name,)+)>,
                    }

                    impl<'de, $($name: Deserialize<'de>),+> Visitor<'de> for TupleVisitor<$($name,)+> {
                        type Value = ($($name,)+);

                        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                            formatter.write_str(concat!("a tuple of size ", $len))
                        }

                        #[inline]
                        #[allow(non_snake_case)]
                        fn visit_seq<A>(self, mut seq: A) -> Result<($($name,)+), A::Error>
                        where
                            A: SeqAccess<'de>,
                        {
                            $(
                                let $name = match try!(seq.next_element()) {
                                    Some(value) => value,
                                    None => return Err(Error::invalid_length($n, &self)),
                                };
                            )+

                            Ok(($($name,)+))
                        }
                    }

                    deserializer.deserialize_tuple($len, TupleVisitor { marker: PhantomData })
                }

                #[inline]
                fn deserialize_from<D>(&mut self, deserializer: D) -> Result<(), D::Error>
                where
                    D: Deserializer<'de>,
                {
                    struct TupleVisitor<'a, $($name: 'a,)+>(&'a mut ($($name,)+));

                    impl<'a, 'de, $($name: Deserialize<'de>),+> Visitor<'de> for TupleVisitor<'a, $($name,)+> {
                        type Value = ();

                        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                            formatter.write_str(concat!("a tuple of size ", $len))
                        }

                        #[inline]
                        #[allow(non_snake_case)]
                        fn visit_seq<A>(self, mut seq:                                                                      A) -> Result<(), A::Error>
                        where
                            A: SeqAccess<'de>,
                        {
                            $(
                                if try!(seq.next_element_seed(DeserializeFromSeed(&mut (self.0).$n))).is_none() {
                                    return Err(Error::invalid_length($n, &self));
                                }
                            )+

                            Ok(())
                        }
                    }

                    deserializer.deserialize_tuple($len, TupleVisitor(self))
                }
            }
        )+
    }
}

tuple_impls! {
    1  => (0 T0)
    2  => (0 T0 1 T1)
    3  => (0 T0 1 T1 2 T2)
    4  => (0 T0 1 T1 2 T2 3 T3)
    5  => (0 T0 1 T1 2 T2 3 T3 4 T4)
    6  => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5)
    7  => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6)
    8  => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7)
    9  => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8)
    10 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9)
    11 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10)
    12 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11)
    13 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12)
    14 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13)
    15 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14)
    16 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15)
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(any(feature = "std", feature = "alloc"))]
macro_rules! map_impl {
    (
        $ty:ident < K $(: $kbound1:ident $(+ $kbound2:ident)*)*, V $(, $typaram:ident : $bound1:ident $(+ $bound2:ident)*)* >,
        $access:ident,
        $ctor:expr,
        $with_capacity:expr
    ) => {
        impl<'de, K, V $(, $typaram)*> Deserialize<'de> for $ty<K, V $(, $typaram)*>
        where
            K: Deserialize<'de> $(+ $kbound1 $(+ $kbound2)*)*,
            V: Deserialize<'de>,
            $($typaram: $bound1 $(+ $bound2)*),*
        {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct MapVisitor<K, V $(, $typaram)*> {
                    marker: PhantomData<$ty<K, V $(, $typaram)*>>,
                }

                impl<'de, K, V $(, $typaram)*> Visitor<'de> for MapVisitor<K, V $(, $typaram)*>
                where
                    K: Deserialize<'de> $(+ $kbound1 $(+ $kbound2)*)*,
                    V: Deserialize<'de>,
                    $($typaram: $bound1 $(+ $bound2)*),*
                {
                    type Value = $ty<K, V $(, $typaram)*>;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("a map")
                    }

                    #[inline]
                    fn visit_map<A>(self, mut $access: A) -> Result<Self::Value, A::Error>
                    where
                        A: MapAccess<'de>,
                    {
                        let mut values = $with_capacity;

                        while let Some((key, value)) = try!($access.next_entry()) {
                            values.insert(key, value);
                        }

                        Ok(values)
                    }
                }

                let visitor = MapVisitor { marker: PhantomData };
                deserializer.deserialize_map(visitor)
            }
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
map_impl!(
    BTreeMap<K: Ord, V>,
    map,
    BTreeMap::new(),
    BTreeMap::new());

#[cfg(feature = "std")]
map_impl!(
    HashMap<K: Eq + Hash, V, S: BuildHasher + Default>,
    map,
    HashMap::with_hasher(S::default()),
    HashMap::with_capacity_and_hasher(size_hint::cautious(map.size_hint()), S::default()));

////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "std")]
macro_rules! parse_ip_impl {
    ($ty:ty; $size: expr) => {
        impl<'de> Deserialize<'de> for $ty {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                if deserializer.is_human_readable() {
                    let s = try!(String::deserialize(deserializer));
                    s.parse().map_err(Error::custom)
                } else {
                    <[u8; $size]>::deserialize(deserializer).map(<$ty>::from)
                }
            }
        }
    }
}

#[cfg(feature = "std")]
macro_rules! variant_identifier {
    (
        $name_kind: ident ( $($variant: ident; $bytes: expr; $index: expr),* )
        $expecting_message: expr,
        $variants_name: ident
    ) => {
        enum $name_kind {
            $( $variant ),*
        }

        static $variants_name: &'static [&'static str] = &[ $( stringify!($variant) ),*];

        impl<'de> Deserialize<'de> for $name_kind {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct KindVisitor;

                impl<'de> Visitor<'de> for KindVisitor {
                    type Value = $name_kind;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str($expecting_message)
                    }

                    fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
                    where
                        E: Error,
                    {
                        match value {
                            $(
                                $index => Ok($name_kind :: $variant),
                            )*
                            _ => Err(Error::invalid_value(Unexpected::Unsigned(value as u64), &self),),
                        }
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: Error,
                    {
                        match value {
                            $(
                                stringify!($variant) => Ok($name_kind :: $variant),
                            )*
                            _ => Err(Error::unknown_variant(value, $variants_name)),
                        }
                    }

                    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
                    where
                        E: Error,
                    {
                        match value {
                            $(
                                $bytes => Ok($name_kind :: $variant),
                            )*
                            _ => {
                                match str::from_utf8(value) {
                                    Ok(value) => Err(Error::unknown_variant(value, $variants_name)),
                                    Err(_) => Err(Error::invalid_value(Unexpected::Bytes(value), &self)),
                                }
                            }
                        }
                    }
                }

                deserializer.deserialize_identifier(KindVisitor)
            }
        }
    }
}

#[cfg(feature = "std")]
macro_rules! deserialize_enum {
    (
        $name: ident $name_kind: ident ( $($variant: ident; $bytes: expr; $index: expr),* )
        $expecting_message: expr,
        $deserializer: expr
    ) => {
        variant_identifier!{
            $name_kind ( $($variant; $bytes; $index),* )
            $expecting_message,
            VARIANTS
        }

        struct EnumVisitor;
        impl<'de> Visitor<'de> for EnumVisitor {
            type Value = $name;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(concat!("a ", stringify!($name)))
            }


            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                match try!(data.variant()) {
                    $(
                        ($name_kind :: $variant, v) => v.newtype_variant().map($name :: $variant),
                    )*
                }
            }
        }
        $deserializer.deserialize_enum(stringify!($name), VARIANTS, EnumVisitor)
    }
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for net::IpAddr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = try!(String::deserialize(deserializer));
            s.parse().map_err(Error::custom)
        } else {
            use lib::net::IpAddr;
            deserialize_enum!{
                IpAddr IpAddrKind (V4; b"V4"; 0, V6; b"V6"; 1)
                "`V4` or `V6`",
                deserializer
            }
        }
    }
}

#[cfg(feature = "std")]
parse_ip_impl!(net::Ipv4Addr; 4);

#[cfg(feature = "std")]
parse_ip_impl!(net::Ipv6Addr; 16);

#[cfg(feature = "std")]
macro_rules! parse_socket_impl {
    ($ty:ty, $new: expr) => {
        impl<'de> Deserialize<'de> for $ty {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                if deserializer.is_human_readable() {
                    let s = try!(String::deserialize(deserializer));
                    s.parse().map_err(Error::custom)
                } else {
                    <(_, u16)>::deserialize(deserializer).map(|(ip, port)| $new(ip, port))
                }
            }
        }
    }
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for net::SocketAddr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = try!(String::deserialize(deserializer));
            s.parse().map_err(Error::custom)
        } else {
            use lib::net::SocketAddr;
            deserialize_enum!{
                SocketAddr SocketAddrKind (V4; b"V4"; 0, V6; b"V6"; 1)
                "`V4` or `V6`",
                deserializer
            }
        }
    }
}

#[cfg(feature = "std")]
parse_socket_impl!(net::SocketAddrV4, net::SocketAddrV4::new);

#[cfg(feature = "std")]
parse_socket_impl!(net::SocketAddrV6, |ip, port| net::SocketAddrV6::new(ip, port, 0, 0));

////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "std")]
struct PathVisitor;

#[cfg(feature = "std")]
impl<'a> Visitor<'a> for PathVisitor {
    type Value = &'a Path;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a borrowed path")
    }

    fn visit_borrowed_str<E>(self, v: &'a str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(v.as_ref())
    }

    fn visit_borrowed_bytes<E>(self, v: &'a [u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        str::from_utf8(v)
            .map(AsRef::as_ref)
            .map_err(|_| Error::invalid_value(Unexpected::Bytes(v), &self))
    }
}

#[cfg(feature = "std")]
impl<'de: 'a, 'a> Deserialize<'de> for &'a Path {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(PathVisitor)
    }
}

#[cfg(feature = "std")]
struct PathBufVisitor;

#[cfg(feature = "std")]
impl<'de> Visitor<'de> for PathBufVisitor {
    type Value = PathBuf;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("path string")
    }

    fn visit_str<E>(self, v: &str) -> Result<PathBuf, E>
    where
        E: Error,
    {
        Ok(From::from(v))
    }

    fn visit_string<E>(self, v: String) -> Result<PathBuf, E>
    where
        E: Error,
    {
        Ok(From::from(v))
    }
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for PathBuf {
    fn deserialize<D>(deserializer: D) -> Result<PathBuf, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(PathBufVisitor)
    }
}

////////////////////////////////////////////////////////////////////////////////

// If this were outside of the serde crate, it would just use:
//
//    #[derive(Deserialize)]
//    #[serde(variant_identifier)]
#[cfg(all(feature = "std", any(unix, windows)))]
variant_identifier!{
    OsStringKind (Unix; b"Unix"; 0, Windows; b"Windows"; 1)
    "`Unix` or `Windows`",
    OSSTR_VARIANTS
}

#[cfg(all(feature = "std", any(unix, windows)))]
struct OsStringVisitor;

#[cfg(all(feature = "std", any(unix, windows)))]
impl<'de> Visitor<'de> for OsStringVisitor {
    type Value = OsString;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("os string")
    }

    #[cfg(unix)]
    fn visit_enum<A>(self, data: A) -> Result<OsString, A::Error>
    where
        A: EnumAccess<'de>,
    {
        use std::os::unix::ffi::OsStringExt;

        match try!(data.variant()) {
            (OsStringKind::Unix, v) => v.newtype_variant().map(OsString::from_vec),
            (OsStringKind::Windows, _) => Err(Error::custom("cannot deserialize Windows OS string on Unix",),),
        }
    }

    #[cfg(windows)]
    fn visit_enum<A>(self, data: A) -> Result<OsString, A::Error>
    where
        A: EnumAccess<'de>,
    {
        use std::os::windows::ffi::OsStringExt;

        match try!(data.variant()) {
            (OsStringKind::Windows, v) => {
                v.newtype_variant::<Vec<u16>>()
                    .map(|vec| OsString::from_wide(&vec))
            }
            (OsStringKind::Unix, _) => Err(Error::custom("cannot deserialize Unix OS string on Windows",),),
        }
    }
}

#[cfg(all(feature = "std", any(unix, windows)))]
impl<'de> Deserialize<'de> for OsString {
    fn deserialize<D>(deserializer: D) -> Result<OsString, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_enum("OsString", OSSTR_VARIANTS, OsStringVisitor)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(any(feature = "std", feature = "alloc"))]
forwarded_impl!((T), Box<T>, Box::new);

#[cfg(any(feature = "std", feature = "alloc"))]
forwarded_impl!((T), Box<[T]>, Vec::into_boxed_slice);

#[cfg(any(feature = "std", feature = "alloc"))]
forwarded_impl!((), Box<str>, String::into_boxed_str);

#[cfg(all(not(feature = "unstable"), feature = "rc", any(feature = "std", feature = "alloc")))]
forwarded_impl!((T), Arc<T>, Arc::new);

#[cfg(all(not(feature = "unstable"), feature = "rc", any(feature = "std", feature = "alloc")))]
forwarded_impl!((T), Rc<T>, Rc::new);

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'de, 'a, T: ?Sized> Deserialize<'de> for Cow<'a, T>
where
    T: ToOwned,
    T::Owned: Deserialize<'de>,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Cow<'a, T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::Owned::deserialize(deserializer).map(Cow::Owned)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(all(feature = "unstable", feature = "rc", any(feature = "std", feature = "alloc")))]
macro_rules! box_forwarded_impl {
    ($t:ident) => {
        impl<'de, T: ?Sized> Deserialize<'de> for $t<T>
        where
            Box<T>: Deserialize<'de>,
        {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                Box::deserialize(deserializer).map(Into::into)
            }
        }
    }
}

#[cfg(all(feature = "unstable", feature = "rc", any(feature = "std", feature = "alloc")))]
box_forwarded_impl!(Rc);

#[cfg(all(feature = "unstable", feature = "rc", any(feature = "std", feature = "alloc")))]
box_forwarded_impl!(Arc);

////////////////////////////////////////////////////////////////////////////////

impl<'de, T> Deserialize<'de> for Cell<T>
where
    T: Deserialize<'de> + Copy,
{
    fn deserialize<D>(deserializer: D) -> Result<Cell<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(deserializer).map(Cell::new)
    }
}

forwarded_impl!((T), RefCell<T>, RefCell::new);

#[cfg(feature = "std")]
forwarded_impl!((T), Mutex<T>, Mutex::new);

#[cfg(feature = "std")]
forwarded_impl!((T), RwLock<T>, RwLock::new);

////////////////////////////////////////////////////////////////////////////////

// This is a cleaned-up version of the impl generated by:
//
//     #[derive(Deserialize)]
//     #[serde(deny_unknown_fields)]
//     struct Duration {
//         secs: u64,
//         nanos: u32,
//     }
#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // If this were outside of the serde crate, it would just use:
        //
        //    #[derive(Deserialize)]
        //    #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Secs,
            Nanos,
        };

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`secs` or `nanos`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: Error,
                    {
                        match value {
                            "secs" => Ok(Field::Secs),
                            "nanos" => Ok(Field::Nanos),
                            _ => Err(Error::unknown_field(value, FIELDS)),
                        }
                    }

                    fn visit_bytes<E>(self, value: &[u8]) -> Result<Field, E>
                    where
                        E: Error,
                    {
                        match value {
                            b"secs" => Ok(Field::Secs),
                            b"nanos" => Ok(Field::Nanos),
                            _ => {
                                let value = String::from_utf8_lossy(value);
                                Err(Error::unknown_field(&value, FIELDS))
                            }
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct DurationVisitor;

        impl<'de> Visitor<'de> for DurationVisitor {
            type Value = Duration;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Duration")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Duration, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let secs: u64 = match try!(seq.next_element()) {
                    Some(value) => value,
                    None => {
                        return Err(Error::invalid_length(0, &self));
                    }
                };
                let nanos: u32 = match try!(seq.next_element()) {
                    Some(value) => value,
                    None => {
                        return Err(Error::invalid_length(1, &self));
                    }
                };
                Ok(Duration::new(secs, nanos))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Duration, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut secs: Option<u64> = None;
                let mut nanos: Option<u32> = None;
                while let Some(key) = try!(map.next_key()) {
                    match key {
                        Field::Secs => {
                            if secs.is_some() {
                                return Err(<A::Error as Error>::duplicate_field("secs"));
                            }
                            secs = Some(try!(map.next_value()));
                        }
                        Field::Nanos => {
                            if nanos.is_some() {
                                return Err(<A::Error as Error>::duplicate_field("nanos"));
                            }
                            nanos = Some(try!(map.next_value()));
                        }
                    }
                }
                let secs = match secs {
                    Some(secs) => secs,
                    None => return Err(<A::Error as Error>::missing_field("secs")),
                };
                let nanos = match nanos {
                    Some(nanos) => nanos,
                    None => return Err(<A::Error as Error>::missing_field("nanos")),
                };
                Ok(Duration::new(secs, nanos))
            }
        }

        const FIELDS: &'static [&'static str] = &["secs", "nanos"];
        deserializer.deserialize_struct("Duration", FIELDS, DurationVisitor)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for SystemTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Reuse duration
        enum Field {
            Secs,
            Nanos,
        };

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`secs_since_epoch` or `nanos_since_epoch`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: Error,
                    {
                        match value {
                            "secs_since_epoch" => Ok(Field::Secs),
                            "nanos_since_epoch" => Ok(Field::Nanos),
                            _ => Err(Error::unknown_field(value, FIELDS)),
                        }
                    }

                    fn visit_bytes<E>(self, value: &[u8]) -> Result<Field, E>
                    where
                        E: Error,
                    {
                        match value {
                            b"secs_since_epoch" => Ok(Field::Secs),
                            b"nanos_since_epoch" => Ok(Field::Nanos),
                            _ => {
                                let value = String::from_utf8_lossy(value);
                                Err(Error::unknown_field(&value, FIELDS))
                            }
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct DurationVisitor;

        impl<'de> Visitor<'de> for DurationVisitor {
            type Value = Duration;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct SystemTime")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Duration, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let secs: u64 = match try!(seq.next_element()) {
                    Some(value) => value,
                    None => {
                        return Err(Error::invalid_length(0, &self));
                    }
                };
                let nanos: u32 = match try!(seq.next_element()) {
                    Some(value) => value,
                    None => {
                        return Err(Error::invalid_length(1, &self));
                    }
                };
                Ok(Duration::new(secs, nanos))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Duration, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut secs: Option<u64> = None;
                let mut nanos: Option<u32> = None;
                while let Some(key) = try!(map.next_key()) {
                    match key {
                        Field::Secs => {
                            if secs.is_some() {
                                return Err(<A::Error as Error>::duplicate_field("secs_since_epoch"));
                            }
                            secs = Some(try!(map.next_value()));
                        }
                        Field::Nanos => {
                            if nanos.is_some() {
                                return Err(<A::Error as Error>::duplicate_field("nanos_since_epoch"));
                            }
                            nanos = Some(try!(map.next_value()));
                        }
                    }
                }
                let secs = match secs {
                    Some(secs) => secs,
                    None => return Err(<A::Error as Error>::missing_field("secs_since_epoch")),
                };
                let nanos = match nanos {
                    Some(nanos) => nanos,
                    None => return Err(<A::Error as Error>::missing_field("nanos_since_epoch")),
                };
                Ok(Duration::new(secs, nanos))
            }
        }

        const FIELDS: &'static [&'static str] = &["secs_since_epoch", "nanos_since_epoch"];
        let duration = try!(deserializer.deserialize_struct("SystemTime", FIELDS, DurationVisitor));
        Ok(UNIX_EPOCH + duration)
    }
}

////////////////////////////////////////////////////////////////////////////////

// Similar to:
//
//     #[derive(Deserialize)]
//     #[serde(deny_unknown_fields)]
//     struct Range {
//         start: u64,
//         end: u32,
//     }
#[cfg(feature = "std")]
impl<'de, Idx> Deserialize<'de> for ops::Range<Idx>
where
    Idx: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // If this were outside of the serde crate, it would just use:
        //
        //    #[derive(Deserialize)]
        //    #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Start,
            End,
        };

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`start` or `end`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: Error,
                    {
                        match value {
                            "start" => Ok(Field::Start),
                            "end" => Ok(Field::End),
                            _ => Err(Error::unknown_field(value, FIELDS)),
                        }
                    }

                    fn visit_bytes<E>(self, value: &[u8]) -> Result<Field, E>
                    where
                        E: Error,
                    {
                        match value {
                            b"start" => Ok(Field::Start),
                            b"end" => Ok(Field::End),
                            _ => {
                                let value = String::from_utf8_lossy(value);
                                Err(Error::unknown_field(&value, FIELDS))
                            }
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct RangeVisitor<Idx> {
            phantom: PhantomData<Idx>,
        }

        impl<'de, Idx> Visitor<'de> for RangeVisitor<Idx>
        where
            Idx: Deserialize<'de>,
        {
            type Value = ops::Range<Idx>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Range")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<ops::Range<Idx>, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let start: Idx = match try!(seq.next_element()) {
                    Some(value) => value,
                    None => {
                        return Err(Error::invalid_length(0, &self));
                    }
                };
                let end: Idx = match try!(seq.next_element()) {
                    Some(value) => value,
                    None => {
                        return Err(Error::invalid_length(1, &self));
                    }
                };
                Ok(start..end)
            }

            fn visit_map<A>(self, mut map: A) -> Result<ops::Range<Idx>, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut start: Option<Idx> = None;
                let mut end: Option<Idx> = None;
                while let Some(key) = try!(map.next_key()) {
                    match key {
                        Field::Start => {
                            if start.is_some() {
                                return Err(<A::Error as Error>::duplicate_field("start"));
                            }
                            start = Some(try!(map.next_value()));
                        }
                        Field::End => {
                            if end.is_some() {
                                return Err(<A::Error as Error>::duplicate_field("end"));
                            }
                            end = Some(try!(map.next_value()));
                        }
                    }
                }
                let start = match start {
                    Some(start) => start,
                    None => return Err(<A::Error as Error>::missing_field("start")),
                };
                let end = match end {
                    Some(end) => end,
                    None => return Err(<A::Error as Error>::missing_field("end")),
                };
                Ok(start..end)
            }
        }

        const FIELDS: &'static [&'static str] = &["start", "end"];
        deserializer.deserialize_struct("Range", FIELDS, RangeVisitor { phantom: PhantomData })
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "unstable")]
impl<'de, T> Deserialize<'de> for NonZero<T>
where
    T: Deserialize<'de> + Zeroable,
{
    fn deserialize<D>(deserializer: D) -> Result<NonZero<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = try!(Deserialize::deserialize(deserializer));
        match NonZero::new(value) {
            Some(nonzero) => Ok(nonzero),
            None => Err(Error::custom("expected a non-zero value")),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<'de, T, E> Deserialize<'de> for Result<T, E>
where
    T: Deserialize<'de>,
    E: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Result<T, E>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // If this were outside of the serde crate, it would just use:
        //
        //    #[derive(Deserialize)]
        //    #[serde(variant_identifier)]
        enum Field {
            Ok,
            Err,
        }

        impl<'de> Deserialize<'de> for Field {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`Ok` or `Err`")
                    }

                    fn visit_u32<E>(self, value: u32) -> Result<Field, E>
                    where
                        E: Error,
                    {
                        match value {
                            0 => Ok(Field::Ok),
                            1 => Ok(Field::Err),
                            _ => {
                                Err(Error::invalid_value(Unexpected::Unsigned(value as u64), &self),)
                            }
                        }
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: Error,
                    {
                        match value {
                            "Ok" => Ok(Field::Ok),
                            "Err" => Ok(Field::Err),
                            _ => Err(Error::unknown_variant(value, VARIANTS)),
                        }
                    }

                    fn visit_bytes<E>(self, value: &[u8]) -> Result<Field, E>
                    where
                        E: Error,
                    {
                        match value {
                            b"Ok" => Ok(Field::Ok),
                            b"Err" => Ok(Field::Err),
                            _ => {
                                match str::from_utf8(value) {
                                    Ok(value) => Err(Error::unknown_variant(value, VARIANTS)),
                                    Err(_) => {
                                        Err(Error::invalid_value(Unexpected::Bytes(value), &self))
                                    }
                                }
                            }
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct ResultVisitor<T, E>(PhantomData<Result<T, E>>);

        impl<'de, T, E> Visitor<'de> for ResultVisitor<T, E>
        where
            T: Deserialize<'de>,
            E: Deserialize<'de>,
        {
            type Value = Result<T, E>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("enum Result")
            }

            fn visit_enum<A>(self, data: A) -> Result<Result<T, E>, A::Error>
            where
                A: EnumAccess<'de>,
            {
                match try!(data.variant()) {
                    (Field::Ok, v) => v.newtype_variant().map(Ok),
                    (Field::Err, v) => v.newtype_variant().map(Err),
                }
            }
        }

        const VARIANTS: &'static [&'static str] = &["Ok", "Err"];

        deserializer.deserialize_enum("Result", VARIANTS, ResultVisitor(PhantomData))
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "std")]
impl<'de, T> Deserialize<'de> for Wrapping<T>
where
    T: Deserialize<'de>
{
    fn deserialize<D>(deserializer: D) -> Result<Wrapping<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer).map(Wrapping)
    }
}
