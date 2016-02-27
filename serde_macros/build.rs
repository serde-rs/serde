#[cfg(feature = "nightly-testing")]
mod inner {
    extern crate skeptic;

    pub fn main() {
        skeptic::generate_doc_tests(&["../README.md"]);
    }
}

#[cfg(not(feature = "nightly-testing"))]
mod inner {
    pub fn main() {}
}

fn main() {
    inner::main()
}
