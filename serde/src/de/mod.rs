//! Generic deserialization framework.

#[cfg(feature = "std")]
use std::error;
#[cfg(not(feature = "std"))]
use error;

#[cfg(all(not(feature = "std"), feature = "collections"))]
use collections::{String, Vec};

use core::fmt::{self, Display};
use core::marker::PhantomData;

///////////////////////////////////////////////////////////////////////////////

pub mod impls;
pub mod value;
mod from_primitive;

// Helpers used by generated code. Not public API.
#[doc(hidden)]
pub mod private;

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
        Error::custom(InvalidType { unexp: unexp, exp: exp })
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
        Error::custom(InvalidValue { unexp: unexp, exp: exp })
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
        Error::custom(InvalidLength { len: len, exp: exp })
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
        Error::custom(UnknownVariant { variant: variant, expected: expected })
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
        Error::custom(UnknownField { field: field, expected: expected })
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

    /// The input contained an unsigned integer `usize`, `u8`, `u16`, `u32` or
    /// `u64` that was not expected.
    Unsigned(u64),

    /// The input contained a signed integer `isize`, `i8`, `i16`, `i32` or
    /// `i64` that was not expected.
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
/// message should complete the sentence "This Visitor expects to receive ...",
/// for example the message could be "an integer between 0 and 64". The message
/// should not be capitalized and should not end with a period.
///
/// Within the context of a `Visitor` implementation, the `Visitor` itself
/// (`&self`) is an implementation of this trait.
///
/// ```rust
/// # use serde::de::{Error, Unexpected, Visitor};
/// # use std::fmt;
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

impl<T> Expected for T where T: Visitor {
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

/// `Deserialize` represents a type that can be deserialized.
pub trait Deserialize: Sized {
    /// Deserialize this value given this `Deserializer`.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer;
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
/// # enum Error {}
/// fn func<T: Deserialize>() -> Result<T, Error>
/// # { unimplemented!() }
/// ```
///
/// Adjusting an API like this to support stateful deserialization is a matter
/// of accepting a seed as input:
///
/// ```rust
/// # use serde::de::DeserializeSeed;
/// # enum Error {}
/// fn func_seed<T: DeserializeSeed>(seed: T) -> Result<T::Value, Error>
/// # { unimplemented!() }
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
/// into it. This requires stateful deserialization using the DeserializeSeed
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
/// # fn example<D: Deserializer>(deserializer: D) -> Result<(), D::Error> {
/// let visitor = FlattenedVecVisitor(PhantomData);
/// let flattened: Vec<u64> = deserializer.deserialize_seq(visitor)?;
/// # Ok(()) }
/// ```
pub trait DeserializeSeed: Sized {
    /// The type produced by using this seed.
    type Value;

    /// Equivalent to the more common `Deserialize::deserialize` method, except
    /// with some initial piece of data (the seed) passed in.
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer;
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

/// `Deserializer` is a trait that can deserialize values by threading a `Visitor` trait through a
/// value. It supports two entry point styles which enables different kinds of deserialization.
///
/// 1) The `deserialize` method. File formats like JSON embed the type of its construct in its file
///    format. This allows the `Deserializer` to deserialize into a generic type like
///    `json::Value`, which can represent all JSON types.
///
/// 2) The `deserialize_*` methods. File formats like bincode do not embed in its format how to
///    decode its values. It relies instead on the `Deserialize` type to hint to the `Deserializer`
///    with the `deserialize_*` methods how it should parse the next value. One downside though to
///    only supporting the `deserialize_*` types is that it does not allow for deserializing into a
///    generic `json::Value`-esque type.
pub trait Deserializer: Sized {
    /// The error type that can be returned if some error occurs during deserialization.
    type Error: Error;

