//! Generic data structure deserialization framework.
//!
//! The two most important traits in this module are `Deserialize` and
//! `Deserializer`.
//!
//!  - **A type that implements `Deserialize` is a data structure** that can be
//!    deserialized from any data format supported by Serde, and conversely
//!  - **A type that implements `Deserializer` is a data format** that can
//!    deserialize any data structure supported by Serde.
//!
//! # The Deserialize trait
//!
//! Serde provides `Deserialize` implementations for many Rust primitive and
//! standard library types. The complete list is below. All of these can be
//! deserialized using Serde out of the box.
//!
//! Additionally, Serde provides a procedural macro called `serde_derive` to
//! automatically generate `Deserialize` implementations for structs and enums
//! in your program. See the [codegen section of the manual][codegen] for how to
//! use this.
//!
//! In rare cases it may be necessary to implement `Deserialize` manually for
//! some type in your program. See the [Implementing
//! `Deserialize`][impl-deserialize] section of the manual for more about this.
//!
//! Third-party crates may provide `Deserialize` implementations for types that
//! they expose. For example the `linked-hash-map` crate provides a
//! `LinkedHashMap<K, V>` type that is deserializable by Serde because the crate
//! provides an implementation of `Deserialize` for it.
//!
//! # The Deserializer trait
//!
//! `Deserializer` implementations are provided by third-party crates, for
//! example [`serde_json`][serde_json], [`serde_yaml`][serde_yaml] and
//! [`bincode`][bincode].
//!
//! A partial list of well-maintained formats is given on the [Serde
//! website][data-formats].
//!
//! # Implementations of Deserialize provided by Serde
//!
//! This is a slightly different set of types than what is supported for
//! serialization. Some types can be serialized by Serde but not deserialized.
//! One example is `&str`.
//!
//!  - **Primitive types**:
//!    - bool
//!    - isize, i8, i16, i32, i64
//!    - usize, u8, u16, u32, u64
//!    - f32, f64
//!    - char
//!  - **Compound types**:
//!    - [T; 0] through [T; 32]
//!    - tuples up to size 16
//!  - **Common standard library types**:
//!    - String
//!    - Option\<T\>
//!    - Result\<T, E\>
//!    - PhantomData\<T\>
//!  - **Wrapper types**:
//!    - Box\<T\>
//!    - Box\<[T]\>
//!    - Box\<str\>
//!    - Rc\<T\>
//!    - Arc\<T\>
//!    - Cow\<'a, T\>
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
//! [impl-deserialize]: https://serde.rs/impl-deserialize.html
//! [serde_json]: https://github.com/serde-rs/json
//! [serde_yaml]: https://github.com/dtolnay/serde-yaml
//! [bincode]: https://github.com/TyOverby/bincode
//! [data-formats]: https://serde.rs/#data-formats

#[cfg(feature = "std")]
use std::error;
#[cfg(not(feature = "std"))]
use error;

#[cfg(all(not(feature = "std"), feature = "collections"))]
use collections::{String, Vec};

use core::fmt::{self, Display};
use core::marker::PhantomData;

///////////////////////////////////////////////////////////////////////////////

#[doc(hidden)]
pub mod impls;
pub mod value;
mod from_primitive;

// Helpers used by generated code. Not public API.
#[doc(hidden)]
pub mod private;
#[cfg(any(feature = "std", feature = "collections"))]
mod content;

///////////////////////////////////////////////////////////////////////////////

/// The `Error` trait allows `Deserialize` implementations to create descriptive
/// error messages belonging to the `Deserializer` against which they are
/// currently running.
///
/// Every `Deserializer` declares an `Error` type that encompasses both
/// general-purpose deserialization errors as well as errors specific to the
/// particular deserialization format. For example the `Error` type of
/// `serde_json` can represent errors like an invalid JSON escape sequence or an
/// unterminated string literal, in addition to the error cases that are part of
/// this trait.
///
/// Most deserializers should only need to provide the `Error::custom` method
/// and inherit the default behavior for the other methods.
pub trait Error: Sized + error::Error {
    /// Raised when there is general error when deserializing a type.
    ///
    /// The message should not be capitalized and should not end with a period.
    ///
    /// ```rust
    /// # use serde::de::{Deserialize, Deserializer, Error};
    /// # use std::str::FromStr;
    /// # #[allow(dead_code)]
    /// # struct IpAddr;
    /// # impl FromStr for IpAddr {
    /// #     type Err = String;
    /// #     fn from_str(_: &str) -> Result<Self, String> { unimplemented!() }
    /// # }
    /// impl Deserialize for IpAddr {
    ///     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    ///         where D: Deserializer
    ///     {
    ///         let s = try!(String::deserialize(deserializer));
    ///         s.parse().map_err(Error::custom)
    ///     }
    /// }
    /// ```
    fn custom<T: Display>(msg: T) -> Self;

