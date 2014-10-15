use std::collections::{HashMap, TreeMap};
use std::hash::Hash;
use std::num;

///////////////////////////////////////////////////////////////////////////////

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

    fn syntax_error(&mut self) -> E;

    fn end_of_stream_error(&mut self) -> E;
}

pub trait Visitor<S: Deserializer<E>, R, E> {
    fn visit_null(&mut self, state: &mut S) -> Result<R, E> {
        Err(state.syntax_error())
    }

    fn visit_bool(&mut self, state: &mut S, _v: bool) -> Result<R, E> {
        Err(state.syntax_error())
    }

    fn visit_int(&mut self, state: &mut S, v: int) -> Result<R, E> {
        self.visit_i64(state, v as i64)
    }

    fn visit_i64(&mut self, state: &mut S, _v: i64) -> Result<R, E> {
        Err(state.syntax_error())
    }

    fn visit_uint(&mut self, state: &mut S, v: uint) -> Result<R, E> {
        self.visit_u64(state, v as u64)
    }

    fn visit_u64(&mut self, state: &mut S, _v: u64) -> Result<R, E> {
        Err(state.syntax_error())
    }

    fn visit_f32(&mut self, state: &mut S, v: f32) -> Result<R, E> {
        self.visit_f64(state, v as f64)
    }

    fn visit_f64(&mut self, state: &mut S, _v: f64) -> Result<R, E> {
        Err(state.syntax_error())
    }

    fn visit_str(&mut self, state: &mut S, _v: &str) -> Result<R, E> {
        Err(state.syntax_error())
    }

    fn visit_string(&mut self, state: &mut S, v: String) -> Result<R, E> {
        self.visit_str(state, v.as_slice())
    }

    fn visit_option<
        V: OptionVisitor<S, E>,
    >(&mut self, state: &mut S, _visitor: V) -> Result<R, E> {
        Err(state.syntax_error())
    }

    fn visit_seq<
        V: SeqVisitor<S, E>,
    >(&mut self, state: &mut S, _visitor: V) -> Result<R, E> {
        Err(state.syntax_error())
    }

    fn visit_map<
        V: MapVisitor<S, E>,
    >(&mut self, state: &mut S, _visitor: V) -> Result<R, E> {
        Err(state.syntax_error())
    }
}

pub trait OptionVisitor<S, E> {
    fn visit<
        T: Deserialize<S, E>,
    >(&mut self, state: &mut S) -> Result<Option<T>, E>;
}

pub trait SeqVisitor<S, E> {
    fn visit<
        T: Deserialize<S, E>,
    >(&mut self, state: &mut S) -> Result<Option<T>, E>;

    fn end(&mut self, state: &mut S) -> Result<(), E>;

    #[inline]
    fn size_hint(&self, _state: &mut S) -> (uint, Option<uint>) {
        (0, None)
    }
}

pub trait MapVisitor<S, E> {
    fn visit<
        K: Deserialize<S, E>,
        V: Deserialize<S, E>,
    >(&mut self, state: &mut S) -> Result<Option<(K, V)>, E>;

    fn end(&mut self, state: &mut S) -> Result<(), E>;