    /// This method walks a visitor through a value as it is being deserialized.
    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting a `bool` value.
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting an `usize` value.
    /// A reasonable default is to forward to `deserialize_u64`.
    fn deserialize_usize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting an `u8` value.
    /// A reasonable default is to forward to `deserialize_u64`.
    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting an `u16` value.
    /// A reasonable default is to forward to `deserialize_u64`.
    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting an `u32` value.
    /// A reasonable default is to forward to `deserialize_u64`.
    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting an `u64` value.
    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting an `isize` value.
    /// A reasonable default is to forward to `deserialize_i64`.
    fn deserialize_isize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting an `i8` value.
    /// A reasonable default is to forward to `deserialize_i64`.
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting an `i16` value.
    /// A reasonable default is to forward to `deserialize_i64`.
    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting an `i32` value.
    /// A reasonable default is to forward to `deserialize_i64`.
    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting an `i64` value.
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting a `f32` value.
    /// A reasonable default is to forward to `deserialize_f64`.
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting a `f64` value.
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting a `char` value.
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting a `&str` value.
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting a `String` value.
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting an `unit` value.
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting an `Option` value. This allows
    /// deserializers that encode an optional value as a nullable value to convert the null value
    /// into a `None`, and a regular value as `Some(value)`.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting a sequence value. This allows
    /// deserializers to parse sequences that aren't tagged as sequences.
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting a fixed size array. This allows
    /// deserializers to parse arrays that aren't tagged as arrays.
    fn deserialize_seq_fixed_size<V>(self,
                                     len: usize,
                                     visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting a `&[u8]`. This allows
    /// deserializers that provide a custom byte vector serialization to properly deserialize the
    /// type.
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting a `Vec<u8>`. This allows
    /// deserializers that provide a custom byte vector serialization to properly deserialize the
    /// type and prevent needless intermediate allocations that would occur when going through
    /// `&[u8]`.
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting a map of values. This allows
    /// deserializers to parse sequences that aren't tagged as maps.
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting a unit struct. This allows
    /// deserializers to a unit struct that aren't tagged as a unit struct.
    fn deserialize_unit_struct<V>(self,
                                  name: &'static str,
                                  visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting a newtype struct. This allows
    /// deserializers to a newtype struct that aren't tagged as a newtype struct.
    /// A reasonable default is to simply deserialize the expected value directly.
    fn deserialize_newtype_struct<V>(self,
                                     name: &'static str,
                                     visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting a tuple struct. This allows
    /// deserializers to parse sequences that aren't tagged as sequences.
    fn deserialize_tuple_struct<V>(self,
                                   name: &'static str,
                                   len: usize,
                                   visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting a struct. This allows
    /// deserializers to parse sequences that aren't tagged as maps.
    fn deserialize_struct<V>(self,
                             name: &'static str,
                             fields: &'static [&'static str],
                             visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting some sort of struct field
    /// name.  This allows deserializers to choose between &str, usize, or &[u8] to properly
    /// deserialize a struct field.
    fn deserialize_struct_field<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting a tuple value. This allows
    /// deserializers that provide a custom tuple serialization to properly deserialize the type.
    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type is expecting an enum value. This allows
    /// deserializers that provide a custom enumeration serialization to properly deserialize the
    /// type.
    fn deserialize_enum<V>(self,
                           name: &'static str,
                           variants: &'static [&'static str],
                           visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// This method hints that the `Deserialize` type needs to deserialize a value whose type
    /// doesn't matter because it is ignored.
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
    /// # struct S { max: usize }
    /// # impl serde::de::Visitor for S {
    /// # type Value = ();
    /// fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    ///     write!(formatter, "an integer between 0 and {}", self.max)
    /// }
    /// # }
    /// ```
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result;

    /// `visit_bool` deserializes a `bool` into a `Value`.
    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
        where E: Error,
    {
        Err(Error::invalid_type(Unexpected::Bool(v), &self))
    }

    /// `visit_isize` deserializes a `isize` into a `Value`.
    fn visit_isize<E>(self, v: isize) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_i64(v as i64)
    }

    /// `visit_i8` deserializes a `i8` into a `Value`.
    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_i64(v as i64)
    }

    /// `visit_i16` deserializes a `i16` into a `Value`.
    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_i64(v as i64)
    }

    /// `visit_i32` deserializes a `i32` into a `Value`.
    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_i64(v as i64)
    }

    /// `visit_i64` deserializes a `i64` into a `Value`.
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where E: Error,
    {
        Err(Error::invalid_type(Unexpected::Signed(v), &self))
    }

    /// `visit_usize` deserializes a `usize` into a `Value`.
    fn visit_usize<E>(self, v: usize) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_u64(v as u64)
    }

    /// `visit_u8` deserializes a `u8` into a `Value`.
    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_u64(v as u64)
    }

