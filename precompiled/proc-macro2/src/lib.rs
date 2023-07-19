#[doc(hidden)]
pub mod watt;

use crate::extra::DelimSpan;
use crate::watt::Identity;
use std::cmp::Ordering;
use std::fmt::{self, Debug, Display};
use std::hash::{Hash, Hasher};
use std::ops::RangeBounds;
use std::str::FromStr;

pub use proc_macro2::{Delimiter, Spacing};

#[derive(Copy, Clone)]
pub struct Span {
    lo: u32,
    hi: u32,
}

impl Span {
    pub fn call_site() -> Self {
        Span { lo: 0, hi: 0 }
    }

    pub fn join(&self, other: Self) -> Option<Self> {
        Some(Span {
            lo: self.lo,
            hi: other.hi,
        })
    }
}

#[derive(Clone)]
pub enum TokenTree {
    Group(Group),
    Ident(Ident),
    Punct(Punct),
    Literal(Literal),
}

impl TokenTree {
    pub fn span(&self) -> Span {
        match self {
            TokenTree::Group(group) => group.span(),
            TokenTree::Ident(ident) => ident.span(),
            TokenTree::Punct(punct) => punct.span(),
            TokenTree::Literal(literal) => literal.span(),
        }
    }

    pub fn set_span(&mut self, span: Span) {
        match self {
            TokenTree::Group(group) => group.set_span(span),
            TokenTree::Ident(ident) => ident.set_span(span),
            TokenTree::Punct(punct) => punct.set_span(span),
            TokenTree::Literal(literal) => literal.set_span(span),
        }
    }
}

impl From<Group> for TokenTree {
    fn from(group: Group) -> Self {
        TokenTree::Group(group)
    }
}

impl From<Ident> for TokenTree {
    fn from(ident: Ident) -> Self {
        TokenTree::Ident(ident)
    }
}

impl From<Punct> for TokenTree {
    fn from(punct: Punct) -> Self {
        TokenTree::Punct(punct)
    }
}

impl From<Literal> for TokenTree {
    fn from(literal: Literal) -> Self {
        TokenTree::Literal(literal)
    }
}

impl Debug for TokenTree {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenTree::Group(group) => Debug::fmt(group, formatter),
            TokenTree::Ident(ident) => {
                let mut debug = formatter.debug_struct("Ident");
                debug.field("sym", &format_args!("{}", ident));
                debug.finish()
            }
            TokenTree::Punct(punct) => Debug::fmt(punct, formatter),
            TokenTree::Literal(literal) => Debug::fmt(literal, formatter),
        }
    }
}

#[derive(Clone)]
pub struct Group {
    delimiter: Delimiter,
    stream: Vec<TokenTree>,
    span: Span,
    span_open: Span,
    span_close: Span,
    identity: u32,
}

impl Group {
    pub fn new(delimiter: Delimiter, stream: TokenStream) -> Self {
        Group {
            delimiter,
            stream: stream.content,
            span: Span::call_site(),
            span_open: Span::call_site(),
            span_close: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn stream(&self) -> TokenStream {
        TokenStream {
            content: self.stream.clone(),
        }
    }

    pub fn delimiter(&self) -> Delimiter {
        self.delimiter
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn span_open(&self) -> Span {
        self.span_open
    }

    pub fn span_close(&self) -> Span {
        self.span_close
    }

    pub fn delim_span(&self) -> DelimSpan {
        DelimSpan {
            join: self.span,
            open: self.span_open,
            close: self.span_close,
        }
    }

    pub fn set_span(&mut self, span: Span) {
        self.span = span;
        self.identity |= Identity::RESPANNED;
    }
}

impl Display for Group {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let (open, close) = match self.delimiter {
            Delimiter::Parenthesis => ("(", ")"),
            Delimiter::Brace => ("{ ", "}"),
            Delimiter::Bracket => ("[", "]"),
            Delimiter::None => ("", ""),
        };

        formatter.write_str(open)?;
        display_tokens(&self.stream, formatter)?;
        if self.delimiter == Delimiter::Brace && !self.stream.is_empty() {
            formatter.write_str(" ")?;
        }
        formatter.write_str(close)?;

        Ok(())
    }
}

impl Debug for Group {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let mut debug = formatter.debug_struct("Group");
        debug.field("delimiter", &self.delimiter);
        debug.field("stream", &self.stream);
        debug.finish()
    }
}

#[derive(Clone)]
pub struct Ident {
    fallback: proc_macro2::Ident,
    span: Span,
    identity: u32,
}

impl Ident {
    pub fn new(string: &str, span: Span) -> Self {
        Ident {
            fallback: proc_macro2::Ident::new(string, proc_macro2::Span::call_site()),
            span,
            identity: Identity::NOVEL,
        }
    }

