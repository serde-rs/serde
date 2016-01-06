use syntax::ast;
use syntax::ext::base::ExtCtxt;

use aster;
use attr::{ContainerAttrs, ContainerAttrsBuilder, FieldAttrs, FieldAttrsBuilder};

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

pub fn container_attrs(
    _cx: &ExtCtxt,
    container: &ast::Item,
) -> ContainerAttrs {
    ContainerAttrsBuilder::new().attrs(container.attrs()).build()
}