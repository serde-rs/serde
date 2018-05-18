extern crate version_check;

fn main() {
    match version_check::is_min_version("1.26.0") {
        Some((true, _)) => {
            println!("cargo:rustc-cfg=integer128");
        },
        Some((false, _)) => {}
        None => {
            println!("could not figure out the rustc version");
        },
    };
}
