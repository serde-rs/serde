//! Generic data structure serialization framework.
//!
//! The two most important traits in this module are `Serialize` and
//! `Serializer`.
//!
//!  - **A type that implements `Serialize` is a data structure** that can be
//!    serialized to any data format supported by Serde, and conversely
//!  - **A type that implements `Serializer` is a data format** that can
//!    serialize any data structure supported by Serde.
//!
//! # The Serialize trait
//!
//! Serde provides `Serialize` implementations for many Rust primitive and
//! standard library types. The complete list is below. All of these can be
//! serialized using Serde out of the box.
//!
//! Additionally, Serde provides a procedural macro called `serde_derive` to
//! automatically generate `Serialize` implementations for structs and enums in
//! your program. See the [codegen section of the manual][codegen] for how to
//! use this.
//!
//! In rare cases it may be necessary to implement `Serialize` manually for some
//! type in your program. See the [Implementing `Serialize`][impl-serialize]
//! section of the manual for more about this.
//!
//! Third-party crates may provide `Serialize` implementations for types that
//! they expose. For example the `linked-hash-map` crate provides a
//! `LinkedHashMap<K, V>` type that is serializable by Serde because the crate
//! provides an implementation of `Serialize` for it.
//!
//! # The Serializer trait
//!
//! `Serializer` implementations are provided by third-party crates, for example
//! [`serde_json`][serde_json], [`serde_yaml`][serde_yaml] and
//! [`bincode`][bincode].
//!
//! A partial list of well-maintained formats is given on the [Serde
//! website][data-formats].
//!
//! # Implementations of Serialize provided by Serde
//!
//!  - **Primitive types**:
//!    - bool
//!    - isize, i8, i16, i32, i64
//!    - usize, u8, u16, u32, u64
//!    - f32, f64
//!    - char
//!    - str
//!    - &T and &mut T
//!  - **Compound types**:
//!    - [T]
//!    - [T; 0] through [T; 32]
//!    - tuples up to size 16
//!  - **Common standard library types**:
//!    - String
//!    - Option\<T\>
//!    - Result\<T, E\>
//!    - PhantomData\<T\>
//!  - **Wrapper types**:
//!    - Box\<T\>
//!    - Rc\<T\>
//!    - Arc\<T\>
//!    - Cow\<'a, T\>
//!    - Cell\<T\>
//!    - RefCell\<T\>
//!    - Mutex\<T\>
//!    - RwLock\<T\>
//!  - **Collection types**:
//!    - BTreeMap\<K, V\>
//!    - BTreeSet\<T\>
//!    - BinaryHeap\<T\>
//!    - HashMap\<K, V, H\>
//!    - HashSet\<T, H\>
//!    - LinkedList\<T\>
//!    - VecDeque\<T\>
//!    - Vec\<T\>
//!    - EnumSet\<T\> (unstable)
//!  - **Miscellaneous standard library types**:
//!    - Duration
//!    - Path
//!    - PathBuf
//!    - Range\<T\>
//!    - NonZero\<T\> (unstable)
//!  - **Net types**:
//!    - IpAddr
//!    - Ipv4Addr
//!    - Ipv6Addr
//!    - SocketAddr
//!    - SocketAddrV4
//!    - SocketAddrV6
//!
//! [codegen]: https://serde.rs/codegen.html
//! [impl-serialize]: https://serde.rs/impl-serialize.html
//! [serde_json]: https://github.com/serde-rs/json
//! [serde_yaml]: https://github.com/dtolnay/serde-yaml
//! [bincode]: https://github.com/TyOverby/bincode
//! [data-formats]: https://serde.rs/#data-formats

#[cfg(feature = "std")]
use std::error;
#[cfg(not(feature = "std"))]
use error;

#[cfg(all(feature = "collections", not(feature = "std")))]
use collections::string::String;
use core::fmt::Display;
#[cfg(any(feature = "std", feature = "collections"))]
use core::fmt::Write;
use core::iter::IntoIterator;

mod impls;
mod impossible;