    /// Raised when a `Deserialize` receives a type different from what it was
    /// expecting.
    ///
    /// The `unexp` argument provides information about what type was received.
    /// This is the type that was present in the input file or other source data
    /// of the Deserializer.
    ///
    /// The `exp` argument provides information about what type was being
    /// expected. This is the type that is written in the program.
    ///
    /// For example if we try to deserialize a String out of a JSON file
    /// containing an integer, the unexpected type is the integer and the
    /// expected type is the string.
    fn invalid_type(unexp: Unexpected, exp: &Expected) -> Self {
        struct InvalidType<'a> {
            unexp: Unexpected<'a>,
            exp: &'a Expected,
        }
        impl<'a> Display for InvalidType<'a> {
            fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "invalid type: {}, expected {}", self.unexp, self.exp)
            }
        }
        Error::custom(InvalidType {
            unexp: unexp,
            exp: exp,
        })
    }

    /// Raised when a `Deserialize` receives a value of the right type but that
    /// is wrong for some other reason.
    ///
    /// The `unexp` argument provides information about what value was received.
    /// This is the value that was present in the input file or other source
    /// data of the Deserializer.
    ///
    /// The `exp` argument provides information about what value was being
    /// expected. This is the type that is written in the program.
    ///
    /// For example if we try to deserialize a String out of some binary data
    /// that is not valid UTF-8, the unexpected value is the bytes and the
    /// expected value is a string.
    fn invalid_value(unexp: Unexpected, exp: &Expected) -> Self {
        struct InvalidValue<'a> {
            unexp: Unexpected<'a>,
            exp: &'a Expected,
        }
        impl<'a> Display for InvalidValue<'a> {
            fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "invalid value: {}, expected {}", self.unexp, self.exp)
            }
        }
        Error::custom(InvalidValue {
            unexp: unexp,
            exp: exp,
        })
    }

    /// Raised when deserializing a sequence or map and the input data contains
    /// too many or too few elements.
    ///
    /// The `len` argument is the number of elements encountered. The sequence
    /// or map may have expected more arguments or fewer arguments.
    ///
    /// The `exp` argument provides information about what data was being
    /// expected. For example `exp` might say that a tuple of size 6 was
    /// expected.
    fn invalid_length(len: usize, exp: &Expected) -> Self {
        struct InvalidLength<'a> {
            len: usize,
            exp: &'a Expected,
        }
        impl<'a> Display for InvalidLength<'a> {
            fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "invalid length {}, expected {}", self.len, self.exp)
            }
        }
        Error::custom(InvalidLength {
            len: len,
            exp: exp,
        })
    }

    /// Raised when a `Deserialize` enum type received a variant with an
    /// unrecognized name.
    fn unknown_variant(variant: &str, expected: &'static [&'static str]) -> Self {
        struct UnknownVariant<'a> {
            variant: &'a str,
            expected: &'static [&'static str],
        }
        impl<'a> Display for UnknownVariant<'a> {
            fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                if self.expected.is_empty() {
                    write!(formatter,
                           "unknown variant `{}`, there are no variants",
                           self.variant)
                } else {
                    write!(formatter,
                           "unknown variant `{}`, expected {}",
                           self.variant,
                           OneOf { names: self.expected })
                }
            }
        }
        Error::custom(UnknownVariant {
            variant: variant,
            expected: expected,
        })
    }

    /// Raised when a `Deserialize` struct type received a field with an
    /// unrecognized name.
    fn unknown_field(field: &str, expected: &'static [&'static str]) -> Self {
        struct UnknownField<'a> {
            field: &'a str,
            expected: &'static [&'static str],
        }
        impl<'a> Display for UnknownField<'a> {
            fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                if self.expected.is_empty() {
                    write!(formatter,
                           "unknown field `{}`, there are no fields",
                           self.field)
                } else {
                    write!(formatter,
                           "unknown field `{}`, expected {}",
                           self.field,
                           OneOf { names: self.expected })
                }
            }
        }
        Error::custom(UnknownField {
            field: field,
            expected: expected,
        })
    }

    /// Raised when a `Deserialize` struct type expected to receive a required
    /// field with a particular name but that field was not present in the
    /// input.
    fn missing_field(field: &'static str) -> Self {
        struct MissingField {
            field: &'static str,
        }
        impl Display for MissingField {
            fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "missing field `{}`", self.field)
            }
        }
        Error::custom(MissingField { field: field })
    }

    /// Raised when a `Deserialize` struct type received more than one of the
    /// same field.
    fn duplicate_field(field: &'static str) -> Self {
        struct DuplicateField {
            field: &'static str,
        }
        impl Display for DuplicateField {
            fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "duplicate field `{}`", self.field)
            }
        }
        Error::custom(DuplicateField { field: field })
    }
}

/// `Unexpected` represents an unexpected invocation of any one of the `Visitor`
/// trait methods.
///
/// This is used as an argument to the `invalid_type`, `invalid_value`, and
/// `invalid_length` methods of the `Error` trait to build error messages.
///
/// ```rust
/// # use serde::de::{Error, Unexpected, Visitor};
/// # use std::fmt;
/// # #[allow(dead_code)]
/// # struct Example;
/// # impl Visitor for Example {
/// # type Value = ();
/// fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
///     where E: Error
/// {
///     Err(Error::invalid_type(Unexpected::Bool(v), &self))
/// }
/// # fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
/// #     write!(formatter, "definitely not a boolean")
/// # }
/// # }
/// ```
#[derive(Clone, PartialEq, Debug)]
pub enum Unexpected<'a> {
    /// The input contained a boolean value that was not expected.
    Bool(bool),

    /// The input contained an unsigned integer `u8`, `u16`, `u32` or `u64` that
    /// was not expected.
    Unsigned(u64),

    /// The input contained a signed integer `i8`, `i16`, `i32` or `i64` that
    /// was not expected.
    Signed(i64),

    /// The input contained a floating point `f32` or `f64` that was not
    /// expected.
    Float(f64),

    /// The input contained a `char` that was not expected.
    Char(char),

    /// The input contained a `&str` or `String` that was not expected.
    Str(&'a str),

    /// The input contained a `&[u8]` or `Vec<u8>` that was not expected.
    Bytes(&'a [u8]),

    /// The input contained a unit `()` that was not expected.
    Unit,

    /// The input contained an `Option<T>` that was not expected.
    Option,

    /// The input contained a newtype struct that was not expected.
    NewtypeStruct,

    /// The input contained a sequence that was not expected.
    Seq,

    /// The input contained a map that was not expected.
    Map,

    /// The input contained an enum that was not expected.
    Enum,

    /// The input contained a unit variant that was not expected.
    UnitVariant,

    /// The input contained a newtype variant that was not expected.
    NewtypeVariant,

    /// The input contained a tuple variant that was not expected.
    TupleVariant,

    /// The input contained a struct variant that was not expected.
    StructVariant,

    /// A message stating what uncategorized thing the input contained that was
    /// not expected.
    ///
    /// The message should be a noun or noun phrase, not capitalized and without
    /// a period. An example message is "unoriginal superhero".
    Other(&'a str),
}

impl<'a> fmt::Display for Unexpected<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use self::Unexpected::*;
        match *self {
            Bool(b) => write!(formatter, "boolean `{}`", b),
            Unsigned(i) => write!(formatter, "integer `{}`", i),
            Signed(i) => write!(formatter, "integer `{}`", i),
            Float(f) => write!(formatter, "floating point `{}`", f),
            Char(c) => write!(formatter, "character `{}`", c),
            Str(s) => write!(formatter, "string {:?}", s),
            Bytes(_) => write!(formatter, "byte array"),
            Unit => write!(formatter, "unit value"),
            Option => write!(formatter, "Option value"),
            NewtypeStruct => write!(formatter, "newtype struct"),
            Seq => write!(formatter, "sequence"),
            Map => write!(formatter, "map"),
            Enum => write!(formatter, "enum"),
            UnitVariant => write!(formatter, "unit variant"),
            NewtypeVariant => write!(formatter, "newtype variant"),
            TupleVariant => write!(formatter, "tuple variant"),
            StructVariant => write!(formatter, "struct variant"),
            Other(other) => formatter.write_str(other),
        }
    }
}

