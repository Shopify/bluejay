use crate::definition::SchemaDefinition;

pub trait Directive: crate::Directive<true> {
    type SchemaDefinition: SchemaDefinition;

    fn definition<'a>(
        &'a self,
        schema_definition: &'a Self::SchemaDefinition,
    ) -> &'a <Self::SchemaDefinition as SchemaDefinition>::DirectiveDefinition;
}

pub trait Directives:
    crate::Directives<true, Directive = <Self::SchemaDefinition as SchemaDefinition>::Directive>
{
    type SchemaDefinition: SchemaDefinition;
}
