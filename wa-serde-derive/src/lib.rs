extern crate proc_macro;
extern crate watt;

use proc_macro::TokenStream;
use watt::WasmMacro;

static MACRO: WasmMacro = WasmMacro::new(WASM);
static WASM: &[u8] = include_bytes!("serde_derive.wasm");

#[proc_macro_derive(Serialize, attributes(serde))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    MACRO.proc_macro_derive("derive_serialize", input)
}

#[proc_macro_derive(Deserialize, attributes(serde))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    MACRO.proc_macro_derive("derive_deserialize", input)
}
