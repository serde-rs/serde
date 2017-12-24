// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt::Display;
use std::cell::RefCell;

#[derive(Default)]
pub struct Ctxt {
    errors: RefCell<Option<Vec<String>>>,
}

impl Ctxt {
    pub fn new() -> Self {
        Ctxt {
            errors: RefCell::new(Some(Vec::new())),
        }
    }

    pub fn error<T: Display>(&self, msg: T) {
        self.errors
            .borrow_mut()
            .as_mut()
            .unwrap()
            .push(msg.to_string());
    }

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
        if self.errors.borrow().is_some() {
            panic!("forgot to check for errors");
        }
    }
}