// Helpers used by generated code. Not public API.
#[doc(hidden)]
pub mod private;
#[cfg(any(feature = "std", feature = "collections"))]
mod content;

pub use self::impossible::Impossible;

///////////////////////////////////////////////////////////////////////////////

/// Trait used by `Serialize` implementations to generically construct errors
/// belonging to the `Serializer` against which they are currently running.
pub trait Error: Sized + error::Error {
    /// Raised when a `Serialize` implementation encounters a general error
    /// while serializing a type.
    ///
    /// The message should not be capitalized and should not end with a period.
    ///
    /// For example, a filesystem `Path` may refuse to serialize itself if it
    /// contains invalid UTF-8 data.
    ///
    /// ```rust
    /// # use serde::ser::{Serialize, Serializer, Error};
    /// # struct Path;
    /// # impl Path { fn to_str(&self) -> Option<&str> { unimplemented!() } }
    /// impl Serialize for Path {
    ///     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    ///         where S: Serializer
    ///     {
    ///         match self.to_str() {
    ///             Some(s) => s.serialize(serializer),
    ///             None => Err(Error::custom("path contains invalid UTF-8 characters")),
    ///         }
    ///     }
    /// }
    /// ```
    fn custom<T: Display>(msg: T) -> Self;
}

///////////////////////////////////////////////////////////////////////////////

/// A **data structure** that can be serialized into any data format supported
/// by Serde.
///
/// Serde provides `Serialize` implementations for many Rust primitive and
/// standard library types. The complete list is [here][ser]. All of these can
/// be serialized using Serde out of the box.
///
/// Additionally, Serde provides a procedural macro called `serde_derive` to
/// automatically generate `Serialize` implementations for structs and enums in
/// your program. See the [codegen section of the manual][codegen] for how to
/// use this.
///
/// In rare cases it may be necessary to implement `Serialize` manually for some
/// type in your program. See the [Implementing `Serialize`][impl-serialize]
/// section of the manual for more about this.
///
/// Third-party crates may provide `Serialize` implementations for types that
/// they expose. For example the `linked-hash-map` crate provides a
/// `LinkedHashMap<K, V>` type that is serializable by Serde because the crate
/// provides an implementation of `Serialize` for it.
///
/// [ser]: https://docs.serde.rs/serde/ser/index.html
/// [codegen]: https://serde.rs/codegen.html
/// [impl-serialize]: https://serde.rs/impl-serialize.html
pub trait Serialize {
    /// Serialize this value into the given Serde serializer.
    ///
    /// See the [Implementing `Serialize`][impl-serialize] section of the manual
    /// for more information about how to implement this method.
    ///
    /// [impl-serialize]: https://serde.rs/impl-serialize.html
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer;
}

///////////////////////////////////////////////////////////////////////////////

