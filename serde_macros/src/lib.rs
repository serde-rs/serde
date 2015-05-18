#![feature(plugin_registrar, rustc_private)]

extern crate serde_codegen;
extern crate rustc;

#[plugin_registrar]
#[doc(hidden)]
pub fn plugin_registrar(reg: &mut rustc::plugin::Registry) {
    serde_codegen::register(reg);
}
