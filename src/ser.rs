use std::collections::hash_state::HashState;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;
use std::path;
use std::rc::Rc;
use std::str;
use std::sync::Arc;

///////////////////////////////////////////////////////////////////////////////

pub trait Serialize {
    fn visit<V>(&self, visitor: &mut V) -> Result<(), V::Error>
        where V: Visitor;
}

///////////////////////////////////////////////////////////////////////////////

pub trait Serializer {
    type Error;

    fn visit<T>(&mut self, value: &T) -> Result<(), Self::Error>
        where T: Serialize;
}

///////////////////////////////////////////////////////////////////////////////

pub trait Visitor {
    type Error;

    fn visit_bool(&mut self, v: bool) -> Result<(), Self::Error>;

    #[inline]
    fn visit_isize(&mut self, v: isize) -> Result<(), Self::Error> {
        self.visit_i64(v as i64)
    }

    #[inline]
    fn visit_i8(&mut self, v: i8) -> Result<(), Self::Error> {
        self.visit_i64(v as i64)
    }

    #[inline]
    fn visit_i16(&mut self, v: i16) -> Result<(), Self::Error> {
        self.visit_i64(v as i64)
    }

    #[inline]
    fn visit_i32(&mut self, v: i32) -> Result<(), Self::Error> {
        self.visit_i64(v as i64)
    }

    #[inline]
    fn visit_i64(&mut self, v: i64) -> Result<(), Self::Error>;

    #[inline]
    fn visit_usize(&mut self, v: usize) -> Result<(), Self::Error> {
        self.visit_u64(v as u64)
    }

    #[inline]
    fn visit_u8(&mut self, v: u8) -> Result<(), Self::Error> {
        self.visit_u64(v as u64)
    }

    #[inline]
    fn visit_u16(&mut self, v: u16) -> Result<(), Self::Error> {
        self.visit_u64(v as u64)
    }

    #[inline]
    fn visit_u32(&mut self, v: u32) -> Result<(), Self::Error> {
        self.visit_u64(v as u64)
    }

    #[inline]
    fn visit_u64(&mut self, v: u64) -> Result<(), Self::Error>;

    #[inline]
    fn visit_f32(&mut self, v: f32) -> Result<(), Self::Error> {
        self.visit_f64(v as f64)
    }

    fn visit_f64(&mut self, v: f64) -> Result<(), Self::Error>;

    #[inline]
    fn visit_char(&mut self, v: char) -> Result<(), Self::Error> {
        // The unwraps in here should be safe.
        let mut s = &mut [0; 4];
        let len = v.encode_utf8(s).unwrap();
        self.visit_str(str::from_utf8(&s[..len]).unwrap())
    }

    fn visit_str(&mut self, value: &str) -> Result<(), Self::Error>;

    fn visit_unit(&mut self) -> Result<(), Self::Error>;

    #[inline]
    fn visit_named_unit(&mut self, _name: &str) -> Result<(), Self::Error> {
        self.visit_unit()
    }

    #[inline]
    fn visit_enum_unit(&mut self,
                       _name: &str,
                       _variant: &str) -> Result<(), Self::Error> {
        self.visit_unit()
    }

    fn visit_none(&mut self) -> Result<(), Self::Error>;

    fn visit_some<V>(&mut self, value: V) -> Result<(), Self::Error>
        where V: Serialize;

    fn visit_seq<V>(&mut self, visitor: V) -> Result<(), Self::Error>
        where V: SeqVisitor;

    #[inline]
    fn visit_named_seq<V>(&mut self,
                          _name: &'static str,
                          visitor: V) -> Result<(), Self::Error>
        where V: SeqVisitor,
    {
        self.visit_seq(visitor)
    }

    #[inline]
    fn visit_enum_seq<V>(&mut self,
                         _name: &'static str,
                         _variant: &'static str,
                         visitor: V) -> Result<(), Self::Error>
        where V: SeqVisitor,
    {
        self.visit_seq(visitor)
    }