/// A **data format** that can serialize any data structure supported by Serde.
///
/// The role of this trait is to define the serialization half of the Serde data
/// model, which is a way to categorize every Rust data structure into one of 28
/// possible types. Each method of the `Serializer` trait corresponds to one of
/// the types of the data model.
///
/// Implementations of `Serialize` map themselves into this data model by
/// invoking exactly one of the `Serializer` methods.
///
/// The types that make up the Serde data model are:
///
///  - 12 primitive types:
///    - bool
///    - i8, i16, i32, i64
///    - u8, u16, u32, u64
///    - f32, f64
///    - char
///  - string
///  - byte array - [u8]
///  - option
///    - either none or some value
///  - unit
///    - unit is the type of () in Rust
///  - unit_struct
///    - for example `struct Unit` or `PhantomData<T>`
///  - unit_variant
///    - the `E::A` and `E::B` in `enum E { A, B }`
///  - newtype_struct
///    - for example `struct Millimeters(u8)`
///  - newtype_variant
///    - the `E::N` in `enum E { N(u8) }`
///  - seq
///    - a dynamically sized sequence of values, for example `Vec<T>` or
///      `HashSet<T>`
///  - seq_fixed_size
///    - a statically sized sequence of values for which the size will be known
///      at deserialization time without looking at the serialized data, for
///      example `[u64; 10]`
///  - tuple
///    - for example `(u8,)` or `(String, u64, Vec<T>)`
///  - tuple_struct
///    - for example `struct Rgb(u8, u8, u8)`
///  - tuple_variant
///    - the `E::T` in `enum E { T(u8, u8) }`
///  - map
///    - for example `BTreeMap<K, V>`
///  - struct
///    - a key-value pairing in which the keys will be known at deserialization
///      time without looking at the serialized data, for example `struct S { r:
///      u8, g: u8, b: u8 }`
///  - struct_variant
///    - the `E::S` in `enum E { S { r: u8, g: u8, b: u8 } }`
///
/// Many Serde serializers produce text or binary data as output, for example
/// JSON or Bincode. This is not a requirement of the `Serializer` trait, and
/// there are serializers that do not produce text or binary output. One example
/// is the `serde_json::value::Serializer` (distinct from the main `serde_json`
/// serializer) that produces a `serde_json::Value` data structure in memory as
/// output.
pub trait Serializer: Sized {
    /// The output type produced by this `Serializer` during successful
    /// serialization. Most serializers that produce text or binary output
    /// should set `Ok = ()` and serialize into an `io::Write` or buffer
    /// contained within the `Serializer` instance. Serializers that build
    /// in-memory data structures may be simplified by using `Ok` to propagate
    /// the data structure around.
    type Ok;

    /// The error type when some error occurs during serialization.
    type Error: Error;

    /// Type returned from `serialize_seq` and `serialize_seq_fixed_size` for
    /// serializing the content of the sequence.
    type SerializeSeq: SerializeSeq<Ok = Self::Ok, Error = Self::Error>;

    /// Type returned from `serialize_tuple` for serializing the content of the
    /// tuple.
    type SerializeTuple: SerializeTuple<Ok = Self::Ok, Error = Self::Error>;

    /// Type returned from `serialize_tuple_struct` for serializing the content
    /// of the tuple struct.
    type SerializeTupleStruct: SerializeTupleStruct<Ok = Self::Ok, Error = Self::Error>;

    /// Type returned from `serialize_tuple_variant` for serializing the content
    /// of the tuple variant.
    type SerializeTupleVariant: SerializeTupleVariant<Ok = Self::Ok, Error = Self::Error>;

    /// Type returned from `serialize_map` for serializing the content of the
    /// map.
    type SerializeMap: SerializeMap<Ok = Self::Ok, Error = Self::Error>;

    /// Type returned from `serialize_struct` for serializing the content of the
    /// struct.
    type SerializeStruct: SerializeStruct<Ok = Self::Ok, Error = Self::Error>;

    /// Type returned from `serialize_struct_variant` for serializing the
    /// content of the struct variant.
    type SerializeStructVariant: SerializeStructVariant<Ok = Self::Ok, Error = Self::Error>;

    /// Serialize a `bool` value.
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error>;

    /// Serialize an `i8` value.
    ///
    /// If the format does not differentiate between `i8` and `i64`, a
    /// reasonable implementation would be to cast the value to `i64` and
    /// forward to `serialize_i64`.
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error>;

    /// Serialize an `i16` value.
    ///
    /// If the format does not differentiate between `i16` and `i64`, a
    /// reasonable implementation would be to cast the value to `i64` and
    /// forward to `serialize_i64`.
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error>;

    /// Serialize an `i32` value.
    ///
    /// If the format does not differentiate between `i32` and `i64`, a
    /// reasonable implementation would be to cast the value to `i64` and
    /// forward to `serialize_i64`.
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error>;

    /// Serialize an `i64` value.
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error>;

    /// Serialize a `u8` value.
    ///
    /// If the format does not differentiate between `u8` and `u64`, a
    /// reasonable implementation would be to cast the value to `u64` and
    /// forward to `serialize_u64`.
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error>;

