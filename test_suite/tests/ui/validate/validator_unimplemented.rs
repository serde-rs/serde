use serde_derive::Deserialize;

#[derive(Deserialize)]
#[serde(validator)]
struct ValidatorStruct {
    mail: String,
}

fn main() {}
