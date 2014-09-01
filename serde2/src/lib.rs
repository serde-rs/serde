use std::io::IoResult;
use std::io;
use std::collections::TreeMap;

///////////////////////////////////////////////////////////////////////////////

pub trait Serialize<S, R> {
    fn serialize(&self, state: &mut S) -> R;
}

///////////////////////////////////////////////////////////////////////////////

pub trait Serializer<S, R> {
    fn hash<T: Serialize<S, R>>(&self, value: &T) -> R;
}

///////////////////////////////////////////////////////////////////////////////

pub trait Visitor<S, R> {
    fn visit(&mut self, state: &mut S) -> Option<R>;

    fn size_hint(&self) -> (uint, Option<uint>) {
        (0, None)
    }
}

pub trait VisitorState<R> {
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

impl<S: VisitorState<R>, R> Serialize<S, R> for int {
    fn serialize(&self, state: &mut S) -> R {
        state.serialize_int(*self)
    }
}

impl<S: VisitorState<R>, R> Serialize<S, R> for &'static str {
    fn serialize(&self, state: &mut S) -> R {
        state.serialize_str(*self)
    }
}

impl<
    S: VisitorState<R>,
    R,
    T: Serialize<S, R>
> Serialize<S, R> for Vec<T> {
    fn serialize(&self, state: &mut S) -> R {
        state.serialize_seq(self.iter())
    }
}

impl<
    S: VisitorState<R>,
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
    S: VisitorState<R>,
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
    S: VisitorState<R>,
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
    S: VisitorState<R>,
    R,
    T: Serialize<S, R>
> Serialize<S, R> for &'a T {
    fn serialize(&self, state: &mut S) -> R {
        (**self).serialize(state)
    }
}

///////////////////////////////////////////////////////////////////////////////

#[deriving(Show)]
pub enum Token {
    Int(int),
    Str(&'static str),
    SeqStart(uint),
    MapStart(uint),
    StructStart(&'static str, uint),
    End,
}

pub trait TokenState<R>: VisitorState<R> {
    fn serialize(&mut self, token: Token) -> R;
}

///////////////////////////////////////////////////////////////////////////////

pub struct GatherTokens {
    tokens: Vec<Token>,
}

impl GatherTokens {
    pub fn new() -> GatherTokens {
        GatherTokens {
            tokens: Vec::new(),
        }
    }

    pub fn unwrap(self) -> Vec<Token> {
        self.tokens
    }
}

impl TokenState<()> for GatherTokens {
    fn serialize(&mut self, token: Token) -> () {
        self.tokens.push(token);
    }
}

impl VisitorState<()> for GatherTokens {
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

pub struct FormatState<W: Writer> {
    writer: W,
}

impl<W: Writer> FormatState<W> {
    pub fn new(writer: W) -> FormatState<W> {
        FormatState {
            writer: writer,
        }
    }

    pub fn unwrap(self) -> W {
        self.writer
    }
}

impl<W: Writer> VisitorState<IoResult<()>> for FormatState<W> {
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

impl VisitorState<Json> for JsonSerializer {
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

pub fn to_format_vec<
    T: Serialize<FormatState<io::MemWriter>, IoResult<()>>
>(value: &T) -> IoResult<Vec<u8>> {
    let writer = io::MemWriter::new();
    let mut state = FormatState::new(writer);
    try!(value.serialize(&mut state));
    Ok(state.unwrap().unwrap())
}

pub fn to_format_string<
    T: Serialize<FormatState<io::MemWriter>, IoResult<()>>
>(value: &T) -> IoResult<Result<String, Vec<u8>>> {
    let vec = try!(to_format_vec(value));
    Ok(String::from_utf8(vec))
}
