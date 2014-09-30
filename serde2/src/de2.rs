use std::collections::{HashMap, TreeMap};
use std::hash::Hash;

///////////////////////////////////////////////////////////////////////////////

trait Deserialize<S, E> {
    fn deserialize(state: &mut S) -> Result<Self, E>;
}

///////////////////////////////////////////////////////////////////////////////

trait Deserializer<R, E> {
    fn visit<S>(&mut self, state: &mut S) -> Result<R, E>;

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
*/

trait Visitor<R> {
    fn visit<S>(&mut self, state: &mut S) -> R;
}

trait VisitorState<
    D: Deserializer<R, E>,
    R,
    E,
> {
    /*
    fn visit_null(&mut self) -> R;
    */

    fn visit_int(&mut self, d: &mut D, _v: int) -> Result<R, E>;

    /*
    fn visit_string(&mut self, _v: String) -> R;

    fn visit_seq<
        V: Visitor<R>,
    >(&mut self, _visitor: V) -> R;

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

impl<
    S: Deserializer<int, E>,
    E,
> Deserialize<S, E> for int {
    fn deserialize(state: &mut S) -> Result<int, E> {
        struct Visitor;

        impl<
            D: Deserializer<int, E>,
            E,
        > VisitorState<S, int, E> for Visitor {
            fn visit_int(&mut self, d: &mut D, v: int) -> Result<int, E> {
                Ok(v)
            }
        }

        state.visit(&mut Visitor)
    }
}


/*
impl<
    D: DeserializerState<E>,
    E,
> Deserialize<D, E> for String {
    fn deserialize(d: &mut D) -> Result<String, E> {
        struct Visitor;

        impl<
            D: DeserializerState<E>,
            E,
        > ::VisitorState<String, D, E> for Visitor {
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
    D: DeserializerState<E>,
    E,
> Deserialize<D, E> for Vec<T> {
    fn deserialize(d: &mut D) -> Result<Vec<T>, E> {
        struct Visitor;

        impl<
            T: Deserialize<D, E>,
            D: DeserializerState<E>,
            E,
        > ::VisitorState<Vec<T>, D, E> for Visitor {
            fn visit_seq<
                Visitor: ::Visitor<D, E>,
            >(&mut self, d: &mut D, mut visitor: Visitor) -> Result<Vec<T>, E> {
                let (len, _) = visitor.size_hint();
                let mut values = Vec::with_capacity(len);

                loop {
                    match visitor.next(d) {
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

mod json {
    use std::collections::TreeMap;

    #[deriving(Show)]
    pub enum Value {
        Null,
        Bool(bool),
        Int(int),
        String(String),
        Vec(Vec<Value>),
        Map(TreeMap<String, Value>),
    }

    impl<
        D: super::DeserializerState<E>,
        E,
    > super::Deserialize<D, E> for Value {
        fn deserialize(d: &mut D) -> Result<Value, E> {
            struct Visitor;

            impl<
                D: super::DeserializerState<E>,
                E,
            > super::VisitorState<Value, D, E> for Visitor {
                fn visit_null(&mut self, _d: &mut D) -> Result<Value, E> {
                    Ok(Null)
                }

                fn visit_int(&mut self, _d: &mut D, v: int) -> Result<Value, E> {
                    Ok(Int(v))
                }

                fn visit_string(&mut self, _d: &mut D, v: String) -> Result<Value, E> {
                    Ok(String(v))
                }

                fn visit_seq<
                    Visitor: ::SeqVisitor<D, E>,
                >(&mut self, d: &mut D, mut visitor: Visitor) -> Result<Value, E> {
                    let (len, _) = visitor.size_hint();
                    let mut values = Vec::with_capacity(len);

                    loop {
                        match visitor.next(d) {
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

                    Ok(Vec(values))
                }

                fn visit_map<
                    Visitor: ::MapVisitor<D, E>,
                >(&mut self, d: &mut D, mut visitor: Visitor) -> Result<Value, E> {
                    let mut values = TreeMap::new();

                    loop {
                        let kv: Option<Result<(String, Value), E>> = visitor.next(d);
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

                    Ok(Map(values))
                }
            }

            d.visit(&mut Visitor)
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

enum Token {
    Null,
    Int(int),
    String(String),
    SeqStart(uint),
    MapStart(uint),
    End,
}

///////////////////////////////////////////////////////////////////////////////

struct MyDeserializer<Iter> {
    tokens: Iter,
    peeked: Option<Token>,
}

impl<Iter: Iterator<Token>> MyDeserializer<Iter> {
    fn new(tokens: Iter) -> MyDeserializer<Iter> {
        MyDeserializer {
            tokens: tokens,
            peeked: None,
        }
    }

    fn next(&mut self) -> Option<Token> {
        match self.peeked.take() {
            Some(token) => { return Some(token); }
            None => { }
        }

        self.tokens.next()
    }

    fn peek<'a>(&'a mut self) -> Option<&'a Token> {
        match self.peeked {
            Some(_) => { }
            None => { self.peeked = self.tokens.next(); }
        }

        self.peeked.as_ref()
    }
}

impl<
    Iter: Iterator<Token>,
> DeserializerState<
    (),
> for MyDeserializer<Iter> {
    fn syntax_error(&mut self) -> () {
        ()
    }

    fn visit<
        V: VisitorState<T, MyDeserializer<Iter>, ()>,
        T: Deserialize<MyDeserializer<Iter>, ()>,
    >(&mut self, visitor: &mut V) -> Result<T, ()> {
        match self.next() {
            Some(Null) => {
                visitor.visit_null(self)
            }
            Some(Int(v)) => {
                visitor.visit_int(self, v)
            }
            Some(String(v)) => {
                visitor.visit_string(self, v)
            }
            Some(SeqStart(len)) => {
                visitor.visit_seq(self, MySeqVisitor { len: len })
            }
            Some(MapStart(len)) => {
                visitor.visit_map(self, MyMapVisitor { len: len })
            }
            Some(End) => {
                Err(())
            }
            None => {
                Err(())
            }
        }
    }
}

struct MySeqVisitor {
    len: uint,
}

impl<
    Iter: Iterator<Token>,
> Visitor<MyDeserializer<Iter>, ()> for MySeqVisitor {
    fn next(&mut self, d: &mut MyDeserializer<Iter>) -> Option<Result<T, ()>> {
        match d.peek() {
            Some(&End) => {
                d.next();
                None
            }
            Some(_) => {
                self.len -= 1;
                Some(d.visit_seq_elt())
            }
            None => {
                Some(Err(d.syntax_error()))
            }
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        (self.len, Some(self.len))
    }
}

struct MyMapVisitor {
    len: uint,
}

impl<
    Iter: Iterator<Token>,
> Visitor<MyDeserializer<Iter>, ()> for MyMapVisitor {
    fn next(&mut self, d: &mut MyDeserializer<Iter>) -> Option<Result<(K, V), ()>> {
        match d.peek() {
            Some(&End) => {
                d.next();
                None
            }
            Some(_) => {
                self.len -= 1;
                Some(d.visit_map_elt())
            }
            None => {
                Some(Err(d.syntax_error()))
            }
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        (self.len, Some(self.len))
    }
}

///////////////////////////////////////////////////////////////////////////////

fn main() {
    let tokens = vec!(
        SeqStart(2),
        Int(1),
        Int(2),
        End
    );
    let mut state = MyDeserializer::new(tokens.move_iter());

    let v: Result<Vec<int>, ()> = Deserialize::deserialize(&mut state);
    println!("{}", v);

    ////

    let tokens = vec!(
        SeqStart(2),
        Int(1),
        Int(2),
        End
    );
    let mut state = MyDeserializer::new(tokens.move_iter());

    let v: Result<(int, int), ()> = Deserialize::deserialize(&mut state);
    println!("{}", v);

    ////

    let tokens = vec!(
        SeqStart(2),
        Int(1),
        Int(2),
        End
    );
    let mut state = MyDeserializer::new(tokens.move_iter());

    let v: Result<json::Value, ()> = Deserialize::deserialize(&mut state);
    println!("{}", v);

    ////

    let tokens = vec!(
        MapStart(2),
        String("a".to_string()),
        Int(1),
        String("b".to_string()),
        Int(2),
        End
    );
    let mut state = MyDeserializer::new(tokens.move_iter());

    let v: Result<HashMap<String, int>, ()> = Deserialize::deserialize(&mut state);
    println!("{}", v);

    ////

    let tokens = vec!(
        MapStart(2),
        String("a".to_string()),
        Int(1),
        String("b".to_string()),
        Int(2),
        End
    );
    let mut state = MyDeserializer::new(tokens.move_iter());

    let v: Result<json::Value, ()> = Deserialize::deserialize(&mut state);
    println!("{}", v);
}
*/
*/

fn main() {}
