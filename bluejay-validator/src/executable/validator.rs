use crate::executable::{Error, Rule, Rules};
use bluejay_core::definition::{
    FieldDefinition, FieldsDefinition, InterfaceTypeDefinition, ObjectTypeDefinition,
    SchemaDefinition, TypeDefinitionReference, TypeDefinitionReferenceFromAbstract,
};
use bluejay_core::executable::{
    ExecutableDocument, Field, FragmentDefinition, InlineFragment,
    OperationDefinitionFromExecutableDocument, Selection,
};
use bluejay_core::OperationType;

pub struct Validator<'a, E: ExecutableDocument, S: SchemaDefinition, R: Rule<'a, E, S>> {
    schema_definition: &'a S,
    executable_document: &'a E,
    rule: R,
}

pub type RulesValidator<'a, E, S> = Validator<'a, E, S, Rules<'a, E, S>>;

impl<'a, E: ExecutableDocument, S: SchemaDefinition, R: Rule<'a, E, S>> Validator<'a, E, S, R> {
    fn new(executable_document: &'a E, schema_definition: &'a S) -> Self {
        Self {
            schema_definition,
            executable_document,
            rule: Rule::new(executable_document, schema_definition),
        }
    }

    fn visit(&mut self) {
        self.executable_document
            .operation_definitions()
            .iter()
            .for_each(|operation_definition| {
                self.visit_operation_definition(operation_definition.as_ref());
            });
        self.executable_document
            .fragment_definitions()
            .iter()
            .for_each(|fragment_definition| {
                self.visit_fragment_definition(fragment_definition);
            });
    }

    fn visit_operation_definition(
        &mut self,
        operation_definition: &'a OperationDefinitionFromExecutableDocument<E>,
    ) {
        self.rule.visit_operation(operation_definition);

        let root_operation_type_definition_name = match operation_definition.operation_type() {
            OperationType::Query => Some(self.schema_definition.query().name()),
            OperationType::Mutation => self
                .schema_definition
                .mutation()
                .map(ObjectTypeDefinition::name),
            OperationType::Subscription => self
                .schema_definition
                .subscription()
                .map(ObjectTypeDefinition::name),
        };

        if let Some(root_operation_type_definition_name) = root_operation_type_definition_name {
            self.visit_selection_set(
                operation_definition.selection_set(),
                self.schema_definition
                    .get_type_definition(root_operation_type_definition_name)
                    .unwrap_or_else(|| {
                        panic!(
                            "Schema definition's `get_type` method returned `None` for {} root",
                            operation_definition.operation_type()
                        )
                    }),
            );
        }
    }

    fn visit_fragment_definition(&mut self, fragment_definition: &'a E::FragmentDefinition) {
        let type_condition = self
            .schema_definition
            .get_type_definition(fragment_definition.type_condition());
        if let Some(type_condition) = type_condition {
            self.visit_selection_set(fragment_definition.selection_set(), type_condition);
        }
    }

    fn visit_selection_set(
        &mut self,
        selection_set: &'a E::SelectionSet,
        scoped_type: &'a TypeDefinitionReferenceFromAbstract<S::TypeDefinitionReference>,
    ) {
        self.rule.visit_selection_set(selection_set, scoped_type);

        selection_set.as_ref().iter().for_each(|selection| {
            let contained_selection_set_and_type = match selection.as_ref() {
                Selection::Field(f) => {
                    let field_definition = Self::fields_definition(scoped_type)
                        .and_then(|fields_definition| fields_definition.get_field(f.name()));

                    if let Some(field_definition) = field_definition {
                        self.visit_field(f, field_definition.r#type());
                    }

                    f.selection_set().and_then(|selection_set| {
                        field_definition
                            .and_then(|fd| {
                                self.schema_definition
                                    .get_type_definition(fd.r#type().as_ref().base().name())
                            })
                            .map(|t| (selection_set, t))
                    })
                }
                Selection::InlineFragment(i) => {
                    let t = if let Some(type_condition) = i.type_condition() {
                        self.schema_definition.get_type_definition(type_condition)
                    } else {
                        Some(scoped_type)
                    };

                    t.map(|t| (i.selection_set(), t))
                }
                Selection::FragmentSpread(_) => None, // this will get checked when the fragment definitions are visited
            };

            if let Some((selection_set, t)) = contained_selection_set_and_type {
                self.visit_selection_set(selection_set, t);
            }
        })
    }

    fn visit_field(&mut self, field: &'a E::Field, scoped_type: &'a S::OutputTypeReference) {
        self.rule.visit_field(field, scoped_type);
    }

    pub fn validate(
        executable_document: &'a E,
        schema_definition: &'a S,
    ) -> <Self as IntoIterator>::IntoIter {
        let mut instance = Self::new(executable_document, schema_definition);
        instance.visit();
        instance.into_iter()
    }

    fn fields_definition(
        t: &'a TypeDefinitionReferenceFromAbstract<S::TypeDefinitionReference>,
    ) -> Option<&'a S::FieldsDefinition> {
        match t {
            TypeDefinitionReference::BuiltinScalarType(_)
            | TypeDefinitionReference::CustomScalarType(_, _)
            | TypeDefinitionReference::EnumType(_, _)
            | TypeDefinitionReference::UnionType(_, _)
            | TypeDefinitionReference::InputObjectType(_, _) => None,
            TypeDefinitionReference::InterfaceType(itd, _) => {
                Some(itd.as_ref().fields_definition())
            }
            TypeDefinitionReference::ObjectType(otd, _) => Some(otd.as_ref().fields_definition()),
        }
    }
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition, R: Rule<'a, E, S>> IntoIterator
    for Validator<'a, E, S, R>
{
    type Item = Error<'a, E, S>;
    type IntoIter = <R as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.rule.into_iter()
    }
}