    pub fn new_raw(string: &str, span: Span) -> Self {
        Ident {
            fallback: proc_macro2::Ident::new_raw(string, proc_macro2::Span::call_site()),
            span,
            identity: Identity::NOVEL,
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn set_span(&mut self, span: Span) {
        self.span = span;
        self.identity |= Identity::RESPANNED;
    }
}

impl Display for Ident {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.fallback, formatter)
    }
}

impl Debug for Ident {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.fallback, formatter)
    }
}

impl Eq for Ident {}

impl PartialEq for Ident {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.fallback, &other.fallback)
    }
}

impl<T> PartialEq<T> for Ident
where
    T: ?Sized + AsRef<str>,
{
    fn eq(&self, other: &T) -> bool {
        PartialEq::eq(&self.fallback, other)
    }
}

impl Ord for Ident {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&self.fallback, &other.fallback)
    }
}

impl PartialOrd for Ident {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(&self.fallback, &other.fallback)
    }
}

impl Hash for Ident {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        Hash::hash(&self.fallback, hasher);
    }
}

#[derive(Clone)]
pub struct Punct {
    fallback: proc_macro2::Punct,
    span: Span,
    identity: u32,
}

impl Punct {
    pub fn new(ch: char, spacing: Spacing) -> Self {
        Punct {
            fallback: proc_macro2::Punct::new(ch, spacing),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn as_char(&self) -> char {
        self.fallback.as_char()
    }

    pub fn spacing(&self) -> Spacing {
        self.fallback.spacing()
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn set_span(&mut self, span: Span) {
        self.span = span;
        self.identity |= Identity::RESPANNED;
    }
}

impl Display for Punct {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.fallback, formatter)
    }
}

impl Debug for Punct {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.fallback, formatter)
    }
}

#[derive(Clone)]
pub struct Literal {
    fallback: proc_macro2::Literal,
    span: Span,
    identity: u32,
}

