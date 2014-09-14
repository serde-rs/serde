use std::collections::{HashMap, TreeMap};
use std::hash::Hash;

///////////////////////////////////////////////////////////////////////////////

trait Deserialize<S, E> {
    fn deserialize(state: &mut S) -> Result<Self, E>;
}

///////////////////////////////////////////////////////////////////////////////

trait VisitorState<E> {
    fn syntax_error(&mut self) -> E;

    fn visit<
        V: Visitor<T, Self, E>,
        T: Deserialize<Self, E>,
    >(&mut self, visitor: &mut V) -> Result<T, E>;
}

trait Visitor<
    T,
    S: VisitorState<E>,
    E,
> {
    fn visit_null(&mut self, state: &mut S) -> Result<T, E> {
        Err(state.syntax_error())
    }

    fn visit_int(&mut self, state: &mut S, _v: int) -> Result<T, E> {
        Err(state.syntax_error())
    }

    fn visit_string(&mut self, state: &mut S, _v: String) -> Result<T, E> {
        Err(state.syntax_error())
    }

    fn visit_seq<
        V: SeqVisitor<S, E>,
    >(&mut self, state: &mut S, _visitor: V) -> Result<T, E> {
        Err(state.syntax_error())
    }

    fn visit_map<
        V: MapVisitor<S, E>,
    >(&mut self, state: &mut S, _visitor: V) -> Result<T, E> {
        Err(state.syntax_error())
    }
}

trait SeqVisitor<S, E> {
    fn next<
        T: Deserialize<S, E>,
    >(&mut self, state: &mut S) -> Option<Result<T, E>>;

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        (0, None)
    }
}

trait MapVisitor<S, E> {
    fn next<
        K: Deserialize<S, E>,
        V: Deserialize<S, E>,
    >(&mut self, state: &mut S) -> Option<Result<(K, V), E>>;

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        (0, None)
    }
}


///////////////////////////////////////////////////////////////////////////////

impl<
    S: VisitorState<E>,
    E,
> Deserialize<S, E> for int {
    fn deserialize(state: &mut S) -> Result<int, E> {
        struct Visitor;

        impl<
            S: VisitorState<E>,
            E,
        > ::Visitor<int, S, E> for Visitor {
            fn visit_int(&mut self, _state: &mut S, v: int) -> Result<int, E> {
                Ok(v)
            }
        }

        state.visit(&mut Visitor)
    }
}

impl<
    S: VisitorState<E>,
    E,
