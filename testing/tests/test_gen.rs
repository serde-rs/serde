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
#[allow(dead_code)]
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

#[derive(Serialize, Deserialize)]
enum TreeNode<D> {
    Split {
        left: Box<TreeNode<D>>,
        right: Box<TreeNode<D>>,
    },
    Leaf {
        data: D,
    },
}

#[derive(Serialize, Deserialize)]
struct ListNode<D> {
    data: D,
    next: Box<ListNode<D>>,
}

#[derive(Serialize, Deserialize)]
#[serde(bound="D: SerializeWith + DeserializeWith")]
struct WithTraits1<D, E> {
    #[serde(serialize_with="SerializeWith::serialize_with",
            deserialize_with="DeserializeWith::deserialize_with")]
    d: D,
    #[serde(serialize_with="SerializeWith::serialize_with",
            deserialize_with="DeserializeWith::deserialize_with",
            bound="E: SerializeWith + DeserializeWith")]
    e: E,
}

#[derive(Serialize, Deserialize)]
#[serde(bound(serialize="D: SerializeWith",
              deserialize="D: DeserializeWith"))]
struct WithTraits2<D, E> {
    #[serde(serialize_with="SerializeWith::serialize_with",
            deserialize_with="DeserializeWith::deserialize_with")]
    d: D,
    #[serde(serialize_with="SerializeWith::serialize_with",
            bound(serialize="E: SerializeWith"))]
    #[serde(deserialize_with="DeserializeWith::deserialize_with",
            bound(deserialize="E: DeserializeWith"))]
    e: E,
}

//////////////////////////////////////////////////////////////////////////

trait SerializeWith {
    fn serialize_with<S: Serializer>(_: &Self, _: &mut S) -> Result<(), S::Error>;
}

trait DeserializeWith: Sized {
    fn deserialize_with<D: Deserializer>(_: &mut D) -> Result<Self, D::Error>;
}

// Implements neither Serialize nor Deserialize
struct X;
fn ser_x<S: Serializer>(_: &X, _: &mut S) -> Result<(), S::Error> { panic!() }
fn de_x<D: Deserializer>(_: &mut D) -> Result<X, D::Error> { panic!() }

