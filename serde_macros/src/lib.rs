#![feature(plugin_registrar, rustc_private)]

extern crate serde_codegen;
extern crate rustc_plugin;

#[plugin_registrar]
#[doc(hidden)]
pub fn plugin_registrar(reg: &mut rustc_plugin::Registry) {
    serde_codegen::register(reg);
}
