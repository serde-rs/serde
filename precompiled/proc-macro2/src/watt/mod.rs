pub mod buffer;
pub mod bytecode;

use crate::watt::buffer::{InputBuffer, OutputBuffer};
use crate::watt::bytecode::Bytecode;
use crate::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use std::str::FromStr;

pub enum Kind {
    Group(Delimiter),
    Ident,
    Punct(Spacing),
    Literal,
}

pub enum Identity {}

impl Identity {
    pub const RESPANNED: u32 = 1 << 31;
    pub const NOVEL: u32 = u32::MAX;
}

impl Span {
    fn is_call_site(&self) -> bool {
        self.lo == 0 && self.hi == 0
    }
}

fn post_increment(counter: &mut u32) -> impl FnMut() -> u32 + '_ {
    || {
        let value = *counter;
        *counter += 1;
        value
    }
}

pub fn load(buf: &mut InputBuffer) -> TokenStream {
    let mut span_counter = 1;
    let mut next_span = post_increment(&mut span_counter);
    let mut next_span = || {
        let next = next_span();
        Span { lo: next, hi: next }
    };

    let [mut group_counter, mut ident_counter, mut punct_counter, mut literal_counter] = [0; 4];
    let mut next_group = post_increment(&mut group_counter);
    let mut next_ident = post_increment(&mut ident_counter);
    let mut next_punct = post_increment(&mut punct_counter);
    let mut next_literal = post_increment(&mut literal_counter);

    let mut trees = Vec::new();
    while !buf.is_empty() {
        match match buf.read_u8() {
            Bytecode::GROUP_PARENTHESIS => Kind::Group(Delimiter::Parenthesis),
            Bytecode::GROUP_BRACE => Kind::Group(Delimiter::Brace),
            Bytecode::GROUP_BRACKET => Kind::Group(Delimiter::Bracket),
            Bytecode::GROUP_NONE => Kind::Group(Delimiter::None),
            Bytecode::IDENT => Kind::Ident,
            Bytecode::PUNCT_ALONE => Kind::Punct(Spacing::Alone),
            Bytecode::PUNCT_JOINT => Kind::Punct(Spacing::Joint),
            Bytecode::LITERAL => Kind::Literal,
            _ => unreachable!(),
        } {
            Kind::Group(delimiter) => {
                let len = buf.read_u32();
                let stream = trees.drain(trees.len() - len as usize..).collect();
                trees.push(TokenTree::Group(Group {
                    delimiter,
                    stream,
                    span: next_span(),
                    span_open: next_span(),
                    span_close: next_span(),
                    identity: next_group(),
                }));
            }
            Kind::Ident => {
                let len = buf.read_u16();
                let repr = buf.read_str(len as usize);
                let ident = if let Some(repr) = repr.strip_prefix("r#") {
                    proc_macro2::Ident::new_raw(repr, proc_macro2::Span::call_site())
                } else if repr == "$crate" {
                    proc_macro2::Ident::new("crate", proc_macro2::Span::call_site())
                } else {
                    proc_macro2::Ident::new(repr, proc_macro2::Span::call_site())
                };
                trees.push(TokenTree::Ident(Ident {
                    fallback: ident,
                    span: next_span(),
                    identity: next_ident(),
                }));
            }
            Kind::Punct(spacing) => {
                let ch = buf.read_u8();
                assert!(ch.is_ascii());
                let punct = proc_macro2::Punct::new(ch as char, spacing);
                trees.push(TokenTree::Punct(Punct {
                    fallback: punct,
                    span: next_span(),
                    identity: next_punct(),
                }));
            }
            Kind::Literal => {
                let len = buf.read_u16();
                let repr = buf.read_str(len as usize);
                let literal = proc_macro2::Literal::from_str(repr).unwrap();
                trees.push(TokenTree::Literal(Literal {
                    fallback: literal,
                    span: next_span(),
                    identity: next_literal(),
                }));
            }
        }
    }

    TokenStream { content: trees }
}

pub fn linearize(tokens: TokenStream) -> Vec<u8> {
    let mut buf = OutputBuffer::new();
    for token in &tokens.content {
        linearize_token(token, &mut buf);
    }
    buf.into_bytes()
}

fn linearize_token(token: &TokenTree, buf: &mut OutputBuffer) {
    let needs_span;
    match token {
        TokenTree::Group(group) => {
            if group.identity < Identity::NOVEL {
                buf.write_u8(Bytecode::LOAD_GROUP);
                buf.write_u32(group.identity & !Identity::RESPANNED);
                needs_span = group.identity >= Identity::RESPANNED;
            } else {
                let len = group.stream.len();
                assert!(len <= u32::MAX as usize);
                for token in &group.stream {
                    linearize_token(token, buf);
                }
                buf.write_u8(match group.delimiter {
                    Delimiter::Parenthesis => Bytecode::GROUP_PARENTHESIS,
                    Delimiter::Brace => Bytecode::GROUP_BRACE,
                    Delimiter::Bracket => Bytecode::GROUP_BRACKET,
                    Delimiter::None => Bytecode::GROUP_NONE,
                });
                buf.write_u32(len as u32);
                needs_span = !group.span.is_call_site();
            }
        }
        TokenTree::Ident(ident) => {
            if ident.identity < Identity::NOVEL {
                buf.write_u8(Bytecode::LOAD_IDENT);
                buf.write_u32(ident.identity & !Identity::RESPANNED);
                needs_span = ident.identity >= Identity::RESPANNED;
            } else {
                buf.write_u8(Bytecode::IDENT);
                let repr = ident.to_string();
                assert!(repr.len() <= u16::MAX as usize);
                buf.write_u16(repr.len() as u16);
                buf.write_str(&repr);
                linearize_span(ident.span, buf);
                needs_span = false;
            }
        }
        TokenTree::Punct(punct) => {
            if punct.identity < Identity::NOVEL {
                buf.write_u8(Bytecode::LOAD_PUNCT);
                buf.write_u32(punct.identity & !Identity::RESPANNED);
                needs_span = punct.identity >= Identity::RESPANNED;
            } else {
                buf.write_u8(match punct.spacing() {
                    Spacing::Alone => Bytecode::PUNCT_ALONE,
                    Spacing::Joint => Bytecode::PUNCT_JOINT,
                });
                let ch = punct.as_char();
                assert!(ch.is_ascii());
                buf.write_u8(ch as u8);
                needs_span = !punct.span.is_call_site();
            }
        }
        TokenTree::Literal(literal) => {
            if literal.identity < Identity::NOVEL {
                buf.write_u8(Bytecode::LOAD_LITERAL);
                buf.write_u32(literal.identity & !Identity::RESPANNED);
                needs_span = literal.identity >= Identity::RESPANNED;
            } else {
                buf.write_u8(Bytecode::LITERAL);
                let repr = literal.to_string();
                assert!(repr.len() <= u16::MAX as usize);
                buf.write_u16(repr.len() as u16);
                buf.write_str(&repr);
                needs_span = !literal.span.is_call_site();
            }
        }
    }
    if needs_span {
        buf.write_u8(Bytecode::SET_SPAN);
        linearize_span(token.span(), buf);
    }
}

fn linearize_span(span: Span, buf: &mut OutputBuffer) {
    buf.write_u32(span.lo);
    buf.write_u32(span.hi);
}
