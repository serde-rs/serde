use std::io::IoResult;
use std::io;
use std::collections::TreeMap;

///////////////////////////////////////////////////////////////////////////////

trait Serialize<S, R> {
    fn serialize(&self, state: &mut S) -> R;
}

///////////////////////////////////////////////////////////////////////////////

trait Serializer<S, R> {
    fn hash<T: Serialize<S, R>>(&self, value: &T) -> R;
}

///////////////////////////////////////////////////////////////////////////////

trait Visitor<S, R> {
    fn visit(&mut self, state: &mut S) -> Option<R>;

    fn size_hint(&self) -> (uint, Option<uint>) {
        (0, None)
    }
}

trait SerializeState<R> {
    fn serialize_int(&mut self, value: int) -> R;

    fn serialize_str(&mut self, value: &'static str) -> R;

    fn serialize_seq<
        T: Serialize<Self, R>,
        Iter: Iterator<T>
    >(&mut self, iter: Iter) -> R;

    fn serialize_seq_elt<
        T: Serialize<Self, R>
    >(&mut self, first: bool, value: T) -> R;

    fn serialize_tuple<
        V: Visitor<Self, R>
    >(&mut self, visitor: V) -> R;

    fn serialize_tuple_struct<
        V: Visitor<Self, R>
    >(&mut self, name: &'static str, mut visitor: V) -> R;

    fn serialize_enum<
        V: Visitor<Self, R>
    >(&mut self, name: &'static str, variant: &'static str, visitor: V) -> R;

    fn serialize_map<
        K: Serialize<Self, R>,
        V: Serialize<Self, R>,
        Iter: Iterator<(K, V)>
    >(&mut self, iter: Iter) -> R;

    fn serialize_map_elt<
        K: Serialize<Self, R>,
        V: Serialize<Self, R>
    >(&mut self, first: bool, key: K, value: V) -> R;

    fn serialize_struct<
        V: Visitor<Self, R>
    >(&mut self, name: &'static str, visitor: V) -> R;
}

///////////////////////////////////////////////////////////////////////////////

impl<S: SerializeState<R>, R> Serialize<S, R> for int {
    fn serialize(&self, state: &mut S) -> R {
        state.serialize_int(*self)
    }
}

impl<S: SerializeState<R>, R> Serialize<S, R> for &'static str {
    fn serialize(&self, state: &mut S) -> R {
        state.serialize_str(*self)
    }
}

impl<
    S: SerializeState<R>,
    R,
    T: Serialize<S, R>
> Serialize<S, R> for Vec<T> {
    fn serialize(&self, state: &mut S) -> R {
        state.serialize_seq(self.iter())
    }
}

impl<
    S: SerializeState<R>,
    R,
    K: Serialize<S, R> + Ord,
    V: Serialize<S, R>
> Serialize<S, R> for TreeMap<K, V> {
    fn serialize(&self, state: &mut S) -> R {
        state.serialize_map(self.iter())
    }
}

impl<
    'a,
    S: SerializeState<R>,
    R,
    T0: Serialize<S, R>,
    T1: Serialize<S, R>
> Serialize<S, R> for (T0, T1) {
    fn serialize(&self, state: &mut S) -> R {
        state.serialize_tuple(Tuple2Serialize { value: self, state: 0 })
    }
}

struct Tuple2Serialize<'a, T0, T1> {
    value: &'a (T0, T1),
    state: uint,
}

impl<
    'a,
    S: SerializeState<R>,
    R,
    T0: Serialize<S, R>,
    T1: Serialize<S, R>
> Visitor<S, R> for Tuple2Serialize<'a, T0, T1> {
    fn visit(&mut self, state: &mut S) -> Option<R> {
        match self.state {
            0 => {
                self.state += 1;
                let (ref value, _) = *self.value;
                Some(state.serialize_seq_elt(true, value))
            }
            1 => {
                self.state += 1;
                let (_, ref value) = *self.value;
                Some(state.serialize_seq_elt(false, value))
            }
            _ => {
                None
            }
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        let size = 2 - self.state;
        (size, Some(size))
    }
}

impl<
    'a,
    S: SerializeState<R>,
    R,
    T: Serialize<S, R>
> Serialize<S, R> for &'a T {
    fn serialize(&self, state: &mut S) -> R {
        (**self).serialize(state)
    }
}

///////////////////////////////////////////////////////////////////////////////

