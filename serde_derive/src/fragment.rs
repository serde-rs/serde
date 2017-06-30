// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use quote::{Tokens, ToTokens};

pub enum Fragment {
    /// Tokens that can be used as an expression.
    Expr(Tokens),
    /// Tokens that can be used inside a block. The surrounding curly braces are
    /// not part of these tokens.
    Block(Tokens),
}

macro_rules! quote_expr {
    ($($tt:tt)*) => {
        $crate::fragment::Fragment::Expr(quote!($($tt)*))
    }
}

macro_rules! quote_block {
    ($($tt:tt)*) => {
        $crate::fragment::Fragment::Block(quote!($($tt)*))
    }
}

/// Interpolate a fragment in place of an expression. This involves surrounding
/// Block fragments in curly braces.
pub struct Expr(pub Fragment);
impl ToTokens for Expr {
    fn to_tokens(&self, out: &mut Tokens) {
        match self.0 {
            Fragment::Expr(ref expr) => expr.to_tokens(out),
            Fragment::Block(ref block) => {
                out.append("{");
                block.to_tokens(out);
                out.append("}");
            }
        }
    }
}

/// Interpolate a fragment as the statements of a block.
pub struct Stmts(pub Fragment);
impl ToTokens for Stmts {
    fn to_tokens(&self, out: &mut Tokens) {
        match self.0 {
            Fragment::Expr(ref expr) => expr.to_tokens(out),
            Fragment::Block(ref block) => block.to_tokens(out),
        }
    }
}

/// Interpolate a fragment as the value part of a `match` expression. This
/// involves putting a comma after expressions and curly braces around blocks.
pub struct Match(pub Fragment);
impl ToTokens for Match {
    fn to_tokens(&self, out: &mut Tokens) {
        match self.0 {
            Fragment::Expr(ref expr) => {
                expr.to_tokens(out);
                out.append(",");
            }
            Fragment::Block(ref block) => {
                out.append("{");
                block.to_tokens(out);
                out.append("}");
            }
        }
    }
}

use internals::attr::StrBoolInt as StrBoolIntInternal;

/// An extended name, used for the type tag for enums.
#[derive(Debug, Clone)]
pub enum StrBoolInt {
    Str(String),
    Bool(bool),
    Int(u64),
}

impl StrBoolInt {
    /// Get a string description for use in error messages.
    ///
    /// It will be used as second argument in
    /// `_serde::Serializer::serialize_struct(__serializer, name, len)`
    /// while serializing the inner struct in adjacently tagged enums.
    ///
    /// It will be used in the `VARIANTS` const array, that is given to
    /// `serde::de::Error::unknown_variant(variant, expected)`.
    /// and
    /// `_serde::Deserializer::deserialize_enum(name, variants, visitor)`
    ///
    pub fn stringify(&self) -> String {
        match *self {
            StrBoolInt::Str(ref s) => s.clone(),
            StrBoolInt::Bool(true) => "bool: true".to_owned(),
            StrBoolInt::Bool(false) => "bool: false".to_owned(),
            StrBoolInt::Int(i) => format!("integer: {}", i),
        }
    }
}

impl From<StrBoolIntInternal> for StrBoolInt {
    fn from(src: StrBoolIntInternal) -> StrBoolInt {
        match src {
            StrBoolIntInternal::Str(s) => StrBoolInt::Str(s),
            StrBoolIntInternal::Bool(b) => StrBoolInt::Bool(b),
            StrBoolIntInternal::Int(i) => StrBoolInt::Int(i),
        }
    }
}

impl ToTokens for StrBoolInt {
    fn to_tokens(&self, out: &mut Tokens) {
        match *self {
            StrBoolInt::Str(ref s) => s.to_tokens(out),
            StrBoolInt::Bool(b) => b.to_tokens(out),
            StrBoolInt::Int(i) => i.to_tokens(out),
        }
    }
}
