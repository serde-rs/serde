<!-- Serde readme rendered on crates.io -->

**Serde Core is a collection of core functionalities and abstractions for Serde, a framework for *ser*ializing and *de*serializing Rust data structures efficiently and generically.**

---

Serde Core exposes core traits and utilities for interacting with Serde's data model.
It is intended to be consumed by Serde's data formats and other crates that implement the Serde traits by hand.

If you are looking to use Serde in your library/application to simply serialize and deserialize data, you most likely want to use [Serde](https://crates.io/crates/serde) directly.
Serde Core does _not_ offer any convenience functions for automatically implementing `Serialize` or `Deserialize` via a `#[derive]` macro.

You may be looking for:

- [API documentation](https://docs.rs/serde_core)
- [An overview of Serde](https://serde.rs/)
- [Data formats supported by Serde](https://serde.rs/#data-formats)
- [Examples](https://serde.rs/examples.html)
- [Release notes](https://github.com/serde-rs/serde_core/releases)

## Getting help

Serde is one of the most widely used Rust libraries so any place that Rustaceans
congregate will be able to help you out. For chat, consider trying the
[#rust-questions] or [#rust-beginners] channels of the unofficial community
Discord (invite: <https://discord.gg/rust-lang-community>), the [#rust-usage]
or [#beginners] channels of the official Rust Project Discord (invite:
<https://discord.gg/rust-lang>), or the [#general][zulip] stream in Zulip. For
asynchronous, consider the [\[rust\] tag on StackOverflow][stackoverflow], the
[/r/rust] subreddit which has a pinned weekly easy questions post, or the Rust
[Discourse forum][discourse]. It's acceptable to file a support issue in this
repo but they tend not to get as many eyes as any of the above and may get
closed without a response after some time.

[#rust-questions]: https://discord.com/channels/273534239310479360/274215136414400513
[#rust-beginners]: https://discord.com/channels/273534239310479360/273541522815713281
[#rust-usage]: https://discord.com/channels/442252698964721669/443150878111694848
[#beginners]: https://discord.com/channels/442252698964721669/448238009733742612
[zulip]: https://rust-lang.zulipchat.com/#narrow/stream/122651-general
[stackoverflow]: https://stackoverflow.com/questions/tagged/rust
[/r/rust]: https://www.reddit.com/r/rust
[discourse]: https://users.rust-lang.org
