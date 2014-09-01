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
    fn visit_int(&mut self, value: int) -> Json {
        Integer(value)
    }

    fn visit_str(&mut self, value: &'static str) -> Json {
        String(value.to_string())
    }

    fn visit_seq<
        T: Serialize<JsonSerializer, Json>,
        Iter: Iterator<T>
    >(&mut self, mut iter: Iter) -> Json {
        let (len, _) = iter.size_hint();
        let mut v = Vec::with_capacity(len);

        let mut first = true;
        for elt in iter {
            v.push(self.visit_seq_elt(first, elt));
            first = false;
        }

        Array(v)
    }

    fn visit_seq_elt<
        T: Serialize<JsonSerializer, Json>
    >(&mut self, _first: bool, value: T) -> Json {
        value.serialize(self)
    }

    fn visit_tuple<
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

    fn visit_tuple_struct<
        V: Visitor<JsonSerializer, Json>
    >(&mut self, _name: &'static str, visitor: V) -> Json {
        self.visit_tuple(visitor)
    }

    fn visit_enum<
        V: Visitor<JsonSerializer, Json>
    >(&mut self, _name: &'static str, _variant: &'static str, visitor: V) -> Json {
        self.visit_tuple(visitor)
    }

    fn visit_map<
        K: Serialize<JsonSerializer, Json>,
        V: Serialize<JsonSerializer, Json>,
        Iter: Iterator<(K, V)>
    >(&mut self, mut iter: Iter) -> Json {
        let mut v = TreeMap::new();
        let mut first = true;

        for (key, value) in iter {
            let value = self.visit_map_elt(first, key, value);
            v.insert(self.key.take().unwrap(), value);
            first = false;
        }

        Object(v)
    }

    fn visit_map_elt<
        K: Serialize<JsonSerializer, Json>,
        V: Serialize<JsonSerializer, Json>
    >(&mut self, _first: bool, key: K, value: V) -> Json {
        match key.serialize(self) {
            String(key) => { self.key = Some(key); }
            _ => { fail!() }
        }
        value.serialize(self)
    }

    fn visit_struct<
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
