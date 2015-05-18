#![cfg_attr(not(feature = "with-syntex"), feature(rustc_private, plugin))]
#![cfg_attr(not(feature = "with-syntex"), plugin(quasi_macros))]

extern crate aster;
extern crate quasi;

#[cfg(feature = "with-syntex")]
extern crate syntex;

#[cfg(feature = "with-syntex")]
extern crate syntex_syntax as syntax;

#[cfg(not(feature = "with-syntex"))]
extern crate syntax;

#[cfg(not(feature = "with-syntex"))]
extern crate rustc;

#[cfg(feature = "with-syntex")]
include!(concat!(env!("OUT_DIR"), "/lib.rs"));

#[cfg(not(feature = "with-syntex"))]
include!("lib.rs.in");

#[cfg(feature = "with-syntex")]
pub fn register(reg: &mut syntex::Registry) {
    use syntax::{ast, fold};

    reg.add_attr("feature(custom_derive)");
    reg.add_attr("feature(custom_attribute)");

    reg.add_decorator("derive_Serialize", ser::expand_derive_serialize);
    reg.add_decorator("derive_Deserialize", de::expand_derive_deserialize);

    reg.add_post_expansion_pass(strip_attributes);

    /// Strip the serde attributes from the crate.
    #[cfg(feature = "with-syntex")]
    fn strip_attributes(krate: ast::Crate) -> ast::Crate {
        /// Helper folder that strips the serde attributes after the extensions have been expanded.
        struct StripAttributeFolder;

        impl fold::Folder for StripAttributeFolder {
            fn fold_attribute(&mut self, attr: ast::Attribute) -> Option<ast::Attribute> {
                match attr.node.value.node {
                    ast::MetaList(ref n, _) if n == &"serde" => { return None; }
                    _ => {}
                }

                Some(attr)
            }

            fn fold_mac(&mut self, mac: ast::Mac) -> ast::Mac {
                fold::noop_fold_mac(mac, self)
            }
        }

        fold::Folder::fold_crate(&mut StripAttributeFolder, krate)
    }
}

#[cfg(not(feature = "with-syntex"))]
pub fn register(reg: &mut rustc::plugin::Registry) {
    reg.register_syntax_extension(
        syntax::parse::token::intern("derive_Serialize"),
        syntax::ext::base::MultiDecorator(
            Box::new(ser::expand_derive_serialize)));

    reg.register_syntax_extension(
        syntax::parse::token::intern("derive_Deserialize"),
        syntax::ext::base::MultiDecorator(
            Box::new(de::expand_derive_deserialize)));
}