/// `Expected` represents an explanation of what data a `Visitor` was expecting
/// to receive.
///
/// This is used as an argument to the `invalid_type`, `invalid_value`, and
/// `invalid_length` methods of the `Error` trait to build error messages. The
/// message should be a noun or noun phrase that completes the sentence "This
/// Visitor expects to receive ...", for example the message could be "an
/// integer between 0 and 64". The message should not be capitalized and should
/// not end with a period.
///
/// Within the context of a `Visitor` implementation, the `Visitor` itself
/// (`&self`) is an implementation of this trait.
///
/// ```rust
/// # use serde::de::{Error, Unexpected, Visitor};
/// # use std::fmt;
/// # #[allow(dead_code)]
/// # struct Example;
/// # impl Visitor for Example {
/// # type Value = ();
/// fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
///     where E: Error
/// {
///     Err(Error::invalid_type(Unexpected::Bool(v), &self))
/// }
/// # fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
/// #     write!(formatter, "definitely not a boolean")
/// # }
/// # }
/// ```
///
/// Outside of a `Visitor`, `&"..."` can be used.
///
/// ```rust
/// # use serde::de::{Error, Unexpected};
/// # #[allow(dead_code)]
/// # fn example<E: Error>() -> Result<(), E> {
/// # let v = true;
/// return Err(Error::invalid_type(Unexpected::Bool(v), &"a negative integer"));
/// # }
/// ```
pub trait Expected {
    /// Format an explanation of what data was being expected. Same signature as
    /// the `Display` and `Debug` traits.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result;
}

impl<T> Expected for T
    where T: Visitor
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.expecting(formatter)
    }
}

impl<'a> Expected for &'a str {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self)
    }
}

impl<'a> Display for Expected + 'a {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Expected::fmt(self, formatter)
    }
}

///////////////////////////////////////////////////////////////////////////////

/// A **data structure** that can be deserialized from any data format supported
/// by Serde.
///
/// Serde provides `Deserialize` implementations for many Rust primitive and
/// standard library types. The complete list is [here][de]. All of these can
/// be deserialized using Serde out of the box.
///
/// Additionally, Serde provides a procedural macro called `serde_derive` to
/// automatically generate `Deserialize` implementations for structs and enums
/// in your program. See the [codegen section of the manual][codegen] for how to
/// use this.
///
/// In rare cases it may be necessary to implement `Deserialize` manually for
/// some type in your program. See the [Implementing
/// `Deserialize`][impl-deserialize] section of the manual for more about this.
///
/// Third-party crates may provide `Deserialize` implementations for types that
/// they expose. For example the `linked-hash-map` crate provides a
/// `LinkedHashMap<K, V>` type that is deserializable by Serde because the crate
/// provides an implementation of `Deserialize` for it.
///
/// [de]: https://docs.serde.rs/serde/de/index.html
/// [codegen]: https://serde.rs/codegen.html
/// [impl-deserialize]: https://serde.rs/impl-deserialize.html
pub trait Deserialize: Sized {
    /// Deserialize this value from the given Serde deserializer.
    ///
    /// See the [Implementing `Deserialize`][impl-deserialize] section of the
    /// manual for more information about how to implement this method.
    ///
    /// [impl-deserialize]: https://serde.rs/impl-deserialize.html
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer;
}