    fn visit_seq_elt<T>(&mut self,
                        first: bool,
                        value: T) -> Result<(), Self::Error>
        where T: Serialize;

    fn visit_map<V>(&mut self, visitor: V) -> Result<(), Self::Error>
        where V: MapVisitor;

    #[inline]
    fn visit_named_map<V>(&mut self,
                          _name: &'static str,
                          visitor: V) -> Result<(), Self::Error>
        where V: MapVisitor,
    {
        self.visit_map(visitor)
    }

    #[inline]
    fn visit_enum_map<V>(&mut self,
                          _name: &'static str,
                          _variant: &'static str,
                          visitor: V) -> Result<(), Self::Error>
        where V: MapVisitor,
    {
        self.visit_map(visitor)
    }

    fn visit_map_elt<K, V>(&mut self,
                           first: bool,
                           key: K,
                           value: V) -> Result<(), Self::Error>
        where K: Serialize,
              V: Serialize;
}

pub trait SeqVisitor {
    fn visit<V>(&mut self, visitor: &mut V) -> Result<Option<()>, V::Error>
        where V: Visitor;

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

pub trait MapVisitor {
    fn visit<V>(&mut self, visitor: &mut V) -> Result<Option<()>, V::Error>
        where V: Visitor;

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

///////////////////////////////////////////////////////////////////////////////

macro_rules! impl_visit {
    ($ty:ty, $method:ident) => {
        impl Serialize for $ty {
            #[inline]
            fn visit<V>(&self, visitor: &mut V) -> Result<(), V::Error>
                where V: Visitor,
            {
                visitor.$method(*self)
            }
        }
    }
}

impl_visit!(bool, visit_bool);
impl_visit!(isize, visit_isize);
impl_visit!(i8, visit_i8);
impl_visit!(i16, visit_i16);
impl_visit!(i32, visit_i32);
impl_visit!(i64, visit_i64);
impl_visit!(usize, visit_usize);
impl_visit!(u8, visit_u8);
impl_visit!(u16, visit_u16);
impl_visit!(u32, visit_u32);
impl_visit!(u64, visit_u64);
impl_visit!(f32, visit_f32);
impl_visit!(f64, visit_f64);
impl_visit!(char, visit_char);

///////////////////////////////////////////////////////////////////////////////

impl<'a> Serialize for &'a str {
    #[inline]
    fn visit<V>(&self, visitor: &mut V) -> Result<(), V::Error>
        where V: Visitor,
    {
        visitor.visit_str(*self)
    }
}

impl Serialize for String {
    #[inline]
    fn visit<V>(&self, visitor: &mut V) -> Result<(), V::Error>
        where V: Visitor,
    {
        (&self[..]).visit(visitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<T> Serialize for Option<T> where T: Serialize {
    #[inline]
    fn visit<V>(&self, visitor: &mut V) -> Result<(), V::Error>
        where V: Visitor,
    {
        match *self {
            Some(ref value) => visitor.visit_some(value),
            None => visitor.visit_none(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

pub struct SeqIteratorVisitor<Iter> {
    iter: Iter,
    first: bool,
}

impl<T, Iter> SeqIteratorVisitor<Iter>
    where Iter: Iterator<Item=T>
{
    #[inline]
    pub fn new(iter: Iter) -> SeqIteratorVisitor<Iter> {
        SeqIteratorVisitor {
            iter: iter,
            first: true,
        }
    }
}

impl<T, Iter> SeqVisitor for SeqIteratorVisitor<Iter>
    where T: Serialize,
          Iter: Iterator<Item=T>,
{
    #[inline]
    fn visit<V>(&mut self, visitor: &mut V) -> Result<Option<()>, V::Error>
        where V: Visitor,
    {
        let first = self.first;
        self.first = false;

        match self.iter.next() {
            Some(value) => {
                let value = try!(visitor.visit_seq_elt(first, value));
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<'a, T> Serialize for &'a [T]
    where T: Serialize,
{
    #[inline]
    fn visit<V>(&self, visitor: &mut V) -> Result<(), V::Error>
        where V: Visitor,
    {
        visitor.visit_seq(SeqIteratorVisitor::new(self.iter()))
    }
}

impl<T> Serialize for Vec<T> where T: Serialize {
    #[inline]
    fn visit<V>(&self, visitor: &mut V) -> Result<(), V::Error>
        where V: Visitor,
    {
        (&self[..]).visit(visitor)
    }
}

impl<T> Serialize for BTreeSet<T> where T: Serialize {
    #[inline]
    fn visit<V>(&self, visitor: &mut V) -> Result<(), V::Error>
        where V: Visitor,
    {
        visitor.visit_seq(SeqIteratorVisitor::new(self.iter()))
    }
}

impl<T, S> Serialize for HashSet<T, S>
    where T: Serialize + Eq + Hash,
          S: HashState,
{
    #[inline]
    fn visit<V: Visitor>(&self, visitor: &mut V) -> Result<(), V::Error> {
        visitor.visit_seq(SeqIteratorVisitor::new(self.iter()))
    }
}

///////////////////////////////////////////////////////////////////////////////

impl Serialize for () {
    #[inline]
    fn visit<V>(&self, visitor: &mut V) -> Result<(), V::Error>
        where V: Visitor,
    {
        visitor.visit_unit()
    }
}

///////////////////////////////////////////////////////////////////////////////

// FIXME(rust #19630) Remove this work-around
macro_rules! e {
    ($e:expr) => { $e }
}

macro_rules! tuple_impls {
    ($(
        $TupleVisitor:ident ($len:expr, $($T:ident),+) {
            $($state:pat => $idx:tt,)+
        }
    )+) => {
        $(
            pub struct $TupleVisitor<'a, $($T: 'a),+> {
                tuple: &'a ($($T,)+),
                state: u8,
                first: bool,
            }

            impl<'a, $($T: 'a),+> $TupleVisitor<'a, $($T),+> {
                pub fn new(tuple: &'a ($($T,)+)) -> $TupleVisitor<'a, $($T),+> {
                    $TupleVisitor {
                        tuple: tuple,
                        state: 0,
                        first: true,
                    }
                }
            }

            impl<'a, $($T),+> SeqVisitor for $TupleVisitor<'a, $($T),+>
                where $($T: Serialize),+
            {
                fn visit<V>(&mut self, visitor: &mut V) -> Result<Option<()>, V::Error>
                    where V: Visitor,
                {
                    let first = self.first;
                    self.first = false;

                    match self.state {
                        $(
                            $state => {
                                self.state += 1;
                                Ok(Some(try!(visitor.visit_seq_elt(first, &e!(self.tuple.$idx)))))
                            }
                        )+
                        _ => {
                            Ok(None)
                        }
                    }
                }

                fn size_hint(&self) -> (usize, Option<usize>) {
                    ($len, Some($len))
                }
            }

            impl<$($T),+> Serialize for ($($T,)+)
                where $($T: Serialize),+
            {
                #[inline]
                fn visit<V: Visitor>(&self, visitor: &mut V) -> Result<(), V::Error> {
                    visitor.visit_seq($TupleVisitor::new(self))
                }
            }
        )+
    }
}

tuple_impls! {
    TupleVisitor1 (1, T0) {
        0 => 0,
    }
    TupleVisitor2 (2, T0, T1) {
        0 => 0,
        1 => 1,
    }
    TupleVisitor3 (3, T0, T1, T2) {
        0 => 0,
        1 => 1,
        2 => 2,
    }
    TupleVisitor4 (4, T0, T1, T2, T3) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
    }
    TupleVisitor5 (5, T0, T1, T2, T3, T4) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
    }
    TupleVisitor6 (6, T0, T1, T2, T3, T4, T5) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
    }
    TupleVisitor7 (7, T0, T1, T2, T3, T4, T5, T6) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
        6 => 6,
    }
    TupleVisitor8 (8, T0, T1, T2, T3, T4, T5, T6, T7) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
        6 => 6,
        7 => 7,
    }
    TupleVisitor9 (9, T0, T1, T2, T3, T4, T5, T6, T7, T8) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
        6 => 6,
        7 => 7,
        8 => 8,
    }
    TupleVisitor10 (10, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
        6 => 6,
        7 => 7,
        8 => 8,
        9 => 9,
    }
    TupleVisitor11 (11, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
        6 => 6,
        7 => 7,
        8 => 8,
        9 => 9,
        10 => 10,
    }
    TupleVisitor12 (12, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
        6 => 6,
        7 => 7,
        8 => 8,
        9 => 9,
        10 => 10,
        11 => 11,
    }
}

///////////////////////////////////////////////////////////////////////////////

pub struct MapIteratorVisitor<Iter> {
    iter: Iter,
    first: bool,
}

impl<K, V, Iter> MapIteratorVisitor<Iter>
    where Iter: Iterator<Item=(K, V)>
{
    #[inline]
    pub fn new(iter: Iter) -> MapIteratorVisitor<Iter> {
        MapIteratorVisitor {
            iter: iter,
            first: true,
        }
    }
}

impl<K, V, I> MapVisitor for MapIteratorVisitor<I>
    where K: Serialize,
          V: Serialize,
          I: Iterator<Item=(K, V)>,
{
    #[inline]
    fn visit<V_>(&mut self, visitor: &mut V_) -> Result<Option<()>, V_::Error>
        where V_: Visitor,
    {
        let first = self.first;
        self.first = false;

        match self.iter.next() {
            Some((key, value)) => {
                let value = try!(visitor.visit_map_elt(first, key, value));
                Ok(Some(value))
            }
            None => Ok(None)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<K, V> Serialize for BTreeMap<K, V>
    where K: Serialize + Ord,
          V: Serialize,
{
    #[inline]
    fn visit<V_: Visitor>(&self, visitor: &mut V_) -> Result<(), V_::Error> {
        visitor.visit_map(MapIteratorVisitor::new(self.iter()))
    }
}

impl<K, V, S> Serialize for HashMap<K, V, S>
    where K: Serialize + Eq + Hash,
          V: Serialize,
          S: HashState,
{
    #[inline]
    fn visit<V_: Visitor>(&self, visitor: &mut V_) -> Result<(), V_::Error> {
        visitor.visit_map(MapIteratorVisitor::new(self.iter()))
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<'a, T> Serialize for &'a T where T: Serialize {
    #[inline]
    fn visit<V>(&self, visitor: &mut V) -> Result<(), V::Error>
        where V: Visitor,
    {
        (**self).visit(visitor)
    }
}

impl<'a, T> Serialize for &'a mut T where T: Serialize {
    #[inline]
    fn visit<V>(&self, visitor: &mut V) -> Result<(), V::Error>
        where V: Visitor,
    {
        (**self).visit(visitor)
    }
}

impl<T> Serialize for Box<T> where T: Serialize {
    #[inline]
    fn visit<V>(&self, visitor: &mut V) -> Result<(), V::Error>
        where V: Visitor,
    {
        (**self).visit(visitor)
    }
}

impl<T> Serialize for Rc<T> where T: Serialize, {
    #[inline]
    fn visit<V>(&self, visitor: &mut V) -> Result<(), V::Error>
        where V: Visitor,
    {
        (**self).visit(visitor)
    }
}

impl<T> Serialize for Arc<T> where T: Serialize, {
    #[inline]
    fn visit<V>(&self, visitor: &mut V) -> Result<(), V::Error>
        where V: Visitor,
    {
        (**self).visit(visitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl Serialize for path::Path {
    fn visit<V>(&self, visitor: &mut V) -> Result<(), V::Error>
        where V: Visitor,
    {
        self.to_str().unwrap().visit(visitor)
    }
}

impl Serialize for path::PathBuf {
    fn visit<V>(&self, visitor: &mut V) -> Result<(), V::Error>
        where V: Visitor,
    {
        self.to_str().unwrap().visit(visitor)
    }
}
