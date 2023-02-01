use crate::definition::{
    AbstractBaseOutputTypeReference, FieldDefinition, FieldsDefinition, ObjectTypeDefinition,
    SchemaDefinition, TypeDefinitionReferenceFromAbstract,
};
use crate::executable::{
    ExecutableDocument, Field, FragmentDefinition, InlineFragment,
    OperationDefinitionFromExecutableDocument, Selection,
};
use crate::validation::executable::{Error, Rule, Rules, Visitor};

pub struct Validator<'a, E: ExecutableDocument<'a>, S: SchemaDefinition<'a>> {
    schema_definition: &'a S,
    executable_document: &'a E,
    rules: Rules<'a, E, S>,
}

impl<'a, E: ExecutableDocument<'a>, S: SchemaDefinition<'a>> Validator<'a, E, S> {
    fn new(executable_document: &'a E, schema_definition: &'a S) -> Self {
        Self {
            schema_definition,
            executable_document,
            rules: Rules::new(executable_document, schema_definition),
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
        operation_definition: &'a OperationDefinitionFromExecutableDocument<'a, E>,
    ) {
        self.rules.visit_operation(operation_definition);

        // TODO: handle mutation case
        self.visit_selection_set(
            operation_definition.selection_set(),
            self.schema_definition
                .get_type(self.schema_definition.query().name())
                .expect("Schema definition's `get_type` method returned `None` for query root"),
        )
    }

    fn visit_fragment_definition(&mut self, fragment_definition: &'a E::FragmentDefinition) {
        let type_condition = self
            .schema_definition
            .get_type(fragment_definition.type_condition());
        if let Some(type_condition) = type_condition {
            self.visit_selection_set(fragment_definition.selection_set(), type_condition);
        }
    }

    fn visit_selection_set(
        &mut self,
        selection_set: &'a E::SelectionSet,
        scoped_type: &'a TypeDefinitionReferenceFromAbstract<S::TypeDefinitionReference>,
    ) {
        self.rules.visit_selection_set(selection_set, scoped_type);

        selection_set.as_ref().iter().for_each(|selection| {
            let contained_selection_set_and_type = match selection.as_ref() {
                Selection::Field(f) => f.selection_set().and_then(|selection_set| {
                    self.schema_definition
                        .query()
                        .fields_definition()
                        .get_field(f.name())
                        .and_then(|fd| {
                            self.schema_definition
                                .get_type(fd.r#type().as_ref().base().name())
                        })
                        .map(|t| (selection_set, t))
                }),
                Selection::InlineFragment(i) => {
                    let t = if let Some(type_condition) = i.type_condition() {
                        self.schema_definition.get_type(type_condition)
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

    pub fn validate(
        executable_document: &'a E,
        schema_definition: &'a S,
    ) -> <Self as IntoIterator>::IntoIter {
        let mut instance = Self::new(executable_document, schema_definition);
        instance.visit();
        instance.into_iter()
    }
}

impl<'a, E: ExecutableDocument<'a>, S: SchemaDefinition<'a>> IntoIterator for Validator<'a, E, S> {
    type Item = Error<'a, E, S>;
    type IntoIter = <Rules<'a, E, S> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.rules.into_iter()
    }
}
