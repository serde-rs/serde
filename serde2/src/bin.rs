extern crate serde2;

use std::collections::HashMap;
use std::option;
use std::string;

use serde2::de;
use serde2::de::{Deserialize, Deserializer};

#[derive(Show)]
pub enum Token {
    Null,
    Int(int),
    String(string::String),
    Option(bool),
    SeqStart(uint),
    MapStart(uint),
    End,
}

#[derive(Show)]
enum Error {
    SyntaxError,
    EndOfStreamError,
}

impl de::Error for Error {
    fn syntax_error() -> Error {
        Error::SyntaxError
    }

    fn end_of_stream_error() -> Error {
        Error::EndOfStreamError
    }
}

///////////////////////////////////////////////////////////////////////////////

struct MyDeserializer<Iter> {
    tokens: Iter,
    peeked: option::Option<Token>,
}

impl<Iter: Iterator<Item=Token>> MyDeserializer<Iter> {
    pub fn new(tokens: Iter) -> MyDeserializer<Iter> {
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

impl<Iter: Iterator<Item=Token>> Deserializer<Error> for MyDeserializer<Iter> {
    fn visit<
        R,
        V: de::Visitor<MyDeserializer<Iter>, R, Error>,
    >(&mut self, visitor: &mut V) -> Result<R, Error> {
        use serde2::de::Error;

        match self.next() {
            Some(Token::Null) => {
                visitor.visit_null()
            }
            Some(Token::Int(v)) => {
                visitor.visit_int(v)
            }
            Some(Token::String(v)) => {
                visitor.visit_string(v)
            }
            Some(Token::Option(is_some)) => {
                visitor.visit_option(MyOptionVisitor { d: self, is_some: is_some })
            }
            Some(Token::SeqStart(len)) => {
                visitor.visit_seq(MySeqVisitor { d: self, len: len })
            }
            Some(Token::MapStart(len)) => {
                visitor.visit_map(MyMapVisitor { d: self, len: len })
            }
            Some(Token::End) => {
                Err(Error::syntax_error())
            }
            None => {
                Err(Error::end_of_stream_error())
            }
        }
    }

    fn visit_option<
        R,
        V: de::Visitor<MyDeserializer<Iter>, R, Error>,
    >(&mut self, visitor: &mut V) -> Result<R, Error> {
        match self.peek() {
            Some(&Token::Null) => {
                self.next();
                visitor.visit_option(MyOptionVisitor {
                    d: self,
                    is_some: false,
                })
            }
            Some(&Token::Option(is_some)) => {
                self.next();
                visitor.visit_option(MyOptionVisitor {
                    d: self,
                    is_some: is_some,
                })
            }
            _ => {
                visitor.visit_option(MyOptionVisitor {
                    d: self,
                    is_some: true,
                })
            }
        }
    }
}

struct MyOptionVisitor<'a, Iter: 'a> {
    d: &'a mut MyDeserializer<Iter>,
    is_some: bool,
}

impl<
    'a,
    Iter: Iterator<Item=Token>,
> de::OptionVisitor<MyDeserializer<Iter>, Error> for MyOptionVisitor<'a, Iter> {
    fn visit<
        T: Deserialize<MyDeserializer<Iter>, Error>,
    >(&mut self) -> Result<option::Option<T>, Error> {
        if self.is_some {
            self.is_some = false;
            let value = try!(Deserialize::deserialize(self.d));
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}

struct MySeqVisitor<'a, Iter: 'a> {
    d: &'a mut MyDeserializer<Iter>,
    len: uint,
}

impl<
    'a,
    Iter: Iterator<Item=Token>,
> de::SeqVisitor<MyDeserializer<Iter>, Error> for MySeqVisitor<'a, Iter> {
    fn visit<
        T: Deserialize<MyDeserializer<Iter>, Error>
    >(&mut self) -> Result<option::Option<T>, Error> {
        use serde2::de::Error;

        match self.d.peek() {
            Some(&Token::End) => {
                self.d.next();
                Ok(None)
            }
            Some(_) => {
                self.len -= 1;
                let value = try!(Deserialize::deserialize(self.d));
                Ok(Some(value))
            }
            None => {
                Err(Error::syntax_error())
            }
        }
    }

    fn end(&mut self) -> Result<(), Error> {
        use serde2::de::Error;

        match self.d.next() {
            Some(Token::End) => Ok(()),
            Some(_) => Err(Error::syntax_error()),
            None => Err(Error::end_of_stream_error()),
        }
    }

    fn size_hint(&self) -> (uint, option::Option<uint>) {
        (self.len, Some(self.len))
    }
}

struct MyMapVisitor<'a, Iter: 'a> {
    d: &'a mut MyDeserializer<Iter>,
    len: uint,
}

impl<
    'a,
    Iter: Iterator<Item=Token>,
> de::MapVisitor<MyDeserializer<Iter>, Error> for MyMapVisitor<'a, Iter> {
    fn visit_key<
        K: Deserialize<MyDeserializer<Iter>, Error>,
    >(&mut self) -> Result<option::Option<K>, Error> {
        use serde2::de::Error;

        match self.d.peek() {
            Some(&Token::End) => {
                self.d.next();
                Ok(None)
            }
            Some(_) => {
                self.len -= 1;

                Ok(Some(try!(Deserialize::deserialize(self.d))))
            }
            None => {
                Err(Error::syntax_error())
            }
        }
    }

    fn visit_value<
        V: Deserialize<MyDeserializer<Iter>, Error>,
    >(&mut self) -> Result<V, Error> {
        Ok(try!(Deserialize::deserialize(self.d)))
    }

    fn end(&mut self) -> Result<(), Error> {
        use serde2::de::Error;

        match self.d.next() {
            Some(Token::End) => Ok(()),
            Some(_) => Err(Error::syntax_error()),
            None => Err(Error::end_of_stream_error()),
        }
    }

    fn size_hint(&self) -> (uint, option::Option<uint>) {
        (self.len, Some(self.len))
    }
}

///////////////////////////////////////////////////////////////////////////////

mod json {
    use std::collections::BTreeMap;
    use serde2::de;

