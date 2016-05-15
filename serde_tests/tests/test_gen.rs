// These just test that serde_codegen is able to produce code that compiles
// successfully when there are a variety of generics and non-(de)serializable
// types involved.

extern crate serde;
use self::serde::ser::{Serialize, Serializer};
use self::serde::de::{Deserialize, Deserializer};

//////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize)]
struct With<T> {
    t: T,
    #[serde(serialize_with="ser_x", deserialize_with="de_x")]
    x: X,
}

#[derive(Serialize, Deserialize)]
struct WithRef<'a, T: 'a> {
    #[serde(skip_deserializing)]
    t: Option<&'a T>,
    #[serde(serialize_with="ser_x", deserialize_with="de_x")]
    x: X,
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
    Unit,
    Newtype(
        #[serde(serialize_with="ser_x", deserialize_with="de_x")]
        X),
    Tuple(
        T,
        #[serde(serialize_with="ser_x", deserialize_with="de_x")]
        X),
    Struct {
        t: T,
        #[serde(serialize_with="ser_x", deserialize_with="de_x")]
        x: X },
}

#[derive(Serialize)]
struct MultipleRef<'a, 'b, 'c, T> where T: 'c, 'c: 'b, 'b: 'a {
    t: T,
    rrrt: &'a &'b &'c T,
}

#[derive(Serialize, Deserialize)]
struct Newtype(
    #[serde(serialize_with="ser_x", deserialize_with="de_x")]
    X
);

#[derive(Serialize, Deserialize)]
struct Tuple<T>(
    T,
    #[serde(serialize_with="ser_x", deserialize_with="de_x")]
    X,
);

//////////////////////////////////////////////////////////////////////////

// Implements neither Serialize nor Deserialize
struct X;
fn ser_x<S: Serializer>(_: &X, _: &mut S) -> Result<(), S::Error> { panic!() }
fn de_x<D: Deserializer>(_: &mut D) -> Result<X, D::Error> { panic!() }

