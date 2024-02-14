use crate::executable::{
    document::{BuiltinRules, Path, PathRoot, Rule, Visitor},
    Cache,
};
use bluejay_core::definition::{
    ArgumentsDefinition, DirectiveDefinition, DirectiveLocation, FieldDefinition, FieldsDefinition,
    ObjectTypeDefinition, OutputType, SchemaDefinition, TypeDefinitionReference,
};
use bluejay_core::executable::{
    ExecutableDocument, Field, FragmentDefinition, FragmentSpread, InlineFragment,
    OperationDefinition, Selection, SelectionReference, VariableDefinition,
};
use bluejay_core::{Argument, AsIter, Directive, OperationType};

pub struct Orchestrator<'a, E: ExecutableDocument, S: SchemaDefinition, V: Visitor<'a, E, S>> {
    schema_definition: &'a S,
    executable_document: &'a E,
    visitor: V,
}

pub type BuiltinRulesValidator<'a, E, S> = Orchestrator<'a, E, S, BuiltinRules<'a, E, S>>;

impl<'a, E: ExecutableDocument, S: SchemaDefinition, V: Visitor<'a, E, S>>
    Orchestrator<'a, E, S, V>
{
    fn new(
        executable_document: &'a E,
        schema_definition: &'a S,
        cache: &'a Cache<'a, E, S>,
    ) -> Self {
        Self {
            schema_definition,
            executable_document,
            visitor: Visitor::new(executable_document, schema_definition, cache),
        }
    }

    fn visit(&mut self) {
        self.executable_document
            .operation_definitions()
            .iter()
            .for_each(|operation_definition| {
                self.visit_operation_definition(operation_definition);
            });
        self.executable_document
            .fragment_definitions()
            .iter()
            .for_each(|fragment_definition| {
                self.visit_fragment_definition(fragment_definition);
            });
    }

    fn visit_operation_definition(&mut self, operation_definition: &'a E::OperationDefinition) {
        let path = Path::new(PathRoot::Operation(operation_definition));
        self.visitor
            .visit_operation_definition(operation_definition);
        let core_operation_definition = operation_definition.as_ref();
        if let Some(directives) = core_operation_definition.directives() {
            self.visit_variable_directives(
                directives,
                core_operation_definition
                    .operation_type()
                    .associated_directive_location(),
                &path,
            );
        }

        if let Some(variable_definitions) = core_operation_definition.variable_definitions() {
            self.visit_variable_definitions(variable_definitions);
        }

        let root_operation_type_definition_name = match core_operation_definition.operation_type() {
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
                core_operation_definition.selection_set(),
                self.schema_definition
                    .get_type_definition(root_operation_type_definition_name)
                    .unwrap_or_else(|| {
                        panic!(
                            "Schema definition's `get_type` method returned `None` for {} root",
                            core_operation_definition.operation_type()
                        )
                    }),
                &path,
            );
        }
    }

    fn visit_fragment_definition(&mut self, fragment_definition: &'a E::FragmentDefinition) {
        let path = Path::new(PathRoot::Fragment(fragment_definition));
        let type_condition = self
            .schema_definition
            .get_type_definition(fragment_definition.type_condition());
        if let Some(type_condition) = type_condition {
            self.visit_selection_set(fragment_definition.selection_set(), type_condition, &path);
        }
        self.visit_variable_directives(
            fragment_definition.directives(),
            DirectiveLocation::FragmentDefinition,
            &path,
        );
        self.visitor.visit_fragment_definition(fragment_definition);
    }

    fn visit_selection_set(
        &mut self,
        selection_set: &'a E::SelectionSet,
        scoped_type: TypeDefinitionReference<'a, S::TypeDefinition>,
        path: &Path<'a, E>,
    ) {
        self.visitor.visit_selection_set(selection_set, scoped_type);

        selection_set.iter().for_each(|selection| {
            let nested_path = path.with_selection(selection);
            match selection.as_ref() {
                SelectionReference::Field(f) => {
                    let field_definition = scoped_type
                        .fields_definition()
                        .and_then(|fields_definition| fields_definition.get(f.name()));

                    if let Some(field_definition) = field_definition {
                        self.visit_field(f, field_definition, &nested_path);
                    }
                }
                SelectionReference::InlineFragment(i) => {
                    self.visit_inline_fragment(i, scoped_type, &nested_path)
                }
                SelectionReference::FragmentSpread(fs) => {
                    self.visit_fragment_spread(fs, scoped_type, path)
                }
            }
        })
    }

    fn visit_field(
        &mut self,
        field: &'a E::Field,
        field_definition: &'a S::FieldDefinition,
        path: &Path<'a, E>,
    ) {
        self.visitor.visit_field(field, field_definition, path);
        self.visit_variable_directives(field.directives(), DirectiveLocation::Field, path);

        if let Some((arguments, arguments_definition)) = field
            .arguments()
            .zip(field_definition.arguments_definition())
        {
            self.visit_variable_arguments(arguments, arguments_definition, path);
        }

        if let Some(selection_set) = field.selection_set() {
            if let Some(nested_type) = self
                .schema_definition
                .get_type_definition(field_definition.r#type().base_name())
            {
                self.visit_selection_set(selection_set, nested_type, path);
            }
        }
    }

    fn visit_variable_directives(
        &mut self,
        directives: &'a E::Directives<false>,
        location: DirectiveLocation,
        path: &Path<'a, E>,
    ) {
        self.visitor.visit_variable_directives(directives, location);
        directives
            .iter()
            .for_each(|directive| self.visit_variable_directive(directive, location, path));
    }

    fn visit_const_directives(
        &mut self,
        directives: &'a E::Directives<true>,
        location: DirectiveLocation,
    ) {
        self.visitor.visit_const_directives(directives, location);
        directives
            .iter()
            .for_each(|directive| self.visit_const_directive(directive, location));
    }

    fn visit_variable_directive(
        &mut self,
        directive: &'a E::Directive<false>,
        location: DirectiveLocation,
        path: &Path<'a, E>,
    ) {
        self.visitor.visit_variable_directive(directive, location);
        if let Some(arguments) = directive.arguments() {
            if let Some(arguments_definition) = self
                .schema_definition
                .get_directive_definition(directive.name())
                .and_then(DirectiveDefinition::arguments_definition)
            {
                self.visit_variable_arguments(arguments, arguments_definition, path);
            }
        }
    }

    fn visit_const_directive(
        &mut self,
        directive: &'a E::Directive<true>,
        location: DirectiveLocation,
    ) {
        self.visitor.visit_const_directive(directive, location);
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
        scoped_type: TypeDefinitionReference<'a, S::TypeDefinition>,
        path: &Path<'a, E>,
    ) {
        self.visit_variable_directives(
            inline_fragment.directives(),
            DirectiveLocation::InlineFragment,
            path,
        );

        let fragment_type = if let Some(type_condition) = inline_fragment.type_condition() {
            self.schema_definition.get_type_definition(type_condition)
        } else {
            Some(scoped_type)
        };

        if let Some(fragment_type) = fragment_type {
            self.visit_selection_set(inline_fragment.selection_set(), fragment_type, path);
        }

        self.visitor
            .visit_inline_fragment(inline_fragment, scoped_type);
    }

    fn visit_fragment_spread(
        &mut self,
        fragment_spread: &'a E::FragmentSpread,
        scoped_type: TypeDefinitionReference<'a, S::TypeDefinition>,
        path: &Path<'a, E>,
    ) {
        self.visit_variable_directives(
            fragment_spread.directives(),
            DirectiveLocation::FragmentSpread,
            path,
        );
        self.visitor
            .visit_fragment_spread(fragment_spread, scoped_type, path);
        // fragment will get checked when definition is visited
    }

    fn visit_variable_definitions(&mut self, variable_definitions: &'a E::VariableDefinitions) {
        self.visitor
            .visit_variable_definitions(variable_definitions);
        variable_definitions.iter().for_each(|variable_definition| {
            self.visit_const_directives(
                variable_definition.directives(),
                DirectiveLocation::VariableDefinition,
            );
            self.visitor.visit_variable_definition(variable_definition);
        });
    }

    fn visit_const_arguments(
        &mut self,
        arguments: &'a E::Arguments<true>,
        arguments_definition: &'a S::ArgumentsDefinition,
    ) {
        arguments.iter().for_each(|argument| {
            if let Some(ivd) = arguments_definition.get(argument.name()) {
                self.visit_const_argument(argument, ivd);
            }
        });
    }

    fn visit_variable_arguments(
        &mut self,
        arguments: &'a E::Arguments<false>,
        arguments_definition: &'a S::ArgumentsDefinition,
        path: &Path<'a, E>,
    ) {
        arguments.iter().for_each(|argument| {
            if let Some(ivd) = arguments_definition.get(argument.name()) {
                self.visit_variable_argument(argument, ivd, path);
            }
        });
    }

    fn visit_const_argument(
        &mut self,
        argument: &'a E::Argument<true>,
        input_value_definition: &'a S::InputValueDefinition,
    ) {
        self.visitor
            .visit_const_argument(argument, input_value_definition);
    }

    fn visit_variable_argument(
        &mut self,
        argument: &'a E::Argument<false>,
        input_value_definition: &'a S::InputValueDefinition,
        path: &Path<'a, E>,
    ) {
        self.visitor
            .visit_variable_argument(argument, input_value_definition, path);
    }

    pub fn validate(
        executable_document: &'a E,
        schema_definition: &'a S,
        cache: &'a Cache<'a, E, S>,
    ) -> <V as IntoIterator>::IntoIter
    where
        V: Rule<'a, E, S>,
    {
        let mut instance = Self::new(executable_document, schema_definition, cache);
        instance.visit();
        instance.visitor.into_iter()
    }
}
