extern crate serde2;

use serde2::de2;
use serde2::de2::{Deserialize, Deserializer};

enum Token {
    //Null,
    Int(int),
    //String(String),
    SeqStart(uint),
    //MapStart(uint),
    End,
}

#[deriving(Show)]
enum Error {
    SyntaxError,
    EndOfStreamError,
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

impl<Iter: Iterator<Token>> Deserializer<Error> for MyDeserializer<Iter> {
    fn visit<
        R,
        V: de2::Visitor<MyDeserializer<Iter>, R, Error>,
    >(&mut self, visitor: &mut V) -> Result<R, Error> {
        match self.next() {
            /*
            Some(Null) => {
                visitor.visit_null(self)
            }
            */
            Some(Int(v)) => {
                visitor.visit_int(self, v)
            }
            /*
            Some(String(v)) => {
                visitor.visit_string(self, v)
            }
            */
            Some(SeqStart(len)) => {
                visitor.visit_seq(self, MySeqVisitor { len: len })
            }
            /*
            Some(MapStart(len)) => {
                visitor.visit_map(self, MyMapVisitor { len: len })
            }
            */
            Some(End) => {
                Err(self.syntax_error())
            }
            None => {
                Err(self.end_of_stream_error())
            }
        }
    }

    fn syntax_error(&mut self) -> Error {
        SyntaxError
    }

    fn end_of_stream_error(&mut self) -> Error {
        EndOfStreamError
    }
}

struct MySeqVisitor {
    len: uint,
}

impl<
    Iter: Iterator<Token>,
> de2::SeqVisitor<MyDeserializer<Iter>, Error> for MySeqVisitor {
    fn next<
        T: Deserialize<MyDeserializer<Iter>, Error>
    >(&mut self, d: &mut MyDeserializer<Iter>) -> Option<Result<T, Error>> {
        match d.peek() {
            Some(&End) => {
                d.next();
                None
            }
            Some(_) => {
                self.len -= 1;
                Some(Deserialize::deserialize(d))
            }
            None => {
                Some(Err(d.syntax_error()))
            }
        }
    }

    fn end(&mut self, d: &mut MyDeserializer<Iter>) -> Result<(), Error> {
        match d.next() {
            Some(End) => Ok(()),
            Some(_) => Err(d.syntax_error()),
            None => Err(d.end_of_stream_error()),
        }
    }

    fn size_hint(&self, _d: &mut MyDeserializer<Iter>) -> (uint, Option<uint>) {
        (self.len, Some(self.len))
    }
}

/*
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
*/

///////////////////////////////////////////////////////////////////////////////

mod json {
    //use std::collections::TreeMap;
    use serde2::de2;

    #[deriving(Show)]
    pub enum Value {
        //Null,
        //Bool(bool),
        Int(int),
        //String(String),
        List(Vec<Value>),
        //Map(TreeMap<String, Value>),
    }

