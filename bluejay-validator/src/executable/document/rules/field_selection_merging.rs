use crate::executable::{
    document::{Error, Rule, Visitor},
    Cache,
};
use bluejay_core::definition::{
    FieldDefinition, FieldsDefinition, ObjectTypeDefinition, OutputType, OutputTypeReference,
    SchemaDefinition, TypeDefinitionReference,
};
use bluejay_core::executable::{
    ExecutableDocument, Field, FragmentDefinition, FragmentSpread, InlineFragment, Selection,
    SelectionReference,
};
use bluejay_core::{Argument, AsIter, Indexed, Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::ops::Not;

pub struct FieldSelectionMerging<'a, E: ExecutableDocument, S: SchemaDefinition> {
    cache: &'a Cache<'a, E, S>,
    schema_definition: &'a S,
    cached_errors: BTreeMap<Indexed<'a, E::SelectionSet>, Vec<Error<'a, E, S>>>,
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition> Visitor<'a, E, S>
    for FieldSelectionMerging<'a, E, S>
{
    fn new(_: &'a E, schema_definition: &'a S, cache: &'a Cache<'a, E, S>) -> Self {
        Self {
            cache,
            schema_definition,
            cached_errors: BTreeMap::new(),
        }
    }

    fn visit_selection_set(
        &mut self,
        selection_set: &'a E::SelectionSet,
        r#type: TypeDefinitionReference<'a, S::TypeDefinition>,
    ) {
        self.selection_set_valid(selection_set, r#type);
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> FieldSelectionMerging<'a, E, S> {
    fn selection_set_valid(
        &mut self,
        selection_set: &'a E::SelectionSet,
        parent_type: TypeDefinitionReference<'a, S::TypeDefinition>,
    ) -> bool {
        if let Some(errors) = self.cached_errors.get(&Indexed(selection_set)) {
            errors.is_empty()
        } else {
            self.cached_errors
                .insert(Indexed(selection_set), Vec::new());

            let grouped_fields = self.selection_set_contained_fields(selection_set, parent_type);

            let errors = self.fields_in_set_can_merge(grouped_fields, selection_set);

            let is_valid = errors.is_empty();

            self.cached_errors.insert(Indexed(selection_set), errors);

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
                        Self::same_output_type_shape(
                            self.schema_definition,
                            first.field_definition.r#type(),
                            other.field_definition.r#type(),
                        )
                        .not()
                        .then_some(
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
                        TypeDefinitionReference::Object(otd) => concrete_groups
                            .entry(otd.name())
                            .or_default()
                            .push(field_context),
                        TypeDefinitionReference::Interface(_) => abstract_group.push(field_context),
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
        parent_type: TypeDefinitionReference<'a, S::TypeDefinition>,
    ) -> HashMap<&'a str, Vec<FieldContext<'a, E, S>>> {
        let mut fields = HashMap::new();
        self.visit_selections_for_fields(
            selection_set.iter(),
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
                if let Some(parent_type) = self
                    .schema_definition
                    .get_type_definition(field_context.field_definition.r#type().base_name())
                {
                    if self.selection_set_valid(selection_set, parent_type) {
                        self.visit_selections_for_fields(
                            selection_set.iter(),
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
        parent_type: TypeDefinitionReference<'a, S::TypeDefinition>,
        parent_fragments: &HashSet<&'a str>,
    ) {
        selections.for_each(|selection| match selection.as_ref() {
            SelectionReference::Field(field) => {
                let fields_definition = parent_type.fields_definition();
                if let Some(field_definition) = fields_definition
                    .and_then(|fields_definition| fields_definition.get(field.name()))
                {
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
            SelectionReference::FragmentSpread(fs) => {
                let fragment_name = fs.name();
                if !parent_fragments.contains(fragment_name) {
                    if let Some(fragment_definition) = self.cache.fragment_definition(fragment_name)
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
                                    fragment_definition.selection_set().iter(),
                                    fields,
                                    scoped_type,
                                    &parent_fragments,
                                );
                            }
                        }
                    }
                }
            }
            SelectionReference::InlineFragment(i) => {
                let scoped_type = match i.type_condition() {
                    Some(type_condition) => {
                        self.schema_definition.get_type_definition(type_condition)
                    }
                    None => Some(parent_type),
                };
                if let Some(scoped_type) = scoped_type {
                    if self.selection_set_valid(i.selection_set(), scoped_type) {
                        self.visit_selections_for_fields(
                            i.selection_set().iter(),
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
        schema_definition: &S,
        type_a: &S::OutputType,
        type_b: &S::OutputType,
    ) -> bool {
        match (
            type_a.as_ref(schema_definition),
            type_b.as_ref(schema_definition),
        ) {
            (
                OutputTypeReference::Base(type_a_base, type_a_required),
                OutputTypeReference::Base(type_b_base, type_b_required),
            ) if type_a_required == type_b_required => {
                !(type_a_base.is_scalar_or_enum() || type_b_base.is_scalar_or_enum())
                    || type_a_base.name() == type_b_base.name()
            }
            (
                OutputTypeReference::List(type_a_inner, type_a_required),
                OutputTypeReference::List(type_b_inner, type_b_required),
            ) if type_a_required == type_b_required => {
                Self::same_output_type_shape(schema_definition, type_a_inner, type_b_inner)
            }
            _ => false,
        }
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

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for FieldSelectionMerging<'a, E, S>
{
    type Error = Error<'a, E, S>;
    type Errors = std::iter::Flatten<
        std::collections::btree_map::IntoValues<Indexed<'a, E::SelectionSet>, Vec<Error<'a, E, S>>>,
    >;

    fn into_errors(self) -> Self::Errors {
        self.cached_errors.into_values().flatten()
    }
}

struct FieldContext<'a, E: ExecutableDocument, S: SchemaDefinition> {
    field: &'a E::Field,
    field_definition: &'a S::FieldDefinition,
    parent_type: TypeDefinitionReference<'a, S::TypeDefinition>,
    parent_fragments: HashSet<&'a str>,
}
