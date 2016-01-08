use syntax::ast;
use syntax::ext::base::ExtCtxt;
use syntax::ptr;

use aster;
use attr;

pub fn struct_field_attrs(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    fields: &[ast::StructField],
) -> Result<Vec<attr::FieldAttrs>, ()> {
    let mut attrs = vec![];
    for field in fields {
        let builder = attr::FieldAttrsBuilder::new(cx, builder);
        let builder = try!(builder.field(field));
        let attr = builder.build();
        attrs.push(attr);
    }

    Ok(attrs)
}

pub fn variant_attrs(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    variants: &[ptr::P<ast::Variant>],
) -> Result<Vec<attr::FieldAttrs>, ()> {
    let mut attrs = vec![];
    for variant in variants {
        let builder = attr::FieldAttrsBuilder::new(cx, builder);
        let builder = try!(builder.variant(variant));
        let attr = builder.build();
        attrs.push(attr);
    }

    Ok(attrs)
}

pub fn container_attrs(
    cx: &ExtCtxt,
    container: &ast::Item,
) -> Result<attr::ContainerAttrs, ()> {
    let builder = attr::ContainerAttrsBuilder::new(cx);
    let builder = try!(builder.attrs(container.attrs()));
    Ok(builder.build())
}
