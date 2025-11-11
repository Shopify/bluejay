use bluejay_parser::ast::{
    definition::{DefinitionDocument, SchemaDefinition},
    Parse,
};
use bluejay_schema_comparator::compare;

#[test]
fn test_schema_compare() {
    let old_schema = r#"
        schema {
            query: Query
        }

        input AInput {
            # a
            a: String = "1"
            b: String!
            options: [Options]
        }

        # The Query Root of this schema
        type Query {
            # Just a simple string
            a(anArg: String): String!
            b: BType
            c(arg: Options): Options
            d: String
        }

        type BType {
            a: String
        }

        type CType {
            a: String @deprecated(reason: "whynot")
            c: Int!
            d(arg: Int): String
        }

        union MyUnion = CType | BType

        interface AnInterface {
            interfaceField: Int!
        }

        interface AnotherInterface {
            anotherInterfaceField: String
        }

        type WithInterfaces implements AnInterface & AnotherInterface {
            a: String!
        }

        type WithArguments {
            a(
                # Meh
                a: Int
                b: String
                option: Options
            ): String
            b(arg: Int = 1): String
        }

        enum Options {
            A
            B
            C
            E
            F @deprecated(reason: "Old")
        }

        # Old
        directive @yolo(
            # Included when true.
            someArg: Boolean!

            anotherArg: String!

            willBeRemoved: Boolean!
        ) on FIELD | FRAGMENT_SPREAD | INLINE_FRAGMENT

        type WillBeRemoved {
            a: String
        }

        directive @willBeRemoved on FIELD
    "#;

    let new_schema = r#"
        schema {
            query: Query
        }

        input AInput {
            # changed
            a: Int = 1
            c: String!
            options: [Options]
        }

        # Query Root description changed
        type Query {
            # This description has been changed
            a: String!
            b: Int!
            c(arg: Options): Options
            d: String @deprecated(reason: "removing soon")
        }

        input BType {
            a: String!
        }

        type CType implements AnInterface {
            a(arg: Int): String @deprecated(reason: "cuz")
            b: Int!
            d(arg: Int = 10): String
        }

        type DType {
            b: Int!
        }

        union MyUnion = CType | DType

        interface AnInterface {
            interfaceField: Int!
        }

        interface AnotherInterface {
            b: Int
        }

        type WithInterfaces implements AnInterface {
            a: String!
        }

        type WithArguments {
            a(
                # Description for a
                a: Int
                b: String!
                option: Options
            ): String
            b(arg: Int = 2): String
        }

        enum Options {
            # Stuff
            A
            B
            D
            E @deprecated
            F @deprecated(reason: "New")
        }

        # New
        directive @yolo(
            # someArg does stuff
            someArg: String!

            anotherArg: String! = "Test"
        ) on FIELD | FIELD_DEFINITION

        directive @yolo2(
            # Included when true.
            someArg: String!
        ) on FIELD
    "#;

    let document_a: DefinitionDocument = DefinitionDocument::parse(old_schema).result.unwrap();
    let document_b: DefinitionDocument = DefinitionDocument::parse(new_schema).result.unwrap();

    let schema_definition_a = SchemaDefinition::try_from(&document_a).unwrap();
    let schema_definition_b = SchemaDefinition::try_from(&document_b).unwrap();

    let result = compare(&schema_definition_a, &schema_definition_b);

    let mut expected_messages = vec![
        "Argument `anArg` was removed from field `Query.a`",
        "Argument `arg` was added to field `CType.a`",
        "Argument `willBeRemoved` was removed from directive `yolo`",
        "Default value `10` was added to argument `arg` on field `CType.d`",
        "Default value `\"Test\"` was added to directive argument `yolo.anotherArg`",
        "Default value for argument `arg` on field `WithArguments.b` was changed from `1` to `2`",
        "Directive `deprecated` was added to enum value `E`",
        "Directive `deprecated` was added to field `d`",
        "Directive `willBeRemoved` was removed",
        "Directive `yolo2` was added",
        "Enum value `C` was removed from enum `Options`",
        "Enum value `D` was added to enum `Options`",
        "Field `Query.b` changed type from `BType` to `Int!`.",
        "Field `anotherInterfaceField` was removed from object type `AnotherInterface`",
        "Field `b` was added to object type `AnotherInterface`",
        "Field `b` was added to object type `CType`",
        "Field `c` was removed from object type `CType`",
        "Input field `AInput.a` changed type from `String` to `Int`",
        "Input field `AInput.a` default value changed from `\"1\"` to `1`",
        "Input field `b` was removed from input object type `AInput`",
        "Input field `c` was added to input object type `AInput`",
        "Location `FIELD_DEFINITION` was added to directive `yolo`",
        "Location `FRAGMENT_SPREAD` was removed from directive `yolo`",
        "Location `INLINE_FRAGMENT` was removed from directive `yolo`",
        "Type `DType` was added",
        "Type `WillBeRemoved` was removed",
        "Type for argument `b` on field `a.WithArguments` changed from `String` to `String!`",
        "Type for argument `someArg` on directive `yolo` changed from `Boolean!` to `String!`",
        "Union member `BType` was removed from union type `MyUnion`",
        "Union member `DType` was added to union type `MyUnion`",
        "Value for argument `reason` on directive `deprecated` changed from \"Old\" to \"New\"",
        "Value for argument `reason` on directive `deprecated` changed from \"whynot\" to \"cuz\"",
        "`BType` kind changed from `OBJECT` to `INPUT_OBJECT`",
        "`CType` object implements `AnInterface` interface",
        "`WithInterfaces` object type no longer implements `AnotherInterface` interface",
    ];

    let mut change_messages: Vec<String> = result
        .changes
        .iter()
        .map(|change| change.message().to_string())
        .collect();

    expected_messages.sort();
    change_messages.sort();

    assert_eq!(expected_messages, change_messages);

    let mut expected_paths: Vec<&str> = vec![
        "@deprecated.reason",
        "@deprecated.reason",
        "@willBeRemoved",
        "@yolo",
        "@yolo",
        "@yolo",
        "@yolo2",
        "@yolo.anotherArg",
        "@yolo.someArg",
        "@yolo.willBeRemoved",
        "AInput.a",
        "AInput.a",
        "AInput.b",
        "AInput.c",
        "AnotherInterface.anotherInterfaceField",
        "AnotherInterface.b",
        "BType",
        "CType",
        "CType.a.arg",
        "CType.b",
        "CType.c",
        "CType.d.arg",
        "DType",
        "MyUnion",
        "MyUnion",
        "Options.C",
        "Options.D",
        "Query.a.anArg",
        "Query.b",
        "WillBeRemoved",
        "WithArguments.a.b",
        "WithArguments.b.arg",
        "WithInterfaces",
        "deprecated",
        "deprecated",
    ];

    let mut change_paths: Vec<String> = result.changes.iter().map(|change| change.path()).collect();

    expected_paths.sort();
    change_paths.sort();

    assert_eq!(expected_paths, change_paths);
}
