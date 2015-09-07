use syntax::ast;
use syntax::ext::base::ExtCtxt;

use aster;
use attr::{FieldAttrs, FieldAttrsBuilder};

pub fn struct_field_attrs(
    _cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    struct_def: &ast::StructDef,
) -> Vec<FieldAttrs> {
    struct_def.fields.iter()
        .map(|field| {
            FieldAttrsBuilder::new(builder).field(field).build()
        })
        .collect()
}
