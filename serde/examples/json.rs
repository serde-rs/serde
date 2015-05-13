#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate serde;

use std::collections::BTreeMap;
use serde::json;

// Creating serializable types with serde is quite simple with `serde_macros`. It implements a
// syntax extension that automatically generates the necessary serde trait implementations.
#[derive(Debug, Serialize, Deserialize)]
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let point = Point { x: 5, y: 6 };

    // Serializing to JSON is pretty simple by using the `to_string` method:
    let serialized_point = json::to_string(&point).unwrap();

    println!("{}", serialized_point);
    // prints:
    //
    // {"x":5,"y":6}

    // There is also support for pretty printing using `to_string_pretty`:
    let serialized_point = json::to_string_pretty(&point).unwrap();

    println!("{}", serialized_point);
    // prints:
    //
    // {
    //   "x":5,
    //   "y":6
    // }

    // Values can also be deserialized with the same style using `from_str`:
    let deserialized_point: Point = json::from_str(&serialized_point).unwrap();

    println!("{:?}", deserialized_point);
    // prints:
    //
    // Point { x: 5, y: 6 }

    // `Point`s aren't the only type that can be serialized to. Because `Point` members have the
    // same type, they can be also serialized into a map. Also, 
    let deserialized_map: BTreeMap<String, i64> = json::from_str(&serialized_point).unwrap();

    println!("{:?}", deserialized_map);
    // prints:
    //
    // {"x": 5, "y": 6}

    // If you need to accept arbitrary data, you can also deserialize into `json::Value`, which
    // can represent all JSON values.
    let deserialized_value: json::Value = json::from_str(&serialized_point).unwrap();

    println!("{:?}", deserialized_value);
    // prints:
    //
    // {"x":5,"y":6}
}
