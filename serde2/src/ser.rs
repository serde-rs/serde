use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::collections::hash_state::HashState;
use std::hash::Hash;
use std::rc::Rc;
use std::str;
use std::sync::Arc;

///////////////////////////////////////////////////////////////////////////////

pub trait Serialize {
    fn visit<
        V: Visitor,
    >(&self, visitor: &mut V) -> Result<V::Value, V::Error>;
}

///////////////////////////////////////////////////////////////////////////////

pub trait Serializer {
    type Value;
    type Error;

    fn visit<T>(&mut self, value: &T) -> Result<Self::Value, Self::Error>
        where T: Serialize;
}

///////////////////////////////////////////////////////////////////////////////

pub trait Visitor {
    type Value;
    type Error;

    fn visit_bool(&mut self, v: bool) -> Result<Self::Value, Self::Error>;

    #[inline]
    fn visit_isize(&mut self, v: isize) -> Result<Self::Value, Self::Error> {
        self.visit_i64(v as i64)
    }

    #[inline]
    fn visit_i8(&mut self, v: i8) -> Result<Self::Value, Self::Error> {
        self.visit_i64(v as i64)
    }

    #[inline]
    fn visit_i16(&mut self, v: i16) -> Result<Self::Value, Self::Error> {
        self.visit_i64(v as i64)
    }

    #[inline]
    fn visit_i32(&mut self, v: i32) -> Result<Self::Value, Self::Error> {
        self.visit_i64(v as i64)
    }

    #[inline]
    fn visit_i64(&mut self, v: i64) -> Result<Self::Value, Self::Error>;

    #[inline]
    fn visit_usize(&mut self, v: usize) -> Result<Self::Value, Self::Error> {
        self.visit_u64(v as u64)
    }

    #[inline]
    fn visit_u8(&mut self, v: u8) -> Result<Self::Value, Self::Error> {
        self.visit_u64(v as u64)
    }

    #[inline]
    fn visit_u16(&mut self, v: u16) -> Result<Self::Value, Self::Error> {
        self.visit_u64(v as u64)
    }

    #[inline]
    fn visit_u32(&mut self, v: u32) -> Result<Self::Value, Self::Error> {
        self.visit_u64(v as u64)
    }

    #[inline]
    fn visit_u64(&mut self, v: u64) -> Result<Self::Value, Self::Error>;

    #[inline]
    fn visit_f32(&mut self, v: f32) -> Result<Self::Value, Self::Error> {
        self.visit_f64(v as f64)
    }

    fn visit_f64(&mut self, v: f64) -> Result<Self::Value, Self::Error>;

    #[inline]
    fn visit_char(&mut self, v: char) -> Result<Self::Value, Self::Error> {
        // The unwraps in here should be safe.
        let mut s = &mut [0; 4];
        let len = v.encode_utf8(s).unwrap();
        self.visit_str(str::from_utf8(&s[..len]).unwrap())
    }

    fn visit_str(&mut self, value: &str) -> Result<Self::Value, Self::Error>;

    fn visit_unit(&mut self) -> Result<Self::Value, Self::Error>;

    #[inline]
    fn visit_named_unit(&mut self, _name: &str) -> Result<Self::Value, Self::Error> {
        self.visit_unit()
    }

    #[inline]
    fn visit_enum_unit(&mut self,
                       _name: &str,
                       _variant: &str) -> Result<Self::Value, Self::Error> {
        self.visit_unit()
    }

    fn visit_none(&mut self) -> Result<Self::Value, Self::Error>;

    fn visit_some<V>(&mut self, value: V) -> Result<Self::Value, Self::Error>
        where V: Serialize;

    fn visit_seq<V>(&mut self, visitor: V) -> Result<Self::Value, Self::Error>
        where V: SeqVisitor;

    #[inline]
    fn visit_named_seq<V>(&mut self,
                          _name: &'static str,
                          visitor: V) -> Result<Self::Value, Self::Error>
        where V: SeqVisitor,
    {
        self.visit_seq(visitor)
    }

