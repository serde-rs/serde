This example demonstrates how to use Serde with Syntex. On stable or nightly
with Syntex, it can be built with:

```
% rustup run stable cargo run
     Running `target/debug/serde-syntex-example`
{"x":1,"y":2}
Point { x: 1, y: 2 }

% rustup run nightly cargo run
     Running `target/debug/serde-syntex-example`
{"x":1,"y":2}
Point { x: 1, y: 2 }
```

On nightly, it can use a plugin with:

```
% rustup run nightly cargo run --features nightly --no-default-features
```