    #[derive(Show)]
    pub enum Value {
        Null,
        //Bool(bool),
        Int(int),
        //String(String),
        List(Vec<Value>),
        Map(BTreeMap<String, Value>),
    }

    impl<
        D: de::Deserializer<E>,
        E: de::Error,
    > de::Deserialize<D, E> for Value {
        fn deserialize(d: &mut D) -> Result<Value, E> {
            struct Visitor;

            impl<
                D: de::Deserializer<E>,
                E: de::Error,
            > de::Visitor<D, Value, E> for Visitor {
                fn visit_null(&mut self) -> Result<Value, E> {
                    Ok(Value::Null)
                }

                fn visit_int(&mut self, v: int) -> Result<Value, E> {
                    Ok(Value::Int(v))
                }

                /*
                fn visit_string(&mut self, _d: &mut D, v: String) -> Result<Value, E> {
                    Ok(Value::String(v))
                }
                */

                fn visit_option<
                    Visitor: de::OptionVisitor<D, E>,
                >(&mut self, mut visitor: Visitor) -> Result<Value, E> {
                    match try!(visitor.visit()) {
                        Some(value) => Ok(value),
                        None => Ok(Value::Null),
                    }
                }

                fn visit_seq<
                    Visitor: de::SeqVisitor<D, E>,
                >(&mut self, mut visitor: Visitor) -> Result<Value, E> {
                    let (len, _) = visitor.size_hint();
                    let mut values = Vec::with_capacity(len);

                    loop {
                        match try!(visitor.visit()) {
                            Some(value) => {
                                values.push(value);
                            }
                            None => {
                                break;
                            }
                        }
                    }

                    Ok(Value::List(values))
                }

                fn visit_map<
                    Visitor: de::MapVisitor<D, E>,
                >(&mut self, mut visitor: Visitor) -> Result<Value, E> {
                    let mut values = BTreeMap::new();

                    loop {
                        match try!(visitor.visit()) {
                            Some((key, value)) => {
                                values.insert(key, value);
                            }
                            None => {
                                break;
                            }
                        }
                    }

                    Ok(Value::Map(values))
                }
            }

            d.visit(&mut Visitor)
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

pub fn main() {
    let tokens = vec!(
        Token::SeqStart(2),
        Token::Int(1),
        Token::Int(2),
        Token::End,
    );
    let mut state = MyDeserializer::new(tokens.into_iter());

    let v: Result<Vec<int>, Error> = Deserialize::deserialize(&mut state);
    println!("vec:           {:?}", v);

    ////

    let tokens = vec!(
        Token::SeqStart(2),
        Token::Int(3),
        Token::Int(4),
        Token::End,
    );
    let mut state = MyDeserializer::new(tokens.into_iter());

    let v: Result<(int, int), Error> = Deserialize::deserialize(&mut state);
    println!("tuple:         {:?}", v);

    ////

    let tokens = vec!(
        Token::SeqStart(2),
        Token::Int(5),
        Token::Int(6),
        Token::End,
    );
    let mut state = MyDeserializer::new(tokens.into_iter());

    let v: Result<json::Value, Error> = Deserialize::deserialize(&mut state);
    println!("value:         {:?}", v);

    ////

    let tokens = vec!(
        Token::Option(true),
        Token::Int(7),
    );
    let mut state = MyDeserializer::new(tokens.into_iter());

    let v: Result<option::Option<int>, Error> = Deserialize::deserialize(&mut state);
    println!("optiony:       {:?}", v);

    ////

    let tokens = vec!(
        Token::Option(false),
    );
    let mut state = MyDeserializer::new(tokens.into_iter());

    let v: Result<option::Option<int>, Error> = Deserialize::deserialize(&mut state);
    println!("optiony:       {:?}", v);

    ////

    let tokens = vec!(
        Token::Option(true),
        Token::Int(8),
    );
    let mut state = MyDeserializer::new(tokens.into_iter());

    let v: Result<json::Value, Error> = Deserialize::deserialize(&mut state);
    println!("optiony value: {:?}", v);

    ////

    let tokens = vec!(
        Token::Option(false),
    );
    let mut state = MyDeserializer::new(tokens.into_iter());

    let v: Result<json::Value, Error> = Deserialize::deserialize(&mut state);
    println!("optiony value: {:?}", v);

    ////

    let tokens = vec!(
        Token::Int(9),
    );
    let mut state = MyDeserializer::new(tokens.into_iter());

    let v: Result<option::Option<int>, Error> = Deserialize::deserialize(&mut state);
    println!("option:        {:?}", v);

    ////

    let tokens = vec!(
        Token::Null,
    );
    let mut state = MyDeserializer::new(tokens.into_iter());

    let v: Result<option::Option<int>, Error> = Deserialize::deserialize(&mut state);
    println!("option:        {:?}", v);

    ////

    let tokens = vec!(
        Token::Int(10),
    );
    let mut state = MyDeserializer::new(tokens.into_iter());

    let v: Result<json::Value, Error> = Deserialize::deserialize(&mut state);
    println!("option value:  {:?}", v);

    ////

    let tokens = vec!(
        Token::Null,
    );
    let mut state = MyDeserializer::new(tokens.into_iter());

    let v: Result<json::Value, Error> = Deserialize::deserialize(&mut state);
    println!("option value:  {:?}", v);

    ////

    let tokens = vec!(
        Token::MapStart(2),
        Token::String("a".to_string()),
        Token::Int(1),
        Token::String("b".to_string()),
        Token::Int(2),
        Token::End
    );
    let mut state = MyDeserializer::new(tokens.into_iter());

    let v: Result<HashMap<string::String, int>, Error> = Deserialize::deserialize(&mut state);
    println!("{:?}", v);

    ////

    let tokens = vec!(
        Token::MapStart(2),
        Token::String("a".to_string()),
        Token::Int(1),
        Token::String("b".to_string()),
        Token::Int(2),
        Token::End
    );
    let mut state = MyDeserializer::new(tokens.into_iter());

    let v: Result<json::Value, Error> = Deserialize::deserialize(&mut state);
    println!("{:?}", v);
}


/*
use std::collections::BTreeMap;
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
    println!("tokens: {:?}", s.unwrap());

    println!("json:   {:?}", json::to_string(&value).unwrap().unwrap());
    println!("");

    ////

    let value = vec!(1i, 2, 3);

    let mut s = GatherTokens::new();
    value.serialize(&mut s);
    println!("tokens: {:?}", s.unwrap());

    println!("json:   {:?}", json::to_string(&value).unwrap().unwrap());
    println!("");

    ////

    let mut value = BTreeMap::new();
    value.insert("a", 1i);
    value.insert("b", 2);
    value.insert("c", 3);

    let mut s = GatherTokens::new();
    value.serialize(&mut s);
    println!("tokens: {:?}", s.unwrap());

    println!("json:   {:?}", json::to_string(&value).unwrap().unwrap());
    println!("");

    ////

    /*
    println!("{:?}", to_format_vec(&5i));
    println!("{:?}", to_format_string(&5i));
    */

    let value = Foo { x: 1, y: 2, z: "abc" };

    let mut s = GatherTokens::new();
    value.serialize(&mut s);
    println!("tokens: {:?}", s.unwrap());

    println!("json:   {:?}", json::to_string(&value).unwrap().unwrap());
    println!("");

    ////

    let value = (1i, "abc");

    let mut s = GatherTokens::new();
    value.serialize(&mut s);
    println!("tokens: {:?}", s.unwrap());

    println!("json:   {:?}", json::to_string(&value).unwrap().unwrap());
    println!("");
}
*/
