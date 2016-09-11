use std::fmt::Display;
use std::cell::Cell;

#[derive(Default)]
pub struct Ctxt {
    err_count: Cell<usize>,
}

impl Ctxt {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn error<T: Display>(&self, msg: T) {
        println!("{}", msg);
        self.err_count.set(self.err_count.get() + 1);
    }
}

impl Drop for Ctxt {
    fn drop(&mut self) {
        let err_count = self.err_count.get();
        if err_count == 1 {
            panic!("1 error");
        } else if err_count > 1 {
            panic!("{} errors", err_count);
        }
    }
}
