use proc_macro2::{Ident, TokenStream};
use quote::format_ident;

use syn;
use try;

pub fn wrap_in_const(
    serde_path: Option<&syn::Path>,
    trait_: &str,
    ty: &Ident,
    code: TokenStream,
) -> TokenStream {
    let try_replacement = try::replacement();

    let dummy_const = if cfg!(no_underscore_consts) {
        format_ident!("_IMPL_{}_FOR_{}", trait_, unraw(ty))
    } else {
        format_ident!("_")
    };

    let use_serde = match serde_path {
        Some(path) => quote! {
            use #path as _serde;
        },
        None => quote! {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
        },
    };

    quote! {
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const #dummy_const: () = {
            #use_serde
            #try_replacement
            #code
        };
    }
}

fn unraw(ident: &Ident) -> String {
    ident.to_string().trim_start_matches("r#").to_owned()
}