    /// Serialize a `u16` value.
    ///
    /// If the format does not differentiate between `u16` and `u64`, a
    /// reasonable implementation would be to cast the value to `u64` and
    /// forward to `serialize_u64`.
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error>;

    /// Serialize a `u32` value.
    ///
    /// If the format does not differentiate between `u32` and `u64`, a
    /// reasonable implementation would be to cast the value to `u64` and
    /// forward to `serialize_u64`.
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error>;

    /// Serialize a `u64` value.
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error>;

    /// Serialize an `f32` value.
    ///
    /// If the format does not differentiate between `f32` and `f64`, a
    /// reasonable implementation would be to cast the value to `f64` and
    /// forward to `serialize_f64`.
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error>;

    /// Serialize an `f64` value.
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error>;

    /// Serialize a character.
    ///
    /// If the format does not support characters, it is reasonable to serialize
    /// it as a single element `str` or a `u32`.
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error>;

    /// Serialize a `&str`.
    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error>;

    /// Serialize a chunk of raw byte data.
    ///
    /// Enables serializers to serialize byte slices more compactly or more
    /// efficiently than other types of slices. If no efficient implementation
    /// is available, a reasonable implementation would be to forward to
    /// `serialize_seq`. If forwarded, the implementation looks usually just
    /// like this:
    ///
    /// ```rust,ignore
    /// let mut seq = self.serialize_seq(Some(value.len()))?;
    /// for b in value {
    ///     seq.serialize_element(b)?;
    /// }
    /// seq.end()
    /// ```
    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok, Self::Error>;

    /// Serialize a `None` value.
    fn serialize_none(self) -> Result<Self::Ok, Self::Error>;

    /// Serialize a `Some(T)` value.
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error>;

    /// Serialize a `()` value.
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error>;

    /// Serialize a unit struct like `struct Unit` or `PhantomData<T>`.
    ///
    /// A reasonable implementation would be to forward to `serialize_unit`.
    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error>;

