use serde_derive::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(tag = "type")]
enum Serializable {
    /// No errors should be reported, equivalent to Unit representation
    Tuple0(),
    /// No errors should be reported, delegates decision to the inner type
    Tuple1(u8),
    /// Error should be reported
    Tuple2(u8, u8),

    /// No errors should be reported, equivalent to Tuple0 representation
    Tuple1as0S(#[serde(skip_serializing)] u8),
    /// No errors should be reported, equivalent to Tuple1 representation
    Tuple1as0D(#[serde(skip_deserializing)] u8),

    /// No errors should be reported, equivalent to Tuple1 representation
    Tuple2as1S(#[serde(skip_serializing)] u8, u8),
    /// Error should be reported, equivalent to Tuple2 representation
    Tuple2as1D(#[serde(skip_deserializing)] u8, u8),

    /// Error should be reported, equivalent to Tuple2 representation
    Tuple3as2S(#[serde(skip_serializing)] u8, u8, u8),
    /// Error should be reported, equivalent to Tuple3 representation
    Tuple3as2D(#[serde(skip_deserializing)] u8, u8, u8),
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum Deserializable {
    /// No errors should be reported, equivalent to Unit representation
    Tuple0(),
    /// No errors should be reported, delegates decision to the inner type
    Tuple1(u8),
    /// Error should be reported
    Tuple2(u8, u8),

    /// No errors should be reported, equivalent to Tuple1 representation
    Tuple1as0S(#[serde(skip_serializing)] u8),
    /// No errors should be reported, equivalent to Tuple0 representation
    Tuple1as0D(#[serde(skip_deserializing)] u8),

    /// Error should be reported, equivalent to Tuple2 representation
    Tuple2as1S(#[serde(skip_serializing)] u8, u8),
    /// No errors should be reported, equivalent to Tuple1 representation
    Tuple2as1D(#[serde(skip_deserializing)] u8, u8),

    /// Error should be reported, equivalent to Tuple3 representation
    Tuple3as2S(#[serde(skip_serializing)] u8, u8, u8),
    /// Error should be reported, equivalent to Tuple2 representation
    Tuple3as2D(#[serde(skip_deserializing)] u8, u8, u8),
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum Both {
    /// No errors should be reported, equivalent to Unit representation
    Tuple0(),
    /// No errors should be reported, delegates decision to the inner type
    Tuple1(u8),
    /// Error should be reported
    Tuple2(u8, u8),

    /// Serialize: No errors should be reported, equivalent to Tuple0 representation
    /// Deserialize: No errors should be reported, equivalent to Tuple1 representation
    Tuple1as0S(#[serde(skip_serializing)] u8),
    /// Serialize: No errors should be reported, equivalent to Tuple1 representation
    /// Deserialize: No errors should be reported, equivalent to Tuple0 representation
    Tuple1as0D(#[serde(skip_deserializing)] u8),

    /// Serialize: No errors should be reported, equivalent to Tuple1 representation
    /// Deserialize: Error should be reported, equivalent to Tuple2 representation
    Tuple2as1S(#[serde(skip_serializing)] u8, u8),
    /// Serialize: Error should be reported, equivalent to Tuple2 representation
    /// Deserialize: No errors should be reported, equivalent to Tuple1 representation
    Tuple2as1D(#[serde(skip_deserializing)] u8, u8),

    /// Serialize: Error should be reported, equivalent to Tuple2
    /// Deserialize: Error should be reported, equivalent to Tuple3
    Tuple3as2S(#[serde(skip_serializing)] u8, u8, u8),
    /// Serialize: Error should be reported, equivalent to Tuple3
    /// Deserialize: Error should be reported, equivalent to Tuple2
    Tuple3as2D(#[serde(skip_deserializing)] u8, u8, u8),
}

fn main() {}
