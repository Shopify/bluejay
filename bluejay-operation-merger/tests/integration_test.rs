use bluejay_operation_merger::MergedExecutableDocument;
use bluejay_parser::ast::{executable::ExecutableDocument, Parse};
use bluejay_printer::executable::ExecutableDocumentPrinter;
use similar_asserts::assert_eq;

#[test]
fn integration_test() {
    let s1 = r#"
        query($x: Int = 1) {
            foo {
                bar
                ... on AbstractType {
                    x(arg: $x)
                }
            }
        }
    "#;
    let s2 = r#"
        query($y: Int = 2) {
            foo {
                baz
                ...MyFragment
            }
        }

        fragment MyFragment on AbstractType {
            y(arg: $y)
        }
    "#;

    let parsed1 = ExecutableDocument::parse(s1).unwrap();
    let parsed2 = ExecutableDocument::parse(s2).unwrap();

    let merged = MergedExecutableDocument::new([&parsed1, &parsed2]).unwrap();

    let expected = r#"query($x: Int = 1, $y: Int = 2) {
  foo {
    bar
    ...on AbstractType {
      x(arg: $x)
      y(arg: $y)
    }
    baz
  }
}
"#;

    assert_eq!(
        expected,
        ExecutableDocumentPrinter::new(&merged).to_string()
    );
}
