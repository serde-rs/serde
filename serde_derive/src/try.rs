use proc_macro2::TokenStream;
use quote::quote;

// None of our generated code requires the `From::from` error conversion
// performed by the standard library's `try!` macro. With this simplified macro
// we see a significant improvement in type checking and borrow checking time of
// the generated code and a slight improvement in binary size.
pub fn replacement() -> TokenStream {
    quote! {
        #[allow(unused_macros)]
        macro_rules! try {
            ($__expr:expr) => {
                match $__expr {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                }
            }
        }
    }
}
