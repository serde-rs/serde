use std::collections::HashMap;
use std::hash::Hash;

///////////////////////////////////////////////////////////////////////////////

trait Deserialize<S, E> {
    fn deserialize(state: &mut S) -> Result<Self, E>;
}

///////////////////////////////////////////////////////////////////////////////

trait VisitorState<E> {
    fn visit_int(&mut self) -> Result<int, E>;

    fn visit_string(&mut self) -> Result<String, E>;

    fn visit_seq<
        C: Deserialize<Self, E>,
        V: Visitor<C, Self, E>
    >(&mut self) -> Result<C, E>;

    fn visit_map<
        C: Deserialize<Self, E>,
        V: Visitor<C, Self, E>
    >(&mut self) -> Result<C, E>;
}

trait Visitor<C, S, E> {
    fn new(len: uint) -> Self;

    fn visit(&mut self, state: &mut S) -> Result<(), E>;

    fn unwrap(self) -> Result<C, E>;
}

///////////////////////////////////////////////////////////////////////////////

impl<S: VisitorState<E>, E> Deserialize<S, E> for int {
    fn deserialize(state: &mut S) -> Result<int, E> {
        state.visit_int()
    }
}

impl<S: VisitorState<E>, E> Deserialize<S, E> for String {
    fn deserialize(state: &mut S) -> Result<String, E> {
        state.visit_string()
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<
    T: Deserialize<S, E>,
    S: VisitorState<E>,
    E
> Deserialize<S, E> for Vec<T> {
    fn deserialize(state: &mut S) -> Result<Vec<T>, E> {
        struct Visitor<T> {
            value: Vec<T>,
        }

        impl<
            T: Deserialize<S, E>,
            S: VisitorState<E>,
            E
        > ::Visitor<Vec<T>, S, E> for Visitor<T> {
            fn new(len: uint) -> Visitor<T> {
                Visitor {
                    value: Vec::with_capacity(len),
                }
            }

            fn visit(&mut self, state: &mut S) -> Result<(), E> {
                let value = try!(Deserialize::deserialize(state));
                self.value.push(value);
                Ok(())
            }

            fn unwrap(self) -> Result<Vec<T>, E> {
                Ok(self.value)
            }
        }

        state.visit_seq::<Vec<T>, Visitor<T>>()
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
        struct Visitor<T0, T1> {
            state: uint,
            t0: Option<T0>,
            t1: Option<T1>,
        }

        impl<
            T0: Deserialize<S, E>,
            T1: Deserialize<S, E>,
            S,
            E
        > ::Visitor<(T0, T1), S, E> for Visitor<T0, T1> {
            fn new(_: uint) -> Visitor<T0, T1> {
                Visitor {
                    state: 0,
                    t0: None,
                    t1: None,
                }
            }

            fn visit(&mut self, state: &mut S) -> Result<(), E> {
                match self.state {
                    0 => {
                        self.state += 1;
                        self.t0 = Some(try!(Deserialize::deserialize(state)));
                    }
                    1 => {
                        self.state += 1;
                        self.t1 = Some(try!(Deserialize::deserialize(state)));
                    }
                    _ => fail!()
                }

                Ok(())
            }

            fn unwrap(self) -> Result<(T0, T1), E> {
                let t0 = match self.t0 {
                    Some(t0) => t0,
                    None => { fail!(); }
                };

                let t1 = match self.t1 {
                    Some(t1) => t1,
                    None => { fail!(); }
                };

                Ok((t0, t1))
            }
        }

        state.visit_seq::<(T0, T1), Visitor<T0, T1>>()
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
        struct Visitor<K, V> {
            value: HashMap<K, V>,
        }

        impl<
            K: Deserialize<S, E> + Eq + Hash,
            V: Deserialize<S, E>,
            S: VisitorState<E>,
            E
        > ::Visitor<HashMap<K, V>, S, E> for Visitor<K, V> {
            fn new(len: uint) -> Visitor<K, V> {
                Visitor {
                    value: HashMap::with_capacity(len),
                }
            }

            fn visit(&mut self, state: &mut S) -> Result<(), E> {
                let key = try!(Deserialize::deserialize(state));
                let value = try!(Deserialize::deserialize(state));
                self.value.insert(key, value);
                Ok(())
            }

            fn unwrap(self) -> Result<HashMap<K, V>, E> {
                Ok(self.value)
            }
        }

        state.visit_map::<HashMap<K, V>, Visitor<K, V>>()
    }
}

///////////////////////////////////////////////////////////////////////////////

trait Deserializer<S, E> {
    fn deserialize<T: Deserialize<S, E>>(&mut self) -> Result<T, E>;
}

///////////////////////////////////////////////////////////////////////////////

enum Token {
    Int(int),
    String(String),
    SeqStart(uint),
    MapStart(uint),
    End,
}

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
}

impl<Iter: Iterator<Token>> VisitorState<()> for MyDeserializerState<Iter> {
    fn visit_int(&mut self) -> Result<int, ()> {
        match self.next() {
            Some(Int(v)) => Ok(v),
            _ => Err(())
        }
    }

    fn visit_string(&mut self) -> Result<String, ()> {
        match self.next() {
            Some(String(v)) => Ok(v),
            _ => Err(())
        }
    }

    fn visit_seq<
        T: Deserialize<MyDeserializerState<Iter>, ()>,
        V: Visitor<T, MyDeserializerState<Iter>, ()>
    >(&mut self) -> Result<T, ()> {
        let len = match self.next() {
            Some(SeqStart(len)) => len,
            _ => { return Err(()); }
        };

        let mut visitor: V = Visitor::new(len);

        loop {
            match self.next() {
                Some(End) => { break; }
                Some(token) => {
                    self.peeked = Some(token);
                    try!(visitor.visit(self));
                }
                _ => { return Err(()); }
            }
        }

        visitor.unwrap()
    }

    fn visit_map<
        T: Deserialize<MyDeserializerState<Iter>, ()>,
        V: Visitor<T, MyDeserializerState<Iter>, ()>
    >(&mut self) -> Result<T, ()> {
        let len = match self.next() {
            Some(MapStart(len)) => len,
            _ => { return Err(()); }
        };

        let mut visitor: V = Visitor::new(len);

        loop {
            match self.next() {
                Some(End) => { break; }
                Some(token) => {
                    self.peeked = Some(token);
                    try!(visitor.visit(self));
                }
                _ => { return Err(()); }
            }
        }

        visitor.unwrap()
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

    let tokens = vec!(
        SeqStart(2),
        Int(1),
        Int(2),
        End
    );
    let mut state = MyDeserializerState::new(tokens.move_iter());

    let v: Result<(int, int), ()> = Deserialize::deserialize(&mut state);
    println!("{}", v);

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
}