/// `DeserializeSeed` is the stateful form of the `Deserialize` trait. If you
/// ever find yourself looking for a way to pass data into a `Deserialize` impl,
/// this trait is the way to do it.
///
/// As one example of stateful deserialization consider deserializing a JSON
/// array into an existing buffer. Using the `Deserialize` trait we could
/// deserialize a JSON array into a `Vec<T>` but it would be a freshly allocated
/// `Vec<T>`; there is no way for `Deserialize` to reuse a previously allocated
/// buffer. Using `DeserializeSeed` instead makes this possible as in the
/// example code below.
///
/// The canonical API for stateless deserialization looks like this:
///
/// ```rust
/// # use serde::Deserialize;
/// # #[allow(dead_code)]
/// # enum Error {}
/// # #[allow(dead_code)]
/// fn func<T: Deserialize>() -> Result<T, Error>
/// # { unimplemented!() }
/// ```
///
/// Adjusting an API like this to support stateful deserialization is a matter
/// of accepting a seed as input:
///
/// ```rust
/// # use serde::de::DeserializeSeed;
/// # #[allow(dead_code)]
/// # enum Error {}
/// # #[allow(dead_code)]
/// fn func_seed<T: DeserializeSeed>(seed: T) -> Result<T::Value, Error>
/// # {
/// #     let _ = seed;
/// #     unimplemented!()
/// # }
/// ```
///
/// In practice the majority of deserialization is stateless. An API expecting a
/// seed can be appeased by passing `std::marker::PhantomData` as a seed in the
/// case of stateless deserialization.
///
/// # Example
///
/// Suppose we have JSON that looks like `[[1, 2], [3, 4, 5], [6]]` and we need
/// to deserialize it into a flat representation like `vec![1, 2, 3, 4, 5, 6]`.
/// Allocating a brand new `Vec<T>` for each subarray would be slow. Instead we
/// would like to allocate a single `Vec<T>` and then deserialize each subarray
/// into it. This requires stateful deserialization using the `DeserializeSeed`
/// trait.
///
/// ```rust
/// # use serde::de::{Deserialize, DeserializeSeed, Deserializer, Visitor, SeqVisitor};
/// # use std::fmt;
/// # use std::marker::PhantomData;
/// #
/// // A DeserializeSeed implementation that uses stateful deserialization to
/// // append array elements onto the end of an existing vector. The preexisting
/// // state ("seed") in this case is the Vec<T>. The `deserialize` method of
/// // `ExtendVec` will be traversing the inner arrays of the JSON input and
/// // appending each integer into the existing Vec.
/// struct ExtendVec<'a, T: 'a>(&'a mut Vec<T>);
///
/// impl<'a, T> DeserializeSeed for ExtendVec<'a, T>
///     where T: Deserialize
/// {
///     // The return type of the `deserialize` method. This implementation
///     // appends onto an existing vector but does not create any new data
///     // structure, so the return type is ().
///     type Value = ();
///
///     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
///         where D: Deserializer
///     {
///         // Visitor implementation that will walk an inner array of the JSON
///         // input.
///         struct ExtendVecVisitor<'a, T: 'a>(&'a mut Vec<T>);
///
///         impl<'a, T> Visitor for ExtendVecVisitor<'a, T>
///             where T: Deserialize
///         {
///             type Value = ();
///
///             fn visit_seq<V>(self, mut visitor: V) -> Result<(), V::Error>
///                 where V: SeqVisitor
///             {
///                 // Visit each element in the inner array and push it onto
///                 // the existing vector.
///                 while let Some(elem) = visitor.visit()? {
///                     self.0.push(elem);
///                 }
///                 Ok(())
///             }
/// #
/// #           fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
/// #               write!(formatter, "an array of integers")
/// #           }
///         }
///
///         deserializer.deserialize_seq(ExtendVecVisitor(self.0))
///     }
/// }
///
/// // Visitor implementation that will walk the outer array of the JSON input.
/// struct FlattenedVecVisitor<T>(PhantomData<T>);
///
/// impl<T> Visitor for FlattenedVecVisitor<T>
///     where T: Deserialize
/// {
///     // This Visitor constructs a single Vec<T> to hold the flattened
///     // contents of the inner arrays.
///     type Value = Vec<T>;
///
///     fn visit_seq<V>(self, mut visitor: V) -> Result<Vec<T>, V::Error>
///         where V: SeqVisitor
///     {
///         // Create a single Vec to hold the flattened contents.
///         let mut vec = Vec::new();
///
///         // Each iteration through this loop is one inner array.
///         while let Some(()) = visitor.visit_seed(ExtendVec(&mut vec))? {
///             // Nothing to do; inner array has been appended into `vec`.
///         }
///
///         // Return the finished vec.
///         Ok(vec)
///     }
/// #
/// #   fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
/// #       write!(formatter, "an array of arrays")
/// #   }
/// }
///
/// # #[allow(dead_code)]
/// # fn example<D: Deserializer>(deserializer: D) -> Result<(), D::Error> {
/// let visitor = FlattenedVecVisitor(PhantomData);
/// let flattened: Vec<u64> = deserializer.deserialize_seq(visitor)?;
/// # let _ = flattened;
/// # Ok(()) }
/// ```
pub trait DeserializeSeed: Sized {
    /// The type produced by using this seed.
    type Value;

    /// Equivalent to the more common `Deserialize::deserialize` method, except
    /// with some initial piece of data (the seed) passed in.
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where D: Deserializer;
}

impl<T> DeserializeSeed for PhantomData<T>
    where T: Deserialize
{
    type Value = T;

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<T, D::Error>
        where D: Deserializer
    {
        T::deserialize(deserializer)
    }
}

///////////////////////////////////////////////////////////////////////////////

/// A **data format** that can deserialize any data structure supported by
/// Serde.
///
/// The role of this trait is to define the deserialization half of the Serde
/// data model, which is a way to categorize every Rust data type into one of 28
/// possible types. Each method of the `Serializer` trait corresponds to one of
/// the types of the data model.
///
/// Implementations of `Deserialize` map themselves into this data model by
/// passing to the `Deserializer` a `Visitor` implementation that can receive
/// these various types.
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
/// The `Deserializer` trait supports two entry point styles which enables
/// different kinds of deserialization.
///
/// 1. The `deserialize` method. Self-describing data formats like JSON are able
///    to look at the serialized data and tell what it represents. For example
///    the JSON deserializer may see an opening curly brace (`{`) and know that
///    it is seeing a map. If the data format supports
///    `Deserializer::deserialize`, it will drive the Visitor using whatever
///    type it sees in the input. JSON uses this approach when deserializing
///    `serde_json::Value` which is an enum that can represent any JSON
///    document. Without knowing what is in a JSON document, we can deserialize
///    it to `serde_json::Value` by going through `Deserializer::deserialize`.
///
/// 2. The various `deserialize_*` methods. Non-self-describing formats like
///    Bincode need to be told what is in the input in order to deserialize it.
///    The `deserialize_*` methods are hints to the deserializer for how to
///    interpret the next piece of input. Non-self-describing formats are not
///    able to deserialize something like `serde_json::Value` which relies on
///    `Deserializer::deserialize`.
///
/// When implementing `Deserialize`, you should avoid relying on
/// `Deserializer::deserialize` unless you need to be told by the Deserializer
/// what type is in the input. Know that relying on `Deserializer::deserialize`
/// means your data type will be able to deserialize from self-describing
/// formats only, ruling out Bincode and many others.
pub trait Deserializer: Sized {
    /// The error type that can be returned if some error occurs during
    /// deserialization.
    type Error: Error;

