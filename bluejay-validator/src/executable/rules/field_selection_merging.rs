use crate::executable::{Error, Rule, Visitor};
use bluejay_core::definition::{
    FieldDefinition, FieldsDefinition, InterfaceTypeDefinition, ObjectTypeDefinition,
    OutputTypeReference, SchemaDefinition, TypeDefinitionReference,
    TypeDefinitionReferenceFromAbstract,
};
use bluejay_core::executable::{
    ExecutableDocument, Field, FragmentDefinition, FragmentSpread, InlineFragment, Selection,
};
use bluejay_core::{Argument, AsIter};
use itertools::Itertools;
use std::collections::HashMap;

pub struct FieldSelectionMerging<'a, E: ExecutableDocument, S: SchemaDefinition> {
    executable_document: &'a E,
    schema_definition: &'a S,
    errors: Vec<Error<'a, E, S>>,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> Visitor<'a, E, S>
    for FieldSelectionMerging<'a, E, S>
{
    fn visit_selection_set(
        &mut self,
        selection_set: &'a E::SelectionSet,
        r#type: &'a TypeDefinitionReferenceFromAbstract<S::TypeDefinitionReference>,
    ) {
        if !self.fields_in_set_can_merge(selection_set.as_ref().iter(), r#type) {
            self.errors
                .push(Error::FieldSelectionsDoNotMerge { selection_set });
        }
    }
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> FieldSelectionMerging<'a, E, S> {
    fn fields_in_set_can_merge(
        &self,
        selection_set: impl Iterator<Item = &'a E::Selection>,
        scoped_type: &'a TypeDefinitionReferenceFromAbstract<S::TypeDefinitionReference>,
    ) -> bool {
        let mut groups: HashMap<&'a str, Vec<FieldContext<'a, E, S>>> = HashMap::new();
        self.group_fields(&mut groups, selection_set, scoped_type);

        groups.values().all(|fields_for_name| {
            fields_for_name
                .iter()
                .tuple_combinations()
                .all(|(field_a, field_b)| {
                    self.same_response_shape(field_a, field_b) && {
                        let parent_type_a = field_a.parent_type;
                        let parent_type_b = field_b.parent_type;

                        if parent_type_a.name() == parent_type_b.name()
                            || !matches!(parent_type_a, TypeDefinitionReference::ObjectType(_, _))
                            || !matches!(parent_type_b, TypeDefinitionReference::ObjectType(_, _))
                        {
                            if field_a.field.name() != field_b.field.name() {
                                return false;
                            }

                            if !Self::arguments_equal(
                                field_a.field.arguments(),
                                field_b.field.arguments(),
                            ) {
                                return false;
                            }

                            let merged_set = field_a
                                .field
                                .selection_set()
                                .map(AsRef::as_ref)
                                .unwrap_or_default()
                                .iter()
                                .chain(
                                    field_b
                                        .field
                                        .selection_set()
                                        .map(AsRef::as_ref)
                                        .unwrap_or_default()
                                        .iter(),
                                );

                            self.fields_in_set_can_merge(merged_set, parent_type_a)
                        } else {
                            true
                        }
                    }
                })
        })
    }

    fn group_fields(
        &self,
        groups: &mut HashMap<&'a str, Vec<FieldContext<'a, E, S>>>,
        selection_set: impl Iterator<Item = &'a E::Selection>,
        parent_type: &'a TypeDefinitionReferenceFromAbstract<S::TypeDefinitionReference>,
    ) {
        selection_set.for_each(|selection| match selection.as_ref() {
            Selection::Field(field) => {
                let fields_definition = match parent_type {
                    TypeDefinitionReference::ObjectType(otd, _) => {
                        Some(otd.as_ref().fields_definition())
                    }
                    TypeDefinitionReference::InterfaceType(itd, _) => {
                        Some(itd.as_ref().fields_definition())
                    }
                    TypeDefinitionReference::BuiltinScalarType(_)
                    | TypeDefinitionReference::CustomScalarType(_, _)
                    | TypeDefinitionReference::EnumType(_, _)
                    | TypeDefinitionReference::InputObjectType(_, _)
                    | TypeDefinitionReference::UnionType(_, _) => None,
                };
                if let Some(fields_definition) = fields_definition {
                    if let Some(field_definition) = fields_definition.get_field(field.name()) {
                        groups
                            .entry(field.response_name())
                            .or_default()
                            .push(FieldContext {
                                field,
                                field_definition,
                                parent_type,
                            });
                    }
                }
            }
            Selection::FragmentSpread(fs) => {
                let fragment_name = fs.name();
                let fragment_definition = self
                    .executable_document
                    .fragment_definitions()
                    .iter()
                    .find(|fd| fd.name() == fragment_name);
                if let Some(fragment_definition) = fragment_definition {
                    let type_condition = fragment_definition.type_condition();
                    if let Some(scoped_type) =
                        self.schema_definition.get_type_definition(type_condition)
                    {
                        self.group_fields(
                            groups,
                            fragment_definition.selection_set().as_ref().iter(),
                            scoped_type,
                        );
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
                    self.group_fields(groups, i.selection_set().as_ref().iter(), scoped_type);
                }
            }
        });
    }

    fn same_response_shape(
        &self,
        field_a: &FieldContext<'a, E, S>,
        field_b: &FieldContext<'a, E, S>,
    ) -> bool {
        let types_match = Self::types_match_for_same_response_shape(
            field_a.field_definition.r#type(),
            field_b.field_definition.r#type(),
        );

        if !types_match {
            return false;
        }

        let mut field_a_groups = {
            let mut groups: HashMap<&str, Vec<FieldContext<'a, E, S>>> = HashMap::new();

            if let Some(selection_set) = field_a.field.selection_set() {
                if let Some(scoped_type) = self
                    .schema_definition
                    .get_type_definition(field_a.field_definition.r#type().as_ref().base().name())
                {
                    self.group_fields(&mut groups, selection_set.as_ref().iter(), scoped_type);
                }
            }

            groups
        };

        let field_b_groups = {
            let mut groups: HashMap<&str, Vec<FieldContext<'a, E, S>>> = HashMap::new();

            if let Some(selection_set) = field_b.field.selection_set() {
                if let Some(scoped_type) = self
                    .schema_definition
                    .get_type_definition(field_b.field_definition.r#type().as_ref().base().name())
                {
                    self.group_fields(&mut groups, selection_set.as_ref().iter(), scoped_type);
                }
            }

            groups
        };

        field_b_groups.into_iter().for_each(|(key, mut value)| {
            field_a_groups.entry(key).or_default().append(&mut value);
        });

        let groups = field_a_groups;

        groups.values().all(|fields_for_name| {
            fields_for_name
                .iter()
                .tuple_combinations()
                .all(|(field_a, field_b)| self.same_response_shape(field_a, field_b) && todo!())
        })
    }

    fn types_match_for_same_response_shape(
        type_a: &S::OutputTypeReference,
        type_b: &S::OutputTypeReference,
    ) -> bool {
        let mut type_a = type_a.as_ref();
        let mut type_b = type_b.as_ref();

        let (type_a, type_b) = loop {
            let type_a_non_null = type_a.is_required();
            let type_b_non_null = type_b.is_required();

            if (type_a_non_null || type_b_non_null) && (!type_a_non_null || !type_b_non_null) {
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

        if type_a.as_ref().is_scalar_or_enum() || type_b.as_ref().is_scalar_or_enum() {
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
    type IntoIter = std::vec::IntoIter<Error<'a, E, S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> Rule<'a, E, S>
    for FieldSelectionMerging<'a, E, S>
{
    fn new(executable_document: &'a E, schema_definition: &'a S) -> Self {
        Self {
            executable_document,
            schema_definition,
            errors: Vec::new(),
        }
    }
}

struct FieldContext<'a, E: ExecutableDocument, S: SchemaDefinition> {
    field: &'a E::Field,
    field_definition: &'a S::FieldDefinition,
    parent_type: &'a TypeDefinitionReferenceFromAbstract<S::TypeDefinitionReference>,
}

#[cfg(test)]
mod tests {}
