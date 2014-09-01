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
    fn visit_null(&mut self) -> R;

    fn visit_bool(&mut self, v: bool) -> R;

    #[inline]
    fn visit_int(&mut self, v: int) -> R {
        self.visit_i64(v as i64)
    }

    #[inline]
    fn visit_i8(&mut self, v: i8) -> R {
        self.visit_i64(v as i64)
    }

    #[inline]
    fn visit_i16(&mut self, v: i16) -> R {
        self.visit_i64(v as i64)
    }

    #[inline]
    fn visit_i32(&mut self, v: i32) -> R {
        self.visit_i64(v as i64)
    }

    #[inline]
    fn visit_i64(&mut self, v: i64) -> R;

    #[inline]
    fn visit_uint(&mut self, v: uint) -> R {
        self.visit_u64(v as u64)
    }

    #[inline]
    fn visit_u8(&mut self, v: u8) -> R {
        self.visit_u64(v as u64)
    }

    #[inline]
    fn visit_u16(&mut self, v: u16) -> R {
        self.visit_u64(v as u64)
    }

    #[inline]
    fn visit_u32(&mut self, v: u32) -> R {
        self.visit_u64(v as u64)
    }

    #[inline]
    fn visit_u64(&mut self, v: u64) -> R;

    #[inline]
    fn visit_f32(&mut self, v: f32) -> R {
        self.visit_f64(v as f64)
    }

    fn visit_f64(&mut self, v: f64) -> R;

    fn visit_char(&mut self, value: char) -> R;

    fn visit_str(&mut self, value: &'static str) -> R;

    fn visit_seq<
        V: Visitor<Self, R>
    >(&mut self, visitor: V) -> R;

    fn visit_named_seq<
        V: Visitor<Self, R>
    >(&mut self, _name: &'static str, visitor: V) -> R {
        self.visit_seq(visitor)
    }

    fn visit_enum<
        V: Visitor<Self, R>
    >(&mut self, _name: &'static str, _variant: &'static str, visitor: V) -> R {
        self.visit_seq(visitor)
    }

    fn visit_seq_elt<
        T: Serialize<Self, R>
    >(&mut self, value: T) -> R;

    fn visit_map<
        V: Visitor<Self, R>
    >(&mut self, visitor: V) -> R;

    fn visit_named_map<
        V: Visitor<Self, R>
    >(&mut self, _name: &'static str, visitor: V) -> R {
        self.visit_map(visitor)
    }

    fn visit_map_elt<
        K: Serialize<Self, R>,
        V: Serialize<Self, R>
    >(&mut self, key: K, value: V) -> R;
}

///////////////////////////////////////////////////////////////////////////////

impl<S: VisitorState<R>, R> Serialize<S, R> for int {
    fn serialize(&self, state: &mut S) -> R {
        state.visit_int(*self)
    }
}

impl<S: VisitorState<R>, R> Serialize<S, R> for &'static str {
    fn serialize(&self, state: &mut S) -> R {
        state.visit_str(*self)
    }
}

///////////////////////////////////////////////////////////////////////////////

struct IteratorVisitor<Iter> {
    iter: Iter,
}

impl<T, Iter: Iterator<T>> IteratorVisitor<Iter> {
    pub fn new(iter: Iter) -> IteratorVisitor<Iter> {
        IteratorVisitor {
            iter: iter,
        }
    }
}

impl<
    T: Serialize<S, R>,
    Iter: Iterator<T>,
    S: VisitorState<R>,
    R
> Visitor<S, R> for IteratorVisitor<Iter> {
    fn visit(&mut self, state: &mut S) -> Option<R> {
        match self.iter.next() {
            Some(value) => Some(state.visit_seq_elt(value)),
            None => None
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<
    S: VisitorState<R>,
    R,
    T: Serialize<S, R>
> Serialize<S, R> for Vec<T> {
    fn serialize(&self, state: &mut S) -> R {
        state.visit_seq(self.iter())
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<
    S: VisitorState<R>,
    R,
    K: Serialize<S, R> + Ord,
    V: Serialize<S, R>
> Serialize<S, R> for TreeMap<K, V> {
    fn serialize(&self, state: &mut S) -> R {
        state.visit_map(self.iter())
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
        state.visit_seq(Tuple2Serialize { value: self, state: 0 })
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
                Some(state.visit_seq_elt(true, value))
            }
            1 => {
                self.state += 1;
                let (_, ref value) = *self.value;
                Some(state.visit_seq_elt(false, value))
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
    Null,
    Bool(bool),
    Int(int),
    I64(i64),
    U64(u64),
    F64(f64),
    Char(char),
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
    fn visit_null(&mut self) -> () {
        self.serialize(Null)
    }

    fn visit_bool(&mut self, value: bool) -> () {
        self.serialize(Bool(value))
    }

    fn visit_i64(&mut self, value: i64) -> () {
        self.serialize(I64(value))
    }

    fn visit_u64(&mut self, value: u64) -> () {
        self.serialize(U64(value))
    }

    fn visit_f64(&mut self, value: f64) -> () {
        self.serialize(F64(value))
    }

    fn visit_char(&mut self, value: char) -> () {
        self.serialize(Char(value))
    }

    fn visit_str(&mut self, value: &'static str) -> () {
        self.serialize(Str(value))
    }

    fn visit_seq<
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

    fn visit_enum<
        V: Visitor<GatherTokens, ()>
    >(&mut self, _name: &'static str, _variant: &'static str, visitor: V) -> () {
        self.visit_seq(visitor)
    }

    fn visit_map<
        V: Visitor<GatherTokens, ()>
    >(&mut self, mut visitor: V) -> () {
        let (len, _) = visitor.size_hint();
        self.serialize(MapStart(len));
        loop {
            match visitor.visit(self) {
                Some(()) => { }
                None => { break; }
            }
        }
        self.serialize(End)
    }

    fn visit_named_map<
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
    fn visit_null(&mut self) -> IoResult<()> {
        write!(self.writer, "()")
    }

    fn visit_bool(&mut self, value: bool) -> IoResult<()> {
        write!(self.writer, "{}", value)
    }

    fn visit_i64(&mut self, value: i64) -> IoResult<()> {
        write!(self.writer, "{}", value)
    }

    fn visit_u64(&mut self, value: u64) -> IoResult<()> {
        write!(self.writer, "{}", value)
    }

    fn visit_f64(&mut self, value: f64) -> IoResult<()> {
        write!(self.writer, "{}", value)
    }

    fn visit_char(&mut self, value: char) -> io::IoResult<()> {
        write!(self.writer, "{}", value)
    }

    fn visit_str(&mut self, value: &'static str) -> IoResult<()> {
        write!(self.writer, "{}", value)
    }

    fn visit_seq<
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

    fn visit_enum<
        V: Visitor<FormatState<W>, IoResult<()>>
    >(&mut self, _name: &'static str, _variant: &'static str, visitor: V) -> IoResult<()> {
        self.visit_tuple(visitor)
    }

    fn visit_map_elt<
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

    fn visit_map<
        V: Visitor<FormatState<W>, IoResult<()>>
    >(&mut self, mut visitor: V) -> IoResult<()> {
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
