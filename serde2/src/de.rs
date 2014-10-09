use std::collections::{HashMap, TreeMap};
use std::hash::Hash;
use std::num;

///////////////////////////////////////////////////////////////////////////////

pub trait Deserialize<D, E> {
    fn deserialize(d: &mut D) -> Result<Self, E>;
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

    fn syntax_error(&mut self) -> E;

    fn end_of_stream_error(&mut self) -> E;
}

pub trait Visitor<D: Deserializer<E>, R, E> {
    fn visit_null(&mut self, d: &mut D) -> Result<R, E> {
        Err(d.syntax_error())
    }

    fn visit_bool(&mut self, d: &mut D, _v: bool) -> Result<R, E> {
        Err(d.syntax_error())
    }

    fn visit_int(&mut self, d: &mut D, v: int) -> Result<R, E> {
        self.visit_i64(d, v as i64)
    }

    fn visit_i64(&mut self, d: &mut D, _v: i64) -> Result<R, E> {
        Err(d.syntax_error())
    }

    fn visit_uint(&mut self, d: &mut D, v: uint) -> Result<R, E> {
        self.visit_u64(d, v as u64)
    }

    fn visit_u64(&mut self, d: &mut D, _v: u64) -> Result<R, E> {
        Err(d.syntax_error())
    }

    fn visit_f32(&mut self, d: &mut D, v: f32) -> Result<R, E> {
        self.visit_f64(d, v as f64)
    }

    fn visit_f64(&mut self, d: &mut D, _v: f64) -> Result<R, E> {
        Err(d.syntax_error())
    }

    fn visit_str(&mut self, d: &mut D, _v: &str) -> Result<R, E> {
        Err(d.syntax_error())
    }

    fn visit_string(&mut self, d: &mut D, v: String) -> Result<R, E> {
        self.visit_str(d, v.as_slice())
    }

    fn visit_option<
        V: OptionVisitor<D, E>,
    >(&mut self, d: &mut D, _visitor: V) -> Result<R, E> {
        Err(d.syntax_error())
    }

    fn visit_seq<
        V: SeqVisitor<D, E>,
    >(&mut self, d: &mut D, _visitor: V) -> Result<R, E> {
        Err(d.syntax_error())
    }

    fn visit_map<
        V: MapVisitor<D, E>,
    >(&mut self, d: &mut D, _visitor: V) -> Result<R, E> {
        Err(d.syntax_error())
    }
}

pub trait OptionVisitor<D, E> {
    fn visit<
        T: Deserialize<D, E>,
    >(&mut self, d: &mut D) -> Result<Option<T>, E>;
}

pub trait SeqVisitor<D, E> {
    fn visit<
        T: Deserialize<D, E>,
    >(&mut self, d: &mut D) -> Result<Option<T>, E>;

    fn end(&mut self, d: &mut D) -> Result<(), E>;

    #[inline]
    fn size_hint(&self, _d: &mut D) -> (uint, Option<uint>) {
        (0, None)
    }
}

pub trait MapVisitor<D, E> {
    fn visit<
        K: Deserialize<D, E>,
        V: Deserialize<D, E>,
    >(&mut self, d: &mut D) -> Result<Option<(K, V)>, E>;

    fn end(&mut self, d: &mut D) -> Result<(), E>;

