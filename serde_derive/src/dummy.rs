use proc_macro2::{Ident, Span, TokenStream};

use try;

pub fn wrap_in_const(trait_: &str, ty: &Ident, code: TokenStream) -> TokenStream {
    let try_replacement = try::replacement();

    let dummy_const = Ident::new(
        &format!("_IMPL_{}_FOR_{}", trait_, unraw(ty)),
        Span::call_site(),
    );

    quote! {
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const #dummy_const: () = {
            #[allow(unknown_lints)]
            #[cfg_attr(feature = "cargo-clippy", allow(useless_attribute))]
            #[allow(rust_2018_idioms)]
            extern crate serde as _serde;
            #try_replacement
            #code
        };
    }
}

#[allow(deprecated)]
fn unraw(ident: &Ident) -> String {
    // str::trim_start_matches was added in 1.30, trim_left_matches deprecated
    // in 1.33. We currently support rustc back to 1.15 so we need to continue
    // to use the deprecated one.
    ident.to_string().trim_left_matches("r#").to_owned()
}
