use std::collections::TreeMap;

///////////////////////////////////////////////////////////////////////////////

pub trait Serialize<S, R, E> {
    fn serialize(&self, state: &mut S) -> Result<R, E>;
}

///////////////////////////////////////////////////////////////////////////////

pub trait Serializer<S, R, E> {
    fn serialize<
        T: Serialize<S, R, E>,
    >(&mut self, value: &T) -> Result<R, E>;
}

///////////////////////////////////////////////////////////////////////////////

pub trait Visitor<R, E> {
    fn visit_null(&mut self) -> Result<R, E>;

    fn visit_bool(&mut self, v: bool) -> Result<R, E>;

    #[inline]
    fn visit_int(&mut self, v: int) -> Result<R, E> {
        self.visit_i64(v as i64)
    }

    #[inline]
    fn visit_i8(&mut self, v: i8) -> Result<R, E> {
        self.visit_i64(v as i64)
    }

    #[inline]
    fn visit_i16(&mut self, v: i16) -> Result<R, E> {
        self.visit_i64(v as i64)
    }

    #[inline]
    fn visit_i32(&mut self, v: i32) -> Result<R, E> {
        self.visit_i64(v as i64)
    }

    #[inline]
    fn visit_i64(&mut self, v: i64) -> Result<R, E>;

    #[inline]
    fn visit_uint(&mut self, v: uint) -> Result<R, E> {
        self.visit_u64(v as u64)
    }

    #[inline]
    fn visit_u8(&mut self, v: u8) -> Result<R, E> {
        self.visit_u64(v as u64)
    }

    #[inline]
    fn visit_u16(&mut self, v: u16) -> Result<R, E> {
        self.visit_u64(v as u64)
    }

    #[inline]
    fn visit_u32(&mut self, v: u32) -> Result<R, E> {
        self.visit_u64(v as u64)
    }

    #[inline]
    fn visit_u64(&mut self, v: u64) -> Result<R, E>;

    #[inline]
    fn visit_f32(&mut self, v: f32) -> Result<R, E> {
        self.visit_f64(v as f64)
    }

    fn visit_f64(&mut self, v: f64) -> Result<R, E>;

    fn visit_char(&mut self, value: char) -> Result<R, E>;

    fn visit_str(&mut self, value: &str) -> Result<R, E>;

    fn visit_seq<
        V: SeqVisitor<Self, R, E>,
    >(&mut self, visitor: V) -> Result<R, E>;

    #[inline]
    fn visit_named_seq<
        V: SeqVisitor<Self, R, E>,
    >(&mut self, _name: &'static str, visitor: V) -> Result<R, E> {
        self.visit_seq(visitor)
    }

    #[inline]
    fn visit_enum<
        V: SeqVisitor<Self, R, E>,
    >(&mut self, _name: &'static str, _variant: &'static str, visitor: V) -> Result<R, E> {
        self.visit_seq(visitor)
    }

    fn visit_seq_elt<
        T: Serialize<Self, R, E>,
    >(&mut self, first: bool, value: T) -> Result<R, E>;

    fn visit_map<
        V: MapVisitor<Self, R, E>,
    >(&mut self, visitor: V) -> Result<R, E>;

    #[inline]
    fn visit_named_map<
        V: MapVisitor<Self, R, E>,
    >(&mut self, _name: &'static str, visitor: V) -> Result<R, E> {
        self.visit_map(visitor)
    }

    fn visit_map_elt<
        K: Serialize<Self, R, E>,
        V: Serialize<Self, R, E>,
    >(&mut self, first: bool, key: K, value: V) -> Result<R, E>;
}

pub trait SeqVisitor<S, R, E> {
    fn visit(&mut self, state: &mut S) -> Result<Option<R>, E>;

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        (0, None)
    }
}

pub trait MapVisitor<S, R, E> {
    fn visit(&mut self, state: &mut S) -> Result<Option<R>, E>;

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        (0, None)
    }
}


///////////////////////////////////////////////////////////////////////////////

macro_rules! impl_serialize {
    ($ty:ty, $method:ident) => {
        impl<S: Visitor<R, E>, R, E> Serialize<S, R, E> for $ty {
            #[inline]
            fn serialize(&self, state: &mut S) -> Result<R, E> {
                state.$method(*self)
            }
        }
    }
}

impl_serialize!(bool, visit_bool)
impl_serialize!(int, visit_int)
impl_serialize!(i8, visit_i8)
impl_serialize!(i16, visit_i16)
impl_serialize!(i32, visit_i32)
impl_serialize!(i64, visit_i64)
impl_serialize!(uint, visit_uint)
impl_serialize!(u8, visit_u8)
impl_serialize!(u16, visit_u16)
impl_serialize!(u32, visit_u32)
impl_serialize!(u64, visit_u64)
impl_serialize!(f32, visit_f32)
impl_serialize!(f64, visit_f64)
impl_serialize!(char, visit_char)

