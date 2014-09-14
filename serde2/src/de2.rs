use std::collections::HashMap;
use std::hash::Hash;

///////////////////////////////////////////////////////////////////////////////

trait Deserialize<S, E> {
    fn deserialize(state: &mut S) -> Result<Self, E>;
}

///////////////////////////////////////////////////////////////////////////////

trait VisitorState<E> {
    fn visit<
        V: Visitor<Value, SeqValue, E>,
        Value,
        SeqValue,
    >(&mut self, visitor: &mut V) -> Result<Value, E>;

    fn visit_null(&mut self) -> Result<(), E>;

    fn visit_int(&mut self) -> Result<int, E>;

    fn visit_string(&mut self) -> Result<String, E>;

    fn visit_seq(&mut self) -> Result<uint, E>;

    fn visit_seq_elt<
        T: Deserialize<Self, E>,
    >(&mut self) -> Option<Result<T, E>>;

    /*
    fn visit_map<
        T: Deserialize<Self, E>,
        V: Visitor<T, Self, E>
    >(&mut self) -> Result<T, E>;

    fn visit_map_elt<
        K: Deserialize<Self, E>,
        V: Deserialize<Self, E>
    >(&mut self) -> Result<(K, V), E>;
    */
}

trait Visitor<Value, SeqValue, E> {
    fn visit_null(&mut self) -> Result<Value, E>;

    fn visit_int(&mut self, v: int) -> Result<Value, E>;

    fn visit_string(&mut self, v: String) -> Result<Value, E>;

    fn visit_seq(&mut self, len: uint) -> Result<SeqValue, E>;

    fn visit_seq_elt(&mut self, values: &mut SeqValue, value: Value) -> Result<(), E>;

    fn visit_seq_end(&mut self, values: SeqValue) -> Result<Value, E>;
}

/*
trait Visitor<VS: VisitorState<Self, E>, E> {
    fn next<
        T: Deserialize<VS, E>,
    >(&mut self) -> Option<Result<T, E>>;

    fn size_hint(&self) -> (uint, Option<uint>);
}
*/

/*
trait Visitor<C, S, E> {
    fn new(len: uint) -> Self;

    fn visit(&mut self, state: &mut S) -> Result<(), E>;

    fn unwrap(self) -> Result<C, E>;
}
*/

///////////////////////////////////////////////////////////////////////////////

impl<
    S: VisitorState<E>,
    E,
> Deserialize<S, E> for int {
    fn deserialize(state: &mut S) -> Result<int, E> {
        state.visit_int()
    }
}

impl<
    S: VisitorState<E>,
    E,
