// These just test that serde_codegen is able to produce code that compiles
// successfully when there are a variety of generics involved.

extern crate serde;
use self::serde::ser::{Serialize, Serializer};
use self::serde::de::{Deserialize, Deserializer};

//////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize)]
struct With<T> {
    t: T,
    #[serde(serialize_with="ser_i32", deserialize_with="de_i32")]
    i: i32,
}

#[derive(Serialize, Deserialize)]
struct WithRef<'a, T: 'a> {
    #[serde(skip_deserializing)]
    t: Option<&'a T>,
    #[serde(serialize_with="ser_i32", deserialize_with="de_i32")]
    i: i32,
}

#[derive(Serialize, Deserialize)]
struct Bounds<T: Serialize + Deserialize> {
    t: T,
    option: Option<T>,
    boxed: Box<T>,
    option_boxed: Option<Box<T>>,
}

#[derive(Serialize, Deserialize)]
struct NoBounds<T> {
    t: T,
    option: Option<T>,
    boxed: Box<T>,
    option_boxed: Option<Box<T>>,
}

#[derive(Serialize, Deserialize)]
enum EnumWith<T> {
    A(
        #[serde(serialize_with="ser_i32", deserialize_with="de_i32")]
        i32),
    B {
        t: T,
        #[serde(serialize_with="ser_i32", deserialize_with="de_i32")]
        i: i32 },
}

//////////////////////////////////////////////////////////////////////////

fn ser_i32<S: Serializer>(_: &i32, _: &mut S) -> Result<(), S::Error> { panic!() }

fn de_i32<D: Deserializer>(_: &mut D) -> Result<i32, D::Error> { panic!() }
