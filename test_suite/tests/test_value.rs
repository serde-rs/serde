use serde::de::{value, IntoDeserializer};
use serde::Deserialize;

#[test]
fn test_u32_to_enum() {
    #[derive(Deserialize, Debug, PartialEq)]
    enum E {
        A,
        B,
    }

    let deserializer = IntoDeserializer::<value::Error>::into_deserializer(1u32);
    let e: E = E::deserialize(deserializer).unwrap();
    assert_eq!(E::B, e);
}

#[cfg(not(any(target_arch = "asmjs", target_arch = "wasm32")))]
#[test]
fn test_integer128() {
    let de_u128 = IntoDeserializer::<value::Error>::into_deserializer(1u128);
    let de_i128 = IntoDeserializer::<value::Error>::into_deserializer(1i128);

    // u128 to u128
    assert_eq!(1u128, u128::deserialize(de_u128).unwrap());

    // u128 to i128
    assert_eq!(1i128, i128::deserialize(de_u128).unwrap());

    // i128 to u128
    assert_eq!(1u128, u128::deserialize(de_i128).unwrap());

    // i128 to i128
    assert_eq!(1i128, i128::deserialize(de_i128).unwrap());
}
