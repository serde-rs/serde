#[cfg_attr(target_os = "emscripten", ignore)]
#[rustversion::attr(not(nightly), ignore)]
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/**/*.rs");
}
