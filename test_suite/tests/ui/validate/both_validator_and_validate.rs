use serde_derive::Deserialize;

#[derive(validator::Validate, Deserialize)]
#[serde(validate = "validator::Validate::validate")]
#[serde(validator)]
struct ValidatorStruct {
    #[validate(email)]
    mail: String,
}

fn main() {}
