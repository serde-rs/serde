#![feature(custom_attribute, custom_derive, plugin)]
#![plugin(serde_macros)]

#[derive(Serialize)]
struct S {
    #[serde(rename(serialize="x"))]
    #[serde(rename(serialize="y"))] //~ ERROR: duplicate serde attribute `rename`
    a: (),                          //~^ ERROR: duplicate serde attribute `rename`
                                    //~^^ ERROR: duplicate serde attribute `rename`

    #[serde(rename(serialize="x"))]
    #[serde(rename="y")] //~ ERROR: duplicate serde attribute `rename`
    b: (),               //~^ ERROR: duplicate serde attribute `rename`
                         //~^^ ERROR: duplicate serde attribute `rename`

    #[serde(rename(serialize="x"))]
    #[serde(rename(deserialize="y"))] // ok
    c: (),

    #[serde(rename="x")]
    #[serde(rename(deserialize="y"))] //~ ERROR: duplicate serde attribute `rename`
    d: (),                            //~^ ERROR: duplicate serde attribute `rename`
}                                     //~^^ ERROR: duplicate serde attribute `rename`

fn main() {}