    /// Serialize a unit variant like `E::A` in `enum E { A, B }`.
    ///
    /// The `name` is the name of the enum, the `variant_index` is the index of
    /// this variant within the enum, and the `variant` is the name of the
    /// variant.
    ///
    /// A reasonable implementation would be to forward to `serialize_unit`.
    ///
    /// ```rust,ignore
    /// match *self {
    ///     E::A => serializer.serialize_unit_variant("E", 0, "A"),
    ///     E::B => serializer.serialize_unit_variant("E", 1, "B"),
    /// }
    /// ```
    fn serialize_unit_variant(self,
                              name: &'static str,
                              variant_index: usize,
                              variant: &'static str)
                              -> Result<Self::Ok, Self::Error>;

    /// Serialize a newtype struct like `struct Millimeters(u8)`.
    ///
    /// Serializers are encouraged to treat newtype structs as insignificant
    /// wrappers around the data they contain. A reasonable implementation would
    /// be to forward to `value.serialize(self)`.
    ///
    /// ```rust,ignore
    /// serializer.serialize_newtype_struct("Millimeters", &self.0)
    /// ```
    fn serialize_newtype_struct<T: ?Sized + Serialize>(self,
                                                       name: &'static str,
                                                       value: &T)
                                                       -> Result<Self::Ok, Self::Error>;

    /// Serialize a newtype variant like `E::N` in `enum E { N(u8) }`.
    ///
    /// The `name` is the name of the enum, the `variant_index` is the index of
    /// this variant within the enum, and the `variant` is the name of the
    /// variant. The `value` is the data contained within this newtype variant.
    ///
    /// ```rust,ignore
    /// match *self {
    ///     E::N(ref n) => serializer.serialize_newtype_variant("E", 0, "N", n),
    /// }
    /// ```
    fn serialize_newtype_variant<T: ?Sized + Serialize>(self,
                                                        name: &'static str,
                                                        variant_index: usize,
                                                        variant: &'static str,
                                                        value: &T)
                                                        -> Result<Self::Ok, Self::Error>;

    /// Begin to serialize a dynamically sized sequence. This call must be
    /// followed by zero or more calls to `serialize_element`, then a call to
    /// `end`.
    ///
    /// The argument is the number of elements in the sequence, which may or may
    /// not be computable before the sequence is iterated. Some serializers only
    /// support sequences whose length is known up front.
    ///
    /// ```rust,ignore
    /// let mut seq = serializer.serialize_seq(Some(self.len()))?;
    /// for element in self {
    ///     seq.serialize_element(element)?;
    /// }
    /// seq.end()
    /// ```
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error>;

    /// Begin to serialize a statically sized sequence whose length will be
    /// known at deserialization time without looking at the serialized data.
    /// This call must be followed by zero or more calls to `serialize_element`,
    /// then a call to `end`.
    ///
    /// ```rust,ignore
    /// let mut seq = serializer.serialize_seq_fixed_size(self.len())?;
    /// for element in self {
    ///     seq.serialize_element(element)?;
    /// }
    /// seq.end()
    /// ```
    fn serialize_seq_fixed_size(self, size: usize) -> Result<Self::SerializeSeq, Self::Error>;

    /// Begin to serialize a tuple. This call must be followed by zero or more
    /// calls to `serialize_element`, then a call to `end`.
    ///
    /// ```rust,ignore
    /// let mut tup = serializer.serialize_tuple(3)?;
    /// tup.serialize_element(&self.0)?;
    /// tup.serialize_element(&self.1)?;
    /// tup.serialize_element(&self.2)?;
    /// tup.end()
    /// ```
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error>;

    /// Begin to serialize a tuple struct like `struct Rgb(u8, u8, u8)`. This
    /// call must be followed by zero or more calls to `serialize_field`, then a
    /// call to `end`.
    ///
    /// The `name` is the name of the tuple struct and the `len` is the number
    /// of data fields that will be serialized.
    ///
    /// ```rust,ignore
    /// let mut ts = serializer.serialize_tuple_struct("Rgb", 3)?;
    /// ts.serialize_field(&self.0)?;
    /// ts.serialize_field(&self.1)?;
    /// ts.serialize_field(&self.2)?;
    /// ts.end()
    /// ```
    fn serialize_tuple_struct(self,
                              name: &'static str,
                              len: usize)
                              -> Result<Self::SerializeTupleStruct, Self::Error>;

    /// Begin to serialize a tuple variant like `E::T` in `enum E { T(u8, u8)
    /// }`. This call must be followed by zero or more calls to
    /// `serialize_field`, then a call to `end`.
    ///
    /// The `name` is the name of the enum, the `variant_index` is the index of
    /// this variant within the enum, the `variant` is the name of the variant,
    /// and the `len` is the number of data fields that will be serialized.
    ///
    /// ```rust,ignore
    /// match *self {
    ///     E::T(ref a, ref b) => {
    ///         let mut tv = serializer.serialize_tuple_variant("E", 0, "T", 2)?;
    ///         tv.serialize_field(a)?;
    ///         tv.serialize_field(b)?;
    ///         tv.end()
    ///     }
    /// }
    /// ```
    fn serialize_tuple_variant(self,
                               name: &'static str,
                               variant_index: usize,
                               variant: &'static str,
                               len: usize)
                               -> Result<Self::SerializeTupleVariant, Self::Error>;

    /// Begin to serialize a map. This call must be followed by zero or more
    /// calls to `serialize_key` and `serialize_value`, then a call to `end`.
    ///
    /// The argument is the number of elements in the map, which may or may not
    /// be computable before the map is iterated. Some serializers only support
    /// maps whose length is known up front.
    ///
    /// ```rust,ignore
    /// let mut map = serializer.serialize_map(Some(self.len()))?;
    /// for (k, v) in self {
    ///     map.serialize_entry(k, v)?;
    /// }
    /// map.end()
    /// ```
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error>;

    /// Begin to serialize a struct like `struct Rgb { r: u8, g: u8, b: u8 }`.
    /// This call must be followed by zero or more calls to `serialize_field`,
    /// then a call to `end`.
    ///
    /// The `name` is the name of the struct and the `len` is the number of
    /// data fields that will be serialized.
    ///
    /// ```rust,ignore
    /// let mut struc = serializer.serialize_struct("Rgb", 3)?;
    /// struc.serialize_field("r", &self.r)?;
    /// struc.serialize_field("g", &self.g)?;
    /// struc.serialize_field("b", &self.b)?;
    /// struc.end()
    /// ```
    fn serialize_struct(self,
                        name: &'static str,
                        len: usize)
                        -> Result<Self::SerializeStruct, Self::Error>;

    /// Begin to serialize a struct variant like `E::S` in `enum E { S { r: u8,
    /// g: u8, b: u8 } }`. This call must be followed by zero or more calls to
    /// `serialize_field`, then a call to `end`.
    ///
    /// The `name` is the name of the enum, the `variant_index` is the index of
    /// this variant within the enum, the `variant` is the name of the variant,
    /// and the `len` is the number of data fields that will be serialized.
    ///
    /// ```rust,ignore
    /// match *self {
    ///     E::S { ref r, ref g, ref b } => {
    ///         let mut sv = serializer.serialize_struct_variant("E", 0, "S", 3)?;
    ///         sv.serialize_field("r", r)?;
    ///         sv.serialize_field("g", g)?;
    ///         sv.serialize_field("b", b)?;
    ///         sv.end()
    ///     }
    /// }
    /// ```
    fn serialize_struct_variant(self,
                                name: &'static str,
                                variant_index: usize,
                                variant: &'static str,
                                len: usize)
                                -> Result<Self::SerializeStructVariant, Self::Error>;

    /// Collect an iterator as a sequence.
    ///
    /// The default implementation serializes each item yielded by the iterator
    /// using `Self::SerializeSeq`. Implementors should not need to override
    /// this method.
    fn collect_seq<I>(self, iter: I) -> Result<Self::Ok, Self::Error>
        where I: IntoIterator,
              <I as IntoIterator>::Item: Serialize
    {
        let iter = iter.into_iter();
        let mut serializer = try!(self.serialize_seq(iter.len_hint()));
        for item in iter {
            try!(serializer.serialize_element(&item));
        }
        serializer.end()
    }

    /// Collect an iterator as a map.
    ///
    /// The default implementation serializes each pair yielded by the iterator
    /// using `Self::SerializeMap`. Implementors should not need to override
    /// this method.
    fn collect_map<K, V, I>(self, iter: I) -> Result<Self::Ok, Self::Error>
        where K: Serialize,
              V: Serialize,
              I: IntoIterator<Item = (K, V)>
    {
        let iter = iter.into_iter();
        let mut serializer = try!(self.serialize_map(iter.len_hint()));
        for (key, value) in iter {
            try!(serializer.serialize_entry(&key, &value));
        }
        serializer.end()
    }

    /// Serialize a string produced by an implementation of `Display`.
    ///
    /// The default implementation builds a heap-allocated `String` and
    /// delegates to `serialize_str`. Serializers are encouraged to provide a
    /// more efficient implementation if possible.
    ///
    /// ```rust
    /// # use serde::{Serialize, Serializer};
    /// # struct DateTime;
    /// # impl DateTime {
    /// #     fn naive_local(&self) -> () { () }
    /// #     fn offset(&self) -> () { () }
    /// # }
    /// impl Serialize for DateTime {
    ///     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    ///         where S: Serializer
    ///     {
    ///         serializer.collect_str(&format_args!("{:?}{:?}",
    ///                                              self.naive_local(),
    ///                                              self.offset()))
    ///     }
    /// }
    /// ```
    #[cfg(any(feature = "std", feature = "collections"))]
    fn collect_str<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
        where T: Display
    {
        let mut string = String::new();
        write!(string, "{}", value).unwrap();
        self.serialize_str(&string)
    }

    /// Serialize a string produced by an implementation of `Display`.
    ///
    /// The default implementation returns an error unconditionally when
    /// compiled with `no_std`.
    ///
    /// ```rust
    /// # use serde::{Serialize, Serializer};
    /// # struct DateTime;
    /// # impl DateTime {
    /// #     fn naive_local(&self) -> () { () }
    /// #     fn offset(&self) -> () { () }
    /// # }
    /// impl Serialize for DateTime {
    ///     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    ///         where S: Serializer
    ///     {
    ///         serializer.collect_str(&format_args!("{:?}{:?}",
    ///                                              self.naive_local(),
    ///                                              self.offset()))
    ///     }
    /// }
    /// ```
    #[cfg(not(any(feature = "std", feature = "collections")))]
    fn collect_str<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
        where T: Display
    {
        // TODO https://github.com/serde-rs/serde/issues/805
        // Remove this impl and force no_std formats to implement collect_str.
        let _ = value;
        Err(Error::custom("this no_std format does not support serializing strings with collect_str"))
    }
}

/// Returned from `Serializer::serialize_seq` and
/// `Serializer::serialize_seq_fixed_size`.
///
/// ```rust,ignore
/// let mut seq = serializer.serialize_seq(Some(self.len()))?;
/// for element in self {
///     seq.serialize_element(element)?;
/// }
/// seq.end()
/// ```
pub trait SerializeSeq {
    /// Must match the `Ok` type of our `Serializer`.
    type Ok;

