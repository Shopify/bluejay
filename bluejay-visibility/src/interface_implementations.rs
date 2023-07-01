use crate::{Cache, InterfaceImplementation, Warden};
use bluejay_core::definition::{self, SchemaDefinition};
use bluejay_core::AsIter;
use once_cell::unsync::OnceCell;

pub struct InterfaceImplementations<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S::InterfaceImplementations,
    cache: &'a Cache<'a, S, W>,
    interface_implementations: OnceCell<Vec<InterfaceImplementation<'a, S, W>>>,
}

impl<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> InterfaceImplementations<'a, S, W> {
    pub(crate) fn new(inner: &'a S::InterfaceImplementations, cache: &'a Cache<'a, S, W>) -> Self {
        Self {
            inner,
            cache,
            interface_implementations: OnceCell::new(),
        }
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> AsIter
    for InterfaceImplementations<'a, S, W>
{
    type Item = InterfaceImplementation<'a, S, W>;
    type Iterator<'b> = std::slice::Iter<'b, Self::Item> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.interface_implementations
            .get_or_init(|| {
                self.inner
                    .iter()
                    .filter_map(|ii| InterfaceImplementation::new(ii, self.cache))
                    .collect()
            })
            .iter()
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>>
    definition::InterfaceImplementations for InterfaceImplementations<'a, S, W>
{
    type InterfaceImplementation = InterfaceImplementation<'a, S, W>;
}
