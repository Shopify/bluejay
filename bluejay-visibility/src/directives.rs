use crate::{Cache, Warden};
use bluejay_core::definition::SchemaDefinition;
use bluejay_core::{AsIter, Directive, Directives as CoreDirectives};
use once_cell::unsync::OnceCell;

pub struct Directives<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S::Directives,
    cache: &'a Cache<'a, S, W>,
    directives: OnceCell<Vec<&'a <S::Directives as CoreDirectives<true>>::Directive>>,
}

impl<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> Directives<'a, S, W> {
    pub(crate) fn new(inner: &'a S::Directives, cache: &'a Cache<'a, S, W>) -> Self {
        Self {
            inner,
            cache,
            directives: OnceCell::new(),
        }
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> AsIter
    for Directives<'a, S, W>
{
    type Item = <S::Directives as CoreDirectives<true>>::Directive;
    type Iterator<'b> = std::iter::Copied<std::slice::Iter<'b, &'b Self::Item>> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.directives
            .get_or_init(|| {
                self.inner
                    .iter()
                    .filter_map(|directive| {
                        self.cache
                            .get_directive_definition(directive.name())
                            .map(|_| directive)
                    })
                    .collect()
            })
            .iter()
            .copied()
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> CoreDirectives<true>
    for Directives<'a, S, W>
{
    type Directive = <S::Directives as CoreDirectives<true>>::Directive;
}
