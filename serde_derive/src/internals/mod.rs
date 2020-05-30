pub mod ast;
pub mod attr;

mod ctxt;
pub use self::ctxt::Ctxt;

mod case;
mod check;
mod symbol;

use syn::Type;

#[derive(Copy, Clone)]
pub enum Derive {
    Serialize,
    Deserialize,
}

pub fn ungroup(mut ty: &Type) -> &Type {
    while let Type::Group(group) = ty {
        ty = &group.elem;
    }
    ty
}