    /// Must match the `Error` type of our `Serializer`.
    type Error: Error;

    /// Serialize a sequence element.
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error>;

    /// Finish serializing a sequence.
    fn end(self) -> Result<Self::Ok, Self::Error>;
}

/// Returned from `Serializer::serialize_tuple`.
///
/// ```rust,ignore
/// let mut tup = serializer.serialize_tuple(3)?;
/// tup.serialize_element(&self.0)?;
/// tup.serialize_element(&self.1)?;
/// tup.serialize_element(&self.2)?;
/// tup.end()
/// ```
pub trait SerializeTuple {
    /// Must match the `Ok` type of our `Serializer`.
    type Ok;

    /// Must match the `Error` type of our `Serializer`.
    type Error: Error;

    /// Serialize a tuple element.
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error>;

    /// Finish serializing a tuple.
    fn end(self) -> Result<Self::Ok, Self::Error>;
}

/// Returned from `Serializer::serialize_tuple_struct`.
///
/// ```rust,ignore
/// let mut ts = serializer.serialize_tuple_struct("Rgb", 3)?;
/// ts.serialize_field(&self.0)?;
/// ts.serialize_field(&self.1)?;
/// ts.serialize_field(&self.2)?;
/// ts.end()
/// ```
pub trait SerializeTupleStruct {
    /// Must match the `Ok` type of our `Serializer`.
    type Ok;

