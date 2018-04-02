// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::HashSet;

use syn::{self, visit};
use syn::punctuated::Punctuated;

use internals::ast::{Data, Container};
use internals::attr;

use proc_macro2::Span;

// Remove the default from every type parameter because in the generated impls
// they look like associated types: "error: associated type bindings are not
// allowed here".
pub fn without_defaults(generics: &syn::Generics) -> syn::Generics {
    syn::Generics {
        params: generics
            .params
            .iter()
            .map(|param| match *param {
                syn::GenericParam::Type(ref param) => {
                    syn::GenericParam::Type(syn::TypeParam {
                        eq_token: None,
                        default: None,
                        ..param.clone()
                    })
                }
                _ => param.clone(),
            })
            .collect(),
        ..generics.clone()
    }
}

pub fn with_where_predicates(
    generics: &syn::Generics,
    predicates: &[syn::WherePredicate],
) -> syn::Generics {
    let mut generics = generics.clone();
    generics.make_where_clause()
        .predicates
        .extend(predicates.into_iter().cloned());
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
    let predicates = cont.data
        .all_fields()
        .flat_map(|field| from_field(&field.attrs))
        .flat_map(|predicates| predicates.to_vec());

    let mut generics = generics.clone();
    generics.make_where_clause()
        .predicates
        .extend(predicates);
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
    F: Fn(&attr::Field, Option<&attr::Variant>) -> bool,
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
    impl<'ast> visit::Visit<'ast> for FindTyParams {
        fn visit_path(&mut self, path: &syn::Path) {
            if let Some(seg) = path.segments.last() {
                if seg.into_value().ident == "PhantomData" {
                    // Hardcoded exception, because PhantomData<T> implements
                    // Serialize and Deserialize whether or not T implements it.
                    return;
                }
            }
            if path.leading_colon.is_none() && path.segments.len() == 1 {
                let id = path.segments[0].ident;
                if self.all_ty_params.contains(&id) {
                    self.relevant_ty_params.insert(id);
                }
            }
            visit::visit_path(self, path);
        }

        // Type parameter should not be considered used by a macro path.
        //
        //     struct TypeMacro<T> {
        //         mac: T!(),
        //         marker: PhantomData<T>,
        //     }
        fn visit_macro(&mut self, _mac: &syn::Macro) {}
    }

    let all_ty_params: HashSet<_> = generics
        .params
        .iter()
        .filter_map(|param| match *param {
            syn::GenericParam::Type(ref param) => Some(param.ident),
            _ => None,
        })
        .collect();

    let mut visitor = FindTyParams {
        all_ty_params: all_ty_params,
        relevant_ty_params: HashSet::new(),
    };
    match cont.data {
        Data::Enum(ref variants) => for variant in variants.iter() {
            let relevant_fields = variant
                .fields
                .iter()
                .filter(|field| filter(&field.attrs, Some(&variant.attrs)));
            for field in relevant_fields {
                visit::visit_type(&mut visitor, field.ty);
            }
        },
        Data::Struct(_, ref fields) => {
            for field in fields.iter().filter(|field| filter(&field.attrs, None)) {
                visit::visit_type(&mut visitor, field.ty);
            }
        }
    }

    let new_predicates = generics
        .params
        .iter()
        .filter_map(|param| match *param {
            syn::GenericParam::Type(ref param) => Some(param.ident),
            _ => None,
        })
        .filter(|id| visitor.relevant_ty_params.contains(id))
        .map(|id| {
            syn::WherePredicate::Type(syn::PredicateType {
                lifetimes: None,
                // the type parameter that is being bounded e.g. T
                bounded_ty: syn::Type::Path(syn::TypePath {
                    qself: None,
                    path: id.into(),
                }),
                colon_token: Default::default(),
                // the bound e.g. Serialize
                bounds: vec![
                    syn::TypeParamBound::Trait(syn::TraitBound {
                        paren_token: None,
                        modifier: syn::TraitBoundModifier::None,
                        lifetimes: None,
                        path: bound.clone(),
                    }),
                ].into_iter().collect(),
            })
        });

    let mut generics = generics.clone();
    generics.make_where_clause()
        .predicates
        .extend(new_predicates);
    generics
}

pub fn with_self_bound(
    cont: &Container,
    generics: &syn::Generics,
    bound: &syn::Path,
) -> syn::Generics {
    let mut generics = generics.clone();
    generics.make_where_clause()
        .predicates
        .push(syn::WherePredicate::Type(syn::PredicateType {
            lifetimes: None,
            // the type that is being bounded e.g. MyStruct<'a, T>
            bounded_ty: type_of_item(cont),
            colon_token: Default::default(),
            // the bound e.g. Default
            bounds: vec![
                syn::TypeParamBound::Trait(syn::TraitBound {
                    paren_token: None,
                    modifier: syn::TraitBoundModifier::None,
                    lifetimes: None,
                    path: bound.clone(),
                }),
            ].into_iter().collect(),
        }));
    generics
}

pub fn with_lifetime_bound(generics: &syn::Generics, lifetime: &str) -> syn::Generics {
    let bound = syn::Lifetime::new(lifetime, Span::call_site());
    let def = syn::LifetimeDef {
        attrs: Vec::new(),
        lifetime: bound,
        colon_token: None,
        bounds: Punctuated::new(),
    };

    let params = Some(syn::GenericParam::Lifetime(def))
        .into_iter()
        .chain(generics.params
            .iter()
            .cloned()
            .map(|mut param| {
                match param {
                    syn::GenericParam::Lifetime(ref mut param) => {
                        param.bounds.push(bound);
                    }
                    syn::GenericParam::Type(ref mut param) => {
                        param.bounds.push(syn::TypeParamBound::Lifetime(bound));
                    }
                    syn::GenericParam::Const(_) => {}
                }
                param
            }))
        .collect();

    syn::Generics {
        params: params,
        ..generics.clone()
    }
}

fn type_of_item(cont: &Container) -> syn::Type {
    syn::Type::Path(syn::TypePath {
        qself: None,
        path: syn::Path {
            leading_colon: None,
            segments: vec![
                syn::PathSegment {
                    ident: cont.ident,
                    arguments: syn::PathArguments::AngleBracketed(
                        syn::AngleBracketedGenericArguments {
                            colon2_token: None,
                            lt_token: Default::default(),
                            args: cont.generics
                                .params
                                .iter()
                                .map(|param| match *param {
                                    syn::GenericParam::Type(ref param) => {
                                        syn::GenericArgument::Type(syn::Type::Path(syn::TypePath {
                                            qself: None,
                                            path: param.ident.into(),
                                        }))
                                    }
                                    syn::GenericParam::Lifetime(ref param) => {
                                        syn::GenericArgument::Lifetime(param.lifetime)
                                    }
                                    syn::GenericParam::Const(_) => {
                                        panic!("Serde does not support const generics yet");
                                    }
                                })
                                .collect(),
                            gt_token: Default::default(),
                        },
                    ),
                },
            ].into_iter().collect(),
        },
    })
}
