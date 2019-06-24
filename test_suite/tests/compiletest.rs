#[cfg(not(target_os = "emscripten"))]
#[rustc::attr(not(nightly), ignore)]
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/**/*.rs");
}
