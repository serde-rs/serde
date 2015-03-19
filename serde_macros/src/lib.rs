#![feature(custom_derive, plugin, plugin_registrar, rustc_private, unboxed_closures)]
#![plugin(quasi_macros)]

extern crate aster;
extern crate quasi;
extern crate rustc;
extern crate syntax;

use syntax::ext::base::Decorator;
use syntax::parse::token;
use rustc::plugin::Registry;

mod ser;
mod de;
mod field;

#[plugin_registrar]
#[doc(hidden)]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(
        token::intern("derive_Serialize"),
        Decorator(Box::new(ser::expand_derive_serialize)));

    reg.register_syntax_extension(
        token::intern("derive_Deserialize"),
        Decorator(Box::new(de::expand_derive_deserialize)));
}