> Deserialize<S, E> for String {
    fn deserialize(state: &mut S) -> Result<String, E> {
        state.visit_string()
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<
    T: Deserialize<S, E>,
    S: VisitorState<E>,
    E,
> Deserialize<S, E> for Vec<T> {
    fn deserialize(state: &mut S) -> Result<Vec<T>, E> {
        let len = try!(state.visit_seq());
        let mut value = Vec::with_capacity(len);

        loop {
            match state.visit_seq_elt() {
                Some(Ok(v)) => { value.push(v); }
                Some(Err(err)) => { return Err(err); }
                None => { break; }
            }
        }

        Ok(value)
    }
}

///////////////////////////////////////////////////////////////////////////////


impl<
    S: VisitorState<E>,
    E
> Deserialize<S, E> for () {
    fn deserialize(state: &mut S) -> Result<(), E> {
        state.visit_null()
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
        let _ = try!(state.visit_seq());

        let t0 = match state.visit_seq_elt() {
            Some(Ok(v)) => v,
            Some(Err(err)) => { return Err(err); }
            None => { fail!(); }
        };

        let t1 = match state.visit_seq_elt() {
            Some(Ok(v)) => v,
            Some(Err(err)) => { return Err(err); }
            None => { fail!(); }
        };

        match state.visit_seq_elt() {
            Some(Ok(())) => { fail!(); }
            Some(Err(err)) => { return Err(err); }
            None => { }
        }

        Ok((t0, t1))

        /*
        struct Visitor<T0, T1> {
            state: uint,
            t0: Option<T0>,
            t1: Option<T1>,
        }

        impl<
            T0: Deserialize<S, E>,
            T1: Deserialize<S, E>,
            S: VisitorState<E>,
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
                        self.t0 = Some(try!(state.visit_seq_elt()));
                    }
                    1 => {
                        self.state += 1;
                        self.t1 = Some(try!(state.visit_seq_elt()));
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
        */
    }
}

/*
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
                let (key, value) = try!(state.visit_map_elt());
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
*/

///////////////////////////////////////////////////////////////////////////////

mod json {
    enum Value {
        Null,
        Bool(bool),
        Int(int),
        String(String),
        Vec(Vec<Value>),
    }

    impl<
        S: super::VisitorState<E>,
        E
    > super::Deserialize<S, E> for Value {
        fn deserialize(state: &mut S) -> Result<Value, E> {
            struct Visitor;

            impl<E> super::Visitor<Value, Vec<Value>, E> for Visitor {
                fn visit_null(&mut self) -> Result<Value, E> {
                    Ok(Null)
                }

                fn visit_int(&mut self, v: int) -> Result<Value, E> {
                    Ok(Int(v))
                }

                fn visit_string(&mut self, v: String) -> Result<Value, E> {
                    Ok(String(v))
                }

                fn visit_seq(&mut self, len: uint) -> Result<Vec<Value>, E> {
                    Ok(Vec::with_capacity(len))
                }

                fn visit_seq_elt(&mut self, values: &mut Vec<Value>, value: Value) -> Result<(), E> {
                    values.push(value);
                    Ok(())
                }

                fn visit_seq_end(&mut self, values: Vec<Value>) -> Result<Value, E> {
                    Ok(Vec(values))
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
    fn visit<
        V: Visitor<Value, SeqValue, ()>,
        Value,
        SeqValue,
    >(&mut self, visitor: &mut V) -> Result<Value, ()> {
        match self.next() {
            Some(Null) => {
                visitor.visit_null()
            }
            Some(Int(v)) => {
                visitor.visit_int(v)
            }
            Some(String(v)) => {
                visitor.visit_string(v)
            }
            Some(SeqStart(len)) => {
                let mut state = try!(visitor.visit_seq(len));

                loop {
                    match self.peek() {
                        Some(&End) => {
                            self.next();
                            break;
                        }
                        Some(_) => {
                            let value = try!(self.visit(visitor));
                            try!(visitor.visit_seq_elt(&mut state, value));
                        }
                        None => {
                            return Err(());
                        }
                    }
                }

                visitor.visit_seq_end(state)
            }
            Some(MapStart(len)) => {
                Err(())
            }
            Some(End) => {
                Err(())
            }
            None => {
                Err(())
            }
        }

    }

    fn visit_null(&mut self) -> Result<(), ()> {
        match self.next() {
            Some(Null) => Ok(()),
            _ => Err(())
        }
    }

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

    fn visit_seq(&mut self) -> Result<uint, ()> {
        match self.next() {
            Some(SeqStart(len)) => Ok(len),
            _ => Err(()),
        }
    }

    fn visit_seq_elt<
        T: Deserialize<MyDeserializerState<Iter>, ()>,
    >(&mut self) -> Option<Result<T, ()>> {
        match self.peek() {
            Some(&End) => {
                self.next();
                None
            }
            Some(_) => {
                Some(Deserialize::deserialize(self))
            }
            None => {
                Some(Err(()))
            }
        }
    }

    /*
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

    fn visit_map_elt<
        K: Deserialize<MyDeserializerState<Iter>, ()>,
        V: Deserialize<MyDeserializerState<Iter>, ()>
    >(&mut self) -> Result<(K, V), ()> {
        let k = try!(Deserialize::deserialize(self));
        let v = try!(Deserialize::deserialize(self));
        Ok((k, v))
    }
    */
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

    /*
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
    */
}
