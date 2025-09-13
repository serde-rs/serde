use serde::de::Deserialize;
use serde::ser::Serialize;

fn to_string<T>(_: &T) -> String
where
    T: Serialize,
{
    unimplemented!()
}

fn from_str<'de, T>(_: &'de str) -> T
where
    T: Deserialize<'de>,
{
    unimplemented!()
}

struct MyStruct;

fn main() {
    to_string(&MyStruct);
    let _: MyStruct = from_str("");
}
