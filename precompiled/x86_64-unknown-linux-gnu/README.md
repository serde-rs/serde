```toml
# Cargo.toml

[dependencies]
serde = "1"  # no features=["derive"]
serde_derive-x86_64-unknown-linux-gnu = "1.0.171-alpha.1"
```

```rust
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct MyStruct {
    /* ... */
}
```
