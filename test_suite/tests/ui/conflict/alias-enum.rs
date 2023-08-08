#![allow(non_camel_case_types)]

use serde_derive::Deserialize;

#[derive(Deserialize)]
enum E {
    S1 {
        /// Expected error on "alias b", because this is a name of other field
        /// Error on "alias a" is not expected because this is a name of this field
        /// Error on "alias c" is not expected because field `c` is skipped
        #[serde(alias = "a", alias = "b", alias = "c")]
        a: (),

        /// Expected error on "alias c", because it is already used as alias of `a`
        #[serde(alias = "c")]
        b: (),

        #[serde(skip_deserializing)]
        c: (),
    },

    S2 {
        /// Expected error on "alias c", because this is a name of other field after
        /// applying rename rules
        #[serde(alias = "b", alias = "c")]
        a: (),

        #[serde(rename = "c")]
        b: (),
    },

    #[serde(rename_all = "UPPERCASE")]
    S3 {
        /// Expected error on "alias B", because this is a name of field after
        /// applying rename rules
        #[serde(alias = "B", alias = "c")]
        a: (),
        b: (),
    },
}

#[derive(Deserialize)]
enum E1 {
    /// Expected error on "alias b", because this is a name of other variant
    /// Error on "alias a" is not expected because this is a name of this variant
    /// Error on "alias c" is not expected because variant `c` is skipped
    #[serde(alias = "a", alias = "b", alias = "c")]
    a,

    /// Expected error on "alias c", because it is already used as alias of `a`
    #[serde(alias = "c")]
    b,

    #[serde(skip_deserializing)]
    c,
}

#[derive(Deserialize)]
enum E2 {
    /// Expected error on "alias c", because this is a name of other variant after
    /// applying rename rules
    #[serde(alias = "b", alias = "c")]
    a,

    #[serde(rename = "c")]
    b,
}

#[derive(Deserialize)]
#[serde(rename_all = "UPPERCASE")]
enum E3 {
    /// Expected error on "alias B", because this is a name of variant after
    /// applying rename rules
    #[serde(alias = "B", alias = "c")]
    a,
    b,
}

fn main() {}
