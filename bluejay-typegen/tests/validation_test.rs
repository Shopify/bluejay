#[test]
fn validation_test() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/validation_cases/error/*.rs");
    t.pass("tests/validation_cases/valid/*.rs");
}
