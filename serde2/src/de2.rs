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
                            match try!(visitor.visit(d)) {
                                Some(value) => {
                                    t0 = Some(value);
                                }
                                None => {
                                    return Err(d.end_of_stream_error());
                                }
                            }
                        }
                        1 => {
                            state += 1;
                            match try!(visitor.visit(d)) {
                                Some(value) => {
                                    t1 = Some(value);
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
