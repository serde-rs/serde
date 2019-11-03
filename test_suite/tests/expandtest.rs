#[cfg(not(target_os = "emscripten"))]
#[rustversion::attr(not(nightly), ignore)]
#[cfg_attr(not(cargo_expand), ignore)]
#[test]
fn expandtest() {
    macrotest::expand("tests/expand/**/enum/*.rs");
    macrotest::expand("tests/expand/**/struct/*.rs");
}