    /// Require the `Deserializer` to figure out how to drive the visitor based
    /// on what data type is in the input.
    ///
    /// When implementing `Deserialize`, you should avoid relying on
    /// `Deserializer::deserialize` unless you need to be told by the
    /// Deserializer what type is in the input. Know that relying on
    /// `Deserializer::deserialize` means your data type will be able to
    /// deserialize from self-describing formats only, ruling out Bincode and
    /// many others.
    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor;

    /// Hint that the `Deserialize` type is expecting a `bool` value.
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor;

    /// Hint that the `Deserialize` type is expecting a `u8` value.
    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor;

    /// Hint that the `Deserialize` type is expecting a `u16` value.
    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor;

    /// Hint that the `Deserialize` type is expecting a `u32` value.
    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor;

    /// Hint that the `Deserialize` type is expecting a `u64` value.
    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor;

    /// Hint that the `Deserialize` type is expecting an `i8` value.
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor;

    /// Hint that the `Deserialize` type is expecting an `i16` value.
    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor;

    /// Hint that the `Deserialize` type is expecting an `i32` value.
    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor;

    /// Hint that the `Deserialize` type is expecting an `i64` value.
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor;

    /// Hint that the `Deserialize` type is expecting a `f32` value.
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor;

    /// Hint that the `Deserialize` type is expecting a `f64` value.
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor;

    /// Hint that the `Deserialize` type is expecting a `char` value.
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor;

    /// Hint that the `Deserialize` type is expecting a string value and does
    /// not benefit from taking ownership of buffered data owned by the
    /// `Deserializer`.
    ///
    /// If the `Visitor` would benefit from taking ownership of `String` data,
    /// indiciate this to the `Deserializer` by using `deserialize_string`
    /// instead.
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor;

    /// Hint that the `Deserialize` type is expecting a string value and would
    /// benefit from taking ownership of buffered data owned by the
    /// `Deserializer`.
    ///
    /// If the `Visitor` would not benefit from taking ownership of `String`
    /// data, indicate that to the `Deserializer` by using `deserialize_str`
    /// instead.
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor;

    /// Hint that the `Deserialize` type is expecting a byte array and does not
    /// benefit from taking ownership of buffered data owned by the
    /// `Deserializer`.
    ///
    /// If the `Visitor` would benefit from taking ownership of `Vec<u8>` data,
    /// indicate this to the `Deserializer` by using `deserialize_byte_buf`
    /// instead.
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor;

    /// Hint that the `Deserialize` type is expecting a byte array and would
    /// benefit from taking ownership of buffered data owned by the
    /// `Deserializer`.
    ///
    /// If the `Visitor` would not benefit from taking ownership of `Vec<u8>`
    /// data, indicate that to the `Deserializer` by using `deserialize_bytes`
    /// instead.
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor;

    /// Hint that the `Deserialize` type is expecting an optional value.
    ///
    /// This allows deserializers that encode an optional value as a nullable
    /// value to convert the null value into `None` and a regular value into
    /// `Some(value)`.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor;

    /// Hint that the `Deserialize` type is expecting a unit value.
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor;

    /// Hint that the `Deserialize` type is expecting a unit struct with a
    /// particular name.
    fn deserialize_unit_struct<V>(self,
                                  name: &'static str,
                                  visitor: V)
                                  -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// Hint that the `Deserialize` type is expecting a newtype struct with a
    /// particular name.
    fn deserialize_newtype_struct<V>(self,
                                     name: &'static str,
                                     visitor: V)
                                     -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// Hint that the `Deserialize` type is expecting a sequence of values.
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor;

    /// Hint that the `Deserialize` type is expecting a sequence of values and
    /// knows how many values there are without looking at the serialized data.
    fn deserialize_seq_fixed_size<V>(self,
                                     len: usize,
                                     visitor: V)
                                     -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// Hint that the `Deserialize` type is expecting a tuple value with a
    /// particular number of elements.
    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// Hint that the `Deserialize` type is expecting a tuple struct with a
    /// particular name and number of fields.
    fn deserialize_tuple_struct<V>(self,
                                   name: &'static str,
                                   len: usize,
                                   visitor: V)
                                   -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// Hint that the `Deserialize` type is expecting a map of key-value pairs.
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor;

    /// Hint that the `Deserialize` type is expecting a struct with a particular
    /// name and fields.
    fn deserialize_struct<V>(self,
                             name: &'static str,
                             fields: &'static [&'static str],
                             visitor: V)
                             -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// Hint that the `Deserialize` type is expecting the name of a struct
    /// field.
    fn deserialize_struct_field<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// Hint that the `Deserialize` type is expecting an enum value with a
    /// particular name and possible variants.
    fn deserialize_enum<V>(self,
                           name: &'static str,
                           variants: &'static [&'static str],
                           visitor: V)
                           -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// Hint that the `Deserialize` type needs to deserialize a value whose type
    /// doesn't matter because it is ignored.
    ///
    /// Deserializers for non-self-describing formats may not support this mode.
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;
}

///////////////////////////////////////////////////////////////////////////////

/// This trait represents a visitor that walks through a deserializer.
///
/// ```rust
/// # use serde::de::{Error, Unexpected, Visitor};
/// # use std::fmt;
/// /// A visitor that deserializes a long string - a string containing at least
/// /// some minimum number of bytes.
/// # #[allow(dead_code)]
/// struct LongString {
///     min: usize,
/// }
///
/// impl Visitor for LongString {
///     type Value = String;
///
///     fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
///         write!(formatter, "a string containing at least {} bytes", self.min)
///     }
///
///     fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
///         where E: Error
///     {
///         if s.len() >= self.min {
///             Ok(s.to_owned())
///         } else {
///             Err(Error::invalid_value(Unexpected::Str(s), &self))
///         }
///     }
/// }
/// ```
pub trait Visitor: Sized {
    /// The value produced by this visitor.
    type Value;

