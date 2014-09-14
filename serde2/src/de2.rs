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
        V: Visitor<T, SeqState, MapState, Self, E>,
        T,
        SeqState,
        MapState,
    >(&mut self, visitor: &mut V) -> Result<T, E>;
}

trait Visitor<
    T,
    SeqState,
    MapState,
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

    fn visit_seq(&mut self, state: &mut S, _len: uint) -> Result<SeqState, E> {
        Err(state.syntax_error())
    }

    fn visit_seq_elt(&mut self, state: &mut S, _values: &mut SeqState) -> Result<(), E> {
        Err(state.syntax_error())
    }

    fn visit_seq_end(&mut self, state: &mut S, _values: SeqState) -> Result<T, E> {
        Err(state.syntax_error())
    }

    fn visit_map(&mut self, state: &mut S, _len: uint) -> Result<MapState, E> {
        Err(state.syntax_error())
    }

    fn visit_map_elt(&mut self, state: &mut S, _values: &mut MapState) -> Result<(), E> {
        Err(state.syntax_error())
    }

    fn visit_map_end(&mut self, state: &mut S, _values: MapState) -> Result<T, E> {
        Err(state.syntax_error())
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
        > ::Visitor<int, (), (), S, E> for Visitor {
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
        > ::Visitor<String, (), (), S, E> for Visitor {
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
        > ::Visitor<Vec<T>, Vec<T>, (), S, E> for Visitor {
            fn visit_seq(&mut self, _state: &mut S, len: uint) -> Result<Vec<T>, E> {
                Ok(Vec::with_capacity(len))
            }

            fn visit_seq_elt(&mut self, state: &mut S, values: &mut Vec<T>) -> Result<(), E> {
                let value = try!(Deserialize::deserialize(state));
                values.push(value);
                Ok(())
            }

            fn visit_seq_end(&mut self, _state: &mut S, values: Vec<T>) -> Result<Vec<T>, E> {
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
        > ::Visitor<(), (), (), S, E> for Visitor {
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
        struct Visitor {
            state: uint,
        }

        impl<
            T0: Deserialize<S, E>,
            T1: Deserialize<S, E>,
            S: VisitorState<E>,
            E
        > ::Visitor<(T0, T1), (Option<T0>, Option<T1>), (), S, E> for Visitor {
            fn visit_seq(&mut self, _state: &mut S, _len: uint) -> Result<(Option<T0>, Option<T1>), E> {
                Ok((None, None))
            }

            fn visit_seq_elt(&mut self, state: &mut S, values: &mut (Option<T0>, Option<T1>)) -> Result<(), E> {
                match self.state {
                    0 => {
                        *values.mut0() = Some(try!(Deserialize::deserialize(state)));
                        self.state += 1;
                        Ok(())
                    }
                    1 => {
                        *values.mut1() = Some(try!(Deserialize::deserialize(state)));
                        self.state += 1;
                        Ok(())
                    }
                    _ => {
                        Err(state.syntax_error())
                    }
                }
            }

            fn visit_seq_end(&mut self, state: &mut S, values: (Option<T0>, Option<T1>)) -> Result<(T0, T1), E> {
                match values {
                    (Some(t0), Some(t1)) => Ok((t0, t1)),
                    _ => Err(state.syntax_error()),
                }
            }
        }

        state.visit(&mut Visitor { state: 0 })
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
        > ::Visitor<HashMap<K, V>, (), HashMap<K, V>, S, E> for Visitor {
            fn visit_map(&mut self, _state: &mut S, len: uint) -> Result<HashMap<K, V>, E> {
                Ok(HashMap::with_capacity(len))
            }

            fn visit_map_elt(&mut self, state: &mut S, values: &mut HashMap<K, V>) -> Result<(), E> {
                let key = try!(Deserialize::deserialize(state));
                let value = try!(Deserialize::deserialize(state));
                values.insert(key, value);
                Ok(())
            }

            fn visit_map_end(&mut self, _state: &mut S, values: HashMap<K, V>) -> Result<HashMap<K, V>, E> {
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
        > ::Visitor<TreeMap<K, V>, (), TreeMap<K, V>, S, E> for Visitor {
            fn visit_map(&mut self, _state: &mut S, _len: uint) -> Result<TreeMap<K, V>, E> {
                Ok(TreeMap::new())
            }

            fn visit_map_elt(&mut self, state: &mut S, values: &mut TreeMap<K, V>) -> Result<(), E> {
                let key = try!(Deserialize::deserialize(state));
                let value = try!(Deserialize::deserialize(state));
                values.insert(key, value);
                Ok(())
            }

            fn visit_map_end(&mut self, _state: &mut S, values: TreeMap<K, V>) -> Result<TreeMap<K, V>, E> {
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
            > super::Visitor<Value, Vec<Value>, TreeMap<String, Value>, S, E> for Visitor {
                fn visit_null(&mut self, _state: &mut S) -> Result<Value, E> {
                    Ok(Null)
                }

                fn visit_int(&mut self, _state: &mut S, v: int) -> Result<Value, E> {
                    Ok(Int(v))
                }

                fn visit_string(&mut self, _state: &mut S, v: String) -> Result<Value, E> {
                    Ok(String(v))
                }

                fn visit_seq(&mut self, _state: &mut S, len: uint) -> Result<Vec<Value>, E> {
                    Ok(Vec::with_capacity(len))
                }

                fn visit_seq_elt(&mut self, state: &mut S, values: &mut Vec<Value>) -> Result<(), E> {
                    let value = try!(::Deserialize::deserialize(state));
                    values.push(value);
                    Ok(())
                }

                fn visit_seq_end(&mut self, _state: &mut S, values: Vec<Value>) -> Result<Value, E> {
                    Ok(Vec(values))
                }

                fn visit_map(&mut self, _state: &mut S, _len: uint) -> Result<TreeMap<String, Value>, E> {
                    Ok(TreeMap::new())
                }

                fn visit_map_elt(&mut self, state: &mut S, values: &mut TreeMap<String, Value>) -> Result<(), E> {
                    let key = try!(::Deserialize::deserialize(state));
                    let value = try!(::Deserialize::deserialize(state));
                    values.insert(key, value);
                    Ok(())
                }

                fn visit_map_end(&mut self, _state: &mut S, values: TreeMap<String, Value>) -> Result<Value, E> {
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
    'a,
    Iter: Iterator<Token>,
> VisitorState<
    (),
> for MyDeserializerState<Iter> {
    fn syntax_error(&mut self) -> () {
        ()
    }

    fn visit<
        V: Visitor<T, SeqState, MapState, MyDeserializerState<Iter>, ()>,
        T,
        SeqState,
        MapState,
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
                let mut state = try!(visitor.visit_seq(self, len));

                loop {
                    match self.peek() {
                        Some(&End) => {
                            self.next();
                            break;
                        }
                        Some(_) => {
                            try!(visitor.visit_seq_elt(self, &mut state));
                        }
                        None => {
                            return Err(());
                        }
                    }
                }

                visitor.visit_seq_end(self, state)
            }
            Some(MapStart(len)) => {
                let mut state = try!(visitor.visit_map(self, len));

                loop {
                    match self.peek() {
                        Some(&End) => {
                            self.next();
                            break;
                        }
                        Some(_) => {
                            try!(visitor.visit_map_elt(self, &mut state));
                        }
                        None => {
                            return Err(());
                        }
                    }
                }

                visitor.visit_map_end(self, state)
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