    #[inline]
    fn size_hint(&self, _d: &mut D) -> (uint, Option<uint>) {
        (0, None)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<
    D: Deserializer<E>,
    E,
> Deserialize<D, E> for () {
    fn deserialize(d: &mut D) -> Result<(), E> {
        struct Visitor;

        impl<D: Deserializer<E>, E> self::Visitor<D, (), E> for Visitor {
            fn visit_null(&mut self, _d: &mut D) -> Result<(), E> {
                Ok(())
            }

            fn visit_seq<
                V: SeqVisitor<D, E>,
            >(&mut self, d: &mut D, mut visitor: V) -> Result<(), E> {
                try!(visitor.end(d));
                Ok(())
            }
        }

        d.visit(&mut Visitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<
    D: Deserializer<E>,
    E,
> Deserialize<D, E> for bool {
    fn deserialize(d: &mut D) -> Result<bool, E> {
        struct Visitor;

        impl<D: Deserializer<E>, E> self::Visitor<D, bool, E> for Visitor {
            fn visit_bool(&mut self, _d: &mut D, v: bool) -> Result<bool, E> {
                Ok(v)
            }
        }

        d.visit(&mut Visitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

macro_rules! impl_deserialize_num_method {
    ($dst_ty:ty, $src_ty:ty, $method:ident) => {
        fn $method(&mut self, d: &mut D, v: $src_ty) -> Result<$dst_ty, E> {
            match num::cast(v) {
                Some(v) => Ok(v),
                None => Err(d.syntax_error()),
            }
        }
    }
}

macro_rules! impl_deserialize_num {
    ($ty:ty) => {
        impl<D: Deserializer<E>, E> Deserialize<D, E> for $ty {
            #[inline]
            fn deserialize(d: &mut D) -> Result<$ty, E> {
                struct Visitor;

                impl<D: Deserializer<E>, E> self::Visitor<D, $ty, E> for Visitor {
                    impl_deserialize_num_method!($ty, int, visit_int)
                    impl_deserialize_num_method!($ty, i64, visit_i64)
                    impl_deserialize_num_method!($ty, uint, visit_uint)
                    impl_deserialize_num_method!($ty, u64, visit_u64)
                    impl_deserialize_num_method!($ty, f32, visit_f32)
                    impl_deserialize_num_method!($ty, f64, visit_f64)
                }

                d.visit(&mut Visitor)
            }
        }
    }
}

impl_deserialize_num!(int)
impl_deserialize_num!(i64)
impl_deserialize_num!(uint)
impl_deserialize_num!(u64)
impl_deserialize_num!(f32)
impl_deserialize_num!(f64)

///////////////////////////////////////////////////////////////////////////////

impl<
    D: Deserializer<E>,
    E,
> Deserialize<D, E> for String {
    fn deserialize(d: &mut D) -> Result<String, E> {
        struct Visitor;

        impl<D: Deserializer<E>, E> self::Visitor<D, String, E> for Visitor {
            fn visit_str(&mut self, _d: &mut D, v: &str) -> Result<String, E> {
                Ok(v.to_string())
            }

            fn visit_string(&mut self, _d: &mut D, v: String) -> Result<String, E> {
                Ok(v)
            }
        }

        d.visit(&mut Visitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<
    T: Deserialize<D, E>,
    D: Deserializer<E>,
    E,
> Deserialize<D, E> for Option<T> {
    fn deserialize(d: &mut D) -> Result<Option<T>, E> {
        struct Visitor;

        impl<
            T: Deserialize<D, E>,
            D: Deserializer<E>,
            E,
        > self::Visitor<D, Option<T>, E> for Visitor {
            fn visit_option<
                V: OptionVisitor<D, E>,
            >(&mut self, d: &mut D, mut visitor: V) -> Result<Option<T>, E> {
                visitor.visit(d)
            }
        }

        d.visit_option(&mut Visitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<
    T: Deserialize<D, E>,
    D: Deserializer<E>,
    E,
> Deserialize<D, E> for Vec<T> {
    fn deserialize(d: &mut D) -> Result<Vec<T>, E> {
        struct Visitor;

        impl<
            T: Deserialize<D, E>,
            D: Deserializer<E>,
            E,
        > self::Visitor<D, Vec<T>, E> for Visitor {
            fn visit_seq<
                V: SeqVisitor<D, E>,
            >(&mut self, d: &mut D, mut visitor: V) -> Result<Vec<T>, E> {
                let (len, _) = visitor.size_hint(d);
                let mut values = Vec::with_capacity(len);

                loop {
                    match try!(visitor.visit(d)) {
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

        d.visit(&mut Visitor)
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
            D: Deserializer<E>,
            E,
            $($name: Deserialize<D, E>),+
        > Deserialize<D, E> for ($($name,)+) {
            #[inline]
            #[allow(non_snake_case)]
            fn deserialize(d: &mut D) -> Result<($($name,)+), E> {
                struct Visitor;

                impl<
                    D: Deserializer<E>,
                    E,
                    $($name: Deserialize<D, E>,)+
                > self::Visitor<D, ($($name,)+), E> for Visitor {
                    fn visit_seq<
                        V: SeqVisitor<D, E>,
                    >(&mut self, d: &mut D, mut visitor: V) -> Result<($($name,)+), E> {
                        $(
                            let $name = match try!(visitor.visit(d)) {
                                Some(value) => value,
                                None => { return Err(d.end_of_stream_error()); }
                            };
                         )+;

                        try!(visitor.end(d));

                        Ok(($($name,)+))
                    }
                }

                d.visit(&mut Visitor)
            }
        }
    }
}

impl_deserialize_tuple! { T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, }

///////////////////////////////////////////////////////////////////////////////

impl<
    K: Deserialize<D, E> + Eq + Hash,
    V: Deserialize<D, E>,
    D: Deserializer<E>,
    E,
> Deserialize<D, E> for HashMap<K, V> {
    fn deserialize(d: &mut D) -> Result<HashMap<K, V>, E> {
        struct Visitor;

        impl<
            K: Deserialize<D, E> + Eq + Hash,
            V: Deserialize<D, E>,
            D: Deserializer<E>,
            E,
        > self::Visitor<D, HashMap<K, V>, E> for Visitor {
            fn visit_map<
                Visitor: MapVisitor<D, E>,
            >(&mut self, d: &mut D, mut visitor: Visitor) -> Result<HashMap<K, V>, E> {
                let (len, _) = visitor.size_hint(d);
                let mut values = HashMap::with_capacity(len);

                loop {
                    match try!(visitor.visit(d)) {
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

        d.visit(&mut Visitor)
    }
}

impl<
    K: Deserialize<D, E> + Eq + Ord,
    V: Deserialize<D, E>,
    D: Deserializer<E>,
    E,
> Deserialize<D, E> for TreeMap<K, V> {
    fn deserialize(d: &mut D) -> Result<TreeMap<K, V>, E> {
        struct Visitor;

        impl<
            K: Deserialize<D, E> + Eq + Ord,
            V: Deserialize<D, E>,
            D: Deserializer<E>,
            E,
        > self::Visitor<D, TreeMap<K, V>, E> for Visitor {
            fn visit_map<
                Visitor: MapVisitor<D, E>,
            >(&mut self, d: &mut D, mut visitor: Visitor) -> Result<TreeMap<K, V>, E> {
                let mut values = TreeMap::new();

                loop {
                    match try!(visitor.visit(d)) {
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

        d.visit(&mut Visitor)
    }
}