    #[inline]
    fn visit_enum_seq<V>(&mut self,
                         _name: &'static str,
                         _variant: &'static str,
                         visitor: V) -> Result<Self::Value, Self::Error>
        where V: SeqVisitor,
    {
        self.visit_seq(visitor)
    }

    fn visit_seq_elt<T>(&mut self,
                        first: bool,
                        value: T) -> Result<Self::Value, Self::Error>
        where T: Serialize;

    fn visit_map<V>(&mut self, visitor: V) -> Result<Self::Value, Self::Error>
        where V: MapVisitor;

    #[inline]
    fn visit_named_map<V>(&mut self,
                          _name: &'static str,
                          visitor: V) -> Result<Self::Value, Self::Error>
        where V: MapVisitor,
    {
        self.visit_map(visitor)
    }

    #[inline]
    fn visit_enum_map<V>(&mut self,
                          _name: &'static str,
                          _variant: &'static str,
                          visitor: V) -> Result<Self::Value, Self::Error>
        where V: MapVisitor,
    {
        self.visit_map(visitor)
    }

    fn visit_map_elt<K, V>(&mut self,
                           first: bool,
                           key: K,
                           value: V) -> Result<Self::Value, Self::Error>
        where K: Serialize,
              V: Serialize;
}

pub trait SeqVisitor {
    fn visit<
        V: Visitor,
    >(&mut self, visitor: &mut V) -> Result<Option<V::Value>, V::Error>;

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

pub trait MapVisitor {
    fn visit<
        V: Visitor,
    >(&mut self, visitor: &mut V) -> Result<Option<V::Value>, V::Error>;

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
            fn visit<
                V: Visitor,
            >(&self, visitor: &mut V) -> Result<V::Value, V::Error> {
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
    fn visit<
        V: Visitor,
    >(&self, visitor: &mut V) -> Result<V::Value, V::Error> {
        visitor.visit_str(*self)
    }
}

impl Serialize for String {
    #[inline]
    fn visit<
        V: Visitor,
    >(&self, visitor: &mut V) -> Result<V::Value, V::Error> {
        (&self[..]).visit(visitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<T> Serialize for Option<T> where T: Serialize {
    #[inline]
    fn visit<
        V: Visitor,
    >(&self, visitor: &mut V) -> Result<V::Value, V::Error> {
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

impl<T, Iter: Iterator<Item=T>> SeqIteratorVisitor<Iter> {
    #[inline]
    pub fn new(iter: Iter) -> SeqIteratorVisitor<Iter> {
        SeqIteratorVisitor {
            iter: iter,
            first: true,
        }
    }
}

impl<
    T: Serialize,
    Iter: Iterator<Item=T>,
> SeqVisitor for SeqIteratorVisitor<Iter> {
    #[inline]
    fn visit<
        V: Visitor,
    >(&mut self, visitor: &mut V) -> Result<Option<V::Value>, V::Error> {
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

impl<
    'a,
    T: Serialize,
> Serialize for &'a [T] {
    #[inline]
    fn visit<
        V: Visitor,
    >(&self, visitor: &mut V) -> Result<V::Value, V::Error> {
        visitor.visit_seq(SeqIteratorVisitor::new(self.iter()))
    }
}

impl<
    T: Serialize,
> Serialize for Vec<T> {
    #[inline]
    fn visit<
        V: Visitor,
    >(&self, visitor: &mut V) -> Result<V::Value, V::Error> {
        (&self[..]).visit(visitor)
    }
}

impl<T> Serialize for BTreeSet<T> where T: Serialize {
    #[inline]
    fn visit<
        V: Visitor,
    >(&self, visitor: &mut V) -> Result<V::Value, V::Error> {
        visitor.visit_seq(SeqIteratorVisitor::new(self.iter()))
    }
}

impl<T, S> Serialize for HashSet<T, S>
    where T: Serialize + Eq + Hash,
          S: HashState,
{
    #[inline]
    fn visit<V: Visitor>(&self, visitor: &mut V) -> Result<V::Value, V::Error> {
        visitor.visit_seq(SeqIteratorVisitor::new(self.iter()))
    }
}

///////////////////////////////////////////////////////////////////////////////

impl Serialize for () {
    #[inline]
    fn visit<
        V: Visitor,
    >(&self, visitor: &mut V) -> Result<V::Value, V::Error> {
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

            impl<
                'a,
                $($T: Serialize),+
            > SeqVisitor for $TupleVisitor<'a, $($T),+> {
                fn visit<
                    V: Visitor,
                >(&mut self, visitor: &mut V) -> Result<Option<V::Value>, V::Error> {
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

            impl<
                $($T: Serialize),+
            > Serialize for ($($T,)+) {
                #[inline]
                fn visit<V: Visitor>(&self, visitor: &mut V) -> Result<V::Value, V::Error> {
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

impl<K, V, Iter: Iterator<Item=(K, V)>> MapIteratorVisitor<Iter> {
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
    fn visit<
        V_: Visitor,
    >(&mut self, visitor: &mut V_) -> Result<Option<V_::Value>, V_::Error> {
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
    fn visit<V_: Visitor>(&self, visitor: &mut V_) -> Result<V_::Value, V_::Error> {
        visitor.visit_map(MapIteratorVisitor::new(self.iter()))
    }
}

impl<K, V, S> Serialize for HashMap<K, V, S>
    where K: Serialize + Eq + Hash,
          V: Serialize,
          S: HashState,
{
    #[inline]
    fn visit<V_: Visitor>(&self, visitor: &mut V_) -> Result<V_::Value, V_::Error> {
        visitor.visit_map(MapIteratorVisitor::new(self.iter()))
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<'a, T> Serialize for &'a T where T: Serialize {
    #[inline]
    fn visit<
        V: Visitor,
    >(&self, visitor: &mut V) -> Result<V::Value, V::Error> {
        (**self).visit(visitor)
    }
}

impl<'a, T> Serialize for Box<T> where T: Serialize {
    #[inline]
    fn visit<
        V: Visitor,
    >(&self, visitor: &mut V) -> Result<V::Value, V::Error> {
        (**self).visit(visitor)
    }
}

impl<
    'a,
    T: Serialize,
> Serialize for Rc<T> {
    #[inline]
    fn visit<
        V: Visitor,
    >(&self, visitor: &mut V) -> Result<V::Value, V::Error> {
        (**self).visit(visitor)
    }
}

impl<
    'a,
    T: Serialize,
> Serialize for Arc<T> {
    #[inline]
    fn visit<
        V: Visitor,
    >(&self, visitor: &mut V) -> Result<V::Value, V::Error> {
        (**self).visit(visitor)
    }
}

//////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::{Serialize, Serializer, Visitor, SeqVisitor, MapVisitor};
    use std::vec;
    use std::collections::BTreeMap;

    #[derive(Clone, PartialEq, Debug)]
    pub enum Token<'a> {
        Bool(bool),
        Isize(isize),
        I8(i8),
        I16(i16),
        I32(i32),
        I64(i64),
        Usize(usize),
        U8(u8),
        U16(u16),
        U32(u32),
        U64(u64),
        F32(f32),
        F64(f64),
        Char(char),
        Str(&'a str),

        Option(bool),

        Unit,
        NamedUnit(&'a str),
        EnumUnit(&'a str, &'a str),

        SeqStart(usize),
        NamedSeqStart(&'a str, usize),
        EnumSeqStart(&'a str, &'a str, usize),
        SeqSep(bool),
        SeqEnd,

        MapStart(usize),
        NamedMapStart(&'a str, usize),
        EnumMapStart(&'a str, &'a str, usize),
        MapSep(bool),
        MapEnd,
    }

    struct AssertSerializer<'a> {
        iter: vec::IntoIter<Token<'a>>,
    }

    impl<'a> AssertSerializer<'a> {
        fn new(values: Vec<Token<'a>>) -> AssertSerializer {
            AssertSerializer {
                iter: values.into_iter(),
            }
        }

        fn visit_sequence<V>(&mut self, mut visitor: V) -> Result<(), ()>
            where V: SeqVisitor
        {
            while let Some(()) = try!(visitor.visit(self)) { }

            assert_eq!(self.iter.next(), Some(Token::SeqEnd));

            Ok(())
        }

        fn visit_mapping<V>(&mut self, mut visitor: V) -> Result<(), ()>
            where V: MapVisitor
        {
            while let Some(()) = try!(visitor.visit(self)) { }

            assert_eq!(self.iter.next(), Some(Token::MapEnd));

            Ok(())
        }
    }

    impl<'a> Serializer for AssertSerializer<'a> {
        type Value = ();
        type Error = ();

        fn visit<T: Serialize>(&mut self, value: &T) -> Result<(), ()> {
            value.visit(self)
        }
    }

    impl<'a> Visitor for AssertSerializer<'a> {
        type Value = ();
        type Error = ();

        fn visit_unit(&mut self) -> Result<(), ()> {
            assert_eq!(self.iter.next(), Some(Token::Unit));
            Ok(())
        }

        fn visit_named_unit(&mut self, name: &str) -> Result<(), ()> {
            assert_eq!(self.iter.next().unwrap(), Token::NamedUnit(name));
            Ok(())
        }

        fn visit_enum_unit(&mut self, name: &str, variant: &str) -> Result<(), ()> {
            assert_eq!(self.iter.next().unwrap(), Token::EnumUnit(name, variant));
            Ok(())
        }

        fn visit_bool(&mut self, v: bool) -> Result<(), ()> {
            assert_eq!(self.iter.next(), Some(Token::Bool(v)));
            Ok(())
        }

        fn visit_isize(&mut self, v: isize) -> Result<(), ()> {
            assert_eq!(self.iter.next(), Some(Token::Isize(v)));
            Ok(())
        }

        fn visit_i8(&mut self, v: i8) -> Result<(), ()> {
            assert_eq!(self.iter.next(), Some(Token::I8(v)));
            Ok(())
        }

        fn visit_i16(&mut self, v: i16) -> Result<(), ()> {
            assert_eq!(self.iter.next(), Some(Token::I16(v)));
            Ok(())
        }

        fn visit_i32(&mut self, v: i32) -> Result<(), ()> {
            assert_eq!(self.iter.next(), Some(Token::I32(v)));
            Ok(())
        }

        fn visit_i64(&mut self, v: i64) -> Result<(), ()> {
            assert_eq!(self.iter.next(), Some(Token::I64(v)));
            Ok(())
        }

        fn visit_usize(&mut self, v: usize) -> Result<(), ()> {
            assert_eq!(self.iter.next(), Some(Token::Usize(v)));
            Ok(())
        }

        fn visit_u8(&mut self, v: u8) -> Result<(), ()> {
            assert_eq!(self.iter.next(), Some(Token::U8(v)));
            Ok(())
        }

        fn visit_u16(&mut self, v: u16) -> Result<(), ()> {
            assert_eq!(self.iter.next(), Some(Token::U16(v)));
            Ok(())
        }

        fn visit_u32(&mut self, v: u32) -> Result<(), ()> {
            assert_eq!(self.iter.next(), Some(Token::U32(v)));
            Ok(())
        }

        fn visit_u64(&mut self, v: u64) -> Result<(), ()> {
            assert_eq!(self.iter.next(), Some(Token::U64(v)));
            Ok(())
        }

        fn visit_f32(&mut self, v: f32) -> Result<(), ()> {
            assert_eq!(self.iter.next(), Some(Token::F32(v)));
            Ok(())
        }

        fn visit_f64(&mut self, v: f64) -> Result<(), ()> {
            assert_eq!(self.iter.next(), Some(Token::F64(v)));
            Ok(())
        }

        fn visit_char(&mut self, v: char) -> Result<(), ()> {
            assert_eq!(self.iter.next(), Some(Token::Char(v)));
            Ok(())
        }

        fn visit_str(&mut self, v: &str) -> Result<(), ()> {
            assert_eq!(self.iter.next().unwrap(), Token::Str(v));
            Ok(())
        }

        fn visit_none(&mut self) -> Result<(), ()> {
            assert_eq!(self.iter.next(), Some(Token::Option(false)));
            Ok(())
        }

        fn visit_some<V>(&mut self, value: V) -> Result<(), ()>
            where V: Serialize,
        {
            assert_eq!(self.iter.next(), Some(Token::Option(true)));
            value.visit(self)
        }


        fn visit_seq<V>(&mut self, visitor: V) -> Result<(), ()>
            where V: SeqVisitor
        {
            let (len, _) = visitor.size_hint();

            assert_eq!(self.iter.next(), Some(Token::SeqStart(len)));

            self.visit_sequence(visitor)
        }

        fn visit_named_seq<V>(&mut self, name: &str, visitor: V) -> Result<(), ()>
            where V: SeqVisitor
        {
            let (len, _) = visitor.size_hint();

            assert_eq!(self.iter.next().unwrap(), Token::NamedSeqStart(name, len));

            self.visit_sequence(visitor)
        }

        fn visit_enum_seq<V>(&mut self,
                             name: &str,
                             variant: &str,
                             visitor: V) -> Result<(), ()>
            where V: SeqVisitor
        {
            let (len, _) = visitor.size_hint();

            assert_eq!(self.iter.next().unwrap(), Token::EnumSeqStart(name, variant, len));

            self.visit_sequence(visitor)
        }

        fn visit_seq_elt<T>(&mut self, first: bool, value: T) -> Result<(), ()>
            where T: Serialize
        {
            assert_eq!(self.iter.next(), Some(Token::SeqSep(first)));
            value.visit(self)
        }

        fn visit_map<V>(&mut self, visitor: V) -> Result<(), ()>
            where V: MapVisitor
        {
            let (len, _) = visitor.size_hint();

            assert_eq!(self.iter.next(), Some(Token::MapStart(len)));

            self.visit_mapping(visitor)
        }

        fn visit_named_map<V>(&mut self, name: &str, visitor: V) -> Result<(), ()>
            where V: MapVisitor
        {
            let (len, _) = visitor.size_hint();

            assert_eq!(self.iter.next().unwrap(), Token::NamedMapStart(name, len));

            self.visit_mapping(visitor)
        }

        fn visit_enum_map<V>(&mut self,
                             name: &str,
                             variant: &str,
                             visitor: V) -> Result<(), ()>
            where V: MapVisitor
        {
            let (len, _) = visitor.size_hint();

            assert_eq!(self.iter.next().unwrap(), Token::EnumMapStart(name, variant, len));

            self.visit_mapping(visitor)
        }

        fn visit_map_elt<K, V>(&mut self,
                               first: bool,
                               key: K,
                               value: V) -> Result<(), ()>
            where K: Serialize,
                  V: Serialize,
        {
            assert_eq!(self.iter.next(), Some(Token::MapSep(first)));

            try!(key.visit(self));
            value.visit(self)
        }
    }

    struct NamedUnit;

    impl Serialize for NamedUnit {
        fn visit<
            V: Visitor,
        >(&self, visitor: &mut V) -> Result<V::Value, V::Error> {
            visitor.visit_named_unit("NamedUnit")
        }
    }

    struct NamedSeq(i32, i32, i32);

    impl Serialize for NamedSeq {
        fn visit<
            V: Visitor,
        >(&self, visitor: &mut V) -> Result<V::Value, V::Error> {
            visitor.visit_named_seq("NamedSeq", NamedSeqVisitor {
                tuple: self,
                state: 0,
            })
        }
    }

    struct NamedSeqVisitor<'a> {
        tuple: &'a NamedSeq,
        state: u8,
    }

    impl<'a> SeqVisitor for NamedSeqVisitor<'a> {
        fn visit<
            V: Visitor,
        >(&mut self, visitor: &mut V) -> Result<Option<V::Value>, V::Error> {
            match self.state {
                0 => {
                    self.state += 1;
                    Ok(Some(try!(visitor.visit_seq_elt(true, &self.tuple.0))))
                }
                1 => {
                    self.state += 1;
                    Ok(Some(try!(visitor.visit_seq_elt(false, &self.tuple.1))))
                }
                2 => {
                    self.state += 1;
                    Ok(Some(try!(visitor.visit_seq_elt(false, &self.tuple.2))))
                }
                _ => Ok(None)
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (3, Some(3))
        }
    }

    enum Enum {
        Unit,
        Seq(i32, i32),
        Map { a: i32, b: i32 },
    }

    impl Serialize for Enum {
        fn visit<
            V: Visitor,
        >(&self, visitor: &mut V) -> Result<V::Value, V::Error> {
            match *self {
                Enum::Unit => {
                    visitor.visit_enum_unit("Enum", "Unit")
                }
                Enum::Seq(ref a, ref b) => {
                    visitor.visit_enum_seq("Enum", "Seq", EnumSeqVisitor {
                        a: a,
                        b: b,
                        state: 0,
                    })
                }
                Enum::Map { ref a, ref b } => {
                    visitor.visit_enum_map("Enum", "Map", EnumMapVisitor {
                        a: a,
                        b: b,
                        state: 0,
                    })
                }
            }
        }
    }

    struct EnumSeqVisitor<'a> {
        a: &'a i32,
        b: &'a i32,
        state: u8,
    }

    impl<'a> SeqVisitor for EnumSeqVisitor<'a> {
        fn visit<
            V: Visitor,
        >(&mut self, visitor: &mut V) -> Result<Option<V::Value>, V::Error> {
            match self.state {
                0 => {
                    self.state += 1;
                    Ok(Some(try!(visitor.visit_seq_elt(true, self.a))))
                }
                1 => {
                    self.state += 1;
                    Ok(Some(try!(visitor.visit_seq_elt(false, self.b))))
                }
                _ => {
                    Ok(None)
                }
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (2, Some(2))
        }
    }

    struct EnumMapVisitor<'a> {
        a: &'a i32,
        b: &'a i32,
        state: u8,
    }

    impl<'a> MapVisitor for EnumMapVisitor<'a> {
        fn visit<
            V: Visitor,
        >(&mut self, visitor: &mut V) -> Result<Option<V::Value>, V::Error> {
            match self.state {
                0 => {
                    self.state += 1;
                    Ok(Some(try!(visitor.visit_map_elt(true, "a", self.a))))
                }
                1 => {
                    self.state += 1;
                    Ok(Some(try!(visitor.visit_map_elt(false, "b", self.b))))
                }
                _ => {
                    Ok(None)
                }
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (2, Some(2))
        }
    }

    macro_rules! btreemap {
        () => {
            BTreeMap::new()
        };
        ($($key:expr => $value:expr),+) => {
            {
                let mut map = BTreeMap::new();
                $(map.insert($key, $value);)+
                map
            }
        }
    }

    macro_rules! declare_test {
        ($name:ident { $($value:expr => $tokens:expr,)+ }) => {
            #[test]
            fn $name() {
                $(
                    let mut ser = AssertSerializer::new($tokens);
                    assert_eq!(ser.visit(&$value), Ok(()));
                )+
            }
        }
    }

    macro_rules! declare_tests {
        ($($name:ident { $($value:expr => $tokens:expr,)+ })+) => {
            $(
                declare_test!($name { $($value => $tokens,)+ });
            )+
        }
    }

    declare_tests! {
        test_unit {
            () => vec![Token::Unit],
        }
        test_bool {
            true => vec![Token::Bool(true)],
            false => vec![Token::Bool(false)],
        }
        test_isizes {
            0isize => vec![Token::Isize(0)],
            0i8 => vec![Token::I8(0)],
            0i16 => vec![Token::I16(0)],
            0i32 => vec![Token::I32(0)],
            0i64 => vec![Token::I64(0)],
        }
        test_usizes {
            0usize => vec![Token::Usize(0)],
            0u8 => vec![Token::U8(0)],
            0u16 => vec![Token::U16(0)],
            0u32 => vec![Token::U32(0)],
            0u64 => vec![Token::U64(0)],
        }
        test_floats {
            0f32 => vec![Token::F32(0.)],
            0f64 => vec![Token::F64(0.)],
        }
        test_char {
            'a' => vec![Token::Char('a')],
        }
        test_str {
            "abc" => vec![Token::Str("abc")],
            "abc".to_string() => vec![Token::Str("abc")],
        }
        test_option {
            None::<i32> => vec![Token::Option(false)],
            Some(1) => vec![
                Token::Option(true),
                Token::I32(1),
            ],
        }
        test_slice {
            &[0][..0] => vec![
                Token::SeqStart(0),
                Token::SeqEnd,
            ],
            &[1, 2, 3][..] => vec![
                Token::SeqStart(3),
                    Token::SeqSep(true),
                    Token::I32(1),

                    Token::SeqSep(false),
                    Token::I32(2),

                    Token::SeqSep(false),
                    Token::I32(3),
                Token::SeqEnd,
            ],
        }
        test_vec {
            Vec::<isize>::new() => vec![
                Token::SeqStart(0),
                Token::SeqEnd,
            ],
            vec![vec![], vec![1], vec![2, 3]] => vec![
                Token::SeqStart(3),
                    Token::SeqSep(true),
                    Token::SeqStart(0),
                    Token::SeqEnd,

                    Token::SeqSep(false),
                    Token::SeqStart(1),
                        Token::SeqSep(true),
                        Token::I32(1),
                    Token::SeqEnd,

                    Token::SeqSep(false),
                    Token::SeqStart(2),
                        Token::SeqSep(true),
                        Token::I32(2),

                        Token::SeqSep(false),
                        Token::I32(3),
                    Token::SeqEnd,
                Token::SeqEnd,
            ],
        }
        test_tuple {
            (1,) => vec![
                Token::SeqStart(1),
                    Token::SeqSep(true),
                    Token::I32(1),
                Token::SeqEnd,
            ],
            (1, 2, 3) => vec![
                Token::SeqStart(3),
                    Token::SeqSep(true),
                    Token::I32(1),

                    Token::SeqSep(false),
                    Token::I32(2),

                    Token::SeqSep(false),
                    Token::I32(3),
                Token::SeqEnd,
            ],
        }
        test_btreemap {
            btreemap![1 => 2] => vec![
                Token::MapStart(1),
                    Token::MapSep(true),
                    Token::I32(1),
                    Token::I32(2),
                Token::MapEnd,
            ],
            btreemap![1 => 2, 3 => 4] => vec![
                Token::MapStart(2),
                    Token::MapSep(true),
                    Token::I32(1),
                    Token::I32(2),

                    Token::MapSep(false),
                    Token::I32(3),
                    Token::I32(4),
                Token::MapEnd,
            ],
            btreemap![1 => btreemap![], 2 => btreemap![3 => 4, 5 => 6]] => vec![
                Token::MapStart(2),
                    Token::MapSep(true),
                    Token::I32(1),
                    Token::MapStart(0),
                    Token::MapEnd,

                    Token::MapSep(false),
                    Token::I32(2),
                    Token::MapStart(2),
                        Token::MapSep(true),
                        Token::I32(3),
                        Token::I32(4),

                        Token::MapSep(false),
                        Token::I32(5),
                        Token::I32(6),
                    Token::MapEnd,
                Token::MapEnd,
            ],
        }
        test_named_unit {
            NamedUnit => vec![Token::NamedUnit("NamedUnit")],
        }
        test_named_seq {
            NamedSeq(1, 2, 3) => vec![
                Token::NamedSeqStart("NamedSeq", 3),
                    Token::SeqSep(true),
                    Token::I32(1),

                    Token::SeqSep(false),
                    Token::I32(2),

                    Token::SeqSep(false),
                    Token::I32(3),
                Token::SeqEnd,
            ],
        }
        test_enum {
            Enum::Unit => vec![Token::EnumUnit("Enum", "Unit")],
            Enum::Seq(1, 2) => vec![
                Token::EnumSeqStart("Enum", "Seq", 2),
                    Token::SeqSep(true),
                    Token::I32(1),

                    Token::SeqSep(false),
                    Token::I32(2),
                Token::SeqEnd,
            ],
            Enum::Map { a: 1, b: 2 } => vec![
                Token::EnumMapStart("Enum", "Map", 2),
                    Token::MapSep(true),
                    Token::Str("a"),
                    Token::I32(1),

                    Token::MapSep(false),
                    Token::Str("b"),
                    Token::I32(2),
                Token::MapEnd,
            ],
        }
    }
}
