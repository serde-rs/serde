use syntax::ast;
use syntax::ext::base::ExtCtxt;

use aster;
use attr::{FieldAttrs, FieldAttrsBuilder};

pub fn struct_field_attrs(
    _cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    fields: &[ast::StructField],
) -> Vec<FieldAttrs> {
    fields.iter()
        .map(|field| {
            FieldAttrsBuilder::new(builder).field(field).build()
        })
        .collect()
}