    impl<
        D: de2::Deserializer<E>,
        E,
    > de2::Deserialize<D, E> for Value {
        fn deserialize(d: &mut D) -> Result<Value, E> {
            struct Visitor;

            impl<
                D: de2::Deserializer<E>,
                E,
            > de2::Visitor<D, Value, E> for Visitor {
                /*
                fn visit_null(&mut self, _d: &mut D) -> Result<Value, E> {
                    Ok(Null)
                }
                */

                fn visit_int(&mut self, _d: &mut D, v: int) -> Result<Value, E> {
                    Ok(Int(v))
                }

                /*
                fn visit_string(&mut self, _d: &mut D, v: String) -> Result<Value, E> {
                    Ok(String(v))
                }
                */

                fn visit_seq<
                    Visitor: de2::SeqVisitor<D, E>,
                >(&mut self, d: &mut D, mut visitor: Visitor) -> Result<Value, E> {
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

                    Ok(List(values))
                }

                /*
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
                */
            }

            d.visit(&mut Visitor)
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
    let mut state = MyDeserializer::new(tokens.into_iter());

    let v: Result<Vec<int>, Error> = Deserialize::deserialize(&mut state);
    println!("{}", v);

    ////

    let tokens = vec!(
        SeqStart(2),
        Int(1),
        Int(2),
        End
    );
    let mut state = MyDeserializer::new(tokens.into_iter());

    let v: Result<(int, int), Error> = Deserialize::deserialize(&mut state);
    println!("{}", v);

    ////

    let tokens = vec!(
        SeqStart(2),
        Int(1),
        Int(2),
        End
    );
    let mut state = MyDeserializer::new(tokens.into_iter());

    let v: Result<json::Value, Error> = Deserialize::deserialize(&mut state);
    println!("{}", v);

    /*
    ////

    let tokens = vec!(
        MapStart(2),
        String("a".to_string()),
        Int(1),
        String("b".to_string()),
        Int(2),
        End
    );
    let mut state = MyDeserializer::new(tokens.into_iter());

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
    let mut state = MyDeserializer::new(tokens.into_iter());

    let v: Result<json::Value, ()> = Deserialize::deserialize(&mut state);
    println!("{}", v);
    */
}


/*
use std::collections::TreeMap;
use serde2::{Serialize, GatherTokens};
use serde2::json;

///////////////////////////////////////////////////////////////////////////////

struct Foo {
    x: int,
    y: int,
    z: &'static str,
}

impl<S: serde2::VisitorState<R>, R> serde2::Serialize<S, R> for Foo {
    fn serialize(&self, state: &mut S) -> R {
        state.visit_named_map("Foo", FooSerialize {
            value: self,
            state: 0,
        })
    }
}

struct FooSerialize<'a> {
    value: &'a Foo,
    state: uint,
}

impl<'a, S: serde2::VisitorState<R>, R> serde2::Visitor<S, R> for FooSerialize<'a> {
    fn visit(&mut self, state: &mut S) -> Option<R> {
        match self.state {
            0 => {
                self.state += 1;
                Some(state.visit_map_elt(true, "x", &self.value.x))
            }
            1 => {
                self.state += 1;
                Some(state.visit_map_elt(false, "y", &self.value.y))
            }
            2 => {
                self.state += 1;
                Some(state.visit_map_elt(false, "z", &self.value.z))
            }
            _ => {
                None
            }
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        let size = 3 - self.state;
        (size, Some(size))
    }
}

///////////////////////////////////////////////////////////////////////////////

fn main() {
    let value = 5i;

    let mut s = GatherTokens::new();
    value.serialize(&mut s);
    println!("tokens: {}", s.unwrap());

    println!("json:   {}", json::to_string(&value).unwrap().unwrap());
    println!("");

    ////

    let value = vec!(1i, 2, 3);

    let mut s = GatherTokens::new();
    value.serialize(&mut s);
    println!("tokens: {}", s.unwrap());

    println!("json:   {}", json::to_string(&value).unwrap().unwrap());
    println!("");

    ////

    let mut value = TreeMap::new();
    value.insert("a", 1i);
    value.insert("b", 2);
    value.insert("c", 3);

    let mut s = GatherTokens::new();
    value.serialize(&mut s);
    println!("tokens: {}", s.unwrap());

    println!("json:   {}", json::to_string(&value).unwrap().unwrap());
    println!("");

    ////

    /*
    println!("{}", to_format_vec(&5i));
    println!("{}", to_format_string(&5i));
    */

    let value = Foo { x: 1, y: 2, z: "abc" };

    let mut s = GatherTokens::new();
    value.serialize(&mut s);
    println!("tokens: {}", s.unwrap());

    println!("json:   {}", json::to_string(&value).unwrap().unwrap());
    println!("");

    ////

    let value = (1i, "abc");

    let mut s = GatherTokens::new();
    value.serialize(&mut s);
    println!("tokens: {}", s.unwrap());

    println!("json:   {}", json::to_string(&value).unwrap().unwrap());
    println!("");
}
*/