    /// Must match the `Error` type of our `Serializer`.
    type Error: Error;

    /// Serialize a tuple struct field.
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error>;

    /// Finish serializing a tuple struct.
    fn end(self) -> Result<Self::Ok, Self::Error>;
}

/// Returned from `Serializer::serialize_tuple_variant`.
///
/// ```rust,ignore
/// match *self {
///     E::T(ref a, ref b) => {
///         let mut tv = serializer.serialize_tuple_variant("E", 0, "T", 2)?;
///         tv.serialize_field(a)?;
///         tv.serialize_field(b)?;
///         tv.end()
///     }
/// }
/// ```
pub trait SerializeTupleVariant {
    /// Must match the `Ok` type of our `Serializer`.
    type Ok;

    /// Must match the `Error` type of our `Serializer`.
    type Error: Error;

    /// Serialize a tuple variant field.
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error>;

    /// Finish serializing a tuple variant.
    fn end(self) -> Result<Self::Ok, Self::Error>;
}

/// Returned from `Serializer::serialize_map`.
///
/// ```rust,ignore
/// let mut map = serializer.serialize_map(Some(self.len()))?;
/// for (k, v) in self {
///     map.serialize_entry(k, v)?;
/// }
/// map.end()
/// ```
pub trait SerializeMap {
    /// Must match the `Ok` type of our `Serializer`.
    type Ok;

