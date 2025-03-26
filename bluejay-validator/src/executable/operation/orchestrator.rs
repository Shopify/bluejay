use crate::executable::{
    operation::{Analyzer, OperationDefinitionValueEvaluationExt, VariableValues, Visitor},
    Cache,
};
use bluejay_core::{
    definition::{ArgumentsDefinition, DirectiveDefinition, DirectiveLocation},
    executable::{
        ExecutableDocument, Field, FragmentDefinition, FragmentSpread, InlineFragment,
        OperationDefinition, Selection, SelectionReference,
    },
};
use bluejay_core::{
    definition::{
        FieldDefinition, FieldsDefinition, ObjectTypeDefinition, OutputType, SchemaDefinition,
        TypeDefinitionReference,
    },
    ValueReference,
};
use bluejay_core::{Argument, AsIter, Directive, OperationType, Value};
use std::borrow::Cow;
use std::collections::HashSet;

pub struct Orchestrator<
    'a,
    E: ExecutableDocument,
    S: SchemaDefinition,
    VV: VariableValues,
    V: Visitor<'a, E, S, VV>,
> {
    schema_definition: &'a S,
    operation_definition: &'a E::OperationDefinition,
    variable_values: &'a VV,
    visitor: V,
    cache: &'a Cache<'a, E, S>,
    currently_spread_fragments: HashSet<&'a str>,
}

