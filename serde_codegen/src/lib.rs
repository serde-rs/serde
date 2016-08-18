#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", allow(too_many_arguments))]
#![cfg_attr(feature = "clippy", allow(used_underscore_binding))]
#![cfg_attr(not(feature = "with-syntex"), feature(rustc_private, plugin))]
#![cfg_attr(not(feature = "with-syntex"), plugin(quasi_macros))]

extern crate aster;
extern crate quasi;
extern crate serde_codegen_internals as internals;

#[cfg(feature = "with-syntex")]
extern crate syntex;

#[cfg(feature = "with-syntex")]
#[macro_use]
extern crate syntex_syntax as syntax;

#[cfg(not(feature = "with-syntex"))]
#[macro_use]
extern crate syntax;

#[cfg(not(feature = "with-syntex"))]
extern crate rustc_plugin;

#[cfg(feature = "with-syntex")]
use std::path::Path;

#[cfg(not(feature = "with-syntex"))]
use syntax::feature_gate::AttributeType;

#[cfg(feature = "with-syntex")]
include!(concat!(env!("OUT_DIR"), "/lib.rs"));

#[cfg(not(feature = "with-syntex"))]
include!("lib.rs.in");

#[cfg(feature = "with-syntex")]
mod env;

#[cfg(feature = "with-syntex")]
pub fn expand<S, D>(src: S, dst: D) -> Result<(), syntex::Error>
    where S: AsRef<Path>,
          D: AsRef<Path>,
{
    let src = src.as_ref().to_owned();
    let dst = dst.as_ref().to_owned();

    let expand_thread = move || {
        use syntax::{ast, fold};

        /// Strip the serde attributes from the crate.
        #[cfg(feature = "with-syntex")]
        fn strip_attributes(krate: ast::Crate) -> ast::Crate {
            /// Helper folder that strips the serde attributes after the extensions have been expanded.
            struct StripAttributeFolder;

            impl fold::Folder for StripAttributeFolder {
                fn fold_attribute(&mut self, attr: ast::Attribute) -> Option<ast::Attribute> {
                    match attr.node.value.node {
                        ast::MetaItemKind::List(ref n, _) if n == &"serde" => { return None; }
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

        let mut reg = syntex::Registry::new();

        reg.add_attr("feature(custom_derive)");
        reg.add_attr("feature(custom_attribute)");

        reg.add_decorator("derive_Serialize", ser::expand_derive_serialize);
        reg.add_decorator("derive_Deserialize", de::expand_derive_deserialize);

        reg.add_post_expansion_pass(strip_attributes);

        reg.expand("", src, dst)
    };

    // 16 MB stack unless otherwise specified
    let _tmp_env = env::set_if_unset("RUST_MIN_STACK", "16777216");

    use std::thread;
    thread::spawn(expand_thread).join().unwrap()
}

#[cfg(not(feature = "with-syntex"))]
pub fn register(reg: &mut rustc_plugin::Registry) {
    reg.register_syntax_extension(
        syntax::parse::token::intern("derive_Serialize"),
        syntax::ext::base::MultiDecorator(
            Box::new(ser::expand_derive_serialize)));

    reg.register_syntax_extension(
        syntax::parse::token::intern("derive_Deserialize"),
        syntax::ext::base::MultiDecorator(
            Box::new(de::expand_derive_deserialize)));

    reg.register_attribute("serde".to_owned(), AttributeType::Normal);
}