    /// Must match the `Error` type of our `Serializer`.
    type Error: Error;

    /// Serialize a map key.
    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error>;

    /// Serialize a map value.
    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error>;

    /// Serialize a map entry consisting of a key and a value.
    ///
    /// Some `Serialize` types are not able to hold a key and value in memory at
    /// the same time so `SerializeMap` implementations are required to support
    /// `serialize_key` and `serialize_value` individually. The
    /// `serialize_entry` method allows serializers to optimize for the case
    /// where key and value are both available. `Serialize` implementations are
    /// encouraged to use `serialize_entry` if possible.
    ///
    /// The default implementation delegates to `serialize_key` and
    /// `serialize_value`. This is appropriate for serializers that do not care
    /// about performance or are not able to optimize `serialize_entry` any
    /// better than this.
    fn serialize_entry<K: ?Sized + Serialize, V: ?Sized + Serialize>(&mut self,
                                                                     key: &K,
                                                                     value: &V)
                                                                     -> Result<(), Self::Error> {
        try!(self.serialize_key(key));
        self.serialize_value(value)
    }

    /// Finish serializing a map.
    fn end(self) -> Result<Self::Ok, Self::Error>;
}

/// Returned from `Serializer::serialize_struct`.
///
/// ```rust,ignore
/// let mut struc = serializer.serialize_struct("Rgb", 3)?;
/// struc.serialize_field("r", &self.r)?;
/// struc.serialize_field("g", &self.g)?;
/// struc.serialize_field("b", &self.b)?;
/// struc.end()
/// ```
pub trait SerializeStruct {
    /// Must match the `Ok` type of our `Serializer`.
    type Ok;

    /// Must match the `Error` type of our `Serializer`.
    type Error: Error;

    /// Serialize a struct field.
    fn serialize_field<T: ?Sized + Serialize>(&mut self,
                                              key: &'static str,
                                              value: &T)
                                              -> Result<(), Self::Error>;

    /// Finish serializing a struct.
    fn end(self) -> Result<Self::Ok, Self::Error>;
}

/// Returned from `Serializer::serialize_struct_variant`.
///
/// ```rust,ignore
/// match *self {
///     E::S { ref r, ref g, ref b } => {
///         let mut sv = serializer.serialize_struct_variant("E", 0, "S", 3)?;
///         sv.serialize_field("r", r)?;
///         sv.serialize_field("g", g)?;
///         sv.serialize_field("b", b)?;
///         sv.end()
///     }
/// }
/// ```
pub trait SerializeStructVariant {
    /// Must match the `Ok` type of our `Serializer`.
    type Ok;

    /// Must match the `Error` type of our `Serializer`.
    type Error: Error;

    /// Serialize a struct variant field.
    fn serialize_field<T: ?Sized + Serialize>(&mut self,
                                              key: &'static str,
                                              value: &T)
                                              -> Result<(), Self::Error>;

    /// Finish serializing a struct variant.
    fn end(self) -> Result<Self::Ok, Self::Error>;
}

trait LenHint: Iterator {
    fn len_hint(&self) -> Option<usize>;
}

impl<I: Iterator> LenHint for I {
    #[cfg(not(feature = "unstable"))]
    fn len_hint(&self) -> Option<usize> {
        iterator_len_hint(self)
    }

    #[cfg(feature = "unstable")]
    default fn len_hint(&self) -> Option<usize> {
        iterator_len_hint(self)
    }
}

#[cfg(feature = "unstable")]
impl<I: ExactSizeIterator> LenHint for I {
    fn len_hint(&self) -> Option<usize> {
        Some(self.len())
    }
}

fn iterator_len_hint<I: Iterator>(iter: &I) -> Option<usize> {
    match iter.size_hint() {
        (lo, Some(hi)) if lo == hi => Some(lo),
        _ => None,
    }
}
