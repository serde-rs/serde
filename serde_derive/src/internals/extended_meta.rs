use proc_macro2::{TokenStream, TokenTree};
use syn::parse::{Parse, Result};
use syn::punctuated::Punctuated;
use syn::token::{Comma, Eq, Paren};
use syn::{Lit, Path};

#[derive(Clone)]
pub enum Meta {
    Path(Path),
    List(MetaList),
    NameValue(MetaNameValue),
}

impl Meta {
    pub fn path(&self) -> &Path {
        match self {
            Meta::Path(path) => path,
            Meta::List(meta) => &meta.path,
            Meta::NameValue(meta) => &meta.path,
        }
    }
}

#[derive(Clone)]
pub struct MetaList {
    pub path: Path,
    pub paren_token: Paren,
    pub nested: MetaListInner,
}

pub type MetaListInner = Punctuated<NestedMeta, Comma>;

#[derive(Clone)]
pub enum NestedMeta {
    Meta(Meta),
    Lit(Lit),
}

#[derive(Clone)]
pub struct MetaNameValue {
    pub path: Path,
    pub eq_token: Eq,
    pub value: TokenStream,
}

impl MetaNameValue {
    pub fn parse_value<T: Parse>(&self) -> Result<T> {
        syn::parse2(self.value.clone())
    }
}

mod parsing {
    use super::*;
    use syn::ext::IdentExt;
    use syn::parse::{Parse, ParseStream, Result};
    use syn::punctuated::Punctuated;
    use syn::token::Paren;
    use syn::{Ident, Lit, LitBool, Path, PathSegment};

    // Like Path::parse_mod_style but accepts keywords in the path.
    fn parse_meta_path(input: ParseStream) -> Result<Path> {
        Ok(Path {
            leading_colon: input.parse()?,
            segments: {
                let mut segments = Punctuated::new();
                while input.peek(Ident::peek_any) {
                    let ident = Ident::parse_any(input)?;
                    segments.push_value(PathSegment::from(ident));
                    if !input.peek(Token![::]) {
                        break;
                    }
                    let punct = input.parse()?;
                    segments.push_punct(punct);
                }
                if segments.is_empty() {
                    return Err(input.error("expected path"));
                } else if segments.trailing_punct() {
                    return Err(input.error("expected path segment"));
                }
                segments
            },
        })
    }

    impl Parse for Meta {
        fn parse(input: ParseStream) -> Result<Self> {
            let path = input.call(parse_meta_path)?;
            parse_meta_after_path(path, input)
        }
    }

    impl Parse for MetaList {
        fn parse(input: ParseStream) -> Result<Self> {
            let path = input.call(parse_meta_path)?;
            parse_meta_list_after_path(path, input)
        }
    }

    impl Parse for MetaNameValue {
        fn parse(input: ParseStream) -> Result<Self> {
            let path = input.call(parse_meta_path)?;
            parse_meta_name_value_after_path(path, input)
        }
    }

    impl Parse for NestedMeta {
        fn parse(input: ParseStream) -> Result<Self> {
            if input.peek(Lit) && !(input.peek(LitBool) && input.peek2(Token![=])) {
                input.parse().map(NestedMeta::Lit)
            } else if input.peek(Ident::peek_any) {
                input.parse().map(NestedMeta::Meta)
            } else {
                Err(input.error("expected identifier or literal"))
            }
        }
    }

    pub fn parse_meta_after_path(path: Path, input: ParseStream) -> Result<Meta> {
        if input.peek(Paren) {
            parse_meta_list_after_path(path, input).map(Meta::List)
        } else if input.peek(Token![=]) {
            parse_meta_name_value_after_path(path, input).map(Meta::NameValue)
        } else {
            Ok(Meta::Path(path))
        }
    }

    fn parse_meta_list_after_path(path: Path, input: ParseStream) -> Result<MetaList> {
        let content;
        let paren_token = parenthesized!(content in input);
        Ok(MetaList {
            path,
            paren_token,
            nested: content.parse_terminated(NestedMeta::parse)?,
        })
    }

    fn parse_meta_name_value_after_path(path: Path, input: ParseStream) -> Result<MetaNameValue> {
        Ok(MetaNameValue {
            path,
            eq_token: input.parse()?,
            value: input.call(parse_token_stream_until_comma)?,
        })
    }

    fn parse_token_stream_until_comma(input: ParseStream) -> Result<TokenStream> {
        let mut stream = TokenStream::new();
        while !input.peek(Comma) && !input.is_empty() {
            stream.extend(Some(input.parse::<TokenTree>()?));
        }
        Ok(stream)
    }
}

mod printing {
    use super::*;
    use proc_macro2::TokenStream;
    use quote::ToTokens;

    impl ToTokens for Meta {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            match self {
                Meta::Path(path) => path.to_tokens(tokens),
                Meta::List(meta) => meta.to_tokens(tokens),
                Meta::NameValue(meta) => meta.to_tokens(tokens),
            }
        }
    }

    impl ToTokens for MetaList {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            self.path.to_tokens(tokens);
            self.paren_token.surround(tokens, |tokens| {
                self.nested.to_tokens(tokens);
            });
        }
    }

    impl ToTokens for NestedMeta {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            match self {
                NestedMeta::Meta(meta) => meta.to_tokens(tokens),
                NestedMeta::Lit(lit) => lit.to_tokens(tokens),
            }
        }
    }

    impl ToTokens for MetaNameValue {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            self.path.to_tokens(tokens);
            self.eq_token.to_tokens(tokens);
            self.value.to_tokens(tokens);
        }
    }
}
