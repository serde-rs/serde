extern crate proc_macro;

mod buffer;
mod bytecode;

use crate::buffer::{InputBuffer, OutputBuffer};
use crate::bytecode::Bytecode;
use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use std::io::{Read, Write};
use std::iter::FromIterator;
use std::process::{Command, Stdio};
use std::str::FromStr;

#[cfg(not(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")))]
compile_error! {
    "this proof of concept is only compiled for x86_64-unknown-linux-gnu"
}

#[proc_macro_derive(Serialize, attributes(serde))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    derive(0, input)
}

#[proc_macro_derive(Deserialize, attributes(serde))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    derive(1, input)
}

fn derive(select: u8, input: TokenStream) -> TokenStream {
    let mut memory = TokenMemory::default();
    let mut buf = OutputBuffer::new();
    buf.write_u8(select);

    memory.spans.push(Span::call_site());
    for token in input {
        memory.linearize_token(token, &mut buf);
    }

    let mut child = Command::new(concat!(env!("CARGO_MANIFEST_DIR"), "/serde_derive"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn process");

    let mut stdin = child.stdin.take().unwrap();
    let mut buf = buf.into_bytes();
    stdin.write_all(&buf).unwrap();
    drop(stdin);

    let mut stdout = child.stdout.take().unwrap();
    buf.clear();
    stdout.read_to_end(&mut buf).unwrap();

    let mut buf = InputBuffer::new(&buf);
    memory.receive(&mut buf)
}

#[derive(Default)]
struct TokenMemory {
    spans: Vec<Span>,
    groups: Vec<Group>,
    idents: Vec<Ident>,
    puncts: Vec<Punct>,
    literals: Vec<Literal>,
}

enum Kind {
    Group(Delimiter),
    Ident,
    Punct(Spacing),
    Literal,
}

impl TokenMemory {
    // Depth-first post-order traversal.
    fn linearize_token(&mut self, token: TokenTree, buf: &mut OutputBuffer) {
        match token {
            TokenTree::Group(group) => {
                let mut len = 0usize;
                for token in group.stream() {
                    self.linearize_token(token, buf);
                    len += 1;
                }
                assert!(len <= u32::MAX as usize);
                buf.write_u8(match group.delimiter() {
                    Delimiter::Parenthesis => Bytecode::GROUP_PARENTHESIS,
                    Delimiter::Brace => Bytecode::GROUP_BRACE,
                    Delimiter::Bracket => Bytecode::GROUP_BRACKET,
                    Delimiter::None => Bytecode::GROUP_NONE,
                });
                buf.write_u32(len as u32);
                self.spans
                    .extend([group.span(), group.span_open(), group.span_close()]);
                self.groups.push(group);
            }
            TokenTree::Ident(ident) => {
                buf.write_u8(Bytecode::IDENT);
                let repr = ident.to_string();
                assert!(repr.len() <= u16::MAX as usize);
                buf.write_u16(repr.len() as u16);
                buf.write_str(&repr);
                self.spans.push(ident.span());
                self.idents.push(ident);
            }
            TokenTree::Punct(punct) => {
                buf.write_u8(match punct.spacing() {
                    Spacing::Alone => Bytecode::PUNCT_ALONE,
                    Spacing::Joint => Bytecode::PUNCT_JOINT,
                });
                let ch = punct.as_char();
                assert!(ch.is_ascii());
                buf.write_u8(ch as u8);
                self.spans.push(punct.span());
                self.puncts.push(punct);
            }
            TokenTree::Literal(literal) => {
                buf.write_u8(Bytecode::LITERAL);
                let repr = literal.to_string();
                assert!(repr.len() <= u16::MAX as usize);
                buf.write_u16(repr.len() as u16);
                buf.write_str(&repr);
                self.spans.push(literal.span());
                self.literals.push(literal);
            }
        }
    }

    fn receive(&self, buf: &mut InputBuffer) -> TokenStream {
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
                Bytecode::LOAD_GROUP => {
                    let identity = buf.read_u32();
                    let group = self.groups[identity as usize].clone();
                    trees.push(TokenTree::Group(group));
                    continue;
                }
                Bytecode::LOAD_IDENT => {
                    let identity = buf.read_u32();
                    let ident = self.idents[identity as usize].clone();
                    trees.push(TokenTree::Ident(ident));
                    continue;
                }
                Bytecode::LOAD_PUNCT => {
                    let identity = buf.read_u32();
                    let punct = self.puncts[identity as usize].clone();
                    trees.push(TokenTree::Punct(punct));
                    continue;
                }
                Bytecode::LOAD_LITERAL => {
                    let identity = buf.read_u32();
                    let literal = self.literals[identity as usize].clone();
                    trees.push(TokenTree::Literal(literal));
                    continue;
                }
                Bytecode::SET_SPAN => {
                    trees.last_mut().unwrap().set_span(self.read_span(buf));
                    continue;
                }
                _ => unreachable!(),
            } {
                Kind::Group(delimiter) => {
                    let len = buf.read_u32();
                    let stream = trees.drain(trees.len() - len as usize..).collect();
                    let group = Group::new(delimiter, stream);
                    trees.push(TokenTree::Group(group));
                }
                Kind::Ident => {
                    let len = buf.read_u16();
                    let repr = buf.read_str(len as usize);
                    let span = self.read_span(buf);
                    let ident = if let Some(repr) = repr.strip_prefix("r#") {
                        Ident::new_raw(repr, span)
                    } else {
                        Ident::new(repr, span)
                    };
                    trees.push(TokenTree::Ident(ident));
                }
                Kind::Punct(spacing) => {
                    let ch = buf.read_u8();
                    assert!(ch.is_ascii());
                    let punct = Punct::new(ch as char, spacing);
                    trees.push(TokenTree::Punct(punct));
                }
                Kind::Literal => {
                    let len = buf.read_u16();
                    let repr = buf.read_str(len as usize);
                    let literal = Literal::from_str(repr).unwrap();
                    trees.push(TokenTree::Literal(literal));
                }
            }
        }

        TokenStream::from_iter(trees)
    }

    fn read_span(&self, buf: &mut InputBuffer) -> Span {
        let lo = buf.read_u32();
        let hi = buf.read_u32();
        let span = self.spans[lo as usize];
        if lo == hi {
            span
        } else {
            #[cfg(any())] // FIXME
            return span.join(self.spans[hi as usize]).unwrap_or(span);
            span
        }
    }
}
