//! Generic serialization framework.

pub mod impls;

///////////////////////////////////////////////////////////////////////////////

pub trait Serialize {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer;
}

///////////////////////////////////////////////////////////////////////////////

pub trait Serializer {
    type Error;

    /// `visit_bool` serializes a `bool` value.
    fn visit_bool(&mut self, v: bool) -> Result<(), Self::Error>;

    /// `visit_isize` serializes a `isize` value. By default it casts the value to a `i64` and
    /// passes it to the `visit_i64` method.
    #[inline]
    fn visit_isize(&mut self, v: isize) -> Result<(), Self::Error> {
        self.visit_i64(v as i64)
    }

    /// `visit_i8` serializes a `i8` value. By default it casts the value to a `i64` and
    /// passes it to the `visit_i64` method.
    #[inline]
    fn visit_i8(&mut self, v: i8) -> Result<(), Self::Error> {
        self.visit_i64(v as i64)
    }

    /// `visit_i16` serializes a `i16` value. By default it casts the value to a `i64` and
    /// passes it to the `visit_i64` method.
    #[inline]
    fn visit_i16(&mut self, v: i16) -> Result<(), Self::Error> {
        self.visit_i64(v as i64)
    }

    /// `visit_i32` serializes a `i32` value. By default it casts the value to a `i64` and
    /// passes it to the `visit_i64` method.
    #[inline]
    fn visit_i32(&mut self, v: i32) -> Result<(), Self::Error> {
        self.visit_i64(v as i64)
    }

    /// `visit_i64` serializes a `i64` value.
    #[inline]
    fn visit_i64(&mut self, v: i64) -> Result<(), Self::Error>;

    /// `visit_usize` serializes a `usize` value. By default it casts the value to a `u64` and
    /// passes it to the `visit_u64` method.
    #[inline]
    fn visit_usize(&mut self, v: usize) -> Result<(), Self::Error> {
        self.visit_u64(v as u64)
    }

    /// `visit_u8` serializes a `u8` value. By default it casts the value to a `u64` and passes
    /// it to the `visit_u64` method.
    #[inline]
    fn visit_u8(&mut self, v: u8) -> Result<(), Self::Error> {
        self.visit_u64(v as u64)
    }

    /// `visit_u32` serializes a `u32` value. By default it casts the value to a `u64` and passes
    /// it to the `visit_u64` method.
    #[inline]
    fn visit_u16(&mut self, v: u16) -> Result<(), Self::Error> {
        self.visit_u64(v as u64)
    }

    /// `visit_u32` serializes a `u32` value. By default it casts the value to a `u64` and passes
    /// it to the `visit_u64` method.
    #[inline]
    fn visit_u32(&mut self, v: u32) -> Result<(), Self::Error> {
        self.visit_u64(v as u64)
    }

    /// `visit_u64` serializes a `u64` value.
    #[inline]
    fn visit_u64(&mut self, v: u64) -> Result<(), Self::Error>;

    /// `visit_f32` serializes a `f32` value. By default it casts the value to a `f64` and passes
    /// it to the `visit_f64` method.
    #[inline]
    fn visit_f32(&mut self, v: f32) -> Result<(), Self::Error> {
        self.visit_f64(v as f64)
    }

    /// `visit_f64` serializes a `f64` value.
    fn visit_f64(&mut self, v: f64) -> Result<(), Self::Error>;

    /// `visit_char` serializes a character. By default it serializes it as a `&str` containing a
    /// single character.
    #[inline]
    fn visit_char(&mut self, v: char) -> Result<(), Self::Error> {
        // FIXME: this allocation is required in order to be compatible with stable rust, which
        // doesn't support encoding a `char` into a stack buffer.
        self.visit_str(&v.to_string())
    }

    /// `visit_str` serializes a `&str`.
    fn visit_str(&mut self, value: &str) -> Result<(), Self::Error>;

    /// `visit_bytes` is a hook that enables those serialization formats that support serializing
    /// byte slices separately from generic arrays. By default it serializes as a regular array.
    #[inline]
    fn visit_bytes(&mut self, value: &[u8]) -> Result<(), Self::Error> {
        self.visit_seq(impls::SeqIteratorVisitor::new(value.iter(), Some(value.len())))
    }

    fn visit_unit(&mut self) -> Result<(), Self::Error>;

    #[inline]
    fn visit_named_unit(&mut self, _name: &str) -> Result<(), Self::Error> {
        self.visit_unit()
    }

    #[inline]
    fn visit_enum_unit(&mut self,
                       _name: &str,
                       _variant: &str) -> Result<(), Self::Error> {
        self.visit_unit()
    }

    fn visit_none(&mut self) -> Result<(), Self::Error>;

    fn visit_some<V>(&mut self, value: V) -> Result<(), Self::Error>
        where V: Serialize;

    fn visit_seq<V>(&mut self, visitor: V) -> Result<(), Self::Error>
        where V: SeqVisitor;

    #[inline]
    fn visit_named_seq<V>(&mut self,
                          _name: &'static str,
                          visitor: V) -> Result<(), Self::Error>
        where V: SeqVisitor,
    {
        self.visit_seq(visitor)
    }

    #[inline]
    fn visit_enum_seq<V>(&mut self,
                         _name: &'static str,
                         _variant: &'static str,
                         visitor: V) -> Result<(), Self::Error>
        where V: SeqVisitor,
    {
        self.visit_seq(visitor)
    }

    fn visit_seq_elt<T>(&mut self, value: T) -> Result<(), Self::Error>
        where T: Serialize;

    fn visit_map<V>(&mut self, visitor: V) -> Result<(), Self::Error>
        where V: MapVisitor;

    #[inline]
    fn visit_named_map<V>(&mut self,
                          _name: &'static str,
                          visitor: V) -> Result<(), Self::Error>
        where V: MapVisitor,
    {
        self.visit_map(visitor)
    }

    #[inline]
    fn visit_enum_map<V>(&mut self,
                          _name: &'static str,
                          _variant: &'static str,
                          visitor: V) -> Result<(), Self::Error>
        where V: MapVisitor,
    {
        self.visit_map(visitor)
    }

    fn visit_map_elt<K, V>(&mut self, key: K, value: V) -> Result<(), Self::Error>
        where K: Serialize,
              V: Serialize;

    /// Specify a format string for the serializer.
    ///
    /// The serializer format is used to determine which format
    /// specific field attributes should be used with the serializer.
    fn format() -> &'static str {
        ""
    }
}

pub trait SeqVisitor {
    fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
        where S: Serializer;

    /// Return the length of the sequence if known.
    #[inline]
    fn len(&self) -> Option<usize> {
        None
    }
}

pub trait MapVisitor {
    fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
        where S: Serializer;

    /// Return the length of the map if known.
    #[inline]
    fn len(&self) -> Option<usize> {
        None
    }
}