///////////////////////////////////////////////////////////////////////////////

impl<'a, S: Visitor<R, E>, R, E> Serialize<S, R, E> for &'a str {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<R, E> {
        s.visit_str(*self)
    }
}

impl<S: Visitor<R, E>, R, E> Serialize<S, R, E> for String {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<R, E> {
        s.visit_str(self.as_slice())
    }
}

///////////////////////////////////////////////////////////////////////////////

pub struct SeqIteratorVisitor<Iter> {
    iter: Iter,
    first: bool,
}

impl<T, Iter: Iterator<T>> SeqIteratorVisitor<Iter> {
    #[inline]
    pub fn new(iter: Iter) -> SeqIteratorVisitor<Iter> {
        SeqIteratorVisitor {
            iter: iter,
            first: true,
        }
    }
}

impl<
    Iter: Iterator<T>,
    S: Visitor<R, E>,
    R,
    E,
    T: Serialize<S, R, E>,
> SeqVisitor<S, R, E> for SeqIteratorVisitor<Iter> {
    #[inline]
    fn visit(&mut self, state: &mut S) -> Result<Option<R>, E> {
        let first = self.first;
        self.first = false;

        match self.iter.next() {
            Some(value) => {
                let value = try!(state.visit_seq_elt(first, value));
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
    S: Visitor<R, E>,
    R,
    E,
    T: Serialize<S, R, E>,
> Serialize<S, R, E> for Vec<T> {
    #[inline]
    fn serialize(&self, state: &mut S) -> Result<R, E> {
        state.visit_seq(SeqIteratorVisitor::new(self.iter()))
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<
    'a,
    S: Visitor<R, E>,
    R,
    E,
    T0: Serialize<S, R, E>,
    T1: Serialize<S, R, E>
> Serialize<S, R, E> for (T0, T1) {
    #[inline]
    fn serialize(&self, state: &mut S) -> Result<R, E> {
        struct Visitor<'a, T0: 'a, T1: 'a> {
            value: &'a (T0, T1),
            state: uint,
        }

        impl<
            'a,
            S: self::Visitor<R, E>,
            R,
            E,
            T0: Serialize<S, R, E>,
            T1: Serialize<S, R, E>,
        > SeqVisitor<S, R, E> for Visitor<'a, T0, T1> {
            #[inline]
            fn visit(&mut self, state: &mut S) -> Result<Option<R>, E> {
                match self.state {
                    0 => {
                        self.state += 1;
                        let (ref value, _) = *self.value;
                        let v = try!(state.visit_seq_elt(true, value));
                        Ok(Some(v))
                    }
                    1 => {
                        self.state += 1;
                        let (_, ref value) = *self.value;
                        let v = try!(state.visit_seq_elt(false, value));
                        Ok(Some(v))
                    }
                    _ => {
                        Ok(None)
                    }
                }
            }

            #[inline]
            fn size_hint(&self) -> (uint, Option<uint>) {
                let size = 2 - self.state;
                (size, Some(size))
            }
        }

        state.visit_seq(Visitor { value: self, state: 0 })
    }
}


///////////////////////////////////////////////////////////////////////////////

pub struct MapIteratorVisitor<Iter> {
    iter: Iter,
    first: bool,
}

impl<K, V, Iter: Iterator<(K, V)>> MapIteratorVisitor<Iter> {
    #[inline]
    pub fn new(iter: Iter) -> MapIteratorVisitor<Iter> {
        MapIteratorVisitor {
            iter: iter,
            first: true,
        }
    }
}

impl<
    S: Visitor<R, E>,
    R,
    E,
    K: Serialize<S, R, E>,
    V: Serialize<S, R, E>,
    Iter: Iterator<(K, V)>,
> MapVisitor<S, R, E> for MapIteratorVisitor<Iter> {
    #[inline]
    fn visit(&mut self, state: &mut S) -> Result<Option<R>, E> {
        let first = self.first;
        self.first = false;

        match self.iter.next() {
            Some((key, value)) => {
                let value = try!(state.visit_map_elt(first, key, value));
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
    S: Visitor<R, E>,
    R,
    E,
    K: Serialize<S, R, E> + Ord,
    V: Serialize<S, R, E>,
> Serialize<S, R, E> for TreeMap<K, V> {
    #[inline]
    fn serialize(&self, state: &mut S) -> Result<R, E> {
        state.visit_map(MapIteratorVisitor::new(self.iter()))
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<
    'a,
    S: Visitor<R, E>,
    R,
    E,
    T: Serialize<S, R, E>
> Serialize<S, R, E> for &'a T {
    #[inline]
    fn serialize(&self, state: &mut S) -> Result<R, E> {
        (**self).serialize(state)
    }
}
