#![feature(plugin_registrar, rustc_private)]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

extern crate serde_codegen;
extern crate rustc_plugin;

#[plugin_registrar]
#[doc(hidden)]
pub fn plugin_registrar(reg: &mut rustc_plugin::Registry) {
    serde_codegen::register(reg);
}
