## v0.7.5 (May 12, 2016)

- Fixes a codegen regression from 0.7.4 that generates invalid code for some
  generic structs (#308)
- Visiting the same struct field or map key twice is now an error (#293)
- Fields with `deserialize_with` are no longer required to implement the
  `Deserialize` trait (#311)
- Fixes a codegen bug related to `deserialize_with` in a struct containing more
  than one field (#308)
- Fixes a codegen bug related to `deserialize_with` in a struct with a type
  parameter called `D` (#315)

## v0.7.4 (May 4, 2016)

- Adds `skip_deserializing` attribute (#265)
- Adds Deserialize impl for `Box<[T]>` (#290)
- Fields with `skip_serializing` are no longer required to implement the
  `Serialize` trait (#260, #285)
- A bound of `std::default::Default` is inferred for fields that use the
  `default` attribute (#285)
- Supports codegen for structs that use defaulted generic type params (#295)
- Supports `extern crate serde` not in the top-level module (#298)

## v0.7.3

*(this release does not exist)*

## v0.7.2 (April 10, 2016)

- Fixes a codegen bug related to `deserialize_with` in a module containing a
  type alias for `Result` (#279)
- Fixes an `unused_variables` warning in codegen for an empty enum (#273)

## v0.7.1 (March 16, 2016)

*(this release is a Syntex bump only)*

## v0.7.0 (February 26, 2016)

*This release contains significant breaking changes compared to v0.6.x.*

#### Deserializer trait

- Renames `Deserializer::visit_*` to `Deserializer::deserialize_*` (#151)
- Adds `deserialize_ignored_any` method to Deserializer trait (#225)
- Overhauls the `de::Error` trait (#160, #166, #169, #249, #254)
- Adds hooks for fixed-sized arrays (#244)

#### Attributes

- Adds `deny_unknown_fields` attribute (#44 and #60)
- Adds `default="..."` attribute (#90, #216)
- Removes support for format-specific renames added in v0.4.0 (#211)
- Supports serialize- and deserialize-specific renames (#233)

#### Impls

- Adds impls for PhantomData (#248)
- Adds impls for std::net::Ip{,v4,v6}Addr (#181)
- Removes dependency on `num` crate added in v0.6.6 (#243)
- Fixes panic during serialization of Path (#57)

## v0.6.15 (February 22, 2016)

*(this release is a Syntex bump only)*

## v0.6.14 (February 18, 2016)

*(this release is a Syntex bump only)*

## v0.6.13 (February 14, 2016)

*(this release is a Syntex bump only)*

## v0.6.12 (February 12, 2016)

- Method `visit_struct_field` has been added to the `Deserializer` trait to hint
  that a struct key is expected (#223 and 064241f)

## v0.6.11 (January 23, 2016)

- Allow options to be deserialized from units (#217)

## v0.6.10 (January 18, 2016)

- Fixes panic when a non-struct/enum is annotated (#206)
- Adds a `clippy` feature to build with
  [Clippy](https://github.com/Manishearth/rust-clippy) (bfa2b69)
- An unknown serde attribute is now an error (#51, #175, #187)

## v0.6.9 (January 13, 2016)

*(this release is a Syntex bump only)*

## v0.6.8 (January 7, 2016)

*(this release is a Syntex bump only)*

## v0.6.7 (December 24, 2015)

*(this release is a Syntex bump only)*

## v0.6.6 (December 8, 2015)

- Adds impls for num::{BigInt, BigUint, Complex, Ratio} (#191)

## v0.6.5 (November 28, 2015)

*(this release is a Syntex bump only)*

## v0.6.4 (November 22, 2015)

*(this release is a Syntex bump only)*

## v0.6.3 (November 9, 2015)

*(this release is a Syntex bump only)*

## v0.6.2 (November 1, 2015)

*(this release is a Syntex bump only)*

## v0.6.1 (October 17, 2015)

- Adds attributes `skip_serializing_if_none` and `skip_serializing_if_empty`
  (c4392ff)

## v0.6.0 (September 2, 2015)

- Removes the unused EnumSeqVisitor and EnumMapVisitor traits (9a8037b)
- Removes the unused `buf` module (c14eb28)
