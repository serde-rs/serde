#[cfg(not(target_os = "emscripten"))]
#[rustversion::attr(not(nightly), ignore)]
#[test]
fn expandtest() {
    macrotest::expand("tests/expand/**/enum/*.rs");
    macrotest::expand("tests/expand/**/struct/*.rs");
}