    /// `visit_u16` deserializes a `u16` into a `Value`.
    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_u64(v as u64)
    }

    /// `visit_u32` deserializes a `u32` into a `Value`.
    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_u64(v as u64)
    }

    /// `visit_u64` deserializes a `u64` into a `Value`.
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where E: Error,
    {
        Err(Error::invalid_type(Unexpected::Unsigned(v), &self))
    }

    /// `visit_f32` deserializes a `f32` into a `Value`.
    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_f64(v as f64)
    }

    /// `visit_f64` deserializes a `f64` into a `Value`.
    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where E: Error,
    {
        Err(Error::invalid_type(Unexpected::Float(v), &self))
    }

    /// `visit_char` deserializes a `char` into a `Value`.
    #[inline]
    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_str(::utils::encode_utf8(v).as_str())
    }

    /// `visit_str` deserializes a `&str` into a `Value`.
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where E: Error,
    {
        Err(Error::invalid_type(Unexpected::Str(v), &self))
    }

    /// `visit_string` deserializes a `String` into a `Value`.  This allows a deserializer to avoid
    /// a copy if it is deserializing a string from a `String` type.  By default it passes a `&str`
    /// to the `visit_str` method.
    #[inline]
    #[cfg(any(feature = "std", feature = "collections"))]
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_str(&v)
    }

    /// `visit_unit` deserializes a `()` into a `Value`.
    fn visit_unit<E>(self) -> Result<Self::Value, E>
        where E: Error,
    {
        Err(Error::invalid_type(Unexpected::Unit, &self))
    }

    /// `visit_none` deserializes a none value into a `Value`.
    fn visit_none<E>(self) -> Result<Self::Value, E>
        where E: Error,
    {
        Err(Error::invalid_type(Unexpected::Option, &self))
    }

    /// `visit_some` deserializes a value into a `Value`.
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer,
    {
        let _ = deserializer;
        Err(Error::invalid_type(Unexpected::Option, &self))
    }

    /// `visit_newtype_struct` deserializes a value into a `Value`.
    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer,
    {
        let _ = deserializer;
        Err(Error::invalid_type(Unexpected::NewtypeStruct, &self))
    }

    /// `visit_seq` deserializes a `SeqVisitor` into a `Value`.
    fn visit_seq<V>(self, visitor: V) -> Result<Self::Value, V::Error>
        where V: SeqVisitor,
    {
        let _ = visitor;
        Err(Error::invalid_type(Unexpected::Seq, &self))
    }

    /// `visit_map` deserializes a `MapVisitor` into a `Value`.
    fn visit_map<V>(self, visitor: V) -> Result<Self::Value, V::Error>
        where V: MapVisitor,
    {
        let _ = visitor;
        Err(Error::invalid_type(Unexpected::Map, &self))
    }

    /// `visit_enum` deserializes a `EnumVisitor` into a `Value`.
    fn visit_enum<V>(self, visitor: V) -> Result<Self::Value, V::Error>
        where V: EnumVisitor,
    {
        let _ = visitor;
        Err(Error::invalid_type(Unexpected::Enum, &self))
    }

    /// `visit_bytes` deserializes a `&[u8]` into a `Value`.
    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where E: Error,
    {
        let _ = v;
        Err(Error::invalid_type(Unexpected::Bytes(v), &self))
    }

    /// `visit_byte_buf` deserializes a `Vec<u8>` into a `Value`.
    #[cfg(any(feature = "std", feature = "collections"))]
    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
        where E: Error,
    {
        self.visit_bytes(&v)
    }
}

///////////////////////////////////////////////////////////////////////////////

/// `SeqVisitor` visits each item in a sequence.
///
/// This is a trait that a `Deserializer` passes to a `Visitor` implementation, which deserializes
/// each item in a sequence.
pub trait SeqVisitor {
    /// The error type that can be returned if some error occurs during deserialization.
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
    /// `SeqVisitor` implementations should not need to override the default
    /// behavior.
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

impl<'a, V> SeqVisitor for &'a mut V where V: SeqVisitor {
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
    /// The error type that can be returned if some error occurs during deserialization.
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
    fn visit_seed<K, V>(&mut self, kseed: K, vseed: V) -> Result<Option<(K::Value, V::Value)>, Self::Error>
        where K: DeserializeSeed,
              V: DeserializeSeed
    {
        match try!(self.visit_key_seed(kseed)) {
            Some(key) => {
                let value = try!(self.visit_value_seed(vseed));
                Ok(Some((key, value)))
            }
            None => Ok(None)
        }
    }