    /// Format a message stating what data this Visitor expects to receive.
    ///
    /// This is used in error messages. The message should complete the sentence
    /// "This Visitor expects to receive ...", for example the message could be
    /// "an integer between 0 and 64". The message should not be capitalized and
    /// should not end with a period.
    ///
    /// ```rust
    /// # use std::fmt;
    /// # #[allow(dead_code)]
    /// # struct S { max: usize }
    /// # impl serde::de::Visitor for S {
    /// # type Value = ();
    /// fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    ///     write!(formatter, "an integer between 0 and {}", self.max)
    /// }
    /// # }
    /// ```
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result;

    /// Deserialize a `bool` into a `Value`.
    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
        where E: Error
    {
        Err(Error::invalid_type(Unexpected::Bool(v), &self))
    }

    /// Deserialize an `i8` into a `Value`.
    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
        where E: Error
    {
        self.visit_i64(v as i64)
    }

    /// Deserialize an `i16` into a `Value`.
    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
        where E: Error
    {
        self.visit_i64(v as i64)
    }

    /// Deserialize an `i32` into a `Value`.
    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
        where E: Error
    {
        self.visit_i64(v as i64)
    }

    /// Deserialize an `i64` into a `Value`.
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where E: Error
    {
        Err(Error::invalid_type(Unexpected::Signed(v), &self))
    }

    /// Deserialize a `u8` into a `Value`.
    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
        where E: Error
    {
        self.visit_u64(v as u64)
    }

    /// Deserialize a `u16` into a `Value`.
    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
        where E: Error
    {
        self.visit_u64(v as u64)
    }

    /// Deserialize a `u32` into a `Value`.
    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
        where E: Error
    {
        self.visit_u64(v as u64)
    }

    /// Deserialize a `u64` into a `Value`.
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where E: Error
    {
        Err(Error::invalid_type(Unexpected::Unsigned(v), &self))
    }

    /// Deserialize a `f32` into a `Value`.
    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
        where E: Error
    {
        self.visit_f64(v as f64)
    }

    /// Deserialize a `f64` into a `Value`.
    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where E: Error
    {
        Err(Error::invalid_type(Unexpected::Float(v), &self))
    }

    /// Deserialize a `char` into a `Value`.
    #[inline]
    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
        where E: Error
    {
        self.visit_str(::utils::encode_utf8(v).as_str())
    }

    /// Deserialize a `&str` into a `Value`.
    ///
    /// This method allows the `Deserializer` to avoid a copy by retaining
    /// ownership of any buffered data. `Deserialize` implementations that do
    /// not benefit from taking ownership of `String` data should indicate that
    /// to the deserializer by using `Deserializer::deserialize_str` rather than
    /// `Deserializer::deserialize_string`.
    ///
    /// It is never correct to implement `visit_string` without implementing
    /// `visit_str`. Implement neither, both, or just `visit_str`.
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where E: Error
    {
        Err(Error::invalid_type(Unexpected::Str(v), &self))
    }

    /// Deserialize a `String` into a `Value`.
    ///
    /// This method allows the `Visitor` to avoid a copy by taking ownership of
    /// a string created by the `Deserializer`. `Deserialize` implementations
    /// that benefit from taking ownership of `String` data should indicate that
    /// to the deserializer by using `Deserializer::deserialize_string` rather
    /// than `Deserializer::deserialize_str`, although not every deserializer
    /// will honor such a request.
    ///
    /// It is never correct to implement `visit_string` without implementing
    /// `visit_str`. Implement neither, both, or just `visit_str`.
    ///
    /// The default implementation forwards to `visit_str` and then drops the
    /// `String`.
    #[inline]
    #[cfg(any(feature = "std", feature = "collections"))]
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where E: Error
    {
        self.visit_str(&v)
    }

    /// Deserialize a `()` into a `Value`.
    fn visit_unit<E>(self) -> Result<Self::Value, E>
        where E: Error
    {
        Err(Error::invalid_type(Unexpected::Unit, &self))
    }

    /// Deserialize an absent optional `Value`.
    fn visit_none<E>(self) -> Result<Self::Value, E>
        where E: Error
    {
        Err(Error::invalid_type(Unexpected::Option, &self))
    }

    /// Deserialize a present optional `Value`.
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        let _ = deserializer;
        Err(Error::invalid_type(Unexpected::Option, &self))
    }

    /// Deserialize `Value` as a newtype struct.
    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        let _ = deserializer;
        Err(Error::invalid_type(Unexpected::NewtypeStruct, &self))
    }

    /// Deserialize `Value` as a sequence of elements.
    fn visit_seq<V>(self, visitor: V) -> Result<Self::Value, V::Error>
        where V: SeqVisitor
    {
        let _ = visitor;
        Err(Error::invalid_type(Unexpected::Seq, &self))
    }

    /// Deserialize `Value` as a key-value map.
    fn visit_map<V>(self, visitor: V) -> Result<Self::Value, V::Error>
        where V: MapVisitor
    {
        let _ = visitor;
        Err(Error::invalid_type(Unexpected::Map, &self))
    }

    /// Deserialize `Value` as an enum.
    fn visit_enum<V>(self, visitor: V) -> Result<Self::Value, V::Error>
        where V: EnumVisitor
    {
        let _ = visitor;
        Err(Error::invalid_type(Unexpected::Enum, &self))
    }

    /// Deserialize a `&[u8]` into a `Value`.
    ///
    /// This method allows the `Deserializer` to avoid a copy by retaining
    /// ownership of any buffered data. `Deserialize` implementations that do
    /// not benefit from taking ownership of `Vec<u8>` data should indicate that
    /// to the deserializer by using `Deserializer::deserialize_bytes` rather
    /// than `Deserializer::deserialize_byte_buf`.
    ///
    /// It is never correct to implement `visit_byte_buf` without implementing
    /// `visit_bytes`. Implement neither, both, or just `visit_bytes`.
    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where E: Error
    {
        let _ = v;
        Err(Error::invalid_type(Unexpected::Bytes(v), &self))
    }

    /// Deserialize a `Vec<u8>` into a `Value`.
    ///
    /// This method allows the `Visitor` to avoid a copy by taking ownership of
    /// a byte buffer created by the `Deserializer`. `Deserialize`
    /// implementations that benefit from taking ownership of `Vec<u8>` data
    /// should indicate that to the deserializer by using
    /// `Deserializer::deserialize_byte_buf` rather than
    /// `Deserializer::deserialize_bytes`, although not every deserializer will
    /// honor such a request.
    ///
    /// It is never correct to implement `visit_byte_buf` without implementing
    /// `visit_bytes`. Implement neither, both, or just `visit_bytes`.
    ///
    /// The default implementation forwards to `visit_bytes` and then drops the
    /// `Vec<u8>`.
    #[cfg(any(feature = "std", feature = "collections"))]
    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
        where E: Error
    {
        self.visit_bytes(&v)
    }
}

