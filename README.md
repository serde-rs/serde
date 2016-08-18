# Serde &emsp; [![Build Status](https://api.travis-ci.org/serde-rs/serde.svg?branch=master)](https://travis-ci.org/serde-rs/serde) [![Coverage Status](https://coveralls.io/repos/serde-rs/serde/badge.svg?branch=master&service=github)](https://coveralls.io/github/serde-rs/serde?branch=master) [![Latest Version](https://img.shields.io/crates/v/serde.svg)](https://crates.io/crates/serde) [![Clippy Linting Result](https://clippy.bashy.io/github/serde-rs/serde/master/badge.svg)](https://clippy.bashy.io/github/serde-rs/serde/master/log)

**Serde is a framework for *ser*ializing and *de*serializing Rust data structures efficiently and generically.**

---

You may be looking for:

- [An overview of Serde](http://serde.rs/)
- [Data formats supported by Serde](http://serde.rs/#data-formats)
- [Setting up `#[derive(Serialize, Deserialize)]`](http://serde.rs/codegen.html)
- [Examples](http://serde.rs/examples.html)
- [API documentation](http://docs.serde.rs/serde/)

## Serde in action

```rust
#![feature(plugin, custom_derive)]
#![plugin(serde_macros)]

extern crate serde_json;

#[derive(Serialize, Deserialize, Debug)]
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let point = Point { x: 1, y: 2 };

    // Convert the Point to a JSON string.
    let serialized = serde_json::to_string(&point).unwrap();

    // Prints serialized = {"x":1,"y":2}
    println!("serialized = {}", serialized);

    // Convert the JSON string back to a Point.
    let deserialized: Point = serde_json::from_str(&serialized).unwrap();

    // Prints deserialized = Point { x: 1, y: 2 }
    println!("deserialized = {:?}", deserialized);
}
```

## Getting help

Serde developers live in the #serde channel on
[`irc.mozilla.org`](https://wiki.mozilla.org/IRC). The #rust channel is also a
good resource with generally faster response time but less specific knowledge
about Serde. If IRC is not your thing, we are happy to respond to [GitHub
issues](https://github.com/serde-rs/serde/issues/new) as well.

## License

Serde is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Serde by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
