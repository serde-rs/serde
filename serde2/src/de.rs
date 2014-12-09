use std::collections::{HashMap, TreeMap};
use std::hash::Hash;

///////////////////////////////////////////////////////////////////////////////

pub trait Error {
    fn syntax_error() -> Self;

    fn end_of_stream_error() -> Self;
}

pub trait Deserialize<S, E> {
    fn deserialize(state: &mut S) -> Result<Self, E>;
}

pub trait Deserializer<E> {
    fn visit<
        R,
        V: Visitor<Self, R, E>,
    >(&mut self, visitor: &mut V) -> Result<R, E>;

    fn visit_option<
        R,
        V: Visitor<Self, R, E>,
    >(&mut self, visitor: &mut V) -> Result<R, E> {
        self.visit(visitor)
    }
}

pub trait Visitor<S: Deserializer<E>, R, E: Error> {
    fn visit_null(&mut self) -> Result<R, E> {
        Err(Error::syntax_error())
    }

    fn visit_bool(&mut self, _v: bool) -> Result<R, E> {
        Err(Error::syntax_error())
    }

    fn visit_int(&mut self, v: int) -> Result<R, E> {
        self.visit_i64(v as i64)
    }

    fn visit_i8(&mut self, v: i8) -> Result<R, E> {
        self.visit_i64(v as i64)
    }

    fn visit_i16(&mut self, v: i16) -> Result<R, E> {
        self.visit_i64(v as i64)
    }

    fn visit_i32(&mut self, v: i32) -> Result<R, E> {
        self.visit_i64(v as i64)
    }

    fn visit_i64(&mut self, _v: i64) -> Result<R, E> {
        Err(Error::syntax_error())
    }

    fn visit_uint(&mut self, v: uint) -> Result<R, E> {
        self.visit_u64(v as u64)
    }

    fn visit_u8(&mut self, v: u8) -> Result<R, E> {
        self.visit_u64(v as u64)
    }

    fn visit_u16(&mut self, v: u16) -> Result<R, E> {
        self.visit_u64(v as u64)
    }

    fn visit_u32(&mut self, v: u32) -> Result<R, E> {
        self.visit_u64(v as u64)
    }

    fn visit_u64(&mut self, _v: u64) -> Result<R, E> {
        Err(Error::syntax_error())
    }

    fn visit_f32(&mut self, v: f32) -> Result<R, E> {
        self.visit_f64(v as f64)
    }

    fn visit_f64(&mut self, _v: f64) -> Result<R, E> {
        Err(Error::syntax_error())
    }

    fn visit_str(&mut self, _v: &str) -> Result<R, E> {
        Err(Error::syntax_error())
    }

    fn visit_string(&mut self, v: String) -> Result<R, E> {
        self.visit_str(v.as_slice())
    }

    fn visit_option<
        V: OptionVisitor<S, E>,
    >(&mut self, _visitor: V) -> Result<R, E> {
        Err(Error::syntax_error())
    }

    fn visit_seq<
        V: SeqVisitor<S, E>,
    >(&mut self, _visitor: V) -> Result<R, E> {
        Err(Error::syntax_error())
    }

    fn visit_map<
        V: MapVisitor<S, E>,
    >(&mut self, _visitor: V) -> Result<R, E> {
        Err(Error::syntax_error())
    }
}

pub trait OptionVisitor<S, E> {
    fn visit<
        T: Deserialize<S, E>,
    >(&mut self) -> Result<Option<T>, E>;
}

pub trait SeqVisitor<S, E> {
    fn visit<
        T: Deserialize<S, E>,
    >(&mut self) -> Result<Option<T>, E>;

    fn end(&mut self) -> Result<(), E>;

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        (0, None)
    }
}

pub trait MapVisitor<S, E> {
    fn visit<
        K: Deserialize<S, E>,
        V: Deserialize<S, E>,
    >(&mut self) -> Result<Option<(K, V)>, E> {
        match try!(self.visit_key()) {
            Some(key) => {
                let value = try!(self.visit_value());
                Ok(Some((key, value)))
            }
            None => Ok(None)
        }
    }

    fn visit_key<
        K: Deserialize<S, E>,
    >(&mut self) -> Result<Option<K>, E>;

    fn visit_value<
        V: Deserialize<S, E>,
    >(&mut self) -> Result<V, E>;

