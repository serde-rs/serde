#![feature(custom_derive, plugin, rustc_private, unboxed_closures)]
#![plugin(quasi_macros)]

extern crate aster;
extern crate quasi;
extern crate rustc;
extern crate syntax;

mod attr;
mod de;
mod field;
mod ser;

pub fn register(reg: &mut rustc::plugin::Registry) {
    reg.register_syntax_extension(
        syntax::parse::token::intern("derive_Serialize"),
        syntax::ext::base::Decorator(
            Box::new(ser::expand_derive_serialize)));

    reg.register_syntax_extension(
        syntax::parse::token::intern("derive_Deserialize"),
        syntax::ext::base::Decorator(
            Box::new(de::expand_derive_deserialize)));
}
