use std::collections::{HashMap, TreeMap};
use std::hash::Hash;

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

    fn visit_int(&mut self, d: &mut D, _v: int) -> Result<R, E> {
        Err(d.syntax_error())
    }

    fn visit_str(&mut self, d: &mut D, v: &str) -> Result<R, E> {
        self.visit_string(d, v.to_string())
    }

    fn visit_string(&mut self, d: &mut D, _v: String) -> Result<R, E> {
        Err(d.syntax_error())
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
    fn next<
        T: Deserialize<D, E>,
    >(&mut self, d: &mut D) -> Option<Result<T, E>>;

    fn end(&mut self, d: &mut D) -> Result<(), E>;

    #[inline]
    fn size_hint(&self, _d: &mut D) -> (uint, Option<uint>) {
        (0, None)
    }
}

pub trait MapVisitor<D, E> {
    fn next<
        K: Deserialize<D, E>,
        V: Deserialize<D, E>,
    >(&mut self, d: &mut D) -> Option<Result<(K, V), E>>;

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
        }

        d.visit(&mut Visitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<
    D: Deserializer<E>,
    E,
> Deserialize<D, E> for int {
    fn deserialize(d: &mut D) -> Result<int, E> {
        struct Visitor;

        impl<D: Deserializer<E>, E> self::Visitor<D, int, E> for Visitor {
            fn visit_int(&mut self, _d: &mut D, v: int) -> Result<int, E> {
                Ok(v)
            }
        }

        d.visit(&mut Visitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<
    D: Deserializer<E>,
    E,
> Deserialize<D, E> for String {
    fn deserialize(d: &mut D) -> Result<String, E> {
        struct Visitor;

        impl<D: Deserializer<E>, E> self::Visitor<D, String, E> for Visitor {
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
                    match visitor.next(d) {
                        Some(value) => {
                            values.push(try!(value));
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
    T0: Deserialize<D, E>,
    T1: Deserialize<D, E>,
    D: Deserializer<E>,
    E,
> Deserialize<D, E> for (T0, T1) {
    fn deserialize(d: &mut D) -> Result<(T0, T1), E> {
        struct Visitor;

        impl<
            T0: Deserialize<D, E>,
            T1: Deserialize<D, E>,
            D: Deserializer<E>,
            E,
        > self::Visitor<D, (T0, T1), E> for Visitor {
            fn visit_seq<
                V: SeqVisitor<D, E>,
            >(&mut self, d: &mut D, mut visitor: V) -> Result<(T0, T1), E> {
                let mut state = 0u;
                let mut t0 = None;
                let mut t1 = None;

                loop {
                    match state {
                        0 => {
                            state += 1;
                            match visitor.next(d) {
                                Some(value) => {
                                    t0 = Some(try!(value));
                                }
                                None => {
                                    return Err(d.end_of_stream_error());
                                }
                            }
                        }
                        1 => {
                            state += 1;
                            match visitor.next(d) {
                                Some(value) => {
                                    t1 = Some(try!(value));
                                }
                                None => {
                                    return Err(d.end_of_stream_error());
                                }
                            }
                        }
                        _ => {
                            try!(visitor.end(d));

                            return Ok((t0.unwrap(), t1.unwrap()));
                        }
                    }
                }
            }
        }

        d.visit(&mut Visitor)
    }
}

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
                    match visitor.next(d) {
                        Some(Ok((key, value))) => {
                            values.insert(key, value);
                        }
                        Some(Err(err)) => {
                            return Err(err);
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
                    match visitor.next(d) {
                        Some(Ok((key, value))) => {
                            values.insert(key, value);
                        }
                        Some(Err(err)) => {
                            return Err(err);
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

/*
trait Deserialize<S, E> {
    fn deserialize(state: &mut S) -> Result<Self, E>;
}

///////////////////////////////////////////////////////////////////////////////

trait Deserializer<S, E> {
    fn visit<R>(&mut self, state: &mut S) -> Result<(), E>;

    fn syntax_error(&mut self) -> E;
}

///////////////////////////////////////////////////////////////////////////////

/*
trait DeserializerState<E> {
    fn syntax_error(&mut self) -> E;

    fn visit<
        V: VisitorState<T, Self, E>,
        T: Deserialize<Self, E>,
    >(&mut self, visitor: &mut V) -> Result<T, E>;
}

trait Visitor<R, E> {
    fn visit<S>(&mut self, state: &mut S) -> Result<(), E>;

    fn size_hint(&self) -> (uint, Option<uint>);
}
*/

trait VisitorState<
    D: Deserializer<Self, E>,
    //R,
    E,
> {
    /*
    fn visit_null(&mut self) -> R;
    */

    fn visit_int(&mut self, d: &mut D, v: int) -> Result<int, E> {
        Err(d.syntax_error())
        //self.visit_i64(d, v as i64)
    }

    /*
    fn visit_i8(&mut self, d: &mut D, v: i8) -> Result<(), E> {
        self.visit_i64(d, v as i64)
    }

    fn visit_i16(&mut self, d: &mut D, v: i16) -> Result<(), E> {
        self.visit_i64(d, v as i64)
    }

    fn visit_i32(&mut self, d: &mut D, v: i32) -> Result<(), E> {
        self.visit_i64(d, v as i64)
    }

    fn visit_i64(&mut self, d: &mut D, v: i64) -> Result<(), E> {
        Err(d.syntax_error())
    }

    fn visit_uint(&mut self, d: &mut D, v: int) -> Result<(), E> {
        self.visit_u64(d, v as u64)
    }

    fn visit_u8(&mut self, d: &mut D, v: u8) -> Result<(), E> {
        self.visit_u64(d, v as u64)
    }

    fn visit_u16(&mut self, d: &mut D, v: u16) -> Result<(), E> {
        self.visit_u64(d, v as u64)
    }

    fn visit_u32(&mut self, d: &mut D, v: u32) -> Result<(), E> {
        self.visit_u64(d, v as u64)
    }

    fn visit_u64(&mut self, d: &mut D, _v: u64) -> Result<(), E> {
        Err(d.syntax_error())
    }

    fn visit_string(&mut self, d: &mut D, _v: String) -> Result<(), E> {
        Err(d.syntax_error())
    }
    */

    fn visit_seq<
        R,
        V: SeqVisitor<D, E>,
    >(&mut self, d: &mut D, _visitor: V) -> Result<(), E> {
        Err(d.syntax_error())
    }

    /*
    /*
    #[inline]
    fn visit_named_seq<
        Elt: Deserialize<D, E>,
        V: SeqVisitor<D, Result<Elt, E>>,
    >(&mut self, d: &mut D, _name: &str, visitor: V) -> Result<T, E> {
        self.visit_seq(d, visitor)
    }
    */

    fn visit_seq_elt<
        T: Deserialize<Self, R>,
    >(&mut self, first: bool, value: T) -> R;
    */

    /*
    #[inline]
    fn visit_map<
        K: Deserialize<D, E>,
        V: Deserialize<D, E>,
        V: Visitor<D, Result<T, E>>,
    >(&mut self, d: &mut D, _visitor: V) -> Result<T, E> {
        Err(d.syntax_error())
    }

    #[inline]
    fn visit_named_map<
        V: Visitor<D, E>,
    >(&mut self, d: &mut D, _name: &str, visitor: V) -> Result<T, E> {
        self.visit_map(d, visitor)
    }

    fn visit_map_elt<
        K: Deserialize<D, E>,
        V: Deserialize<D, E>,
    >(&mut self, first: bool, value: T) -> Result<(K, V), E>;
    */
}

trait SeqVisitor<D, E> {
    fn next<
        T: Deserialize<D, E>,
    >(&mut self, d: &mut D) -> Option<Result<T, E>>;

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        (0, None)
    }
}

/*
trait MapVisitor<D, E> {
    fn next<
        K: Deserialize<D, E>,
        V: Deserialize<D, E>,
    >(&mut self, d: &mut D) -> Option<Result<(K, V), E>>;

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        (0, None)
    }
}
*/

///////////////////////////////////////////////////////////////////////////////

/*
macro_rules! impl_deserialize {
    ($ty:ty, $method:ident) => {
        impl<
            S: Deserializer<E>,
            E,
        > Deserialize<S, E> for $ty {
            #[inline]
            fn deserialize(state: &mut S) -> Result<$ty, E> {
                struct Visitor;

                impl<
                    D: Deserializer<$ty, E>,
                    E,
                > VisitorState<S, $ty, E> for Visitor {
                    fn visit_int(&mut self, _d: &mut D, v: int) -> Result<$ty, E> {
                        Ok(v)
                    }
                }

                state.visit(&mut Visitor)


                d.$method(token)
            }
        }
    }
}
*/

impl<
    D: Deserializer<int, E>,
    E,
> Deserialize<D, E> for int {
    fn deserialize(state: &mut D) -> Result<int, E> {
        struct Visitor;

        impl<
            D: Deserializer<Visitor, E>,
            E,
        > VisitorState<D, E> for Visitor {
            fn visit_int(&mut self, _state: &mut D, v: int) -> Result<int, E> {
                Ok(v)
            }
        }

        state.visit(&mut Visitor)
    }
}

/*
impl<
    S: Deserializer<String, E>,
    E,
> Deserialize<S, E> for String {
    fn deserialize(state: &mut S) -> Result<String, E> {
        struct Visitor;

        impl<
            D: Deserializer<String, E>,
            E,
        > VisitorState<S, String, E> for Visitor {
            fn visit_string(&mut self, _d: &mut D, v: String) -> Result<String, E> {
                Ok(v)
            }
        }

        state.visit(&mut Visitor)
    }
}
*/

///////////////////////////////////////////////////////////////////////////////

/*
impl<
    T: Deserialize<D, E>,
    D: Deserializer<Vec<T>, E>,
    E,
> Deserialize<D, E> for Vec<T> {
    fn deserialize(d: &mut D) -> Result<Vec<T>, E> {
        struct Visitor;

        impl<
            D: Deserializer<Visitor, E>,
            T: Deserialize<D, E>,
            E,
        > VisitorState<D, E> for Visitor {
            fn visit_seq<
                V: SeqVisitor<D, E>,
            >(&mut self, state: &mut D, mut visitor: V) -> Result<Vec<T>, E> {
                let (len, _) = visitor.size_hint();
                let mut values = Vec::with_capacity(len);

                loop {
                    match visitor.next(state) {
                        Some(Ok(value)) => {
                            values.push(value);
                        }
                        Some(Err(err)) => {
                            return Err(err);
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
*/

/*
///////////////////////////////////////////////////////////////////////////////

impl<
    D: DeserializerState<E>,
    E
> Deserialize<D, E> for () {
    fn deserialize(d: &mut D) -> Result<(), E> {
        struct Visitor;

        impl<
            D: DeserializerState<E>,
            E,
        > ::VisitorState<(), D, E> for Visitor {
            fn visit_null(&mut self, _d: &mut D) -> Result<(), E> {
                Ok(())
            }
        }

        d.visit(&mut Visitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<
    T0: Deserialize<D, E>,
    T1: Deserialize<D, E>,
    D: DeserializerState<E>,
    E
> Deserialize<D, E> for (T0, T1) {
    fn deserialize(d: &mut D) -> Result<(T0, T1), E> {
        struct Visitor;

        impl<
            T0: Deserialize<D, E>,
            T1: Deserialize<D, E>,
            D: DeserializerState<E>,
            E
        > ::VisitorState<(T0, T1), D, E> for Visitor {
            fn visit_seq<
                Visitor: ::SeqVisitor<D, E>,
            >(&mut self, d: &mut D, mut visitor: Visitor) -> Result<(T0, T1), E> {
                let mut state = 0u;
                let mut t0 = None;
                let mut t1 = None;

                loop {
                    match state {
                        0 => {
                            t0 = match visitor.next(d) {
                                Some(Ok(v)) => Some(v),
                                Some(Err(err)) => { return Err(err); }
                                None => { return Err(d.syntax_error()); }
                            };
                            state += 1;
                        }
                        1 => {
                            t1 = match visitor.next(d) {
                                Some(Ok(v)) => Some(v),
                                Some(Err(err)) => { return Err(err); }
                                None => { return Err(d.syntax_error()); }
                            };
                            state += 1;
                        }
                        _ => {
                            match visitor.next(d) {
                                Some(Ok(())) => { return Err(d.syntax_error()); }
                                Some(Err(err)) => { return Err(err); }
                                None => { break; }
                            }
                        }
                    }
                }

                match (t0, t1) {
                    (Some(t0), Some(t1)) => Ok((t0, t1)),
                    _ => Err(d.syntax_error()),
                }
            }
        }

        d.visit(&mut Visitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<
    K: Deserialize<D, E> + Eq + Hash,
    V: Deserialize<D, E>,
    D: DeserializerState<E>,
    E
> Deserialize<D, E> for HashMap<K, V> {
    fn deserialize(d: &mut D) -> Result<HashMap<K, V>, E> {
        struct Visitor;

        impl<
            K: Deserialize<D, E> + Eq + Hash,
            V: Deserialize<D, E>,
            D: DeserializerState<E>,
            E,
        > ::VisitorState<HashMap<K, V>, D, E> for Visitor {
            fn visit_map<
                Visitor: ::MapVisitor<D, E>,
            >(&mut self, d: &mut D, mut visitor: Visitor) -> Result<HashMap<K, V>, E> {
                let (len, _) = visitor.size_hint();
                let mut values = HashMap::with_capacity(len);

                loop {
                    let kv: Option<Result<(K, V), E>> = visitor.next(d);
                    match kv {
                        Some(Ok((key, value))) => {
                            values.insert(key, value);
                        }
                        Some(Err(err)) => {
                            return Err(err);
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
    D: DeserializerState<E>,
    E
> Deserialize<D, E> for TreeMap<K, V> {
    fn deserialize(d: &mut D) -> Result<TreeMap<K, V>, E> {
        struct Visitor;

        impl<
            K: Deserialize<D, E> + Eq + Ord,
            V: Deserialize<D, E>,
            D: DeserializerState<E>,
            E,
        > ::VisitorState<TreeMap<K, V>, D, E> for Visitor {
            fn visit_map<
                Visitor: ::MapVisitor<D, E>,
            >(&mut self, d: &mut D, mut visitor: Visitor) -> Result<TreeMap<K, V>, E> {
                let mut values = TreeMap::new();

                loop {
                    let kv: Option<Result<(K, V), E>> = visitor.next(d);
                    match kv {
                        Some(Ok((key, value))) => {
                            values.insert(key, value);
                        }
                        Some(Err(err)) => {
                            return Err(err);
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

*/
*/
