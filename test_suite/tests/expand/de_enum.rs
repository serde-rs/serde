use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
enum DeEnum<B, C, D> {
    Unit,
    Seq(i8, B, C, D),
    Map { a: i8, b: B, c: C, d: D },

    // Make sure we can support more than one variant.
    _Unit2,
    _Seq2(i8, B, C, D),
    _Map2 { a: i8, b: B, c: C, d: D },
}