///////////////////////////////////////////////////////////////////////////////

/// `SeqVisitor` visits each item in a sequence.
///
/// This is a trait that a `Deserializer` passes to a `Visitor` implementation,
/// which deserializes each item in a sequence.
pub trait SeqVisitor {
    /// The error type that can be returned if some error occurs during
    /// deserialization.
    type Error: Error;

    /// This returns `Ok(Some(value))` for the next value in the sequence, or
    /// `Ok(None)` if there are no more remaining items.
    ///
    /// `Deserialize` implementations should typically use `SeqVisitor::visit`
    /// instead.
    fn visit_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
        where T: DeserializeSeed;

    /// This returns `Ok(Some(value))` for the next value in the sequence, or
    /// `Ok(None)` if there are no more remaining items.
    ///
    /// This method exists as a convenience for `Deserialize` implementations.
    /// `SeqVisitor` implementations should not override the default behavior.
    #[inline]
    fn visit<T>(&mut self) -> Result<Option<T>, Self::Error>
        where T: Deserialize
    {
        self.visit_seed(PhantomData)
    }

    /// Return the lower and upper bound of items remaining in the sequence.
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl<'a, V> SeqVisitor for &'a mut V
    where V: SeqVisitor
{
    type Error = V::Error;

    #[inline]
    fn visit_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, V::Error>
        where T: DeserializeSeed
    {
        (**self).visit_seed(seed)
    }

    #[inline]
    fn visit<T>(&mut self) -> Result<Option<T>, V::Error>
        where T: Deserialize
    {
        (**self).visit()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (**self).size_hint()
    }
}

///////////////////////////////////////////////////////////////////////////////

/// `MapVisitor` visits each item in a sequence.
///
/// This is a trait that a `Deserializer` passes to a `Visitor` implementation.
pub trait MapVisitor {
    /// The error type that can be returned if some error occurs during
    /// deserialization.
    type Error: Error;

    /// This returns `Ok(Some(key))` for the next key in the map, or `Ok(None)`
    /// if there are no more remaining entries.
    ///
    /// `Deserialize` implementations should typically use
    /// `MapVisitor::visit_key` or `MapVisitor::visit` instead.
    fn visit_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
        where K: DeserializeSeed;

    /// This returns a `Ok(value)` for the next value in the map.
    ///
    /// `Deserialize` implementations should typically use
    /// `MapVisitor::visit_value` instead.
    fn visit_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
        where V: DeserializeSeed;

    /// This returns `Ok(Some((key, value)))` for the next (key-value) pair in
    /// the map, or `Ok(None)` if there are no more remaining items.
    ///
    /// `MapVisitor` implementations should override the default behavior if a
    /// more efficient implementation is possible.
    ///
    /// `Deserialize` implementations should typically use `MapVisitor::visit`
    /// instead.
    #[inline]
    fn visit_seed<K, V>(&mut self,
                        kseed: K,
                        vseed: V)
                        -> Result<Option<(K::Value, V::Value)>, Self::Error>
        where K: DeserializeSeed,
              V: DeserializeSeed
    {
        match try!(self.visit_key_seed(kseed)) {
            Some(key) => {
                let value = try!(self.visit_value_seed(vseed));
                Ok(Some((key, value)))
            }
            None => Ok(None),
        }
    }

    /// This returns `Ok(Some(key))` for the next key in the map, or `Ok(None)`
    /// if there are no more remaining entries.
    ///
    /// This method exists as a convenience for `Deserialize` implementations.
    /// `MapVisitor` implementations should not override the default behavior.
    #[inline]
    fn visit_key<K>(&mut self) -> Result<Option<K>, Self::Error>
        where K: Deserialize
    {
        self.visit_key_seed(PhantomData)
    }

    /// This returns a `Ok(value)` for the next value in the map.
    ///
    /// This method exists as a convenience for `Deserialize` implementations.
    /// `MapVisitor` implementations should not override the default behavior.
    #[inline]
    fn visit_value<V>(&mut self) -> Result<V, Self::Error>
        where V: Deserialize
    {
        self.visit_value_seed(PhantomData)
    }

    /// This returns `Ok(Some((key, value)))` for the next (key-value) pair in
    /// the map, or `Ok(None)` if there are no more remaining items.
    ///
    /// This method exists as a convenience for `Deserialize` implementations.
    /// `MapVisitor` implementations should not override the default behavior.
    #[inline]
    fn visit<K, V>(&mut self) -> Result<Option<(K, V)>, Self::Error>
        where K: Deserialize,
              V: Deserialize
    {
        self.visit_seed(PhantomData, PhantomData)
    }

