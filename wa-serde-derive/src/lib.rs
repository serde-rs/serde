extern crate proc_macro;
extern crate watt;

use proc_macro::TokenStream;

static WASM: &[u8] = include_bytes!("serde_derive.wasm");

#[proc_macro_derive(Serialize, attributes(serde))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    watt::proc_macro_derive("derive_serialize", input, WASM)
}

#[proc_macro_derive(Deserialize, attributes(serde))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    watt::proc_macro_derive("derive_deserialize", input, WASM)
}
