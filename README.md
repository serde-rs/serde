Experimental Rust Serialization Library.

[![Build Status](https://travis-ci.org/erickt/rust-serde.png?branch=master)](https://travis-ci.org/erickt/rust-serde)

This is an experiment to modernize rust's `libserialize` library. It is designed to implement https://github.com/rust-lang/rfcs/pull/22. `rust-serde` is an attempt to address a major shortcoming in `libserialize`. For normal structures, when you say you want to deserialize into:

```rust
struct Foo {
    x: int,
    y: int,
}
```

`libserialize`'s deserializer essentially asks for:

* Is the next value a struct named "Foo"? If not, error.
* Is the next field named "x"? If not, error.
* Is the next value an "int"? If not, error.
* Is the next field named "y"? If not, error.
* Is the next value an "int"? If not, error.
* Is the struct finished? If not, error.

While this works for user defined structures, it cannot support deserializing into a value like `json::Json`, which is an enum that can represent every JSON value. In order to support that, it needs to be able to do some lookahead:

* What is the next value type?
    * If a struct, parse a struct.
    * If an integer, parse an integer.
    * ...

More formally, `libserialize` implements a LL(0) grammar, whereas `json::Json` requires a LL(1) grammar. `rust-serde` provides this by implementing a serializer and deserializer that produces a tagged token stream of values. This enables a `Deserializable` for `json::Json` to look at the next token before deciding on how to parse the value.

---

There is now also a new library variation called `serde2`. This removes the need for tagged values and replaces them with a `Visitor` pattern. This pattern is very similar to the `Iterator` pattern, but it threads some custom state through visiting each type. This gets many of the benefits of the `serde` library without needing to always pay for tagging the variants.
