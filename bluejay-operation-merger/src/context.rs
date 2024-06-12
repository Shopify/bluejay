use crate::{id::Id, IdGenerator};
use bluejay_core::executable::{ExecutableDocument, FragmentDefinition};

pub(crate) struct Context<'a, E: ExecutableDocument> {
    id_generator: IdGenerator,
    #[allow(dead_code)]
    executable_document: &'a E,
}

impl<'a, E: ExecutableDocument> Context<'a, E> {
    pub(crate) fn new(id_generator: IdGenerator, executable_document: &'a E) -> Self {
        Self {
            id_generator,
            executable_document,
        }
    }

    pub(crate) fn next_id(&self) -> Id {
        self.id_generator.next()
    }

    pub(crate) fn fragment_definition(&self, name: &str) -> Option<&'a E::FragmentDefinition> {
        self.executable_document
            .fragment_definitions()
            .find(|fd| fd.name() == name)
    }
}
