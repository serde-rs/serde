use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, DeriveInput};

fn inject_serde_crate_attr(attrs: &mut Vec<Attribute>) {
    let serde_attr: Attribute = syn::parse_quote!(#[serde(crate = "my_library::_serde")]);
    attrs.push(serde_attr);
}

fn inject_derive_attr(attrs: &mut Vec<Attribute>, path: proc_macro2::TokenStream) {
    let derive_attr: Attribute = syn::parse_quote!(#[derive(#path)]);
    attrs.push(derive_attr);
}

#[proc_macro_attribute] 
pub fn my_serialize(_args: TokenStream, input: TokenStream) -> TokenStream { 
    let mut ast = parse_macro_input!(input as DeriveInput);

    inject_serde_crate_attr(&mut ast.attrs);
    inject_derive_attr(&mut ast.attrs, quote!(my_library::_serde::Serialize));

    // Because this is an attribute macro, returning the AST replaces the original struct perfectly.
    TokenStream::from(quote!(#ast))
}

#[proc_macro_attribute]
pub fn my_deserialize(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);

    inject_serde_crate_attr(&mut ast.attrs);
    inject_derive_attr(&mut ast.attrs, quote!(my_library::_serde::Deserialize));

    TokenStream::from(quote!(#ast))
}