    /// This returns `Ok(Some(key))` for the next key in the map, or `Ok(None)`
    /// if there are no more remaining entries.
    ///
    /// This method exists as a convenience for `Deserialize` implementations.
    /// `MapVisitor` implementations should not need to override the default
    /// behavior.
    #[inline]
    fn visit_key<K>(&mut self) -> Result<Option<K>, Self::Error>
        where K: Deserialize
    {
        self.visit_key_seed(PhantomData)
    }

    /// This returns a `Ok(value)` for the next value in the map.
    ///
    /// This method exists as a convenience for `Deserialize` implementations.
    /// `MapVisitor` implementations should not need to override the default
    /// behavior.
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
    /// `MapVisitor` implementations should not need to override the default
    /// behavior.
    #[inline]
    fn visit<K, V>(&mut self) -> Result<Option<(K, V)>, Self::Error>
        where K: Deserialize,
              V: Deserialize,
    {
        self.visit_seed(PhantomData, PhantomData)
    }

    /// Return the lower and upper bound of items remaining in the sequence.
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl<'a, V_> MapVisitor for &'a mut V_ where V_: MapVisitor {
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
    fn visit_seed<K, V>(&mut self, kseed: K, vseed: V) -> Result<Option<(K::Value, V::Value)>, Self::Error>
        where K: DeserializeSeed,
              V: DeserializeSeed
    {
        (**self).visit_seed(kseed, vseed)
    }

    #[inline]
    fn visit<K, V>(&mut self) -> Result<Option<(K, V)>, V_::Error>
        where K: Deserialize,
              V: Deserialize,
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
    /// The error type that can be returned if some error occurs during deserialization.
    type Error: Error;
    /// The `Visitor` that will be used to deserialize the content of the enum
    /// variant.
    type Variant: VariantVisitor<Error=Self::Error>;

    /// `visit_variant` is called to identify which variant to deserialize.
    ///
    /// `Deserialize` implementations should typically use
    /// `EnumVisitor::visit_variant` instead.
    fn visit_variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
        where V: DeserializeSeed;

    /// `visit_variant` is called to identify which variant to deserialize.
    ///
    /// This method exists as a convenience for `Deserialize` implementations.
    /// `EnumVisitor` implementations should not need to override the default
    /// behavior.
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
    /// The error type that can be returned if some error occurs during deserialization.
    type Error: Error;

    /// `visit_unit` is called when deserializing a variant with no values.
    fn visit_unit(self) -> Result<(), Self::Error>;

    /// `visit_newtype` is called when deserializing a variant with a single value.
    /// A good default is often to use the `visit_tuple` method to deserialize a `(value,)`.
    ///
    /// `Deserialize` implementations should typically use
    /// `VariantVisitor::visit_newtype` instead.
    fn visit_newtype_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
        where T: DeserializeSeed;

    /// `visit_newtype` is called when deserializing a variant with a single value.
    /// A good default is often to use the `visit_tuple` method to deserialize a `(value,)`.
    ///
    /// This method exists as a convenience for `Deserialize` implementations.
    /// `VariantVisitor` implementations should not need to override the default
    /// behavior.
    #[inline]
    fn visit_newtype<T>(self) -> Result<T, Self::Error>
        where T: Deserialize
    {
        self.visit_newtype_seed(PhantomData)
    }

    /// `visit_tuple` is called when deserializing a tuple-like variant.
    /// If no tuple variants are expected, yield a
    /// `Err(serde::de::Error::invalid_type(serde::de::Type::TupleVariant))`
    fn visit_tuple<V>(self,
                      len: usize,
                      visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;

    /// `visit_struct` is called when deserializing a struct-like variant.
    /// If no struct variants are expected, yield a
    /// `Err(serde::de::Error::invalid_type(serde::de::Type::StructVariant))`
    fn visit_struct<V>(self,
                       fields: &'static [&'static str],
                       visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor;
}

///////////////////////////////////////////////////////////////////////////////

/// Used in error messages.
///
/// - expected `a`
/// - expected `a` or `b`
/// - expected one of `a`, `b`, `c`
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
