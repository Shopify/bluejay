use crate::{Cache, Directive, Warden};
use bluejay_core::definition::{
    prelude::*, Directives as CoreDefinitionDirectives, SchemaDefinition,
};
use bluejay_core::{AsIter, Directives as CoreDirectives};
use once_cell::unsync::OnceCell;

pub struct Directives<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a <S as SchemaDefinition>::Directives,
    cache: &'a Cache<'a, S, W>,
    directives: OnceCell<Vec<Directive<'a, S, W>>>,
}

impl<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> Directives<'a, S, W> {
    pub(crate) fn new(
        inner: &'a <S as SchemaDefinition>::Directives,
        cache: &'a Cache<'a, S, W>,
    ) -> Self {
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
    type Item = Directive<'a, S, W>;
    type Iterator<'b> = std::slice::Iter<'b, Self::Item> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.directives
            .get_or_init(|| {
                let warden = self.cache.warden();
                self.inner
                    .iter()
                    .filter_map(|directive| {
                        warden
                            .is_directive_definition_visible(directive.definition())
                            .then(|| Directive::new(directive, self.cache))
                    })
                    .collect()
            })
            .iter()
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> CoreDirectives<true>
    for Directives<'a, S, W>
{
    type Directive = Directive<'a, S, W>;
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> CoreDefinitionDirectives
    for Directives<'a, S, W>
{
    type Directive = Directive<'a, S, W>;
}
