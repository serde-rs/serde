use syntax::ast;
use syntax::codemap::{self, ExpnId, Span};
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::fold::{self, Folder};
use syntax::parse::token::intern;
use syntax::ptr::P;

pub fn record_expansion(cx: &ExtCtxt, item: P<ast::Item>, derive: &str) -> Annotatable {
    let info = codemap::ExpnInfo {
        call_site: codemap::DUMMY_SP,
        callee: codemap::NameAndSpan {
            format: codemap::MacroAttribute(intern(&format!("derive({})", derive))),
            span: None,
            allow_internal_unstable: false,
        },
    };
    let expn_id = cx.codemap().record_expansion(info);

    let mut respanner = Respanner {
        expn_id: expn_id,
    };
    let item = item.map(|item| respanner.fold_item_simple(item));
    Annotatable::Item(item)
}

struct Respanner {
    expn_id: ExpnId,
}

impl Folder for Respanner {
    fn new_span(&mut self, span: Span) -> Span {
        Span {
            expn_id: self.expn_id,
            ..span
        }
    }

    fn fold_mac(&mut self, mac: ast::Mac) -> ast::Mac {
        fold::noop_fold_mac(mac, self)
    }
}
