pub mod ast;
pub mod attr;
pub mod name;

mod case;
mod check;
mod ctxt;
mod receiver;
mod respan;
mod symbol;

use syn::Type;

pub use self::ctxt::Ctxt;
pub use self::receiver::replace_receiver;

#[derive(Copy, Clone, PartialEq, Eq)]
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
