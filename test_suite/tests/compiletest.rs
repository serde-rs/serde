#[cfg_attr(target_os = "emscripten", ignore)]
#[rustversion::attr(not(nightly), ignore)]
#[cfg_attr(miri, ignore)]
#[allow(unused_attributes)]
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/**/*.rs");
}