impl<
        'a,
        E: ExecutableDocument,
        S: SchemaDefinition,
        VV: VariableValues,
        V: Visitor<'a, E, S, VV>,
    > Orchestrator<'a, E, S, VV, V>
{
    const SKIP_DIRECTIVE_NAME: &'static str = "skip";
    const INCLUDE_DIRECTIVE_NAME: &'static str = "include";
    const SKIP_INCLUDE_CONDITION_ARGUMENT: &'static str = "if";

    fn new(
        operation_definition: &'a E::OperationDefinition,
        schema_definition: &'a S,
        variable_values: &'a VV,
        cache: &'a Cache<'a, E, S>,
        extra_info: V::ExtraInfo,
    ) -> Self {
        Self {
            schema_definition,
            operation_definition,
            variable_values,
            visitor: Visitor::new(
                operation_definition,
                schema_definition,
                variable_values,
                cache,
                extra_info,
            ),
            cache,
            currently_spread_fragments: HashSet::new(),
        }
    }

    fn visit(&mut self) {
        self.visit_operation_definition(self.operation_definition);
    }

    fn visit_variable_directives(
        &mut self,
        directives: &'a E::Directives<false>,
        location: DirectiveLocation,
    ) {
        directives
            .iter()
            .for_each(|directive| self.visit_variable_directive(directive, location));
    }

    fn visit_variable_directive(
        &mut self,
        directive: &'a E::Directive<false>,
        _location: DirectiveLocation,
    ) {
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

    fn visit_operation_definition(&mut self, operation_definition: &'a E::OperationDefinition) {
        let core_operation_definition = operation_definition.as_ref();

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

        if let Some(directives) = core_operation_definition.directives() {
            self.visit_variable_directives(
                directives,
                core_operation_definition
                    .operation_type()
                    .associated_directive_location(),
            )
        }

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
                true,
            );
        }

        if let Some(variable_definitions) = operation_definition.as_ref().variable_definitions() {
            variable_definitions.iter().for_each(|variable_definition| {
                self.visitor.visit_variable_definition(variable_definition)
            });
        }
    }

    fn visit_selection_set(
        &mut self,
        selection_set: &'a E::SelectionSet,
        scoped_type: TypeDefinitionReference<'a, S::TypeDefinition>,
        included: bool,
    ) {
        selection_set
            .iter()
            .for_each(|selection| match selection.as_ref() {
                SelectionReference::Field(f) => {
                    let field_definition = scoped_type
                        .fields_definition()
                        .and_then(|fields_definition| fields_definition.get(f.name()));

                    if let Some(field_definition) = field_definition {
                        self.visit_field(f, field_definition, scoped_type, included);
                    }
                }
                SelectionReference::InlineFragment(i) => {
                    self.visit_inline_fragment(i, scoped_type, included)
                }
                SelectionReference::FragmentSpread(fs) => self.visit_fragment_spread(fs, included),
            })
    }

    fn visit_field(
        &mut self,
        field: &'a E::Field,
        field_definition: &'a S::FieldDefinition,
        owner_type: TypeDefinitionReference<'a, S::TypeDefinition>,
        included: bool,
    ) {
        if let Some(directives) = field.directives() {
            self.visit_variable_directives(directives, DirectiveLocation::Field);
        }
        let included = included
            && field
                .directives()
                .is_none_or(|directives| self.evaluate_selection_inclusion(directives));

        self.visitor
            .visit_field(field, field_definition, owner_type, included);

        if let Some(arguments) = field.arguments() {
            if let Some(arguments_definition) = field_definition.arguments_definition() {
                arguments.iter().for_each(|argument| {
                    if let Some(ivd) = arguments_definition.get(argument.name()) {
                        self.visit_variable_argument(argument, ivd);
                    }
                })
            }
        }

        if let Some(selection_set) = field.selection_set() {
            if let Some(nested_type) = self
                .schema_definition
                .get_type_definition(field_definition.r#type().base_name())
            {
                self.visit_selection_set(selection_set, nested_type, included);
            }
        }

        self.visitor
            .leave_field(field, field_definition, owner_type, included);
    }

    fn visit_variable_arguments(
        &mut self,
        arguments: &'a E::Arguments<false>,
        arguments_definition: &'a S::ArgumentsDefinition,
    ) {
        arguments.iter().for_each(|argument| {
            if let Some(ivd) = arguments_definition.get(argument.name()) {
                self.visit_variable_argument(argument, ivd);
            }
        });
    }

    fn visit_variable_argument(
        &mut self,
        argument: &'a E::Argument<false>,
        input_value_definition: &'a S::InputValueDefinition,
    ) {
        self.visitor
            .visit_variable_argument(argument, input_value_definition);
    }

    fn visit_inline_fragment(
        &mut self,
        inline_fragment: &'a E::InlineFragment,
        scoped_type: TypeDefinitionReference<'a, S::TypeDefinition>,
        included: bool,
    ) {
        if let Some(directives) = inline_fragment.directives() {
            self.visit_variable_directives(directives, DirectiveLocation::InlineFragment);
        }

        let included = included
            && inline_fragment
                .directives()
                .is_none_or(|directives| self.evaluate_selection_inclusion(directives));

        let fragment_type = if let Some(type_condition) = inline_fragment.type_condition() {
            self.schema_definition.get_type_definition(type_condition)
        } else {
            Some(scoped_type)
        };

        if let Some(fragment_type) = fragment_type {
            self.visit_selection_set(inline_fragment.selection_set(), fragment_type, included);
        }
    }

    fn visit_fragment_spread(&mut self, fragment_spread: &'a E::FragmentSpread, included: bool) {
        if let Some(directives) = fragment_spread.directives() {
            self.visit_variable_directives(directives, DirectiveLocation::FragmentSpread);
        }

        let included = included
            && fragment_spread
                .directives()
                .is_none_or(|directives| self.evaluate_selection_inclusion(directives));
        if self
            .currently_spread_fragments
            .insert(fragment_spread.name())
        {
            if let Some(fragment_definition) =
                self.cache.fragment_definition(fragment_spread.name())
            {
                if let Some(type_condition) = self
                    .schema_definition
                    .get_type_definition(fragment_definition.type_condition())
                {
                    self.visit_selection_set(
                        fragment_definition.selection_set(),
                        type_condition,
                        included,
                    );
                }
            }
            self.currently_spread_fragments
                .remove(fragment_spread.name());
        }
    }

    fn evaluate_selection_inclusion(&mut self, directives: &'a E::Directives<false>) -> bool {
        let skip_directive_value = self.evaluate_boolean_directive_argument_value(
            directives,
            Self::SKIP_DIRECTIVE_NAME,
            Self::SKIP_INCLUDE_CONDITION_ARGUMENT,
        );

        let include_directive_value = self.evaluate_boolean_directive_argument_value(
            directives,
            Self::INCLUDE_DIRECTIVE_NAME,
            Self::SKIP_INCLUDE_CONDITION_ARGUMENT,
        );

        !matches!(
            (skip_directive_value, include_directive_value),
            (Some(true), _) | (_, Some(false))
        )
    }

    fn evaluate_boolean_directive_argument_value(
        &self,
        directives: &'a E::Directives<false>,
        directive_name: &str,
        arg_name: &str,
    ) -> Option<bool> {
        directives
            .iter()
            .find(|directive| directive.name() == directive_name)
            .and_then(|directive| {
                directive
                    .arguments()
                    .and_then(|arguments| arguments.iter().find(|arg| arg.name() == arg_name))
                    .and_then(|argument| match argument.value().as_ref() {
                        ValueReference::Boolean(val) => Some(val),
                        ValueReference::Variable(v) => self
                            .operation_definition
                            .evaluate_bool(v, self.variable_values),
                        _ => None,
                    })
            })
    }

    pub fn analyze<'b>(
        executable_document: &'a E,
        schema_definition: &'a S,
        operation_name: Option<&'b str>,
        variable_values: &'a VV,
        cache: &'a Cache<'a, E, S>,
        extra_info: V::ExtraInfo,
    ) -> Result<<V as Analyzer<'a, E, S, VV>>::Output, OperationResolutionError<'b>>
    where
        V: Analyzer<'a, E, S, VV>,
    {
        let operation_definition = match operation_name {
            Some(operation_name) => executable_document
                .operation_definitions()
                .find(|operation_definition| {
                    operation_definition.as_ref().name() == Some(operation_name)
                })
                .ok_or(OperationResolutionError::NoOperationWithName {
                    name: operation_name,
                })?,
            None => {
                let [operation_definition]: [&'a E::OperationDefinition; 1] = executable_document
                    .operation_definitions()
                    .collect::<Vec<_>>()
                    .as_slice()
                    .try_into()
                    .map_err(|_| OperationResolutionError::AnonymousNotEligible)?;
                operation_definition
            }
        };
        let mut instance = Self::new(
            operation_definition,
            schema_definition,
            variable_values,
            cache,
            extra_info,
        );
        instance.visit();
        Ok(instance.visitor.into_output())
    }
}

#[derive(Debug)]
pub enum OperationResolutionError<'a> {
    NoOperationWithName { name: &'a str },
    AnonymousNotEligible,
}

impl OperationResolutionError<'_> {
    pub fn message(&self) -> Cow<'static, str> {
        match self {
            Self::NoOperationWithName { name } => format!("No operation defined with name {}", name).into(),
            Self::AnonymousNotEligible => "Anonymous operation can only be used when the document contains exactly one operation definition".into(),
        }
    }
}
