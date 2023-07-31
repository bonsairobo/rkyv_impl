#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("test_files/01-parse.rs");
    t.pass("test_files/02-call-simple-methods.rs");
    t.pass("test_files/03-generic-self-type.rs");
    t.compile_fail("test_files/04-unexpected-input.rs");
    t.pass("test_files/05-impl-trait.rs");
    t.pass("test_files/06-parse-external-path.rs");
}
