#[test]
fn compile_errors() {
    let t = trybuild::TestCases::new();
    t.compile_fail("test_files/unexpected-input.rs");
    t.compile_fail("test_files/unsupported-self-type.rs");
}
