extern crate serde2;

use std::collections::HashMap;
use std::option;
use std::string;

use serde2::de;
use serde2::de::{Deserialize, Deserializer};

#[deriving(Show)]
enum Token {
    Null,
    Int(int),
    String(string::String),
    Option(bool),
    SeqStart(uint),
    MapStart(uint),
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
    peeked: option::Option<Token>,
}

impl<Iter: Iterator<Token>> MyDeserializer<Iter> {
    fn new(tokens: Iter) -> MyDeserializer<Iter> {
        MyDeserializer {
            tokens: tokens,
            peeked: None,
        }
    }

    fn next(&mut self) -> option::Option<Token> {
        match self.peeked.take() {
            Some(token) => { return Some(token); }
            None => { }
        }

        self.tokens.next()
    }

    fn peek<'a>(&'a mut self) -> option::Option<&'a Token> {
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
        V: de::Visitor<MyDeserializer<Iter>, R, Error>,
    >(&mut self, visitor: &mut V) -> Result<R, Error> {
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
            Some(Option(is_some)) => {
                visitor.visit_option(self, MyOptionVisitor {
                    is_some: is_some,
                })
            }
            Some(SeqStart(len)) => {
                visitor.visit_seq(self, MySeqVisitor { len: len })
            }
            Some(MapStart(len)) => {
                visitor.visit_map(self, MyMapVisitor { len: len })
            }
            Some(End) => {
                Err(self.syntax_error())
            }
            None => {
                Err(self.end_of_stream_error())
            }
        }
    }

    fn visit_option<
        R,
        V: de::Visitor<MyDeserializer<Iter>, R, Error>,
    >(&mut self, visitor: &mut V) -> Result<R, Error> {
        match self.peek() {
            Some(&Null) => {
                self.next();
                visitor.visit_option(self, MyOptionVisitor {
                    is_some: false,
                })
            }
            Some(&Option(is_some)) => {
                self.next();
                visitor.visit_option(self, MyOptionVisitor {
                    is_some: is_some,
                })
            }
            _ => {
                visitor.visit_option(self, MyOptionVisitor {
                    is_some: true,
                })
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

struct MyOptionVisitor {
    is_some: bool,
}

impl<
    Iter: Iterator<Token>,
> de::OptionVisitor<MyDeserializer<Iter>, Error> for MyOptionVisitor {
    fn visit<
        T: Deserialize<MyDeserializer<Iter>, Error>,
    >(&mut self, d: &mut MyDeserializer<Iter>) -> Result<option::Option<T>, Error> {
        if self.is_some {
            self.is_some = false;
            let value = try!(Deserialize::deserialize(d));
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}

struct MySeqVisitor {
    len: uint,
}

impl<
    Iter: Iterator<Token>,
> de::SeqVisitor<MyDeserializer<Iter>, Error> for MySeqVisitor {
    fn visit<
        T: Deserialize<MyDeserializer<Iter>, Error>
    >(&mut self, d: &mut MyDeserializer<Iter>) -> Result<option::Option<T>, Error> {
        match d.peek() {
            Some(&End) => {
                d.next();
                Ok(None)
            }
            Some(_) => {
                self.len -= 1;
                let value = try!(Deserialize::deserialize(d));
                Ok(Some(value))
            }
            None => {
                Err(d.syntax_error())
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

    fn size_hint(&self, _d: &mut MyDeserializer<Iter>) -> (uint, option::Option<uint>) {
        (self.len, Some(self.len))
    }
}

struct MyMapVisitor {
    len: uint,
}

impl<
    Iter: Iterator<Token>,
> de::MapVisitor<MyDeserializer<Iter>, Error> for MyMapVisitor {
    fn visit<
        K: Deserialize<MyDeserializer<Iter>, Error>,
        V: Deserialize<MyDeserializer<Iter>, Error>,
    >(&mut self, d: &mut MyDeserializer<Iter>) -> Result<option::Option<(K, V)>, Error> {
        match d.peek() {
            Some(&End) => {
                d.next();
                Ok(None)
            }
            Some(_) => {
                self.len -= 1;

                let key = try!(Deserialize::deserialize(d));
                let value = try!(Deserialize::deserialize(d));

                Ok(Some((key, value)))
            }
            None => {
                Err(d.syntax_error())
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

    fn size_hint(&self, _d: &mut MyDeserializer<Iter>) -> (uint, option::Option<uint>) {
        (self.len, Some(self.len))
    }
}

///////////////////////////////////////////////////////////////////////////////

mod json {
    use std::collections::TreeMap;
    use serde2::de;

    #[deriving(Show)]
    pub enum Value {
        Null,
        //Bool(bool),
        Int(int),
        //String(String),
        List(Vec<Value>),
        Map(TreeMap<String, Value>),
    }

    impl<
        D: de::Deserializer<E>,
        E,
    > de::Deserialize<D, E> for Value {
        fn deserialize(d: &mut D) -> Result<Value, E> {
            struct Visitor;

            impl<
                D: de::Deserializer<E>,
                E,
            > de::Visitor<D, Value, E> for Visitor {
                fn visit_null(&mut self, _d: &mut D) -> Result<Value, E> {
                    Ok(Null)
                }

                fn visit_int(&mut self, _d: &mut D, v: int) -> Result<Value, E> {
                    Ok(Int(v))
                }

                /*
                fn visit_string(&mut self, _d: &mut D, v: String) -> Result<Value, E> {
                    Ok(String(v))
                }
                */

                fn visit_option<
                    Visitor: de::OptionVisitor<D, E>,
                >(&mut self, d: &mut D, mut visitor: Visitor) -> Result<Value, E> {
                    match try!(visitor.visit(d)) {
                        Some(value) => Ok(value),
                        None => Ok(Null),
                    }
                }

                fn visit_seq<
                    Visitor: de::SeqVisitor<D, E>,
                >(&mut self, d: &mut D, mut visitor: Visitor) -> Result<Value, E> {
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

                    Ok(List(values))
                }

                fn visit_map<
                    Visitor: de::MapVisitor<D, E>,
                >(&mut self, d: &mut D, mut visitor: Visitor) -> Result<Value, E> {
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

                    Ok(Map(values))
                }
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
        End,
    );
    let mut state = MyDeserializer::new(tokens.into_iter());

    let v: Result<Vec<int>, Error> = Deserialize::deserialize(&mut state);
    println!("vec:           {}", v);

    ////

    let tokens = vec!(
        SeqStart(2),
        Int(3),
        Int(4),
        End,
    );
    let mut state = MyDeserializer::new(tokens.into_iter());

    let v: Result<(int, int), Error> = Deserialize::deserialize(&mut state);
    println!("tuple:         {}", v);

    ////

    let tokens = vec!(
        SeqStart(2),
        Int(5),
        Int(6),
        End,
    );
    let mut state = MyDeserializer::new(tokens.into_iter());

    let v: Result<json::Value, Error> = Deserialize::deserialize(&mut state);
    println!("value:         {}", v);

    ////

    let tokens = vec!(
        Option(true),
        Int(7),
    );
    let mut state = MyDeserializer::new(tokens.into_iter());

    let v: Result<option::Option<int>, Error> = Deserialize::deserialize(&mut state);
    println!("optiony:       {}", v);

    ////

    let tokens = vec!(
        Option(false),
    );
    let mut state = MyDeserializer::new(tokens.into_iter());

    let v: Result<option::Option<int>, Error> = Deserialize::deserialize(&mut state);
    println!("optiony:       {}", v);

    ////

    let tokens = vec!(
        Option(true),
        Int(8),
    );
    let mut state = MyDeserializer::new(tokens.into_iter());

    let v: Result<json::Value, Error> = Deserialize::deserialize(&mut state);
    println!("optiony value: {}", v);

    ////

    let tokens = vec!(
        Option(false),
    );
    let mut state = MyDeserializer::new(tokens.into_iter());

    let v: Result<json::Value, Error> = Deserialize::deserialize(&mut state);
    println!("optiony value: {}", v);

    ////

    let tokens = vec!(
        Int(9),
    );
    let mut state = MyDeserializer::new(tokens.into_iter());

    let v: Result<option::Option<int>, Error> = Deserialize::deserialize(&mut state);
    println!("option:        {}", v);

    ////

    let tokens = vec!(
        Null,
    );
    let mut state = MyDeserializer::new(tokens.into_iter());

    let v: Result<option::Option<int>, Error> = Deserialize::deserialize(&mut state);
    println!("option:        {}", v);

    ////

    let tokens = vec!(
        Int(10),
    );
    let mut state = MyDeserializer::new(tokens.into_iter());

    let v: Result<json::Value, Error> = Deserialize::deserialize(&mut state);
    println!("option value:  {}", v);

    ////

    let tokens = vec!(
        Null,
    );
    let mut state = MyDeserializer::new(tokens.into_iter());

    let v: Result<json::Value, Error> = Deserialize::deserialize(&mut state);
    println!("option value:  {}", v);

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

    let v: Result<HashMap<string::String, int>, Error> = Deserialize::deserialize(&mut state);
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

    let v: Result<json::Value, Error> = Deserialize::deserialize(&mut state);
    println!("{}", v);
}


/*
use std::collections::TreeMap;
use serde::{Serialize, GatherTokens};
use serde::json;

///////////////////////////////////////////////////////////////////////////////

struct Foo {
    x: int,
    y: int,
    z: &'static str,
}

impl<S: serde::VisitorState<R>, R> serde::Serialize<S, R> for Foo {
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

impl<'a, S: serde::VisitorState<R>, R> serde::Visitor<S, R> for FooSerialize<'a> {
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
