use serde_derive::Deserialize;

#[derive(Deserialize)]
struct S1 {
    #[serde(alias = "a", alias = "b", alias = "c")]
    a: (),

    // Warning on "c" and "b"
    #[serde(alias = "c")]
    b: (),

    #[serde(skip_deserializing)]
    c: (),
}

#[derive(Deserialize)]
struct S2 {
    #[serde(alias = "b", alias = "c")]
    a: (),

    // Warning on "c"
    #[serde(rename = "c")]
    b: (),
}

#[derive(Deserialize)]
#[serde(rename_all = "UPPERCASE")]
struct S3 {
    #[serde(alias = "B", alias = "c")]
    a: (),

    // Warning on "b" because this collides with the "B" above after applying
    // rename rules
    b: (),
}

fn main() {
    @//fail
}
