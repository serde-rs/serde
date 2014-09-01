extern crate serde2;

use std::io;
use std::collections::TreeMap;
use serde2::{Serialize, FormatState, GatherTokens};

///////////////////////////////////////////////////////////////////////////////

struct Foo {
    x: int,
    y: int,
    z: &'static str,
}

impl<S: serde2::VisitorState<R>, R> serde2::Serialize<S, R> for Foo {
    fn serialize(&self, state: &mut S) -> R {
        state.visit_struct("Foo", FooSerialize {
            value: self,
            state: 0,
        })
    }
}

struct FooSerialize<'a> {
    value: &'a Foo,
    state: uint,
}

impl<'a, S: serde2::VisitorState<R>, R> serde2::Visitor<S, R> for FooSerialize<'a> {
    fn visit(&mut self, state: &mut S) -> Option<R> {
        match self.state {
            0 => {
                self.state += 1;
                Some(state.visit_map_elt(true, "x", &self.value.x))
            }
            1 => {
                self.state += 1;
                Some(state.visit_map_elt(false, "y", &self.value.y))
            }
            2 => {
                self.state += 1;
                Some(state.visit_map_elt(false, "z", &self.value.z))
            }
            _ => {
                None
            }
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        let size = 3 - self.state;
        (size, Some(size))
    }
}

///////////////////////////////////////////////////////////////////////////////

fn main() {
    let value = 5i;

    let mut s = GatherTokens::new();
    value.serialize(&mut s);
    println!("tokens: {}", s.unwrap());

    value.serialize(&mut FormatState::new(io::stdout())).unwrap();
    println!("");

    ////

    let value = vec!(1i, 2, 3);

    let mut s = GatherTokens::new();
    value.serialize(&mut s);
    println!("tokens: {}", s.unwrap());

    value.serialize(&mut FormatState::new(io::stdout())).unwrap();
    println!("");

    ////

    let mut value = TreeMap::new();
    value.insert("a", 1i);
    value.insert("b", 2);
    value.insert("c", 3);

    let mut s = GatherTokens::new();
    value.serialize(&mut s);
    println!("tokens: {}", s.unwrap());

    value.serialize(&mut FormatState::new(io::stdout())).unwrap();
    println!("");

    ////

    /*
    println!("{}", to_format_vec(&5i));
    println!("{}", to_format_string(&5i));
    */

    let value = Foo { x: 1, y: 2, z: "abc" };

    let mut s = GatherTokens::new();
    value.serialize(&mut s);
    println!("tokens: {}", s.unwrap());

    value.serialize(&mut FormatState::new(io::stdout())).unwrap();
    println!("");

    ////

    let value = (1i, "abc");

    let mut s = GatherTokens::new();
    value.serialize(&mut s);
    println!("tokens: {}", s.unwrap());

    value.serialize(&mut FormatState::new(io::stdout())).unwrap();
    println!("");
}