    /// Return the lower and upper bound of items remaining in the sequence.
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl<'a, V_> MapVisitor for &'a mut V_
    where V_: MapVisitor
{
    type Error = V_::Error;

    #[inline]
    fn visit_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
        where K: DeserializeSeed
    {
        (**self).visit_key_seed(seed)
    }

    #[inline]
    fn visit_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
        where V: DeserializeSeed
    {
        (**self).visit_value_seed(seed)
    }

    #[inline]
    fn visit_seed<K, V>(&mut self,
                        kseed: K,
                        vseed: V)
                        -> Result<Option<(K::Value, V::Value)>, Self::Error>
        where K: DeserializeSeed,
              V: DeserializeSeed
    {
        (**self).visit_seed(kseed, vseed)
    }

    #[inline]
    fn visit<K, V>(&mut self) -> Result<Option<(K, V)>, V_::Error>
        where K: Deserialize,
              V: Deserialize
    {
        (**self).visit()
    }

    #[inline]
    fn visit_key<K>(&mut self) -> Result<Option<K>, V_::Error>
        where K: Deserialize
    {
        (**self).visit_key()
    }

    #[inline]
    fn visit_value<V>(&mut self) -> Result<V, V_::Error>
        where V: Deserialize
    {
        (**self).visit_value()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (**self).size_hint()
    }
}

///////////////////////////////////////////////////////////////////////////////

/// `EnumVisitor` is a visitor that is created by the `Deserializer` and passed
/// to the `Deserialize` in order to identify which variant of an enum to
/// deserialize.
pub trait EnumVisitor: Sized {
    /// The error type that can be returned if some error occurs during
    /// deserialization.
    type Error: Error;
    /// The `Visitor` that will be used to deserialize the content of the enum
    /// variant.
    type Variant: VariantVisitor<Error = Self::Error>;

    /// `visit_variant` is called to identify which variant to deserialize.
    ///
    /// `Deserialize` implementations should typically use
    /// `EnumVisitor::visit_variant` instead.
    fn visit_variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
        where V: DeserializeSeed;

    /// `visit_variant` is called to identify which variant to deserialize.
    ///
    /// This method exists as a convenience for `Deserialize` implementations.
    /// `EnumVisitor` implementations should not override the default behavior.
    #[inline]
    fn visit_variant<V>(self) -> Result<(V, Self::Variant), Self::Error>
        where V: Deserialize
    {
        self.visit_variant_seed(PhantomData)
    }
}

/// `VariantVisitor` is a visitor that is created by the `Deserializer` and
/// passed to the `Deserialize` to deserialize the content of a particular enum
/// variant.
pub trait VariantVisitor: Sized {
    /// The error type that can be returned if some error occurs during
    /// deserialization. Must match the error type of our `EnumVisitor`.
    type Error: Error;

    /// Called when deserializing a variant with no values.
    ///
    /// If the data contains a different type of variant, the following
    /// `invalid_type` error should be constructed:
    ///
    /// ```rust,ignore
    /// fn visit_unit(self) -> Result<(), Self::Error> {
    ///     // What the data actually contained; suppose it is a tuple variant.
    ///     let unexp = Unexpected::TupleVariant;
    ///     Err(de::Error::invalid_type(unexp, &"unit variant"))
    /// }
    /// ```
    fn visit_unit(self) -> Result<(), Self::Error>;

    /// Called when deserializing a variant with a single value.
    ///
    /// `Deserialize` implementations should typically use
    /// `VariantVisitor::visit_newtype` instead.
    ///
    /// If the data contains a different type of variant, the following
    /// `invalid_type` error should be constructed:
    ///
    /// ```rust,ignore
    /// fn visit_newtype_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
    ///     where T: de::DeserializeSeed
    /// {
    ///     // What the data actually contained; suppose it is a unit variant.
    ///     let unexp = Unexpected::UnitVariant;
    ///     Err(de::Error::invalid_type(unexp, &"newtype variant"))
    /// }
    /// ```
    fn visit_newtype_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
        where T: DeserializeSeed;

    /// Called when deserializing a variant with a single value.
    ///
    /// This method exists as a convenience for `Deserialize` implementations.
    /// `VariantVisitor` implementations should not override the default
    /// behavior.
    #[inline]
    fn visit_newtype<T>(self) -> Result<T, Self::Error>
        where T: Deserialize
    {
        self.visit_newtype_seed(PhantomData)
    }

    /// Called when deserializing a tuple-like variant.
    ///
    /// The `len` is the number of fields expected in the tuple variant.
    ///
    /// If the data contains a different type of variant, the following
    /// `invalid_type` error should be constructed:
    ///
    /// ```rust,ignore
    /// fn visit_tuple<V>(self,
    ///                   _len: usize,
    ///                   _visitor: V) -> Result<V::Value, Self::Error>
    ///     where V: Visitor
    /// {
    ///     // What the data actually contained; suppose it is a unit variant.
    ///     let unexp = Unexpected::UnitVariant;
    ///     Err(Error::invalid_type(unexp, &"tuple variant"))
    /// }
    /// ```
    fn visit_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// Called when deserializing a struct-like variant.
    ///
    /// The `fields` are the names of the fields of the struct variant.
    ///
    /// If the data contains a different type of variant, the following
    /// `invalid_type` error should be constructed:
    ///
    /// ```rust,ignore
    /// fn visit_struct<V>(self,
    ///                    _fields: &'static [&'static str],
    ///                    _visitor: V) -> Result<V::Value, Self::Error>
    ///     where V: Visitor
    /// {
    ///     // What the data actually contained; suppose it is a unit variant.
    ///     let unexp = Unexpected::UnitVariant;
    ///     Err(Error::invalid_type(unexp, &"struct variant"))
    /// }
    /// ```
    fn visit_struct<V>(self,
                       fields: &'static [&'static str],
                       visitor: V)
                       -> Result<V::Value, Self::Error>
        where V: Visitor;
}

///////////////////////////////////////////////////////////////////////////////

/// Used in error messages.
///
/// - expected `a`
/// - expected `a` or `b`
/// - expected one of `a`, `b`, `c`
///
/// The slice of names must not be empty.
struct OneOf {
    names: &'static [&'static str],
}

impl Display for OneOf {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.names.len() {
            0 => panic!(), // special case elsewhere
            1 => write!(formatter, "`{}`", self.names[0]),
            2 => write!(formatter, "`{}` or `{}`", self.names[0], self.names[1]),
            _ => {
                try!(write!(formatter, "one of "));
                for (i, alt) in self.names.iter().enumerate() {
                    if i > 0 {
                        try!(write!(formatter, ", "));
                    }
                    try!(write!(formatter, "`{}`", alt));
                }
                Ok(())
            }
        }
    }
}
