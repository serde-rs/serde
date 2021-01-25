use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::parse::Parse;

pub(crate) fn respan<T>(node: &T, span: Span) -> T
where
    T: ToTokens + Parse,
{
    let tokens = node.to_token_stream();
    let respanned = respan_tokens(tokens, span);
    syn::parse2(respanned).unwrap()
}

fn respan_tokens(tokens: TokenStream, span: Span) -> TokenStream {
    tokens
        .into_iter()
        .map(|mut token| {
            token.set_span(span);
            token
        })
        .collect()
}
