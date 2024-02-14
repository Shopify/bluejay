use crate::executable::{
    document::{Path, Visitor},
    Cache,
};
use bluejay_core::definition::{DirectiveLocation, SchemaDefinition, TypeDefinitionReference};
use bluejay_core::executable::ExecutableDocument;
use std::marker::PhantomData;

pub trait Rule<'a, E: ExecutableDocument, S: SchemaDefinition>: Visitor<'a, E, S> {
    type Error;
    type Errors: Iterator<Item = Self::Error>;

    fn into_errors(self) -> Self::Errors;
}

macro_rules! impl_rule {
    ($n:literal) => {
        seq_macro::seq!(N in 0..$n {
            #[warn(clippy::missing_trait_methods)]
            impl<'a, E: ExecutableDocument, S: SchemaDefinition, ER, #(T~N: Rule<'a, E, S, Error = ER>,)*> Rule<'a, E, S> for (#(T~N,)*) {
                type Error = ER;
                type Errors = #(std::iter::Chain<)* std::iter::Empty<ER> #(, <T~N as Rule<'a, E, S>>::Errors>)*;

                fn into_errors(self) -> Self::Errors {
                    std::iter::empty() #(.chain(self.N.into_errors()))*
                }
            }
        });
    }
}

seq_macro::seq!(N in 2..=10 {
    impl_rule!(N);
});

impl_rule!(26);

pub struct RuleErrorAdapter<R, ER> {
    rule: R,
    error: PhantomData<ER>,
}

#[warn(clippy::missing_trait_methods)]
impl<'a, E: ExecutableDocument, S: SchemaDefinition, R: Rule<'a, E, S>, ER> Visitor<'a, E, S>
    for RuleErrorAdapter<R, ER>
{
    fn new(
        executable_document: &'a E,
        schema_definition: &'a S,
        cache: &'a Cache<'a, E, S>,
    ) -> Self {
        Self {
            rule: R::new(executable_document, schema_definition, cache),
            error: PhantomData,
        }
    }

    fn visit_operation_definition(&mut self, operation_definition: &'a E::OperationDefinition) {
        self.rule.visit_operation_definition(operation_definition);
    }

    fn visit_selection_set(
        &mut self,
        selection_set: &'a E::SelectionSet,
        r#type: TypeDefinitionReference<'a, S::TypeDefinition>,
    ) {
        self.rule.visit_selection_set(selection_set, r#type);
    }

    fn visit_field(
        &mut self,
        field: &'a E::Field,
        field_definition: &'a S::FieldDefinition,
        path: &Path<'a, E>,
    ) {
        self.rule.visit_field(field, field_definition, path);
    }

    fn visit_const_directive(
        &mut self,
        directive: &'a E::Directive<true>,
        location: DirectiveLocation,
    ) {
        self.rule.visit_const_directive(directive, location);
    }

    fn visit_variable_directive(
        &mut self,
        directive: &'a E::Directive<false>,
        location: DirectiveLocation,
    ) {
        self.rule.visit_variable_directive(directive, location);
    }

    fn visit_const_directives(
        &mut self,
        directives: &'a E::Directives<true>,
        location: DirectiveLocation,
    ) {
        self.rule.visit_const_directives(directives, location);
    }

    fn visit_variable_directives(
        &mut self,
        directives: &'a E::Directives<false>,
        location: DirectiveLocation,
    ) {
        self.rule.visit_variable_directives(directives, location);
    }

    fn visit_fragment_definition(&mut self, fragment_definition: &'a E::FragmentDefinition) {
        self.rule.visit_fragment_definition(fragment_definition);
    }

    fn visit_inline_fragment(
        &mut self,
        inline_fragment: &'a E::InlineFragment,
        scoped_type: TypeDefinitionReference<'a, S::TypeDefinition>,
    ) {
        self.rule
            .visit_inline_fragment(inline_fragment, scoped_type);
    }

    fn visit_fragment_spread(
        &mut self,
        fragment_spread: &'a E::FragmentSpread,
        scoped_type: TypeDefinitionReference<'a, S::TypeDefinition>,
        path: &Path<'a, E>,
    ) {
        self.rule
            .visit_fragment_spread(fragment_spread, scoped_type, path);
    }

    fn visit_const_argument(
        &mut self,
        argument: &'a E::Argument<true>,
        input_value_definition: &'a S::InputValueDefinition,
    ) {
        self.rule
            .visit_const_argument(argument, input_value_definition);
    }

    fn visit_variable_argument(
        &mut self,
        argument: &'a E::Argument<false>,
        input_value_definition: &'a S::InputValueDefinition,
        path: &Path<'a, E>,
    ) {
        self.rule
            .visit_variable_argument(argument, input_value_definition, path)
    }

    fn visit_variable_definition(&mut self, variable_definition: &'a E::VariableDefinition) {
        self.rule.visit_variable_definition(variable_definition);
    }

    fn visit_variable_definitions(&mut self, variable_definitions: &'a E::VariableDefinitions) {
        self.rule.visit_variable_definitions(variable_definitions);
    }
}

#[warn(clippy::missing_trait_methods)]
impl<
        'a,
        E: ExecutableDocument,
        S: SchemaDefinition,
        R: Rule<'a, E, S>,
        ER: From<<R as Rule<'a, E, S>>::Error>,
    > Rule<'a, E, S> for RuleErrorAdapter<R, ER>
{
    type Error = ER;
    type Errors =
        std::iter::Map<<R as Rule<'a, E, S>>::Errors, fn(<R as Rule<'a, E, S>>::Error) -> ER>;

    fn into_errors(self) -> Self::Errors {
        self.rule.into_errors().map(ER::from)
    }
}
