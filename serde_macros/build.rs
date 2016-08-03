#[cfg(feature = "unstable-testing")]
mod inner {
    extern crate skeptic;

    pub fn main() {
        println!("cargo:rerun-if-changed=../README.md");
        skeptic::generate_doc_tests(&["../README.md"]);
    }
}

#[cfg(not(feature = "unstable-testing"))]
mod inner {
    pub fn main() {}
}

fn main() {
    inner::main()
}
