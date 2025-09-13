The `serde_core` crate contains Serde's trait definitions with **no support for
#\[derive()\]**.

In crates that derive an implementation of `Serialize` or `Deserialize`, you
must depend on the [`serde`] crate, not `serde_core`.

In crates that handwrite implementations of Serde traits, or only use them as
trait bounds, depending on `serde_core` is permitted. But `serde` re-exports all
of these traits and can be used for this use case too. If in doubt, disregard
`serde_core` and always use `serde`.

[`serde`]: https://crates.io/crates/serde
