use super::respan::respan;
use proc_macro2::{Group, Spacing, Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned};
use std::{iter::FromIterator, mem};
use syn::{
    parse_quote,
    punctuated::Punctuated,
    visit_mut::{self, VisitMut},
    DeriveInput, ExprPath, Macro, Path, PathArguments, QSelf, Type, TypePath,
};

pub fn replace_receiver(input: &mut DeriveInput) {
    let ident = &input.ident;
    let ty_generics = input.generics.split_for_impl().1;
    let self_ty = parse_quote!(#ident #ty_generics);
    let mut visitor = ReplaceReceiver(&self_ty);
    visitor.visit_generics_mut(&mut input.generics);
    visitor.visit_data_mut(&mut input.data);
}

struct ReplaceReceiver<'a>(&'a TypePath);

impl ReplaceReceiver<'_> {
    fn self_ty(&self, span: Span) -> TypePath {
        respan(self.0, span)
    }

    fn self_to_qself(&self, qself: &mut Option<QSelf>, path: &mut Path) {
        if path.leading_colon.is_some() {
            return;
        }

        let first = &path.segments[0];
        if first.ident != "Self" || !first.arguments.is_empty() {
            return;
        }

        if path.segments.len() == 1 {
            self.self_to_expr_path(path);
            return;
        }

        let span = first.ident.span();
        *qself = Some(QSelf {
            lt_token: Token![<](span),
            ty: Box::new(self.self_ty(span).into()),
            position: 0,
            as_token: None,
            gt_token: Token![>](span),
        });

        path.leading_colon = Some(**path.segments.pairs().next().unwrap().punct().unwrap());

        let segments = mem::replace(&mut path.segments, Punctuated::new());
        path.segments = segments.into_pairs().skip(1).collect();
    }

    fn self_to_expr_path(&self, path: &mut Path) {
        if path.leading_colon.is_some() {
            return;
        }

        let first = &path.segments[0];
        if first.ident != "Self" || !first.arguments.is_empty() {
            return;
        }

        let self_ty = self.self_ty(first.ident.span());
        let variant = mem::replace(path, self_ty.path);
        for segment in &mut path.segments {
            if let PathArguments::AngleBracketed(bracketed) = &mut segment.arguments {
                if bracketed.colon2_token.is_none() && !bracketed.args.is_empty() {
                    bracketed.colon2_token = Some(<Token![::]>::default());
                }
            }
        }
        if variant.segments.len() > 1 {
            path.segments.push_punct(<Token![::]>::default());
            path.segments.extend(variant.segments.into_pairs().skip(1));
        }
    }

    fn visit_token_stream(&self, tokens: &mut TokenStream) -> bool {
        let mut out = Vec::new();
        let mut modified = false;
        let mut iter = tokens.clone().into_iter().peekable();
        while let Some(tt) = iter.next() {
            match tt {
                TokenTree::Ident(ident) => {
                    if ident == "Self" {
                        modified = true;
                        let self_ty = self.self_ty(ident.span());
                        match iter.peek() {
                            Some(TokenTree::Punct(p))
                                if p.as_char() == ':' && p.spacing() == Spacing::Joint =>
                            {
                                let next = iter.next().unwrap();
                                match iter.peek() {
                                    Some(TokenTree::Punct(p)) if p.as_char() == ':' => {
                                        let span = ident.span();
                                        out.extend(quote_spanned!(span=> <#self_ty>));
                                    }
                                    _ => out.extend(quote!(#self_ty)),
                                }
                                out.push(next);
                            }
                            _ => out.extend(quote!(#self_ty)),
                        }
                    } else {
                        out.push(TokenTree::Ident(ident));
                    }
                }
                TokenTree::Group(group) => {
                    let mut content = group.stream();
                    modified |= self.visit_token_stream(&mut content);
                    let mut new = Group::new(group.delimiter(), content);
                    new.set_span(group.span());
                    out.push(TokenTree::Group(new));
                }
                other => out.push(other),
            }
        }
        if modified {
            *tokens = TokenStream::from_iter(out);
        }
        modified
    }
}

impl VisitMut for ReplaceReceiver<'_> {
    // `Self` -> `Receiver`
    fn visit_type_mut(&mut self, ty: &mut Type) {
        if let Type::Path(node) = ty {
            if node.qself.is_none() && node.path.is_ident("Self") {
                *ty = self.self_ty(node.path.segments[0].ident.span()).into();
            } else {
                self.visit_type_path_mut(node);
            }
        } else {
            visit_mut::visit_type_mut(self, ty);
        }
    }

    // `Self::Assoc` -> `<Receiver>::Assoc`
    fn visit_type_path_mut(&mut self, ty: &mut TypePath) {
        if ty.qself.is_none() {
            self.self_to_qself(&mut ty.qself, &mut ty.path);
        }
        visit_mut::visit_type_path_mut(self, ty);
    }

    // `Self::method` -> `<Receiver>::method`
    fn visit_expr_path_mut(&mut self, expr: &mut ExprPath) {
        if expr.qself.is_none() {
            self.self_to_qself(&mut expr.qself, &mut expr.path);
        }
        visit_mut::visit_expr_path_mut(self, expr);
    }

    fn visit_macro_mut(&mut self, mac: &mut Macro) {
        // We can't tell in general whether `self` inside a macro invocation
        // refers to the self in the argument list or a different self
        // introduced within the macro. Heuristic: if the macro input contains
        // `fn`, then `self` is more likely to refer to something other than the
        // outer function's self argument.
        if !contains_fn(mac.tokens.clone()) {
            self.visit_token_stream(&mut mac.tokens);
        }
    }
}

fn contains_fn(tokens: TokenStream) -> bool {
    tokens.into_iter().any(|tt| match tt {
        TokenTree::Ident(ident) => ident == "fn",
        TokenTree::Group(group) => contains_fn(group.stream()),
        _ => false,
    })
}