    #[inline]
    fn size_hint(&self, _state: &mut S) -> (uint, Option<uint>) {
        (0, None)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<
    S: Deserializer<E>,
    E,
> Deserialize<S, E> for () {
    fn deserialize(state: &mut S) -> Result<(), E> {
        struct Visitor;

        impl<S: Deserializer<E>, E> self::Visitor<S, (), E> for Visitor {
            fn visit_null(&mut self, _state: &mut S) -> Result<(), E> {
                Ok(())
            }

            fn visit_seq<
                V: SeqVisitor<S, E>,
            >(&mut self, state: &mut S, mut visitor: V) -> Result<(), E> {
                try!(visitor.end(state));
                Ok(())
            }
        }

        state.visit(&mut Visitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<
    S: Deserializer<E>,
    E,
> Deserialize<S, E> for bool {
    fn deserialize(state: &mut S) -> Result<bool, E> {
        struct Visitor;

        impl<S: Deserializer<E>, E> self::Visitor<S, bool, E> for Visitor {
            fn visit_bool(&mut self, _state: &mut S, v: bool) -> Result<bool, E> {
                Ok(v)
            }
        }

        state.visit(&mut Visitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

macro_rules! impl_deserialize_num_method {
    ($dst_ty:ty, $src_ty:ty, $method:ident) => {
        fn $method(&mut self, state: &mut S, v: $src_ty) -> Result<$dst_ty, E> {
            match num::cast(v) {
                Some(v) => Ok(v),
                None => Err(state.syntax_error()),
            }
        }
    }
}

macro_rules! impl_deserialize_num {
    ($ty:ty) => {
        impl<S: Deserializer<E>, E> Deserialize<S, E> for $ty {
            #[inline]
            fn deserialize(state: &mut S) -> Result<$ty, E> {
                struct Visitor;

                impl<S: Deserializer<E>, E> self::Visitor<S, $ty, E> for Visitor {
                    impl_deserialize_num_method!($ty, int, visit_int)
                    impl_deserialize_num_method!($ty, i64, visit_i64)
                    impl_deserialize_num_method!($ty, uint, visit_uint)
                    impl_deserialize_num_method!($ty, u64, visit_u64)
                    impl_deserialize_num_method!($ty, f32, visit_f32)
                    impl_deserialize_num_method!($ty, f64, visit_f64)
                }

                state.visit(&mut Visitor)
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
    S: Deserializer<E>,
    E,
> Deserialize<S, E> for String {
    fn deserialize(state: &mut S) -> Result<String, E> {
        struct Visitor;

        impl<S: Deserializer<E>, E> self::Visitor<S, String, E> for Visitor {
            fn visit_str(&mut self, _state: &mut S, v: &str) -> Result<String, E> {
                Ok(v.to_string())
            }

            fn visit_string(&mut self, _state: &mut S, v: String) -> Result<String, E> {
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
    E,
> Deserialize<S, E> for Option<T> {
    fn deserialize(state: &mut S) -> Result<Option<T>, E> {
        struct Visitor;

        impl<
            T: Deserialize<S, E>,
            S: Deserializer<E>,
            E,
        > self::Visitor<S, Option<T>, E> for Visitor {
            fn visit_option<
                V: OptionVisitor<S, E>,
            >(&mut self, state: &mut S, mut visitor: V) -> Result<Option<T>, E> {
                visitor.visit(state)
            }
        }

        state.visit_option(&mut Visitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<
    T: Deserialize<S, E>,
    S: Deserializer<E>,
    E,
> Deserialize<S, E> for Vec<T> {
    fn deserialize(state: &mut S) -> Result<Vec<T>, E> {
        struct Visitor;

        impl<
            T: Deserialize<S, E>,
            S: Deserializer<E>,
            E,
        > self::Visitor<S, Vec<T>, E> for Visitor {
            fn visit_seq<
                V: SeqVisitor<S, E>,
            >(&mut self, state: &mut S, mut visitor: V) -> Result<Vec<T>, E> {
                let (len, _) = visitor.size_hint(state);
                let mut values = Vec::with_capacity(len);

                loop {
                    match try!(visitor.visit(state)) {
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
            E,
            $($name: Deserialize<S, E>),+
        > Deserialize<S, E> for ($($name,)+) {
            #[inline]
            #[allow(non_snake_case)]
            fn deserialize(state: &mut S) -> Result<($($name,)+), E> {
                struct Visitor;

                impl<
                    S: Deserializer<E>,
                    E,
                    $($name: Deserialize<S, E>,)+
                > self::Visitor<S, ($($name,)+), E> for Visitor {
                    fn visit_seq<
                        V: SeqVisitor<S, E>,
                    >(&mut self, state: &mut S, mut visitor: V) -> Result<($($name,)+), E> {
                        $(
                            let $name = match try!(visitor.visit(state)) {
                                Some(value) => value,
                                None => { return Err(state.end_of_stream_error()); }
                            };
                         )+;

                        try!(visitor.end(state));

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
    E,
> Deserialize<S, E> for HashMap<K, V> {
    fn deserialize(state: &mut S) -> Result<HashMap<K, V>, E> {
        struct Visitor;

        impl<
            K: Deserialize<S, E> + Eq + Hash,
            V: Deserialize<S, E>,
            S: Deserializer<E>,
            E,
        > self::Visitor<S, HashMap<K, V>, E> for Visitor {
            fn visit_map<
                Visitor: MapVisitor<S, E>,
            >(&mut self, state: &mut S, mut visitor: Visitor) -> Result<HashMap<K, V>, E> {
                let (len, _) = visitor.size_hint(state);
                let mut values = HashMap::with_capacity(len);

                loop {
                    match try!(visitor.visit(state)) {
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
    E,
> Deserialize<S, E> for TreeMap<K, V> {
    fn deserialize(state: &mut S) -> Result<TreeMap<K, V>, E> {
        struct Visitor;

        impl<
            K: Deserialize<S, E> + Eq + Ord,
            V: Deserialize<S, E>,
            S: Deserializer<E>,
            E,
        > self::Visitor<S, TreeMap<K, V>, E> for Visitor {
            fn visit_map<
                Visitor: MapVisitor<S, E>,
            >(&mut self, state: &mut S, mut visitor: Visitor) -> Result<TreeMap<K, V>, E> {
                let mut values = TreeMap::new();

                loop {
                    match try!(visitor.visit(state)) {
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
