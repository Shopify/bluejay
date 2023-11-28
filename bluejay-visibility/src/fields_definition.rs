use crate::{Cache, FieldDefinition, Warden};
use bluejay_core::definition::{self, prelude::*, SchemaDefinition};
use bluejay_core::AsIter;
use elsa::FrozenMap;
use once_cell::unsync::OnceCell;
use std::rc::Rc;

type IndexedFieldsDefinition<'a, S, W> = FrozenMap<&'a str, Box<Rc<FieldDefinition<'a, S, W>>>>;

pub struct FieldsDefinition<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S::FieldsDefinition,
    cache: &'a Cache<'a, S, W>,
    fields_definition: OnceCell<Vec<Rc<FieldDefinition<'a, S, W>>>>,
    indexed_fields_definition: IndexedFieldsDefinition<'a, S, W>,
}

impl<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> FieldsDefinition<'a, S, W> {
    pub(crate) fn new(inner: &'a S::FieldsDefinition, cache: &'a Cache<'a, S, W>) -> Self {
        Self {
            inner,
            cache,
            fields_definition: OnceCell::new(),
            indexed_fields_definition: Default::default(),
        }
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> AsIter
    for FieldsDefinition<'a, S, W>
{
    type Item = FieldDefinition<'a, S, W>;
    type Iterator<'b> = std::iter::Map<std::slice::Iter<'b, Rc<Self::Item>>, fn(&'b Rc<Self::Item>) -> &'b Self::Item> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.fields_definition
            .get_or_init(|| {
                self.inner
                    .iter()
                    .filter(|fd| self.cache.warden().is_field_definition_visible(fd))
                    .filter_map(|fd| {
                        self.indexed_fields_definition
                            .get(fd.name())
                            .or_else(|| {
                                let scoped_fd = FieldDefinition::new(fd, self.cache).map(Rc::new);
                                if let Some(scoped_fd) = scoped_fd {
                                    Some(
                                        self.indexed_fields_definition
                                            .insert(fd.name(), Box::new(scoped_fd)),
                                    )
                                } else {
                                    None
                                }
                            })
                            .cloned()
                    })
                    .collect()
            })
            .iter()
            .map(std::ops::Deref::deref)
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> definition::FieldsDefinition
    for FieldsDefinition<'a, S, W>
{
    type FieldDefinition = FieldDefinition<'a, S, W>;

    fn get(&self, name: &str) -> Option<&Self::FieldDefinition> {
        self.indexed_fields_definition
            .get(name)
            .or_else(|| {
                if let Some((fd, fd_scoped)) = self
                    .inner
                    .iter()
                    .filter(|fd| fd.name() == name)
                    .find_map(|fd| {
                        FieldDefinition::new(fd, self.cache).map(|fd_scoped| (fd, fd_scoped))
                    })
                {
                    Some(
                        self.indexed_fields_definition
                            .insert(fd.name(), Box::new(Rc::new(fd_scoped))),
                    )
                } else {
                    None
                }
            })
            .map(std::ops::Deref::deref)
    }

    fn contains_field(&self, name: &str) -> bool {
        self.get(name).is_some()
    }
}
