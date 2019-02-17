#### To run unit tests

```sh
cargo test
```

#### To run ui tests

```sh
(cd deps && cargo clean && cargo update && cargo build)
cargo test --features compiletest
```

#### To update goldens after running ui tests

```sh
tests/ui/update-references.sh
```
