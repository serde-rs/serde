use internals::respan::respan;
use proc_macro2::Span;
use quote::ToTokens;
use std::mem;
use syn::punctuated::Punctuated;
use syn::visit_mut::{self, VisitMut};
use syn::{parse_quote, DeriveInput, ExprPath, Path, PathArguments, QSelf, Type, TypePath};

pub fn replace_receiver(input: &mut DeriveInput) {
    let self_ty = {
        let ident = &input.ident;
        let ty_generics = input.generics.split_for_impl().1;
        parse_quote!(#ident #ty_generics)
    };
    let mut visitor = ReplaceReceiver(&self_ty);
    visitor.visit_generics_mut(&mut input.generics);
    visitor.visit_data_mut(&mut input.data);
}

struct ReplaceReceiver<'a>(&'a TypePath);

impl ReplaceReceiver<'_> {
    fn self_ty(&self, span: Span) -> TypePath {
        let tokens = self.0.to_token_stream();
        let respanned = respan(tokens, span);
        syn::parse2(respanned).unwrap()
    }

    fn self_to_qself(&self, qself: &mut Option<QSelf>, path: &mut Path) {
        if path.leading_colon.is_some() || path.segments[0].ident != "Self" {
            return;
        }

        if path.segments.len() == 1 {
            self.self_to_expr_path(path);
            return;
        }

        let span = path.segments[0].ident.span();
        *qself = Some(QSelf {
            lt_token: Token![<](span),
            ty: Box::new(Type::Path(self.self_ty(span))),
            position: 0,
            as_token: None,
            gt_token: Token![>](span),
        });

        path.leading_colon = Some(**path.segments.pairs().next().unwrap().punct().unwrap());

        let segments = mem::replace(&mut path.segments, Punctuated::new());
        path.segments = segments.into_pairs().skip(1).collect();
    }

    fn self_to_expr_path(&self, path: &mut Path) {
        let self_ty = self.self_ty(path.segments[0].ident.span());
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
}

impl VisitMut for ReplaceReceiver<'_> {
    // `Self` -> `Receiver`
    fn visit_type_mut(&mut self, ty: &mut Type) {
        let span = if let Type::Path(node) = ty {
            if node.qself.is_none() && node.path.is_ident("Self") {
                node.path.segments[0].ident.span()
            } else {
                self.visit_type_path_mut(node);
                return;
            }
        } else {
            visit_mut::visit_type_mut(self, ty);
            return;
        };
        *ty = self.self_ty(span).into();
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
}
