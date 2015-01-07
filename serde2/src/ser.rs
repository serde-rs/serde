use std::collections::BTreeMap;

///////////////////////////////////////////////////////////////////////////////

pub trait Serialize {
    fn visit<
        S,
        R,
        E,
        V: Visitor<S, R, E>,
    >(&self, state: &mut S, visitor: V) -> Result<R, E>;
}

///////////////////////////////////////////////////////////////////////////////

pub trait Serializer<S, R, E> {
    fn visit<
        T: Serialize,
    >(&mut self, value: &T) -> Result<R, E>;
}

///////////////////////////////////////////////////////////////////////////////

pub trait Visitor<S, R, E> {
    fn visit_null(&self, state: &mut S) -> Result<R, E>;

    fn visit_bool(&self, state: &mut S, v: bool) -> Result<R, E>;

    #[inline]
    fn visit_int(&self, state: &mut S, v: int) -> Result<R, E> {
        self.visit_i64(state, v as i64)
    }

    #[inline]
    fn visit_i8(&self, state: &mut S, v: i8) -> Result<R, E> {
        self.visit_i64(state, v as i64)
    }

    #[inline]
    fn visit_i16(&self, state: &mut S, v: i16) -> Result<R, E> {
        self.visit_i64(state, v as i64)
    }

    #[inline]
    fn visit_i32(&self, state: &mut S, v: i32) -> Result<R, E> {
        self.visit_i64(state, v as i64)
    }

    #[inline]
    fn visit_i64(&self, state: &mut S, v: i64) -> Result<R, E>;

    #[inline]
    fn visit_uint(&self, state: &mut S, v: uint) -> Result<R, E> {
        self.visit_u64(state, v as u64)
    }

    #[inline]
    fn visit_u8(&self, state: &mut S, v: u8) -> Result<R, E> {
        self.visit_u64(state, v as u64)
    }

    #[inline]
    fn visit_u16(&self, state: &mut S, v: u16) -> Result<R, E> {
        self.visit_u64(state, v as u64)
    }

    #[inline]
    fn visit_u32(&self, state: &mut S, v: u32) -> Result<R, E> {
        self.visit_u64(state, v as u64)
    }

    #[inline]
    fn visit_u64(&self, state: &mut S, v: u64) -> Result<R, E>;

    #[inline]
    fn visit_f32(&self, state: &mut S, v: f32) -> Result<R, E> {
        self.visit_f64(state, v as f64)
    }

    fn visit_f64(&self, state: &mut S, v: f64) -> Result<R, E>;

    fn visit_char(&self, state: &mut S, value: char) -> Result<R, E>;

    fn visit_str(&self, state: &mut S, value: &str) -> Result<R, E>;

    fn visit_seq<
        V: SeqVisitor<S, R, E>,
    >(&self, state: &mut S, visitor: V) -> Result<R, E>;

    #[inline]
    fn visit_named_seq<
        V: SeqVisitor<S, R, E>,
    >(&self, state: &mut S, _name: &'static str, visitor: V) -> Result<R, E> {
        self.visit_seq(state, visitor)
    }

    #[inline]
    fn visit_enum<
        V: SeqVisitor<S, R, E>,
    >(&self, state: &mut S, _name: &'static str, _variant: &'static str, visitor: V) -> Result<R, E> {
        self.visit_seq(state, visitor)
    }

    fn visit_seq_elt<
        T: Serialize,
    >(&self, state: &mut S, first: bool, value: T) -> Result<R, E>;

    fn visit_map<
        V: MapVisitor<S, R, E>,
    >(&self, state: &mut S, visitor: V) -> Result<R, E>;

    #[inline]
    fn visit_named_map<
        V: MapVisitor<S, R, E>,
    >(&self, state: &mut S, _name: &'static str, visitor: V) -> Result<R, E> {
        self.visit_map(state, visitor)
    }

    fn visit_map_elt<
        K: Serialize,
        V: Serialize,
    >(&self, state: &mut S, first: bool, key: K, value: V) -> Result<R, E>;
}

pub trait SeqVisitor<S, R, E> {
    fn visit<
        V: Visitor<S, R, E>,
    >(&mut self, state: &mut S, visitor: V) -> Result<Option<R>, E>;

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        (0, None)
    }
}

pub trait MapVisitor<S, R, E> {
    fn visit<
        V: Visitor<S, R, E>,
    >(&mut self, state: &mut S, visitor: V) -> Result<Option<R>, E>;

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        (0, None)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl Serialize for () {
    #[inline]
    fn visit<
        S,
        R,
        E,
        V: Visitor<S, R, E>,
    >(&self, state: &mut S, visitor: V) -> Result<R, E> {
        visitor.visit_null(state)
    }
}

///////////////////////////////////////////////////////////////////////////////

macro_rules! impl_visit {
    ($ty:ty, $method:ident) => {
        impl Serialize for $ty {
            #[inline]
            fn visit<
                S,
                R,
                E,
                V: Visitor<S, R, E>,
            >(&self, state: &mut S, visitor: V) -> Result<R, E> {
                visitor.$method(state, *self)
            }
        }
    }
}

impl_visit!(bool, visit_bool);
impl_visit!(int, visit_int);
impl_visit!(i8, visit_i8);
impl_visit!(i16, visit_i16);
impl_visit!(i32, visit_i32);
impl_visit!(i64, visit_i64);
impl_visit!(uint, visit_uint);
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
        S,
        R,
        E,
        V: Visitor<S, R, E>,
    >(&self, state: &mut S, visitor: V) -> Result<R, E> {
        visitor.visit_str(state, *self)
    }
}