> Deserialize<S, E> for String {
    fn deserialize(state: &mut S) -> Result<String, E> {
        struct Visitor;

        impl<
            S: VisitorState<E>,
            E,
        > ::Visitor<String, S, E> for Visitor {
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
    S: VisitorState<E>,
    E,
> Deserialize<S, E> for Vec<T> {
    fn deserialize(state: &mut S) -> Result<Vec<T>, E> {
        struct Visitor;

        impl<
            T: Deserialize<S, E>,
            S: VisitorState<E>,
            E,
        > ::Visitor<Vec<T>, S, E> for Visitor {
            fn visit_seq<
                Visitor: SeqVisitor<S, E>,
            >(&mut self, state: &mut S, mut visitor: Visitor) -> Result<Vec<T>, E> {
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

        state.visit(&mut Visitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<
    S: VisitorState<E>,
    E
> Deserialize<S, E> for () {
    fn deserialize(state: &mut S) -> Result<(), E> {
        struct Visitor;

        impl<
            S: VisitorState<E>,
            E,
        > ::Visitor<(), S, E> for Visitor {
            fn visit_null(&mut self, _state: &mut S) -> Result<(), E> {
                Ok(())
            }
        }

        state.visit(&mut Visitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<
    T0: Deserialize<S, E>,
    T1: Deserialize<S, E>,
    S: VisitorState<E>,
    E
> Deserialize<S, E> for (T0, T1) {
    fn deserialize(state: &mut S) -> Result<(T0, T1), E> {
        struct Visitor;

        impl<
            T0: Deserialize<S, E>,
            T1: Deserialize<S, E>,
            S: VisitorState<E>,
            E
        > ::Visitor<(T0, T1), S, E> for Visitor {
            fn visit_seq<
                Visitor: SeqVisitor<S, E>,
            >(&mut self, visitor_state: &mut S, mut visitor: Visitor) -> Result<(T0, T1), E> {
                let mut state = 0u;
                let mut t0 = None;
                let mut t1 = None;

                loop {
                    match state {
                        0 => {
                            t0 = match visitor.next(visitor_state) {
                                Some(Ok(v)) => Some(v),
                                Some(Err(err)) => { return Err(err); }
                                None => { return Err(visitor_state.syntax_error()); }
                            };
                            state += 1;
                        }
                        1 => {
                            t1 = match visitor.next(visitor_state) {
                                Some(Ok(v)) => Some(v),
                                Some(Err(err)) => { return Err(err); }
                                None => { return Err(visitor_state.syntax_error()); }
                            };
                            state += 1;
                        }
                        _ => {
                            match visitor.next(visitor_state) {
                                Some(Ok(())) => { return Err(visitor_state.syntax_error()); }
                                Some(Err(err)) => { return Err(err); }
                                None => { break; }
                            }
                        }
                    }
                }

                match (t0, t1) {
                    (Some(t0), Some(t1)) => Ok((t0, t1)),
                    _ => Err(visitor_state.syntax_error()),
                }
            }
        }

        state.visit(&mut Visitor)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<
    K: Deserialize<S, E> + Eq + Hash,
    V: Deserialize<S, E>,
    S: VisitorState<E>,
    E
> Deserialize<S, E> for HashMap<K, V> {
    fn deserialize(state: &mut S) -> Result<HashMap<K, V>, E> {
        struct Visitor;

        impl<
            K: Deserialize<S, E> + Eq + Hash,
            V: Deserialize<S, E>,
            S: VisitorState<E>,
            E,
        > ::Visitor<HashMap<K, V>, S, E> for Visitor {
            fn visit_map<
                Visitor: MapVisitor<S, E>,
            >(&mut self, state: &mut S, mut visitor: Visitor) -> Result<HashMap<K, V>, E> {
                let (len, _) = visitor.size_hint();
                let mut values = HashMap::with_capacity(len);

                loop {
                    match visitor.next(state) {
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

        state.visit(&mut Visitor)
    }
}

impl<
    K: Deserialize<S, E> + Eq + Ord,
    V: Deserialize<S, E>,
    S: VisitorState<E>,
    E
> Deserialize<S, E> for TreeMap<K, V> {
    fn deserialize(state: &mut S) -> Result<TreeMap<K, V>, E> {
        struct Visitor;

        impl<
            K: Deserialize<S, E> + Eq + Ord,
            V: Deserialize<S, E>,
            S: VisitorState<E>,
            E,
        > ::Visitor<TreeMap<K, V>, S, E> for Visitor {
            fn visit_map<
                Visitor: MapVisitor<S, E>,
            >(&mut self, state: &mut S, mut visitor: Visitor) -> Result<TreeMap<K, V>, E> {
                let mut values = TreeMap::new();

                loop {
                    match visitor.next(state) {
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

        state.visit(&mut Visitor)
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
        S: super::VisitorState<E>,
        E,
    > super::Deserialize<S, E> for Value {
        fn deserialize(state: &mut S) -> Result<Value, E> {
            struct Visitor;

            impl<
                S: super::VisitorState<E>,
                E,
            > super::Visitor<Value, S, E> for Visitor {
                fn visit_null(&mut self, _state: &mut S) -> Result<Value, E> {
                    Ok(Null)
                }

                fn visit_int(&mut self, _state: &mut S, v: int) -> Result<Value, E> {
                    Ok(Int(v))
                }

                fn visit_string(&mut self, _state: &mut S, v: String) -> Result<Value, E> {
                    Ok(String(v))
                }

                fn visit_seq<
                    Visitor: ::SeqVisitor<S, E>,
                >(&mut self, state: &mut S, mut visitor: Visitor) -> Result<Value, E> {
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

                    Ok(Vec(values))
                }

                fn visit_map<
                    Visitor: ::MapVisitor<S, E>,
                >(&mut self, state: &mut S, mut visitor: Visitor) -> Result<Value, E> {
                    let mut values = TreeMap::new();

                    loop {
                        match visitor.next(state) {
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

            state.visit(&mut Visitor)
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

trait Deserializer<S, E> {
    fn deserialize<T: Deserialize<S, E>>(&mut self) -> Result<T, E>;
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

struct MyDeserializerState<Iter> {
    tokens: Iter,
    peeked: Option<Token>,
}

impl<Iter: Iterator<Token>> MyDeserializerState<Iter> {
    fn new(tokens: Iter) -> MyDeserializerState<Iter> {
        MyDeserializerState {
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
> VisitorState<
    (),
> for MyDeserializerState<Iter> {
    fn syntax_error(&mut self) -> () {
        ()
    }

    fn visit<
        V: Visitor<T, MyDeserializerState<Iter>, ()>,
        T: Deserialize<MyDeserializerState<Iter>, ()>,
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
> SeqVisitor<MyDeserializerState<Iter>, ()> for MySeqVisitor {
    fn next<
        T: Deserialize<MyDeserializerState<Iter>, ()>,
    >(&mut self, state: &mut MyDeserializerState<Iter>) -> Option<Result<T, ()>> {
        match state.peek() {
            Some(&End) => {
                state.next();
                None
            }
            Some(_) => {
                self.len -= 1;
                Some(Deserialize::deserialize(state))
            }
            None => {
                Some(Err(state.syntax_error()))
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
> MapVisitor<MyDeserializerState<Iter>, ()> for MyMapVisitor {
    fn next<
        K: Deserialize<MyDeserializerState<Iter>, ()>,
        V: Deserialize<MyDeserializerState<Iter>, ()>,
    >(&mut self, state: &mut MyDeserializerState<Iter>) -> Option<Result<(K, V), ()>> {
        match state.peek() {
            Some(&End) => {
                state.next();
                None
            }
            Some(_) => {
                self.len -= 1;

                let key = match Deserialize::deserialize(state) {
                    Ok(key) => key,
                    Err(err) => { return Some(Err(err)); }
                };

                let value = match Deserialize::deserialize(state) {
                    Ok(value) => value,
                    Err(err) => { return Some(Err(err)); }
                };

                Some(Ok((key, value)))
            }
            None => {
                Some(Err(state.syntax_error()))
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
    let mut state = MyDeserializerState::new(tokens.move_iter());

    let v: Result<Vec<int>, ()> = Deserialize::deserialize(&mut state);
    println!("{}", v);

    ////

    let tokens = vec!(
        SeqStart(2),
        Int(1),
        Int(2),
        End
    );
    let mut state = MyDeserializerState::new(tokens.move_iter());

    let v: Result<(int, int), ()> = Deserialize::deserialize(&mut state);
    println!("{}", v);

    ////

    let tokens = vec!(
        SeqStart(2),
        Int(1),
        Int(2),
        End
    );
    let mut state = MyDeserializerState::new(tokens.move_iter());

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
    let mut state = MyDeserializerState::new(tokens.move_iter());

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
    let mut state = MyDeserializerState::new(tokens.move_iter());

    let v: Result<json::Value, ()> = Deserialize::deserialize(&mut state);
    println!("{}", v);
}
