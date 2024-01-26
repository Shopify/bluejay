use crate::definition::{DirectiveDefinition, SchemaDefinition};

pub trait Directive: crate::Directive<true> {
    type DirectiveDefinition: DirectiveDefinition;

    fn definition<'a, S: SchemaDefinition<DirectiveDefinition = Self::DirectiveDefinition>>(
        &'a self,
        schema_definition: &'a S,
    ) -> &'a Self::DirectiveDefinition;
}

pub trait Directives: crate::Directives<true, Directive = <Self as Directives>::Directive> {
    type Directive: Directive;
}
