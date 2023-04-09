use crate::executable::{Error, Rule, Visitor};
use bluejay_core::definition::{
    AbstractBaseOutputTypeReference, AbstractOutputTypeReference, FieldDefinition,
    FieldsDefinition, InterfaceTypeDefinition, ObjectTypeDefinition, OutputTypeReference,
    SchemaDefinition, TypeDefinitionReference, TypeDefinitionReferenceFromAbstract,
};
use bluejay_core::executable::{
    ExecutableDocument, Field, FragmentDefinition, FragmentSpread, InlineFragment, Selection,
};
use bluejay_core::{Argument, AsIter};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::ops::Not;

pub struct FieldSelectionMerging<'a, E: ExecutableDocument, S: SchemaDefinition> {
    indexed_fragment_definitions: HashMap<&'a str, &'a E::FragmentDefinition>,
    schema_definition: &'a S,
    cache: BTreeMap<&'a E::SelectionSet, Vec<Error<'a, E, S>>>,
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition> Visitor<'a, E, S>
    for FieldSelectionMerging<'a, E, S>
{
    fn visit_selection_set(
        &mut self,
        selection_set: &'a E::SelectionSet,
        r#type: TypeDefinitionReferenceFromAbstract<'a, S::TypeDefinitionReference>,
    ) {
        self.selection_set_valid(selection_set, r#type);
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition> FieldSelectionMerging<'a, E, S> {
    fn selection_set_valid(
        &mut self,
        selection_set: &'a E::SelectionSet,
        parent_type: TypeDefinitionReferenceFromAbstract<'a, S::TypeDefinitionReference>,
    ) -> bool {
        if let Some(errors) = self.cache.get(selection_set) {
            errors.is_empty()
        } else {
            self.cache.insert(selection_set, Vec::new());

            let grouped_fields = self.selection_set_contained_fields(selection_set, parent_type);

            let errors = self.fields_in_set_can_merge(grouped_fields, selection_set);

            let is_valid = errors.is_empty();

            self.cache.insert(selection_set, errors);

            is_valid
        }
    }

    fn fields_in_set_can_merge(
        &mut self,
        grouped_fields: HashMap<&'a str, Vec<FieldContext<'a, E, S>>>,
        selection_set: &'a E::SelectionSet,
    ) -> Vec<Error<'a, E, S>> {
        let mut errors = Vec::new();

        grouped_fields.values().for_each(|fields_for_name| {
            errors.append(&mut self.same_response_shape(fields_for_name, selection_set));
            errors.append(
                &mut self
                    .same_for_common_parents_by_name(fields_for_name.as_slice(), selection_set),
            );
        });

        errors
    }

    fn same_response_shape(
        &mut self,
        fields_for_name: &[FieldContext<'a, E, S>],
        selection_set: &'a E::SelectionSet,
    ) -> Vec<Error<'a, E, S>> {
        if fields_for_name.len() <= 1 {
            return Vec::new();
        }

        fields_for_name
            .split_first()
            .and_then(|(first, rest)| {
                let errors: Vec<_> = rest
                    .iter()
                    .filter_map(|other| {
                        Self::same_output_type_shape(first, other).not().then_some(
                            Error::FieldSelectionsDoNotMergeIncompatibleTypes {
                                selection_set,
                                field_a: first.field,
                                field_definition_a: first.field_definition,
                                field_b: other.field,
                                field_definition_b: other.field_definition,
                            },
                        )
                    })
                    .collect();

                errors.is_empty().not().then_some(errors)
            })
            .unwrap_or_else(|| {
                let nested_grouped_fields =
                    self.field_contexts_contained_fields(fields_for_name.iter());

                nested_grouped_fields
                    .values()
                    .flat_map(|nested_fields_for_name| {
                        self.same_response_shape(nested_fields_for_name, selection_set)
                    })
                    .collect()
            })
    }

    fn same_for_common_parents_by_name(
        &mut self,
        fields_for_name: &[FieldContext<'a, E, S>],
        selection_set: &'a E::SelectionSet,
    ) -> Vec<Error<'a, E, S>> {
        if fields_for_name.len() <= 1 {
            return Vec::new();
        }

        type Group<'a, 'b, E, S> = Vec<&'b FieldContext<'a, E, S>>;
        type ConcreteGroups<'a, 'b, E, S> = HashMap<&'a str, Group<'a, 'b, E, S>>;

        let (abstract_group, concrete_groups): (Group<'a, '_, E, S>, ConcreteGroups<'a, '_, E, S>) =
            fields_for_name.iter().fold(
                (Vec::new(), HashMap::new()),
                |(mut abstract_group, mut concrete_groups), field_context| {
                    match field_context.parent_type {
                        TypeDefinitionReference::ObjectType(otd) => concrete_groups
                            .entry(otd.name())
                            .or_default()
                            .push(field_context),
                        TypeDefinitionReference::InterfaceType(_) => {
                            abstract_group.push(field_context)
                        }
                        _ => {}
                    }
                    (abstract_group, concrete_groups)
                },
            );

        let groups = if concrete_groups.is_empty() {
            vec![abstract_group]
        } else {
            concrete_groups
                .into_values()
                .map(|mut group| {
                    group.extend(&abstract_group);
                    group
                })
                .collect()
        };

        groups
            .iter()
            .flat_map(|fields_for_common_parent| {
                fields_for_common_parent
                    .split_first()
                    .and_then(|(first, rest)| {
                        let errors: Vec<_> = rest
                            .iter()
                            .filter_map(|other| {
                                if first.field.name() != other.field.name() {
                                    Some(Error::FieldSelectionsDoNotMergeDifferingNames {
                                        selection_set,
                                        field_a: first.field,
                                        field_b: other.field,
                                    })
                                } else if !Self::arguments_equal(
                                    first.field.arguments(),
                                    other.field.arguments(),
                                ) {
                                    Some(Error::FieldSelectionsDoNotMergeDifferingArguments {
                                        selection_set,
                                        field_a: first.field,
                                        field_b: other.field,
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect();

                        errors.is_empty().not().then_some(errors)
                    })
                    .unwrap_or_else(|| {
                        let nested_grouped_fields = self.field_contexts_contained_fields(
                            fields_for_common_parent.iter().copied(),
                        );

                        nested_grouped_fields
                            .values()
                            .flat_map(|nested_fields_for_name| {
                                self.same_for_common_parents_by_name(
                                    nested_fields_for_name.as_slice(),
                                    selection_set,
                                )
                            })
                            .collect()
                    })
            })
            .collect()
    }

    fn selection_set_contained_fields(
        &mut self,
        selection_set: &'a E::SelectionSet,
        parent_type: TypeDefinitionReferenceFromAbstract<'a, S::TypeDefinitionReference>,
    ) -> HashMap<&'a str, Vec<FieldContext<'a, E, S>>> {
        let mut fields = HashMap::new();
        self.visit_selections_for_fields(
            selection_set.as_ref().iter(),
            &mut fields,
            parent_type,
            &HashSet::new(),
        );
        fields
    }

    fn field_contexts_contained_fields<'b>(
        &mut self,
        field_contexts: impl Iterator<Item = &'b FieldContext<'a, E, S>>,
    ) -> HashMap<&'a str, Vec<FieldContext<'a, E, S>>>
    where
        'a: 'b,
    {
        let mut fields = HashMap::new();
        field_contexts.for_each(|field_context| {
            if let Some(selection_set) = field_context.field.selection_set() {
                if let Some(parent_type) = self.schema_definition.get_type_definition(
                    field_context
                        .field_definition
                        .r#type()
                        .as_ref()
                        .base()
                        .as_ref()
                        .name(),
                ) {
                    if self.selection_set_valid(selection_set, parent_type) {
                        self.visit_selections_for_fields(
                            selection_set.as_ref().iter(),
                            &mut fields,
                            parent_type,
                            &field_context.parent_fragments,
                        );
                    }
                }
            }
        });
        fields
    }

    fn visit_selections_for_fields(
        &mut self,
        selections: impl Iterator<Item = &'a E::Selection>,
        fields: &mut HashMap<&'a str, Vec<FieldContext<'a, E, S>>>,
        parent_type: TypeDefinitionReferenceFromAbstract<'a, S::TypeDefinitionReference>,
        parent_fragments: &HashSet<&'a str>,
    ) {
        selections.for_each(|selection| match selection.as_ref() {
            Selection::Field(field) => {
                let fields_definition = match parent_type {
                    TypeDefinitionReference::ObjectType(otd) => Some(otd.fields_definition()),
                    TypeDefinitionReference::InterfaceType(itd) => Some(itd.fields_definition()),
                    TypeDefinitionReference::BuiltinScalarType(_)
                    | TypeDefinitionReference::CustomScalarType(_)
                    | TypeDefinitionReference::EnumType(_)
                    | TypeDefinitionReference::InputObjectType(_)
                    | TypeDefinitionReference::UnionType(_) => None,
                };
                if let Some(fields_definition) = fields_definition {
                    if let Some(field_definition) = fields_definition.get_field(field.name()) {
                        fields
                            .entry(field.response_name())
                            .or_default()
                            .push(FieldContext {
                                field,
                                field_definition,
                                parent_type: parent_type.to_owned(),
                                parent_fragments: parent_fragments.to_owned(),
                            });
                    }
                }
            }
            Selection::FragmentSpread(fs) => {
                let fragment_name = fs.name();
                if !parent_fragments.contains(fragment_name) {
                    if let Some(fragment_definition) = self
                        .indexed_fragment_definitions
                        .get(fragment_name)
                        .copied()
                    {
                        let type_condition = fragment_definition.type_condition();
                        if let Some(scoped_type) =
                            self.schema_definition.get_type_definition(type_condition)
                        {
                            let mut parent_fragments = parent_fragments.clone();
                            parent_fragments.insert(fragment_name);
                            if self.selection_set_valid(
                                fragment_definition.selection_set(),
                                parent_type,
                            ) {
                                self.visit_selections_for_fields(
                                    fragment_definition.selection_set().as_ref().iter(),
                                    fields,
                                    scoped_type,
                                    &parent_fragments,
                                );
                            }
                        }
                    }
                }
            }
            Selection::InlineFragment(i) => {
                let scoped_type = match i.type_condition() {
                    Some(type_condition) => {
                        self.schema_definition.get_type_definition(type_condition)
                    }
                    None => Some(parent_type),
                };
                if let Some(scoped_type) = scoped_type {
                    if self.selection_set_valid(i.selection_set(), scoped_type) {
                        self.visit_selections_for_fields(
                            i.selection_set().as_ref().iter(),
                            fields,
                            scoped_type,
                            parent_fragments,
                        );
                    }
                }
            }
        });
    }

    fn same_output_type_shape(
        field_context_a: &FieldContext<'a, E, S>,
        field_context_b: &FieldContext<'a, E, S>,
    ) -> bool {
        let mut type_a = field_context_a.field_definition.r#type().as_ref();
        let mut type_b = field_context_b.field_definition.r#type().as_ref();

        let (type_a, type_b) = loop {
            let type_a_non_null = type_a.is_required();
            let type_b_non_null = type_b.is_required();

            if type_a_non_null != type_b_non_null {
                return false;
            }

            let double_base = if let OutputTypeReference::Base(type_a_base, _) = &type_a {
                Some(type_a_base.as_ref())
            } else {
                None
            }
            .and_then(|type_a_base| {
                if let OutputTypeReference::Base(type_b_base, _) = &type_b {
                    Some((type_a_base, type_b_base.as_ref()))
                } else {
                    None
                }
            });

            if let Some(double_base) = double_base {
                break double_base;
            } else {
                let type_a_list = matches!(type_a, OutputTypeReference::List(_, _));
                let type_b_list = matches!(type_b, OutputTypeReference::List(_, _));

                if !type_a_list || !type_b_list {
                    return false;
                }

                if let OutputTypeReference::List(list, _) = type_a {
                    type_a = list.as_ref();
                }

                if let OutputTypeReference::List(list, _) = type_b {
                    type_b = list.as_ref();
                }
            }
        };

        if type_a.is_scalar_or_enum() || type_b.is_scalar_or_enum() {
            return type_a.name() == type_b.name();
        }

        true
    }

    fn arguments_equal(
        args_a: Option<&'a E::Arguments<false>>,
        args_b: Option<&'a E::Arguments<false>>,
    ) -> bool {
        let lhs: HashMap<&str, _> = args_a
            .map(|args| {
                HashMap::from_iter(
                    args.iter()
                        .map(|arg: &E::Argument<false>| (arg.name(), arg.value().as_ref())),
                )
            })
            .unwrap_or_default();
        let rhs: HashMap<&str, _> = args_b
            .map(|args| {
                HashMap::from_iter(
                    args.iter()
                        .map(|arg: &E::Argument<false>| (arg.name(), arg.value().as_ref())),
                )
            })
            .unwrap_or_default();
        lhs == rhs
    }
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> IntoIterator
    for FieldSelectionMerging<'a, E, S>
{
    type Item = Error<'a, E, S>;
    type IntoIter = std::iter::Flatten<
        std::collections::btree_map::IntoValues<&'a E::SelectionSet, Vec<Error<'a, E, S>>>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.cache.into_values().flatten()
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for FieldSelectionMerging<'a, E, S>
{
    fn new(executable_document: &'a E, schema_definition: &'a S) -> Self {
        Self {
            indexed_fragment_definitions: HashMap::from_iter(
                executable_document
                    .fragment_definitions()
                    .as_ref()
                    .iter()
                    .map(|fragment_definition| (fragment_definition.name(), fragment_definition)),
            ),
            schema_definition,
            cache: BTreeMap::new(),
        }
    }
}

struct FieldContext<'a, E: ExecutableDocument, S: SchemaDefinition> {
    field: &'a E::Field,
    field_definition: &'a S::FieldDefinition,
    parent_type: TypeDefinitionReferenceFromAbstract<'a, S::TypeDefinitionReference>,
    parent_fragments: HashSet<&'a str>,
}
