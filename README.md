# serde_state &emsp; [![Build Status]][travis] [![Latest Version]][crates.io] [![Rustc Version 1.13+]][rustc]

[Build Status]: https://api.travis-ci.org/Marwes/serde_state.svg?branch=master
[travis]: https://travis-ci.org/Marwes/serde_state
[Latest Version]: https://img.shields.io/crates/v/serde_state.svg
[crates.io]: https://crates.io/crates/serde_state
[Rustc Version 1.13+]: https://img.shields.io/badge/rustc-1.13+-lightgray.svg
[rustc]: https://blog.rust-lang.org/2016/11/10/Rust-1.13.html


**serde_state is an experimental addition to serde which makes it possible to pass state to the [de]serialized data structures.**

## Serde in action

See https://docs.rs/serde_state for examples

```toml
[dependencies]

# The core APIs, including the Serialize and Deserialize traits. Always
# required when using Serde.
serde = "1.0"

# Support for #[derive(Serialize, Deserialize)]. Required if you want Serde
# to work for structs and enums defined in your crate.
serde_derive = "1.0"

# Each data format lives in its own crate; the sample code below uses JSON
# but you may be using a different one.
serde_json = "1.0"
```

</details>
<p></p>

```rust
#[macro_use]
extern crate serde_derive;

extern crate serde;
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
about Serde. If IRC is not your thing or you don't get a good response, we are
happy to respond to [GitHub issues](https://github.com/serde-rs/serde/issues/new)
as well.

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
