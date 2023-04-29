use crate::executable::{Cache, Error, Rule, Rules};
use bluejay_core::definition::{
    AbstractBaseOutputTypeReference, AbstractOutputTypeReference, ArgumentsDefinition,
    DirectiveDefinition, DirectiveLocation, FieldDefinition, FieldsDefinition,
    InputValueDefinition, InterfaceTypeDefinition, ObjectTypeDefinition, SchemaDefinition,
    TypeDefinitionReference, TypeDefinitionReferenceFromAbstract,
};
use bluejay_core::executable::{
    ExecutableDocument, Field, FragmentDefinition, FragmentSpread, InlineFragment,
    OperationDefinitionFromExecutableDocument, Selection, VariableDefinition,
};
use bluejay_core::{Argument, AsIter, Directive, OperationType};

pub struct Validator<'a, E: ExecutableDocument, S: SchemaDefinition, R: Rule<'a, E, S>> {
    schema_definition: &'a S,
    executable_document: &'a E,
    rule: R,
}

pub type RulesValidator<'a, E, S> = Validator<'a, E, S, Rules<'a, E, S>>;

impl<'a, E: ExecutableDocument, S: SchemaDefinition, R: Rule<'a, E, S>> Validator<'a, E, S, R> {
    fn new(
        executable_document: &'a E,
        schema_definition: &'a S,
        cache: &'a Cache<'a, E, S>,
    ) -> Self {
        Self {
            schema_definition,
            executable_document,
            rule: Rule::new(executable_document, schema_definition, cache),
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
        self.rule.visit_operation_definition(operation_definition);
        if let Some(directives) = operation_definition.directives() {
            self.visit_variable_directives(
                directives,
                operation_definition
                    .operation_type()
                    .associated_directive_location(),
            );
        }

        if let Some(variable_definitions) = operation_definition.variable_definitions() {
            self.visit_variable_definitions(variable_definitions);
        }

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
        self.visit_variable_directives(
            fragment_definition.directives(),
            DirectiveLocation::FragmentDefinition,
        );
        self.rule.visit_fragment_definition(fragment_definition);
    }

    fn visit_selection_set(
        &mut self,
        selection_set: &'a E::SelectionSet,
        scoped_type: TypeDefinitionReferenceFromAbstract<'a, S::TypeDefinitionReference>,
    ) {
        self.rule.visit_selection_set(selection_set, scoped_type);

        selection_set
            .as_ref()
            .iter()
            .for_each(|selection| match selection.as_ref() {
                Selection::Field(f) => {
                    let field_definition = Self::fields_definition(scoped_type)
                        .and_then(|fields_definition| fields_definition.get(f.name()));

                    if let Some(field_definition) = field_definition {
                        self.visit_field(f, field_definition);
                    }
                }
                Selection::InlineFragment(i) => self.visit_inline_fragment(i, scoped_type),
                Selection::FragmentSpread(fs) => self.visit_fragment_spread(fs, scoped_type),
            })
    }

    fn visit_field(&mut self, field: &'a E::Field, field_definition: &'a S::FieldDefinition) {
        self.rule.visit_field(field, field_definition);
        self.visit_variable_directives(field.directives(), DirectiveLocation::Field);

        if let Some((arguments, arguments_definition)) = field
            .arguments()
            .zip(field_definition.arguments_definition())
        {
            self.visit_variable_arguments(arguments, arguments_definition);
        }

        if let Some(selection_set) = field.selection_set() {
            if let Some(nested_type) = self
                .schema_definition
                .get_type_definition(field_definition.r#type().as_ref().base().as_ref().name())
            {
                self.visit_selection_set(selection_set, nested_type);
            }
        }
    }

    fn visit_variable_directives(
        &mut self,
        directives: &'a E::Directives<false>,
        location: DirectiveLocation,
    ) {
        self.rule.visit_variable_directives(directives, location);
        directives
            .iter()
            .for_each(|directive| self.visit_variable_directive(directive, location));
    }

    fn visit_const_directives(
        &mut self,
        directives: &'a E::Directives<true>,
        location: DirectiveLocation,
    ) {
        self.rule.visit_const_directives(directives, location);
        directives
            .iter()
            .for_each(|directive| self.visit_const_directive(directive, location));
    }

    fn visit_variable_directive(
        &mut self,
        directive: &'a E::Directive<false>,
        location: DirectiveLocation,
    ) {
        self.rule.visit_variable_directive(directive, location);
        if let Some(arguments) = directive.arguments() {
            if let Some(arguments_definition) = self
                .schema_definition
                .get_directive_definition(directive.name())
                .and_then(DirectiveDefinition::arguments_definition)
            {
                self.visit_variable_arguments(arguments, arguments_definition);
            }
        }
    }

    fn visit_const_directive(
        &mut self,
        directive: &'a E::Directive<true>,
        location: DirectiveLocation,
    ) {
        self.rule.visit_const_directive(directive, location);
        if let Some(arguments) = directive.arguments() {
            if let Some(arguments_definition) = self
                .schema_definition
                .get_directive_definition(directive.name())
                .and_then(DirectiveDefinition::arguments_definition)
            {
                self.visit_const_arguments(arguments, arguments_definition);
            }
        }
    }

    fn visit_inline_fragment(
        &mut self,
        inline_fragment: &'a E::InlineFragment,
        scoped_type: TypeDefinitionReferenceFromAbstract<'a, S::TypeDefinitionReference>,
    ) {
        self.visit_variable_directives(
            inline_fragment.directives(),
            DirectiveLocation::InlineFragment,
        );

        let fragment_type = if let Some(type_condition) = inline_fragment.type_condition() {
            self.schema_definition.get_type_definition(type_condition)
        } else {
            Some(scoped_type)
        };

        if let Some(fragment_type) = fragment_type {
            self.visit_selection_set(inline_fragment.selection_set(), fragment_type);
        }

        self.rule
            .visit_inline_fragment(inline_fragment, scoped_type);
    }

    fn visit_fragment_spread(
        &mut self,
        fragment_spread: &'a E::FragmentSpread,
        scoped_type: TypeDefinitionReferenceFromAbstract<'a, S::TypeDefinitionReference>,
    ) {
        self.visit_variable_directives(
            fragment_spread.directives(),
            DirectiveLocation::FragmentSpread,
        );
        self.rule
            .visit_fragment_spread(fragment_spread, scoped_type);
        // fragment will get checked when definition is visited
    }

    fn visit_variable_definitions(&mut self, variable_definitions: &'a E::VariableDefinitions) {
        self.rule.visit_variable_definitions(variable_definitions);
        variable_definitions.iter().for_each(|variable_definition| {
            self.visit_const_directives(
                variable_definition.directives(),
                DirectiveLocation::VariableDefinition,
            );
            self.rule.visit_variable_definition(variable_definition);
        });
    }

    fn visit_const_arguments(
        &mut self,
        arguments: &'a E::Arguments<true>,
        arguments_definition: &'a S::ArgumentsDefinition,
    ) {
        arguments.iter().for_each(|argument| {
            if let Some(ivd) = arguments_definition.get(argument.name()) {
                self.visit_const_value(argument.value(), ivd.r#type());
            }
        });
    }

    fn visit_variable_arguments(
        &mut self,
        arguments: &'a E::Arguments<false>,
        arguments_definition: &'a S::ArgumentsDefinition,
    ) {
        arguments.iter().for_each(|argument| {
            if let Some(ivd) = arguments_definition.get(argument.name()) {
                self.visit_variable_value(argument.value(), ivd.r#type());
            }
        });
    }

    fn visit_const_value(
        &mut self,
        value: &'a E::Value<true>,
        expected_type: &'a S::InputTypeReference,
    ) {
        self.rule.visit_const_value(value, expected_type);
    }

    fn visit_variable_value(
        &mut self,
        value: &'a E::Value<false>,
        expected_type: &'a S::InputTypeReference,
    ) {
        self.rule.visit_variable_value(value, expected_type);
    }

    pub fn validate(
        executable_document: &'a E,
        schema_definition: &'a S,
        cache: &'a Cache<'a, E, S>,
    ) -> <Self as IntoIterator>::IntoIter {
        let mut instance = Self::new(executable_document, schema_definition, cache);
        instance.visit();
        instance.into_iter()
    }

    fn fields_definition(
        t: TypeDefinitionReferenceFromAbstract<'a, S::TypeDefinitionReference>,
    ) -> Option<&'a S::FieldsDefinition> {
        match t {
            TypeDefinitionReference::BuiltinScalarType(_)
            | TypeDefinitionReference::CustomScalarType(_)
            | TypeDefinitionReference::EnumType(_)
            | TypeDefinitionReference::UnionType(_)
            | TypeDefinitionReference::InputObjectType(_) => None,
            TypeDefinitionReference::InterfaceType(itd) => Some(itd.fields_definition()),
            TypeDefinitionReference::ObjectType(otd) => Some(otd.fields_definition()),
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
