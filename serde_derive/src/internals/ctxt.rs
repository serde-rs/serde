// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::cell::RefCell;
use std::fmt::Display;
use std::thread;

/// A type to collect errors together and format them.
///
/// Dropping this object will cause a panic. It must be consumed using `check`.
///
/// References can be shared since this type uses run-time exclusive mut checking.
#[derive(Default)]
pub struct Ctxt {
    // The contents will be set to `None` during checking. This is so that checking can be
    // enforced.
    errors: RefCell<Option<Vec<String>>>,
}

impl Ctxt {
    /// Create a new context object.
    ///
    /// This object contains no errors, but will still trigger a panic if it is not `check`ed.
    pub fn new() -> Self {
        Ctxt {
            errors: RefCell::new(Some(Vec::new())),
        }
    }

    /// Add an error to the context object.
    pub fn error<T: Display>(&self, msg: T) {
        self.errors
            .borrow_mut()
            .as_mut()
            .unwrap()
            .push(msg.to_string());
    }

    /// Consume this object, producing a formatted error string if there are errors.
    pub fn check(self) -> Result<(), String> {
        let mut errors = self.errors.borrow_mut().take().unwrap();
        match errors.len() {
            0 => Ok(()),
            1 => Err(errors.pop().unwrap()),
            n => {
                let mut msg = format!("{} errors:", n);
                for err in errors {
                    msg.push_str("\n\t# ");
                    msg.push_str(&err);
                }
                Err(msg)
            }
        }
    }
}

impl Drop for Ctxt {
    fn drop(&mut self) {
        if !thread::panicking() && self.errors.borrow().is_some() {
            panic!("forgot to check for errors");
        }
    }
}