#[deriving(Show)]
enum Token {
    Int(int),
    Str(&'static str),
    SeqStart(uint),
    MapStart(uint),
    StructStart(&'static str, uint),
    End,
}

trait TokenState<R>: SerializeState<R> {
    fn serialize(&mut self, token: Token) -> R;
}

///////////////////////////////////////////////////////////////////////////////

struct GatherTokens {
    tokens: Vec<Token>,
}

impl GatherTokens {
    fn new() -> GatherTokens {
        GatherTokens {
            tokens: Vec::new(),
        }
    }

    fn unwrap(self) -> Vec<Token> {
        self.tokens
    }
}

impl TokenState<()> for GatherTokens {
    fn serialize(&mut self, token: Token) -> () {
        self.tokens.push(token);
    }
}

impl SerializeState<()> for GatherTokens {
    fn serialize_int(&mut self, value: int) -> () {
        self.serialize(Int(value))
    }

    fn serialize_str(&mut self, value: &'static str) -> () {
        self.serialize(Str(value))
    }

    fn serialize_seq<
        T: Serialize<GatherTokens, ()>,
        Iter: Iterator<T>
    >(&mut self, mut iter: Iter) -> () {
        let (len, _) = iter.size_hint();
        self.serialize(SeqStart(len));
        let mut first = false;
        for elt in iter {
            self.serialize_seq_elt(first, elt);

            if first {
                first = false;
            }
        }
        self.serialize(End)
    }

    fn serialize_seq_elt<
        T: Serialize<GatherTokens, ()>
    >(&mut self, _first: bool, value: T) -> () {
        value.serialize(self);
    }

    fn serialize_tuple<
        V: Visitor<GatherTokens, ()>
    >(&mut self, mut visitor: V) -> () {
        let (len, _) = visitor.size_hint();
        self.tokens.push(SeqStart(len));
        loop {
            match visitor.visit(self) {
                Some(()) => { }
                None => { break; }
            }
        }
        self.tokens.push(End)
    }

    fn serialize_tuple_struct<
        V: Visitor<GatherTokens, ()>
    >(&mut self, _name: &'static str, visitor: V) -> () {
        self.serialize_tuple(visitor)
    }

    fn serialize_enum<
        V: Visitor<GatherTokens, ()>
    >(&mut self, _name: &'static str, _variant: &'static str, visitor: V) -> () {
        self.serialize_tuple(visitor)
    }

    fn serialize_map<
        K: Serialize<GatherTokens, ()>,
        V: Serialize<GatherTokens, ()>,
        Iter: Iterator<(K, V)>
    >(&mut self, mut iter: Iter) -> () {
        let (len, _) = iter.size_hint();
        self.serialize(MapStart(len));
        let mut first = true;
        for (key, value) in iter {
            self.serialize_map_elt(first, key, value);
            first = false;
        }
        self.serialize(End)
    }

    fn serialize_map_elt<
        K: Serialize<GatherTokens, ()>,
        V: Serialize<GatherTokens, ()>
    >(&mut self, _first: bool, key: K, value: V) -> () {
        key.serialize(self);
        value.serialize(self);
    }

    fn serialize_struct<
        V: Visitor<GatherTokens, ()>
    >(&mut self, name: &'static str, mut visitor: V) -> () {
        let (len, _) = visitor.size_hint();
        self.serialize(StructStart(name, len));
        loop {
            match visitor.visit(self) {
                Some(()) => { }
                None => { break; }
            }
        }
        self.serialize(End)
    }
}

///////////////////////////////////////////////////////////////////////////////

struct FormatState<W: Writer> {
    writer: W,
}

impl<W: Writer> FormatState<W> {
    fn new(writer: W) -> FormatState<W> {
        FormatState {
            writer: writer,
        }
    }
}

impl<W: Writer> SerializeState<IoResult<()>> for FormatState<W> {
    fn serialize_int(&mut self, value: int) -> IoResult<()> {
        write!(self.writer, "{}", value)
    }

    fn serialize_str(&mut self, value: &'static str) -> IoResult<()> {
        write!(self.writer, "{}", value)
    }

    fn serialize_seq<
        T: Serialize<FormatState<W>, IoResult<()>>,
        Iter: Iterator<T>
    >(&mut self, mut iter: Iter) -> IoResult<()> {
        try!(write!(self.writer, "["));
        let mut first = true;
        for elt in iter {
            try!(self.serialize_seq_elt(first, elt));
            first = false;

        }
        write!(self.writer, "]")
    }

    fn serialize_seq_elt<
        T: Serialize<FormatState<W>, IoResult<()>>
    >(&mut self, first: bool, value: T) -> IoResult<()> {
        if !first {
            try!(write!(self.writer, ", "));
        }

        value.serialize(self)
    }

    fn serialize_tuple<
        V: Visitor<FormatState<W>, IoResult<()>>
    >(&mut self, mut visitor: V) -> IoResult<()> {
        try!(write!(self.writer, "["));
        loop {
            match visitor.visit(self) {
                Some(Ok(())) => { }
                Some(Err(err)) => { return Err(err); }
                None => { break; }
            }
        }
        write!(self.writer, "]")
    }

    fn serialize_tuple_struct<
        V: Visitor<FormatState<W>, IoResult<()>>
    >(&mut self, _name: &'static str, visitor: V) -> IoResult<()> {
        self.serialize_tuple(visitor)
    }


    fn serialize_enum<
        V: Visitor<FormatState<W>, IoResult<()>>
    >(&mut self, _name: &'static str, _variant: &'static str, visitor: V) -> IoResult<()> {
        self.serialize_tuple(visitor)
    }

    fn serialize_map<
        K: Serialize<FormatState<W>, IoResult<()>>,
        V: Serialize<FormatState<W>, IoResult<()>>,
        Iter: Iterator<(K, V)>
    >(&mut self, mut iter: Iter) -> IoResult<()> {
        try!(write!(self.writer, "{{"));
        let mut first = true;
        for (key, value) in iter {
            try!(self.serialize_map_elt(first, &key, &value))
            first = false;
        }
        write!(self.writer, "}}")
    }

    fn serialize_map_elt<
        K: Serialize<FormatState<W>, IoResult<()>>,
        V: Serialize<FormatState<W>, IoResult<()>>
    >(&mut self, first: bool, key: K, value: V) -> IoResult<()> {
        if !first {
            try!(write!(self.writer, ", "));
        }

        try!(key.serialize(self));
        try!(write!(self.writer, ": "));
        value.serialize(self)
    }

    fn serialize_struct<
        V: Visitor<FormatState<W>, IoResult<()>>
    >(&mut self, _name: &'static str, mut visitor: V) -> IoResult<()> {
        try!(write!(self.writer, "{{"));
        loop {
            match visitor.visit(self) {
                Some(Ok(())) => { }
                Some(Err(err)) => { return Err(err); }
                None => { break; }
            }
        }
        write!(self.writer, "}}")
    }
}

///////////////////////////////////////////////////////////////////////////////

#[deriving(PartialEq)]
pub enum Json {
    Integer(int),
    String(String),
    Array(Vec<Json>),
    Object(TreeMap<String, Json>),
}

pub struct JsonSerializer {
    key: Option<String>
}

impl JsonSerializer {
    pub fn new() -> JsonSerializer {
        JsonSerializer {
            key: None
        }
    }
}

impl SerializeState<Json> for JsonSerializer {
    fn serialize_int(&mut self, value: int) -> Json {
        Integer(value)
    }

    fn serialize_str(&mut self, value: &'static str) -> Json {
        String(value.to_string())
    }

    fn serialize_seq<
        T: Serialize<JsonSerializer, Json>,
        Iter: Iterator<T>
    >(&mut self, mut iter: Iter) -> Json {
        let (len, _) = iter.size_hint();
        let mut v = Vec::with_capacity(len);

        let mut first = true;
        for elt in iter {
            v.push(self.serialize_seq_elt(first, elt));
            first = false;
        }

        Array(v)
    }

    fn serialize_seq_elt<
        T: Serialize<JsonSerializer, Json>
    >(&mut self, _first: bool, value: T) -> Json {
        value.serialize(self)
    }

    fn serialize_tuple<
        V: Visitor<JsonSerializer, Json>
    >(&mut self, mut visitor: V) -> Json {
        let (len, _) = visitor.size_hint();
        let mut v = Vec::with_capacity(len);

        loop {
            match visitor.visit(self) {
                Some(value) => { v.push(value); }
                None => { break; }
            }
        }

        Array(v)
    }

    fn serialize_tuple_struct<
        V: Visitor<JsonSerializer, Json>
    >(&mut self, _name: &'static str, visitor: V) -> Json {
        self.serialize_tuple(visitor)
    }

    fn serialize_enum<
        V: Visitor<JsonSerializer, Json>
    >(&mut self, _name: &'static str, _variant: &'static str, visitor: V) -> Json {
        self.serialize_tuple(visitor)
    }

    fn serialize_map<
        K: Serialize<JsonSerializer, Json>,
        V: Serialize<JsonSerializer, Json>,
        Iter: Iterator<(K, V)>
    >(&mut self, mut iter: Iter) -> Json {
        let mut v = TreeMap::new();
        let mut first = true;

        for (key, value) in iter {
            let value = self.serialize_map_elt(first, key, value);
            v.insert(self.key.take().unwrap(), value);
            first = false;
        }

        Object(v)
    }

    fn serialize_map_elt<
        K: Serialize<JsonSerializer, Json>,
        V: Serialize<JsonSerializer, Json>
    >(&mut self, _first: bool, key: K, value: V) -> Json {
        match key.serialize(self) {
            String(key) => { self.key = Some(key); }
            _ => { fail!() }
        }
        value.serialize(self)
    }

    fn serialize_struct<
        V: Visitor<JsonSerializer, Json>
    >(&mut self, _name: &'static str, mut visitor: V) -> Json {
        let mut v = TreeMap::new();

        loop {
            match visitor.visit(self) {
                Some(value) => { v.insert(self.key.take().unwrap(), value); }
                None => { break; }
            }
        }

        Object(v)
    }
}

///////////////////////////////////////////////////////////////////////////////

/*
pub fn to_format_vec<
    W: Writer,
    T: Serialize<FormatState<W>, IoResult<()>>
>(value: &T) -> IoResult<Vec<u8>> {
    let mut writer = io::MemWriter::new();
    {
        let mut w = FormatState::new(writer.by_ref());
        try!(value.serialize(&mut w));
    }
    Ok(writer.unwrap())
}

pub fn to_format_string<
    T: Serialize<FormatState<io::MemWriter>, IoResult<()>>
>(value: &T) -> IoResult<Result<String, Vec<u8>>> {
    let vec = try!(to_format_vec(value));
    Ok(String::from_utf8(vec))
}
*/

///////////////////////////////////////////////////////////////////////////////

struct Foo {
    x: int,
    y: int,
    z: &'static str,
}

impl<S: SerializeState<R>, R> Serialize<S, R> for Foo {
    fn serialize(&self, state: &mut S) -> R {
        let mut x = FooSerialize {
            value: self,
            state: 0,
            foo_state: state,
        };
        x.foo_state.serialize_struct("Foo", &mut x)
        /*
        state.serialize_struct("Foo", FooSerialize {
            value: self,
            state: 0,
            foo_state: state,
        })
        */
    }
}

struct FooSerialize<'a, S> {
    value: &'a Foo,
    state: uint,
    foo_state: &'a mut S,
}

impl<'a, S: SerializeState<R>, R> Visitor<S, R> for FooSerialize<'a, S> {
    fn visit(&mut self, state: &mut S) -> Option<R> {
        match self.state {
            0 => {
                self.state += 1;
                Some(state.serialize_map_elt(true, "x", &self.value.x))
            }
            1 => {
                self.state += 1;
                Some(state.serialize_map_elt(false, "y", &self.value.y))
            }
            2 => {
                self.state += 1;
                Some(state.serialize_map_elt(false, "z", &self.value.z))
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

fn main() {

    let value = 5i;

    let mut s = GatherTokens::new();
    value.serialize(&mut s);
    println!("tokens: {}", s.unwrap());

    value.serialize(&mut FormatState::new(io::stdout())).unwrap();
    println!("");

    ////

    let value = vec!(1i, 2, 3);

    let mut s = GatherTokens::new();
    value.serialize(&mut s);
    println!("tokens: {}", s.unwrap());

    value.serialize(&mut FormatState::new(io::stdout())).unwrap();
    println!("");

    ////

    let mut value = TreeMap::new();
    value.insert("a", 1i);
    value.insert("b", 2);
    value.insert("c", 3);

    let mut s = GatherTokens::new();
    value.serialize(&mut s);
    println!("tokens: {}", s.unwrap());

    value.serialize(&mut FormatState::new(io::stdout())).unwrap();
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

    value.serialize(&mut FormatState::new(io::stdout())).unwrap();
    println!("");

    ////

    let value = (1i, "abc");

    let mut s = GatherTokens::new();
    value.serialize(&mut s);
    println!("tokens: {}", s.unwrap());

    value.serialize(&mut FormatState::new(io::stdout())).unwrap();
    println!("");
}
