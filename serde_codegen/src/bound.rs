use std::collections::HashSet;

use aster::AstBuilder;

use syntax::ast;
use syntax::ptr::P;
use syntax::visit;

use item::{attr, Item};

// Remove the default from every type parameter because in the generated impls
// they look like associated types: "error: associated type bindings are not
// allowed here".
pub fn without_defaults(generics: &ast::Generics) -> ast::Generics {
    ast::Generics {
        ty_params: generics.ty_params.iter().map(|ty_param| {
            ast::TyParam {
                default: None,
                .. ty_param.clone()
            }}).collect(),
        .. generics.clone()
    }
}

pub fn with_where_predicates(
    builder: &AstBuilder,
    generics: &ast::Generics,
    predicates: &[ast::WherePredicate],
) -> ast::Generics {
    builder.from_generics(generics.clone())
        .with_predicates(predicates.to_vec())
        .build()
}

pub fn with_where_predicates_from_fields<F>(
    builder: &AstBuilder,
    item: &Item,
    generics: &ast::Generics,
    from_field: F,
) -> ast::Generics
    where F: Fn(&attr::Field) -> Option<&[ast::WherePredicate]>,
{
    builder.from_generics(generics.clone())
        .with_predicates(
            item.body.all_fields()
                .flat_map(|field| from_field(&field.attrs))
                .flat_map(|predicates| predicates.to_vec()))
        .build()
}

pub fn with_bound<F>(
    builder: &AstBuilder,
    item: &Item,
    generics: &ast::Generics,
    filter: F,
    bound: &ast::Path,
) -> ast::Generics
    where F: Fn(&attr::Field) -> bool,
{
    builder.from_generics(generics.clone())
        .with_predicates(
            item.body.all_fields()
                .filter(|&field| filter(&field.attrs))
                .map(|field| &field.ty)
                // TODO this filter can be removed later, see comment on function
                .filter(|ty| contains_generic(ty, generics))
                .filter(|ty| !contains_recursion(ty, item.ident))
                .map(|ty| strip_reference(ty))
                .map(|ty| builder.where_predicate()
                    // the type that is being bounded e.g. T
                    .bound().build(ty.clone())
                    // the bound e.g. Serialize
                    .bound().trait_(bound.clone()).build()
                    .build()))
        .build()
}

// Rust <1.7 enforces that `where` clauses involve generic type parameters. The
// corresponding compiler error is E0193. It is no longer enforced in Rust >=1.7
// so this filtering can be removed in the future when we stop supporting <1.7.
//
// E0193 means we must not generate a `where` clause like `i32: Serialize`
// because even though i32 implements Serialize, i32 is not a generic type
// parameter. Clauses like `T: Serialize` and `Option<T>: Serialize` are okay.
// This function decides whether a given type references any of the generic type
// parameters in the input `Generics`.
fn contains_generic(ty: &ast::Ty, generics: &ast::Generics) -> bool {
    struct FindGeneric<'a> {
        generic_names: &'a HashSet<ast::Name>,
        found_generic: bool,
    }
    impl<'a, 'v> visit::Visitor<'v> for FindGeneric<'a> {
        fn visit_path(&mut self, path: &'v ast::Path, _id: ast::NodeId) {
            if !path.global
                    && path.segments.len() == 1
                    && self.generic_names.contains(&path.segments[0].identifier.name) {
                self.found_generic = true;
            } else {
                visit::walk_path(self, path);
            }
        }
    }

    let generic_names: HashSet<_> = generics.ty_params.iter()
        .map(|ty_param| ty_param.ident.name)
        .collect();

    let mut visitor = FindGeneric {
        generic_names: &generic_names,
        found_generic: false,
    };
    visit::walk_ty(&mut visitor, ty);
    visitor.found_generic
}

// We do not attempt to generate any bounds based on field types that are
// directly recursive, as in:
//
//    struct Test<D> {
//        next: Box<Test<D>>,
//    }
//
// This does not catch field types that are mutually recursive with some other
// type. For those, we require bounds to be specified by a `bound` attribute if
// the inferred ones are not correct.
//
//    struct Test<D> {
//        #[serde(bound="D: Serialize + Deserialize")]
//        next: Box<Other<D>>,
//    }
//    struct Other<D> {
//        #[serde(bound="D: Serialize + Deserialize")]
//        next: Box<Test<D>>,
//    }
fn contains_recursion(ty: &ast::Ty, ident: ast::Ident) -> bool {
    struct FindRecursion {
        ident: ast::Ident,
        found_recursion: bool,
    }
    impl<'v> visit::Visitor<'v> for FindRecursion {
        fn visit_path(&mut self, path: &'v ast::Path, _id: ast::NodeId) {
            if !path.global
                    && path.segments.len() == 1
                    && path.segments[0].identifier == self.ident {
                self.found_recursion = true;
            } else {
                visit::walk_path(self, path);
            }
        }
    }

    let mut visitor = FindRecursion {
        ident: ident,
        found_recursion: false,
    };
    visit::walk_ty(&mut visitor, ty);
    visitor.found_recursion
}

// This is required to handle types that use both a reference and a value of
// the same type, as in:
//
//    enum Test<'a, T> where T: 'a {
//        Lifetime(&'a T),
//        NoLifetime(T),
//    }
//
// Preserving references, we would generate an impl like:
//
//    impl<'a, T> Serialize for Test<'a, T>
//        where &'a T: Serialize,
//              T: Serialize { ... }
//
// And taking a reference to one of the elements would fail with:
//
//    error: cannot infer an appropriate lifetime for pattern due
//    to conflicting requirements [E0495]
//        Test::NoLifetime(ref v) => { ... }
//                         ^~~~~
//
// Instead, we strip references before adding `T: Serialize` bounds in order to
// generate:
//
//    impl<'a, T> Serialize for Test<'a, T>
//        where T: Serialize { ... }
fn strip_reference(mut ty: &P<ast::Ty>) -> &P<ast::Ty> {
    while let ast::TyKind::Rptr(_, ref mut_ty) = ty.node {
        ty = &mut_ty.ty;
    }
    ty
}
