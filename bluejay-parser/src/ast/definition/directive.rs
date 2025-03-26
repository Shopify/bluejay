use crate::ast::{
    self,
    definition::{Context, DirectiveDefinition},
};
use crate::{HasSpan, Span};
use bluejay_core::definition::SchemaDefinition;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Directive<'a, C: Context + 'a> {
    inner: ast::Directive<'a, true>,
    context: PhantomData<C>,
}

impl<'a, C: Context> bluejay_core::Directive<true> for Directive<'a, C> {
    type Arguments = ast::Arguments<'a, true>;

    fn name(&self) -> &str {
        self.inner.name().as_str()
    }

    fn arguments(&self) -> Option<&Self::Arguments> {
        self.inner.arguments()
    }
}

impl<'a, C: Context> bluejay_core::definition::Directive for Directive<'a, C> {
    type DirectiveDefinition = DirectiveDefinition<'a, C>;

    fn definition<'b, S: SchemaDefinition<DirectiveDefinition = Self::DirectiveDefinition>>(
        &'b self,
        schema_definition: &'b S,
    ) -> &'b Self::DirectiveDefinition {
        schema_definition
            .get_directive_definition(self.inner.name().as_str())
            .unwrap()
    }
}

impl<'a, C: Context> From<ast::Directive<'a, true>> for Directive<'a, C> {
    fn from(value: ast::Directive<'a, true>) -> Self {
        Self {
            inner: value,
            context: PhantomData,
        }
    }
}

impl<C: Context> HasSpan for Directive<'_, C> {
    fn span(&self) -> &Span {
        self.inner.span()
    }
}

#[derive(Debug)]
pub struct Directives<'a, C: Context> {
    directives: Vec<Directive<'a, C>>,
}

impl<'a, C: Context> bluejay_core::AsIter for Directives<'a, C> {
    type Item = Directive<'a, C>;
    type Iterator<'b>
        = std::slice::Iter<'b, Self::Item>
    where
        'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.directives.iter()
    }
}

impl<'a, C: Context> bluejay_core::Directives<true> for Directives<'a, C> {
    type Directive = Directive<'a, C>;
}

impl<'a, C: Context> bluejay_core::definition::Directives for Directives<'a, C> {
    type Directive = Directive<'a, C>;
}

impl<'a, C: Context> From<ast::Directives<'a, true>> for Directives<'a, C> {
    fn from(value: ast::Directives<'a, true>) -> Self {
        Self {
            directives: value.into_iter().map(Directive::from).collect(),
        }
    }
}
