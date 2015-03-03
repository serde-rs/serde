#![feature(plugin)]
#![plugin(serde2_macros)]

extern crate serde2;

/*
trait Trait {
    type Type;
}
*/

#[derive_serialize]
//#[derive_deserialize]
enum Enum<'a, A: 'a, B: /* Trait + */ 'a, C> where C: /* Trait + */ 'a {
    Unit,
    Seq(
        i8,
        &'a A,
        &'a B,
        //B::Type,
        &'a C,
        //<C as Trait>::Type,
    ),
    Map {
        a: i8,
        b: &'a A,
        c: &'a B,
        //d: B::Type,
        e: &'a C,
        //f: <C as Trait>::Type,
    },
}

#[test]
fn test() {
}
