use bluejay_core::{
    definition::{SchemaDefinition, UnionTypeDefinition},
    executable::ExecutableDocument,
};
use bluejay_parser::{
    ast::executable::ExecutableDocument as ParserExecutableDocument,
    error::{Annotation, Error as ParserError},
    HasSpan,
};

pub(crate) enum Error<'a, E: ExecutableDocument, S: SchemaDefinition> {
    InlineFragmentOnObject {
        inline_fragment: &'a E::InlineFragment,
    },
    InlineFragmentOnInterface {
        inline_fragment: &'a E::InlineFragment,
    },
    FragmentSpreadNotIsolated {
        selection_set: &'a E::SelectionSet,
        fragment_spread: &'a E::FragmentSpread,
    },
    NoTypenameSelectionOnUnion {
        selection_set: &'a E::SelectionSet,
    },
    InlineFragmentOnUnionDoesNotTargetMember {
        inline_fragment: &'a E::InlineFragment,
        union_type_definition: &'a S::UnionTypeDefinition,
    },
    NonUniqueInlineFragmentTypeConditions {
        type_condition: &'a str,
        selection_set: &'a E::SelectionSet,
        inline_fragments: Vec<&'a E::InlineFragment>,
    },
    FieldSelectionOnUnion {
        field: &'a E::Field,
    },
    FragmentSpreadOnInterfaceInvalidTarget {
        fragment_spread: &'a E::FragmentSpread,
    },
    FragmentAndOperationNamesClash {
        operation_definition: &'a E::OperationDefinition,
        fragment_definition: &'a E::FragmentDefinition,
    },
}

const MACRO_NAME: &str = "typegen";

impl<'a, S: SchemaDefinition> From<Error<'a, ParserExecutableDocument<'a>, S>> for ParserError {
    fn from(value: Error<'a, ParserExecutableDocument<'a>, S>) -> Self {
        match value {
            Error::FragmentSpreadNotIsolated {
                selection_set,
                fragment_spread,
            } => Self::new(
                format!(
                    "{MACRO_NAME} requires a fragment spread to be the only selection in the set"
                ),
                Some(Annotation::new(
                    "Selection set contains a fragment spread and other selections",
                    selection_set.span().clone(),
                )),
                vec![Annotation::new(
                    "Fragment spread",
                    fragment_spread.span().clone(),
                )],
            ),
            Error::InlineFragmentOnObject { inline_fragment } => Self::new(
                format!("{MACRO_NAME} does not allow inline fragments on objects"),
                Some(Annotation::new(
                    "Inline fragment on object type",
                    inline_fragment.span().clone(),
                )),
                Vec::new(),
            ),
            Error::InlineFragmentOnInterface { inline_fragment } => Self::new(
                format!("{MACRO_NAME} does not allow inline fragments on interfaces"),
                Some(Annotation::new(
                    "Inline fragment on interface type",
                    inline_fragment.span().clone(),
                )),
                Vec::new(),
            ),
            Error::NoTypenameSelectionOnUnion { selection_set } => Self::new(
                format!("{MACRO_NAME} requires unaliased selection of `__typename` on union types to properly deserialize, and for that to be the first in the selection set"),
                Some(Annotation::new(
                    "Selection set does not contain an unaliased `__typename` selection as the first selection",
                    selection_set.span().clone(),
                )),
                Vec::new(),
            ),
            Error::InlineFragmentOnUnionDoesNotTargetMember { inline_fragment, union_type_definition } => Self::new(
                format!("{MACRO_NAME} requires inline fragments on union to target a union member type"),
                Some(Annotation::new(
                    format!(
                        "{} is not a member type of {}",
                        inline_fragment.type_condition().map_or(union_type_definition.name(), |tc| tc.named_type().as_ref()),
                        union_type_definition.name(),
                    ),
                    inline_fragment.span().clone(),
                )),
                Vec::new(),
            ),
            Error::NonUniqueInlineFragmentTypeConditions { type_condition, selection_set, inline_fragments } => Self::new(
                format!("{MACRO_NAME} requires the inline fragments in a selection set have unique type conditions"),
                Some(Annotation::new(
                    format!("Selection set contains multiple inline fragments targeting {type_condition}"),
                    selection_set.span().clone(),
                )),
                inline_fragments.into_iter().map(|inline_fragment| Annotation::new(
                    format!("Inline fragment targeting {type_condition}"),
                    inline_fragment.span().clone(),
                )).collect(),
            ),
            Error::FieldSelectionOnUnion { field } => Self::new(
                format!("{MACRO_NAME} does not allow field selections directly on union types, with the exception of unaliased `__typename` as the first selection in the set"),
                Some(Annotation::new(
                    "Field selection on union type",
                    field.name().span().clone(),
                )),
                Vec::new(),
            ),
            Error::FragmentSpreadOnInterfaceInvalidTarget { fragment_spread } => Self::new(
                format!("{MACRO_NAME} requires fragment spreads on interfaces to target either the interface or one of the interfaces it implements"),
                Some(Annotation::new(
                    "Fragment spread on interface type",
                    fragment_spread.span().clone(),
                )),
                Vec::new(),
            ),
            Error::FragmentAndOperationNamesClash { operation_definition, fragment_definition } => Self::new(
                format!("{MACRO_NAME} requires fragment and operation names to be unique, but encountered a clash with name `{}`", fragment_definition.name().as_ref()),
                Some(Annotation::new(
                    "Fragment definition name collides with operation definition name",
                    fragment_definition.span().clone(),
                )),
                vec![
                    Annotation::new(
                        "Operation definition",
                        operation_definition.span().clone(),
                    ),
                ],
            ),
        }
    }
}