impl Literal {
    pub fn u8_suffixed(n: u8) -> Self {
        Literal {
            fallback: proc_macro2::Literal::u8_suffixed(n),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn u16_suffixed(n: u16) -> Self {
        Literal {
            fallback: proc_macro2::Literal::u16_suffixed(n),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn u32_suffixed(n: u32) -> Self {
        Literal {
            fallback: proc_macro2::Literal::u32_suffixed(n),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn u64_suffixed(n: u64) -> Self {
        Literal {
            fallback: proc_macro2::Literal::u64_suffixed(n),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn u128_suffixed(n: u128) -> Self {
        Literal {
            fallback: proc_macro2::Literal::u128_suffixed(n),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn usize_suffixed(n: usize) -> Self {
        Literal {
            fallback: proc_macro2::Literal::usize_suffixed(n),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn i8_suffixed(n: i8) -> Self {
        Literal {
            fallback: proc_macro2::Literal::i8_suffixed(n),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn i16_suffixed(n: i16) -> Self {
        Literal {
            fallback: proc_macro2::Literal::i16_suffixed(n),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn i32_suffixed(n: i32) -> Self {
        Literal {
            fallback: proc_macro2::Literal::i32_suffixed(n),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn i64_suffixed(n: i64) -> Self {
        Literal {
            fallback: proc_macro2::Literal::i64_suffixed(n),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn i128_suffixed(n: i128) -> Self {
        Literal {
            fallback: proc_macro2::Literal::i128_suffixed(n),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn isize_suffixed(n: isize) -> Self {
        Literal {
            fallback: proc_macro2::Literal::isize_suffixed(n),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn u8_unsuffixed(n: u8) -> Self {
        Literal {
            fallback: proc_macro2::Literal::u8_unsuffixed(n),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn u16_unsuffixed(n: u16) -> Self {
        Literal {
            fallback: proc_macro2::Literal::u16_unsuffixed(n),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn u32_unsuffixed(n: u32) -> Self {
        Literal {
            fallback: proc_macro2::Literal::u32_unsuffixed(n),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn u64_unsuffixed(n: u64) -> Self {
        Literal {
            fallback: proc_macro2::Literal::u64_unsuffixed(n),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn u128_unsuffixed(n: u128) -> Self {
        Literal {
            fallback: proc_macro2::Literal::u128_unsuffixed(n),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn usize_unsuffixed(n: usize) -> Self {
        Literal {
            fallback: proc_macro2::Literal::usize_unsuffixed(n),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn i8_unsuffixed(n: i8) -> Self {
        Literal {
            fallback: proc_macro2::Literal::i8_unsuffixed(n),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn i16_unsuffixed(n: i16) -> Self {
        Literal {
            fallback: proc_macro2::Literal::i16_unsuffixed(n),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn i32_unsuffixed(n: i32) -> Self {
        Literal {
            fallback: proc_macro2::Literal::i32_unsuffixed(n),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn i64_unsuffixed(n: i64) -> Self {
        Literal {
            fallback: proc_macro2::Literal::i64_unsuffixed(n),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn i128_unsuffixed(n: i128) -> Self {
        Literal {
            fallback: proc_macro2::Literal::i128_unsuffixed(n),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn isize_unsuffixed(n: isize) -> Self {
        Literal {
            fallback: proc_macro2::Literal::isize_unsuffixed(n),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn f64_unsuffixed(f: f64) -> Self {
        Literal {
            fallback: proc_macro2::Literal::f64_unsuffixed(f),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn f64_suffixed(f: f64) -> Self {
        Literal {
            fallback: proc_macro2::Literal::f64_suffixed(f),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn f32_unsuffixed(f: f32) -> Self {
        Literal {
            fallback: proc_macro2::Literal::f32_unsuffixed(f),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn f32_suffixed(f: f32) -> Self {
        Literal {
            fallback: proc_macro2::Literal::f32_suffixed(f),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn string(string: &str) -> Self {
        Literal {
            fallback: proc_macro2::Literal::string(string),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn character(ch: char) -> Self {
        Literal {
            fallback: proc_macro2::Literal::character(ch),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn byte_string(s: &[u8]) -> Self {
        Literal {
            fallback: proc_macro2::Literal::byte_string(s),
            span: Span::call_site(),
            identity: Identity::NOVEL,
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn set_span(&mut self, span: Span) {
        self.span = span;
        self.identity |= Identity::RESPANNED;
    }

    pub fn subspan<R: RangeBounds<usize>>(&self, range: R) -> Option<Span> {
        let _ = range;
        None
    }
}

impl FromStr for Literal {
    type Err = LexError;

    fn from_str(repr: &str) -> Result<Self, Self::Err> {
        let fallback = match proc_macro2::Literal::from_str(repr) {
            Ok(literal) => literal,
            Err(error) => {
                return Err(LexError {
                    fallback: error,
                    span: Span::call_site(),
                });
            }
        };
        Ok(Literal {
            fallback,
            span: Span::call_site(),
            identity: Identity::NOVEL,
        })
    }
}

impl Display for Literal {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.fallback, formatter)
    }
}

impl Debug for Literal {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.fallback, formatter)
    }
}

#[derive(Clone)]
pub struct TokenStream {
    content: Vec<TokenTree>,
}

impl TokenStream {
    pub fn new() -> Self {
        TokenStream {
            content: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}

impl IntoIterator for TokenStream {
    type Item = TokenTree;
    type IntoIter = token_stream::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        token_stream::IntoIter {
            iter: self.content.into_iter(),
        }
    }
}

impl Extend<TokenStream> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenStream>>(&mut self, streams: I) {
        self.content.extend(streams.into_iter().flatten());
    }
}

impl Extend<TokenTree> for TokenStream {
    fn extend<I: IntoIterator<Item = TokenTree>>(&mut self, streams: I) {
        self.content.extend(streams);
    }
}

impl FromIterator<TokenStream> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenStream>>(streams: I) -> Self {
        let content = streams.into_iter().flatten().collect();
        TokenStream { content }
    }
}

impl FromIterator<TokenTree> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenTree>>(streams: I) -> Self {
        let content = streams.into_iter().collect();
        TokenStream { content }
    }
}

impl FromStr for TokenStream {
    type Err = LexError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let fallback = match proc_macro2::TokenStream::from_str(string) {
            Ok(token_stream) => token_stream,
            Err(error) => {
                return Err(LexError {
                    fallback: error,
                    span: Span::call_site(),
                });
            }
        };

        fn convert_token_stream(stream: proc_macro2::TokenStream) -> TokenStream {
            TokenStream {
                content: stream.into_iter().map(convert_token_tree).collect(),
            }
        }

        fn convert_token_tree(token: proc_macro2::TokenTree) -> TokenTree {
            match token {
                proc_macro2::TokenTree::Group(group) => TokenTree::Group(Group::new(
                    group.delimiter(),
                    convert_token_stream(group.stream()),
                )),
                proc_macro2::TokenTree::Ident(ident) => TokenTree::Ident(Ident {
                    fallback: ident,
                    span: Span::call_site(),
                    identity: Identity::NOVEL,
                }),
                proc_macro2::TokenTree::Punct(punct) => TokenTree::Punct(Punct {
                    fallback: punct,
                    span: Span::call_site(),
                    identity: Identity::NOVEL,
                }),
                proc_macro2::TokenTree::Literal(literal) => TokenTree::Literal(Literal {
                    fallback: literal,
                    span: Span::call_site(),
                    identity: Identity::NOVEL,
                }),
            }
        }

        Ok(convert_token_stream(fallback))
    }
}

fn display_tokens(tokens: &[TokenTree], formatter: &mut fmt::Formatter) -> fmt::Result {
    let mut joint = false;
    for (i, token) in tokens.iter().enumerate() {
        if i != 0 && !joint {
            write!(formatter, " ")?;
        }
        joint = false;
        match token {
            TokenTree::Group(group) => Display::fmt(group, formatter),
            TokenTree::Ident(ident) => Display::fmt(ident, formatter),
            TokenTree::Punct(punct) => {
                joint = punct.spacing() == Spacing::Joint;
                Display::fmt(punct, formatter)
            }
            TokenTree::Literal(literal) => Display::fmt(literal, formatter),
        }?;
    }

    Ok(())
}

impl Display for TokenStream {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        display_tokens(&self.content, formatter)
    }
}

impl Debug for TokenStream {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("TokenStream ")?;
        formatter.debug_list().entries(&self.content).finish()
    }
}

pub struct LexError {
    fallback: proc_macro2::LexError,
    span: Span,
}

impl LexError {
    pub fn span(&self) -> Span {
        self.span
    }
}

impl Debug for LexError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.fallback, formatter)
    }
}

pub mod token_stream {
    use super::*;

    #[derive(Clone)]
    pub struct IntoIter {
        pub(crate) iter: <Vec<TokenTree> as IntoIterator>::IntoIter,
    }

    impl Iterator for IntoIter {
        type Item = TokenTree;

        fn next(&mut self) -> Option<Self::Item> {
            self.iter.next()
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            self.iter.size_hint()
        }
    }
}

pub mod extra {
    use crate::Span;

    #[derive(Copy, Clone)]
    pub struct DelimSpan {
        pub(crate) join: Span,
        pub(crate) open: Span,
        pub(crate) close: Span,
    }

    impl DelimSpan {
        pub fn join(&self) -> Span {
            self.join
        }

        pub fn open(&self) -> Span {
            self.open
        }

        pub fn close(&self) -> Span {
            self.close
        }
    }
}
