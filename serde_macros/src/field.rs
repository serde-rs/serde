use syntax::ast;

pub fn field_alias(field: &ast::StructField) -> Option<&ast::Lit> {
    field.node.attrs.iter()
        .find(|sa| {
            if let ast::MetaList(ref n, _) = sa.node.value.node {
                n == &"serde"
            } else {
                false
            }
        })
        .and_then(|sa| {
            if let ast::MetaList(_, ref vals) = sa.node.value.node {
                vals.iter().fold(None, |v, mi| {
                    if let ast::MetaNameValue(ref n, ref lit) = mi.node {
                        if n == &"alias" {
                            Some(lit)
                        } else {
                            v
                        }
                    } else {
                        v
                    }
                })
            } else {
                None
            }
        })
}
