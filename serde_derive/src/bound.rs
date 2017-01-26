use std::collections::HashSet;

use syn::{self, aster, visit};

use internals::ast::Item;
use internals::attr;

// Remove the default from every type parameter because in the generated impls
// they look like associated types: "error: associated type bindings are not
// allowed here".
pub fn without_defaults(generics: &syn::Generics) -> syn::Generics {
    syn::Generics {
        ty_params: generics.ty_params.iter().map(|ty_param| {
            syn::TyParam {
                default: None,
                .. ty_param.clone()
            }}).collect(),
        .. generics.clone()
    }
}

pub fn with_where_predicates(
    generics: &syn::Generics,
    predicates: &[syn::WherePredicate],
) -> syn::Generics {
    aster::from_generics(generics.clone())
        .with_predicates(predicates.to_vec())
        .build()
}

pub fn with_where_predicates_from_fields<F>(
    item: &Item,
    generics: &syn::Generics,
    from_field: F,
) -> syn::Generics
    where F: Fn(&attr::Field) -> Option<&[syn::WherePredicate]>,
{
    aster::from_generics(generics.clone())
        .with_predicates(
            item.body.all_fields()
                .flat_map(|field| from_field(&field.attrs))
                .flat_map(|predicates| predicates.to_vec()))
        .build()
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
    item: &Item,
    generics: &syn::Generics,
    filter: F,
    bound: &syn::Path,
) -> syn::Generics
    where F: Fn(&attr::Field) -> bool,
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

    let all_ty_params: HashSet<_> = generics.ty_params.iter()
        .map(|ty_param| ty_param.ident.clone())
        .collect();

    let relevant_tys = item.body.all_fields()
        .filter(|&field| filter(&field.attrs))
        .map(|field| &field.ty);

    let mut visitor = FindTyParams {
        all_ty_params: all_ty_params,
        relevant_ty_params: HashSet::new(),
    };
    for ty in relevant_tys {
        visit::walk_ty(&mut visitor, ty);
    }

    aster::from_generics(generics.clone())
        .with_predicates(
            generics.ty_params.iter()
                .map(|ty_param| ty_param.ident.clone())
                .filter(|id| visitor.relevant_ty_params.contains(id))
                .map(|id| aster::where_predicate()
                    // the type parameter that is being bounded e.g. T
                    .bound().build(aster::ty().id(id))
                    // the bound e.g. Serialize
                    .bound().trait_(bound.clone()).build()
                    .build()))
        .build()
}
