// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::HashSet;

use proc_macro2::Term;
use syn::{self, visit};
use syn::Span;
use syn::tokens;
use syn::delimited::Delimited;

use internals::ast::Container;
use internals::attr;

macro_rules! path {
    ($($path:tt)+) => {
        syn::parse_str::<syn::Path>(stringify!($($path)+)).unwrap()
    };
}

// Remove the default from every type parameter because in the generated impls
// they look like associated types: "error: associated type bindings are not
// allowed here".
pub fn without_defaults(generics: &syn::Generics) -> syn::Generics {
    syn::Generics {
        ty_params: generics
            .ty_params
            .iter()
            .map(
                |ty_param| {
                    syn::TyParam {
                        default: None,
                        ..ty_param.into_item().clone()
                    }
                },
            )
            .collect(),
        ..generics.clone()
    }
}

pub fn with_where_predicates(
    generics: &syn::Generics,
    predicates: &[syn::WherePredicate],
) -> syn::Generics {
    let mut generics = generics.clone();
    for predicate in predicates {
        if generics.where_clause.where_token.is_none() {
            generics.where_clause.where_token = Some(tokens::Where::default());
        }
        generics.where_clause.predicates.push_default(predicate.clone());
    }
    generics
}

pub fn with_where_predicates_from_fields<F>(
    cont: &Container,
    generics: &syn::Generics,
    from_field: F,
) -> syn::Generics
where
    F: Fn(&attr::Field) -> Option<&[syn::WherePredicate]>,
{
    let predicates = cont.body
        .all_fields()
        .flat_map(|field| from_field(&field.attrs))
        .flat_map(|predicates| predicates.to_vec());

    let mut generics = generics.clone();
    for predicate in predicates {
        if generics.where_clause.where_token.is_none() {
            generics.where_clause.where_token = Some(tokens::Where::default());
        }
        generics.where_clause.predicates.push_default(predicate);
    }
    generics
}

// Puts the given bound on any generic type parameters that are used in fields
// for which filter returns true.
//
// For example, the following struct needs the bound `A: Serialize, B: Serialize`.
//
//     struct S<'b, A, B: 'b, C> {
//         a: A,
//         b: Option<&'b B>
//         #[serde(skip_serializing)]
//         c: C,
//     }
pub fn with_bound<F>(
    cont: &Container,
    generics: &syn::Generics,
    filter: F,
    bound: &syn::Path,
) -> syn::Generics
where
    F: Fn(&attr::Field) -> bool,
{
    struct FindTyParams {
        // Set of all generic type parameters on the current struct (A, B, C in
        // the example). Initialized up front.
        all_ty_params: HashSet<syn::Ident>,
        // Set of generic type parameters used in fields for which filter
        // returns true (A and B in the example). Filled in as the visitor sees
        // them.
        relevant_ty_params: HashSet<syn::Ident>,
    }
    impl visit::Visitor for FindTyParams {
        fn visit_path(&mut self, path: &syn::Path) {
            if path.segments.len() > 0 {
                let seg = *path.segments.last().unwrap().item();
                if seg.ident == "PhantomData" {
                    // Hardcoded exception, because PhantomData<T> implements
                    // Serialize and Deserialize whether or not T implements it.
                    return;
                }
            }
            if !path.global() && path.segments.len() == 1 {
                let id = path.segments.first().unwrap().item().ident.clone();
                if self.all_ty_params.contains(&id) {
                    self.relevant_ty_params.insert(id);
                }
            }
            visit::walk_path(self, path);
        }
    }

    let all_ty_params: HashSet<_> = generics
        .ty_params
        .iter()
        .map(|ty_param| ty_param.item().ident.clone())
        .collect();

    let relevant_tys = cont.body
        .all_fields()
        .filter(|&field| filter(&field.attrs))
        .map(|field| &field.ty);

    let mut visitor = FindTyParams {
        all_ty_params: all_ty_params,
        relevant_ty_params: HashSet::new(),
    };
    for ty in relevant_tys {
        visit::walk_ty(&mut visitor, ty);
    }

    let new_predicates = generics
        .ty_params
        .iter()
        .map(|ty_param| ty_param.item().ident.clone())
        .filter(|id| visitor.relevant_ty_params.contains(id))
        .map(
            |id| {
                syn::WherePredicate::BoundPredicate(
                    syn::WhereBoundPredicate {
                        colon_token: tokens::Colon::default(),
                        bound_lifetimes: None,
                        // the type parameter that is being bounded e.g. T
                        bounded_ty: syn::Ty::Path(syn::TyPath {
                            qself: None,
                            path: id.into(),
                        }),
                        // the bound e.g. Serialize
                        bounds: vec![
                            syn::TyParamBound::Trait(
                                syn::PolyTraitRef {
                                    bound_lifetimes: None,
                                    trait_ref: bound.clone(),
                                },
                                syn::TraitBoundModifier::None,
                            ),
                        ].into(),
                    },
                )
            },
        );

    let mut generics = generics.clone();
    for predicate in new_predicates {
        if generics.where_clause.where_token.is_none() {
            generics.where_clause.where_token = Some(tokens::Where::default());
        }
        generics.where_clause.predicates.push_default(predicate);
    }
    generics
}

