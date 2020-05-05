# Serde &emsp; [![Build Status]][actions] [![Latest Version]][crates.io] [![serde: rustc 1.13+]][Rust 1.13] [![serde_derive: rustc 1.31+]][Rust 1.31]

[Build Status]: https://img.shields.io/github/workflow/status/serde-rs/serde/CI/master
[actions]: https://github.com/serde-rs/serde/actions?query=branch%3Amaster
[Latest Version]: https://img.shields.io/crates/v/serde.svg
[crates.io]: https://crates.io/crates/serde
[serde: rustc 1.13+]: https://img.shields.io/badge/serde-rustc_1.13+-lightgray.svg
[serde_derive: rustc 1.31+]: https://img.shields.io/badge/serde_derive-rustc_1.31+-lightgray.svg
[Rust 1.13]: https://blog.rust-lang.org/2016/11/10/Rust-1.13.html
[Rust 1.31]: https://blog.rust-lang.org/2018/12/06/Rust-1.31-and-rust-2018.html

**Serde is a framework for *ser*ializing and *de*serializing Rust data structures efficiently and generically.**

---

You may be looking for:

- [An overview of Serde](https://serde.rs/)
- [Data formats supported by Serde](https://serde.rs/#data-formats)
- [Setting up `#[derive(Serialize, Deserialize)]`](https://serde.rs/derive.html)
- [Examples](https://serde.rs/examples.html)
- [API documentation](https://docs.serde.rs/serde/)
- [Release notes](https://github.com/serde-rs/serde/releases)

## Serde in action

<details>
<summary>
Click to show Cargo.toml.
<a href="https://play.rust-lang.org/?edition=2018&gist=72755f28f99afc95e01d63174b28c1f5" target="_blank">Run this code in the playground.</a>
</summary>

```toml
[dependencies]

# The core APIs, including the Serialize and Deserialize traits. Always
# required when using Serde. The "derive" feature is only required when
# using #[derive(Serialize, Deserialize)] to make Serde work with structs
# and enums defined in your crate.
serde = { version = "1.0", features = ["derive"] }

# Each data format lives in its own crate; the sample code below uses JSON
# but you may be using a different one.
serde_json = "1.0"
```

</details>
<p></p>

```rust
use serde::{Serialize, Deserialize};

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

Serde developers live in the #serde channel on [`irc.mozilla.org`][irc]. The
\#rust channel is also a good resource with generally faster response time but
less specific knowledge about Serde. If IRC is not your thing or you don't get a
good response, we are happy to respond to [GitHub issues][issues] as well.

[irc]: https://wiki.mozilla.org/IRC
[issues]: https://github.com/serde-rs/serde/issues/new/choose

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Serde by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
</sub>