    fn end(&mut self) -> Result<(), E>;

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        (0, None)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<
    S: Deserializer<E>,
    E: Error,
> Deserialize<S, E> for () {
    fn deserialize(state: &mut S) -> Result<(), E> {
        struct Visitor;

        impl<
            S: Deserializer<E>,
            E: Error,
        > self::Visitor<S, (), E> for Visitor {
            fn visit_null(&mut self) -> Result<(), E> {
                Ok(())
            }

            fn visit_seq<
                V: SeqVisitor<S, E>,
            >(&mut self, mut visitor: V) -> Result<(), E> {
                try!(visitor.end());
                Ok(())
            }
        }

        state.visit(&mut Visitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<
    S: Deserializer<E>,
    E: Error,
> Deserialize<S, E> for bool {
    fn deserialize(state: &mut S) -> Result<bool, E> {
        struct Visitor;

        impl<
            S: Deserializer<E>,
            E: Error,
        > self::Visitor<S, bool, E> for Visitor {
            fn visit_bool(&mut self, v: bool) -> Result<bool, E> {
                Ok(v)
            }
        }

        state.visit(&mut Visitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

macro_rules! impl_deserialize_num_method {
    ($src_ty:ty, $method:ident, $from_method:ident) => {
        fn $method(&mut self, v: $src_ty) -> Result<T, E> {
            match FromPrimitive::$from_method(v) {
                Some(v) => Ok(v),
                None => Err(Error::syntax_error()),
            }
        }
    }
}

#[inline]
pub fn deserialize_from_primitive<
    S: Deserializer<E>,
    E: Error,
    T: Deserialize<S, E> + FromPrimitive
>(state: &mut S) -> Result<T, E> {
    struct Visitor;

    impl<
        S: Deserializer<E>,
        E: Error,
        T: Deserialize<S, E> + FromPrimitive
    > self::Visitor<S, T, E> for Visitor {
        impl_deserialize_num_method!(int, visit_int, from_int)
        impl_deserialize_num_method!(i8, visit_i8, from_i8)
        impl_deserialize_num_method!(i16, visit_i16, from_i16)
        impl_deserialize_num_method!(i32, visit_i32, from_i32)
        impl_deserialize_num_method!(i64, visit_i64, from_i64)
        impl_deserialize_num_method!(uint, visit_uint, from_uint)
        impl_deserialize_num_method!(u8, visit_u8, from_u8)
        impl_deserialize_num_method!(u16, visit_u16, from_u16)
        impl_deserialize_num_method!(u32, visit_u32, from_u32)
        impl_deserialize_num_method!(u64, visit_u64, from_u64)
        impl_deserialize_num_method!(f32, visit_f32, from_f32)
        impl_deserialize_num_method!(f64, visit_f64, from_f64)
    }

    state.visit(&mut Visitor)
}

macro_rules! impl_deserialize_num {
    ($ty:ty) => {
        impl<
            S: Deserializer<E>,
            E: Error,
        > Deserialize<S, E> for $ty {
            #[inline]
            fn deserialize(state: &mut S) -> Result<$ty, E> {
                deserialize_from_primitive(state)
            }
        }
    }
}

impl_deserialize_num!(int)
impl_deserialize_num!(i8)
impl_deserialize_num!(i16)
impl_deserialize_num!(i32)
impl_deserialize_num!(i64)
impl_deserialize_num!(uint)
impl_deserialize_num!(u8)
impl_deserialize_num!(u16)
impl_deserialize_num!(u32)
impl_deserialize_num!(u64)
impl_deserialize_num!(f32)
impl_deserialize_num!(f64)

///////////////////////////////////////////////////////////////////////////////

impl<
    S: Deserializer<E>,
    E: Error,
> Deserialize<S, E> for String {
    fn deserialize(state: &mut S) -> Result<String, E> {
        struct Visitor;

        impl<
            S: Deserializer<E>,
            E: Error,
        > self::Visitor<S, String, E> for Visitor {
            fn visit_str(&mut self, v: &str) -> Result<String, E> {
                Ok(v.to_string())
            }

            fn visit_string(&mut self, v: String) -> Result<String, E> {
                Ok(v)
            }
        }

        state.visit(&mut Visitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<
    T: Deserialize<S, E>,
    S: Deserializer<E>,
    E: Error,
> Deserialize<S, E> for Option<T> {
    fn deserialize(state: &mut S) -> Result<Option<T>, E> {
        struct Visitor;

        impl<
            T: Deserialize<S, E>,
            S: Deserializer<E>,
            E: Error,
        > self::Visitor<S, Option<T>, E> for Visitor {
            fn visit_option<
                V: OptionVisitor<S, E>,
            >(&mut self, mut visitor: V) -> Result<Option<T>, E> {
                visitor.visit()
            }
        }

        state.visit_option(&mut Visitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<
    T: Deserialize<S, E>,
    S: Deserializer<E>,
    E: Error,
> Deserialize<S, E> for Vec<T> {
    fn deserialize(state: &mut S) -> Result<Vec<T>, E> {
        struct Visitor;

        impl<
            T: Deserialize<S, E>,
            S: Deserializer<E>,
            E: Error,
        > self::Visitor<S, Vec<T>, E> for Visitor {
            fn visit_seq<
                V: SeqVisitor<S, E>,
            >(&mut self, mut visitor: V) -> Result<Vec<T>, E> {
                let (len, _) = visitor.size_hint();
                let mut values = Vec::with_capacity(len);

                loop {
                    match try!(visitor.visit()) {
                        Some(value) => {
                            values.push(value);
                        }
                        None => {
                            break;
                        }
                    }
                }

                Ok(values)
            }
        }

        state.visit(&mut Visitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

macro_rules! peel {
    ($name:ident, $($other:ident,)*) => {
        impl_deserialize_tuple!($($other,)*)
    }
}

macro_rules! impl_deserialize_tuple {
    () => {};
    ( $($name:ident,)+ ) => {
        peel!($($name,)*)

        impl<
            S: Deserializer<E>,
            E: Error,
            $($name: Deserialize<S, E>),+
        > Deserialize<S, E> for ($($name,)+) {
            #[inline]
            #[allow(non_snake_case)]
            fn deserialize(state: &mut S) -> Result<($($name,)+), E> {
                struct Visitor;

                impl<
                    S: Deserializer<E>,
                    E: Error,
                    $($name: Deserialize<S, E>,)+
                > self::Visitor<S, ($($name,)+), E> for Visitor {
                    fn visit_seq<
                        V: SeqVisitor<S, E>,
                    >(&mut self, mut visitor: V) -> Result<($($name,)+), E> {
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

                state.visit(&mut Visitor)
            }
        }
    }
}

impl_deserialize_tuple! { T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, }

///////////////////////////////////////////////////////////////////////////////

impl<
    K: Deserialize<S, E> + Eq + Hash,
    V: Deserialize<S, E>,
    S: Deserializer<E>,
    E: Error,
> Deserialize<S, E> for HashMap<K, V> {
    fn deserialize(state: &mut S) -> Result<HashMap<K, V>, E> {
        struct Visitor;

        impl<
            K: Deserialize<S, E> + Eq + Hash,
            V: Deserialize<S, E>,
            S: Deserializer<E>,
            E: Error,
        > self::Visitor<S, HashMap<K, V>, E> for Visitor {
            fn visit_map<
                Visitor: MapVisitor<S, E>,
            >(&mut self, mut visitor: Visitor) -> Result<HashMap<K, V>, E> {
                let (len, _) = visitor.size_hint();
                let mut values = HashMap::with_capacity(len);

                loop {
                    match try!(visitor.visit()) {
                        Some((key, value)) => {
                            values.insert(key, value);
                        }
                        None => {
                            break;
                        }
                    }
                }

                Ok(values)
            }
        }

        state.visit(&mut Visitor)
    }
}

impl<
    K: Deserialize<S, E> + Eq + Ord,
    V: Deserialize<S, E>,
    S: Deserializer<E>,
    E: Error,
> Deserialize<S, E> for TreeMap<K, V> {
    fn deserialize(state: &mut S) -> Result<TreeMap<K, V>, E> {
        struct Visitor;

        impl<
            K: Deserialize<S, E> + Eq + Ord,
            V: Deserialize<S, E>,
            S: Deserializer<E>,
            E: Error,
        > self::Visitor<S, TreeMap<K, V>, E> for Visitor {
            fn visit_map<
                Visitor: MapVisitor<S, E>,
            >(&mut self, mut visitor: Visitor) -> Result<TreeMap<K, V>, E> {
                let mut values = TreeMap::new();

                loop {
                    match try!(visitor.visit()) {
                        Some((key, value)) => {
                            values.insert(key, value);
                        }
                        None => {
                            break;
                        }
                    }
                }

                Ok(values)
            }
        }

        state.visit(&mut Visitor)
    }
}
