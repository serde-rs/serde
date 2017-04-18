// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::HashSet;

use syn::{self, visit};

use internals::ast::Container;
use internals::attr;

macro_rules! path {
    ($($path:tt)+) => {
        syn::parse_path(stringify!($($path)+)).unwrap()
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
                        ..ty_param.clone()
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
    generics
        .where_clause
        .predicates
        .extend_from_slice(predicates);
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
    generics.where_clause.predicates.extend(predicates);
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
            if let Some(seg) = path.segments.last() {
                if seg.ident == "PhantomData" {
                    // Hardcoded exception, because PhantomData<T> implements
                    // Serialize and Deserialize whether or not T implements it.
                    return;
                }
            }
            if !path.global && path.segments.len() == 1 {
                let id = path.segments[0].ident.clone();
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
        .map(|ty_param| ty_param.ident.clone())
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
        .map(|ty_param| ty_param.ident.clone())
        .filter(|id| visitor.relevant_ty_params.contains(id))
        .map(
            |id| {
                syn::WherePredicate::BoundPredicate(
                    syn::WhereBoundPredicate {
                        bound_lifetimes: Vec::new(),
                        // the type parameter that is being bounded e.g. T
                        bounded_ty: syn::Ty::Path(None, id.into()),
                        // the bound e.g. Serialize
                        bounds: vec![
                            syn::TyParamBound::Trait(
                                syn::PolyTraitRef {
                                    bound_lifetimes: Vec::new(),
                                    trait_ref: bound.clone(),
                                },
                                syn::TraitBoundModifier::None,
                            ),
                        ],
                    },
                )
            },
        );

    let mut generics = generics.clone();
    generics.where_clause.predicates.extend(new_predicates);
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
        .push(
            syn::WherePredicate::BoundPredicate(
                syn::WhereBoundPredicate {
                    bound_lifetimes: Vec::new(),
                    // the type that is being bounded e.g. MyStruct<'a, T>
                    bounded_ty: type_of_item(cont),
                    // the bound e.g. Default
                    bounds: vec![
                        syn::TyParamBound::Trait(
                            syn::PolyTraitRef {
                                bound_lifetimes: Vec::new(),
                                trait_ref: bound.clone(),
                            },
                            syn::TraitBoundModifier::None,
                        ),
                    ],
                },
            ),
        );
    generics
}

pub fn with_lifetime_bound(generics: &syn::Generics, lifetime: &str) -> syn::Generics {
    let mut generics = generics.clone();

    for lifetime_def in &mut generics.lifetimes {
        lifetime_def.bounds.push(syn::Lifetime::new(lifetime));
    }

    for ty_param in &mut generics.ty_params {
        ty_param
            .bounds
            .push(syn::TyParamBound::Region(syn::Lifetime::new(lifetime)));
    }

    generics
        .lifetimes
        .push(
            syn::LifetimeDef {
                attrs: Vec::new(),
                lifetime: syn::Lifetime::new(lifetime),
                bounds: Vec::new(),
            },
        );

    generics
}

fn type_of_item(cont: &Container) -> syn::Ty {
    syn::Ty::Path(
        None,
        syn::Path {
            global: false,
            segments: vec![
                syn::PathSegment {
                    ident: cont.ident.clone(),
                    parameters: syn::PathParameters::AngleBracketed(
                        syn::AngleBracketedParameterData {
                            lifetimes: cont.generics
                                .lifetimes
                                .iter()
                                .map(|def| def.lifetime.clone())
                                .collect(),
                            types: cont.generics
                                .ty_params
                                .iter()
                                .map(|param| syn::Ty::Path(None, param.ident.clone().into()))
                                .collect(),
                            bindings: Vec::new(),
                        },
                    ),
                },
            ],
        },
    )
}