impl Serialize for String {
    #[inline]
    fn visit<
        S,
        R,
        E,
        V: Visitor<S, R, E>,
    >(&self, state: &mut S, visitor: V) -> Result<R, E> {
        visitor.visit_str(state, self.as_slice())
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
    S,
    R,
    E,
> SeqVisitor<S, R, E> for SeqIteratorVisitor<Iter> {
    #[inline]
    fn visit<
        V: Visitor<S, R, E>,
    >(&mut self, state: &mut S, visitor: V) -> Result<Option<R>, E> {
        let first = self.first;
        self.first = false;

        match self.iter.next() {
            Some(value) => {
                let value = try!(visitor.visit_seq_elt(state, first, value));
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<
    T: Serialize,
> Serialize for Vec<T> {
    #[inline]
    fn visit<
        S,
        R,
        E,
        V: Visitor<S, R, E>,
    >(&self, state: &mut S, visitor: V) -> Result<R, E> {
        visitor.visit_seq(state, SeqIteratorVisitor::new(self.iter()))
    }
}

///////////////////////////////////////////////////////////////////////////////

// FIXME(rust #19630) Remove this work-around
macro_rules! e {
    ($e:expr) => { $e }
}

macro_rules! tuple_impls {
    ($(
        ($($T:ident),+) {
            $($state:pat => $idx:tt,)+
        }
    )+) => {
        $(
            impl<
                $($T: Serialize),+
            > Serialize for ($($T,)+) {
                #[inline]
                fn visit<
                    S,
                    R,
                    E,
                    V: Visitor<S, R, E>,
                >(&self, state: &mut S, visitor: V) -> Result<R, E> {
                    struct Visitor<'a, $($T: 'a),+> {
                        state: uint,
                        tuple: &'a ($($T,)+),
                    }

                    impl<
                        'a,
                        S,
                        R,
                        E,
                        $($T: Serialize),+
                    > SeqVisitor<S, R, E> for Visitor<'a, $($T),+> {
                        fn visit<
                            V: self::Visitor<S, R, E>,
                        >(&mut self, state: &mut S, visitor: V) -> Result<Option<R>, E> {
                            match self.state {
                                $(
                                    $state => {
                                        self.state += 1;
                                        let value = try!(visitor.visit_seq_elt(
                                            state,
                                            true,
                                            &e!(self.tuple.$idx)));
                                        Ok(Some(value))
                                    }
                                )+
                                _ => {
                                    Ok(None)
                                }
                            }
                        }
                    }

                    visitor.visit_seq(state, Visitor {
                        state: 0,
                        tuple: self,
                    })
                }
            }
        )+
    }
}

tuple_impls! {
    (T0) {
        0 => 0,
    }
    (T0, T1) {
        0 => 0,
        1 => 1,
    }
    (T0, T1, T2, T3) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
    }
    (T0, T1, T2, T3, T4) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
    }
    (T0, T1, T2, T3, T4, T5) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
    }
    (T0, T1, T2, T3, T4, T5, T6) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
        6 => 6,
    }
    (T0, T1, T2, T3, T4, T5, T6, T7) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
        6 => 6,
        7 => 7,
    }
    (T0, T1, T2, T3, T4, T5, T6, T7, T8) {
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
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9) {
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
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10) {
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
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11) {
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

impl<
    K: Serialize,
    V: Serialize,
    Iter: Iterator<Item=(K, V)>,
    S,
    R,
    E,
> MapVisitor<S, R, E> for MapIteratorVisitor<Iter> {
    #[inline]
    fn visit<
        V: Visitor<S, R, E>,
    >(&mut self, state: &mut S, visitor: V) -> Result<Option<R>, E> {
        let first = self.first;
        self.first = false;

        match self.iter.next() {
            Some((key, value)) => {
                let value = try!(visitor.visit_map_elt(state, first, key, value));
                Ok(Some(value))
            }
            None => Ok(None)
        }
    }

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<
    K: Serialize + Ord,
    V: Serialize,
> Serialize for BTreeMap<K, V> {
    #[inline]
    fn visit<
        S,
        R,
        E,
        V: Visitor<S, R, E>,
    >(&self, state: &mut S, visitor: V) -> Result<R, E> {
        visitor.visit_map(state, MapIteratorVisitor::new(self.iter()))
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<
    'a,
    T: Serialize,
> Serialize for &'a T {
    #[inline]
    fn visit<
        S,
        R,
        E,
        V: Visitor<S, R, E>,
    >(&self, state: &mut S, visitor: V) -> Result<R, E> {
        (**self).visit(state, visitor)
    }
}