pub fn with_self_bound(
    cont: &Container,
    generics: &syn::Generics,
    bound: &syn::Path,
) -> syn::Generics {
    let mut generics = generics.clone();
    generics
        .where_clause
        .predicates
        .push_default(
            syn::WherePredicate::BoundPredicate(
                syn::WhereBoundPredicate {
                    colon_token: tokens::Colon::default(),
                    bound_lifetimes: None,
                    // the type that is being bounded e.g. MyStruct<'a, T>
                    bounded_ty: type_of_item(cont),
                    // the bound e.g. Default
                    bounds: vec![
                        syn::TyParamBound::Trait(
                            syn::PolyTraitRef {
                                bound_lifetimes: None,
                                trait_ref: bound.clone(),
                            },
                            syn::TraitBoundModifier::None,
                        ),
                    ].into(),
                },
            ),
        );
    generics
}

pub fn with_lifetime_bound(generics: &syn::Generics, lifetime: &str) -> syn::Generics {
    let mut generics = generics.clone();

    let lifetime = syn::Lifetime::new(Term::intern(lifetime), Span::default());

    for lifetime_def in generics.lifetimes.iter_mut().map(|m| m.into_item()) {
        if lifetime_def.colon_token.is_none() {
            lifetime_def.colon_token = Some(tokens::Colon::default());
        }
        lifetime_def
            .bounds
            .push_default(lifetime.clone());
    }

    for ty_param in generics.ty_params.iter_mut().map(|i| i.into_item()) {
        if ty_param.colon_token.is_none() {
            ty_param.colon_token = Some(tokens::Colon::default());
        }
        ty_param
            .bounds
            .push_default(syn::TyParamBound::Region(lifetime.clone()));
    }

    if generics.lt_token.is_none() {
        generics.lt_token = Some(tokens::Lt::default());
        generics.gt_token = Some(tokens::Gt::default());
    }

    generics
        .lifetimes
        .push_default(
            syn::LifetimeDef {
                attrs: Vec::new(),
                lifetime: lifetime,
                bounds: Delimited::default(),
                colon_token: None,
            },
        );

    if generics.ty_params.len() > 0 && !generics.lifetimes.trailing_delim() {
        generics.lifetimes.push_trailing(tokens::Comma::default());
    }

    generics
}

fn type_of_item(cont: &Container) -> syn::Ty {
    syn::Ty::Path(syn::TyPath {
        qself: None,
        path: syn::Path {
            leading_colon: None,
            segments: vec![
                syn::PathSegment {
                    ident: cont.ident.clone(),
                    parameters: syn::PathParameters::AngleBracketed(
                        syn::AngleBracketedParameterData {
                            gt_token: tokens::Gt::default(),
                            lt_token: tokens::Lt::default(),
                            turbofish: None,
                            lifetimes: cont.generics
                                .lifetimes
                                .iter()
                                .map(|def| def.item().lifetime.clone())
                                .collect(),
                            types: cont.generics
                                .ty_params
                                .iter()
                                .map(|param| {
                                    syn::Ty::Path(syn::TyPath {
                                        qself: None,
                                        path: param.item().ident.clone().into(),
                                    })
                                })
                                .collect(),
                            bindings: Delimited::default(),
                        },
                    ),
                },
            ].into(),
        },
    })
}
