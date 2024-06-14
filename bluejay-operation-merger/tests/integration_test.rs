use bluejay_operation_merger::{ExecutableDocumentEntry, MergedExecutableDocument};
use bluejay_parser::ast::{executable::ExecutableDocument, Parse};
use bluejay_printer::executable::ExecutableDocumentPrinter;
use similar_asserts::assert_eq;
use std::collections::HashMap;

#[test]
fn integration_test() {
    let s1 = r#"
        query($x: Int = 1 @suffixOnMerge(contextKey: "myKey")) {
            foo {
                bar
                ... on AbstractType {
                    x(arg: $x) @suffixOnMerge(contextKey: "myKey")
                }
            }
        }
    "#;
    let s2 = r#"
        query($x: Int = 2 @suffixOnMerge(contextKey: "myOtherKey")) {
            foo {
                baz
                ...MyFragment
            }
        }

        fragment MyFragment on AbstractType {
            x(arg: $x) @suffixOnMerge(contextKey: "myOtherKey")
        }
    "#;

    let parsed1 = ExecutableDocument::parse(s1).unwrap();
    let parsed2 = ExecutableDocument::parse(s2).unwrap();

    let user_provided_context = HashMap::from([
        ("myKey".to_string(), "1".to_string()),
        ("myOtherKey".to_string(), "2".to_string()),
    ]);

    let entry1 = ExecutableDocumentEntry::new(&parsed1, &user_provided_context);
    let entry2 = ExecutableDocumentEntry::new(&parsed2, &user_provided_context);

    let merged = MergedExecutableDocument::new([entry1, entry2]).unwrap();

    let expected = r#"query($x1: Int = 1, $x2: Int = 2) {
  foo {
    bar
    ...on AbstractType {
      x1: x(arg: $x1)
      x2: x(arg: $x2)
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
