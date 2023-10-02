use crate::definition::DirectiveDefinition;

pub trait Directive: crate::Directive<true> {
    type DirectiveDefinition: DirectiveDefinition;

    fn definition(&self) -> &Self::DirectiveDefinition;
}

pub trait Directives: crate::Directives<true, Directive = <Self as Directives>::Directive> {
    type Directive: Directive;
}
