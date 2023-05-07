use crate::executable::{Cache, Error, Path, PathRoot, Rule, Visitor};
use bluejay_core::definition::{SchemaDefinition, TypeDefinitionReferenceFromAbstract};
use bluejay_core::executable::{
    AbstractOperationDefinition, ExecutableDocument, FragmentSpread, VariableDefinition,
};
use bluejay_core::{AbstractValue, AsIter, ObjectValue, Value, Variable};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::ops::Not;

pub struct AllVariablesUsed<'a, E: ExecutableDocument, S: SchemaDefinition> {
    fragment_references: HashMap<PathRoot<'a, E>, BTreeSet<&'a E::FragmentDefinition>>,
    variable_usages: HashMap<PathRoot<'a, E>, HashSet<&'a str>>,
    cache: &'a Cache<'a, E, S>,
    executable_document: &'a E,
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Visitor<'a, E, S>
    for AllVariablesUsed<'a, E, S>
{
    fn visit_variable_value(
        &mut self,
        value: &'a <E as ExecutableDocument>::Value<false>,
        _: &'a <S as SchemaDefinition>::InputTypeReference,
        path: &Path<'a, E>,
    ) {
        self.visit_value(value, *path.root());
    }

    fn visit_fragment_spread(
        &mut self,
        fragment_spread: &'a E::FragmentSpread,
        _: TypeDefinitionReferenceFromAbstract<'a, S::TypeDefinitionReference>,
        path: &Path<'a, E>,
    ) {
        if let Some(fragment_definition) = self.cache.fragment_definition(fragment_spread.name()) {
            self.fragment_references
                .entry(*path.root())
                .or_default()
                .insert(fragment_definition);
        }
    }
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> AllVariablesUsed<'a, E, S> {
    fn visit_value(
        &mut self,
        value: &'a <E as ExecutableDocument>::Value<false>,
        root: PathRoot<'a, E>,
    ) {
        match value.as_ref() {
            Value::Variable(v) => {
                self.variable_usages
                    .entry(root)
                    .or_default()
                    .insert(v.name());
            }
            Value::List(l) => l.iter().for_each(|value| self.visit_value(value, root)),
            Value::Object(o) => o
                .iter()
                .for_each(|(_, value)| self.visit_value(value, root)),
            _ => {}
        }
    }

    fn fragment_usages(
        &self,
        operation_definition: &'a E::OperationDefinition,
    ) -> impl Iterator<Item = &'a E::FragmentDefinition> {
        let mut references = HashSet::new();
        self.visit_fragment_references(&PathRoot::Operation(operation_definition), &mut references);
        references.into_iter()
    }

    fn visit_fragment_references(
        &self,
        executable_definition: &PathRoot<'a, E>,
        visited: &mut HashSet<&'a E::FragmentDefinition>,
    ) {
        if let Some(references) = self.fragment_references.get(executable_definition) {
            references.iter().for_each(|&reference| {
                if visited.insert(reference) {
                    self.visit_fragment_references(&PathRoot::Fragment(reference), visited);
                }
            });
        }
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> IntoIterator
    for AllVariablesUsed<'a, E, S>
{
    type Item = Error<'a, E, S>;
    type IntoIter = std::vec::IntoIter<Error<'a, E, S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.executable_document
            .operation_definitions()
            .iter()
            .filter(|operation_definition| {
                operation_definition
                    .as_ref()
                    .variable_definitions()
                    .map_or(false, |variable_definitions| {
                        !variable_definitions.is_empty()
                    })
            })
            .flat_map(|operation_definition| {
                let variable_usages: HashSet<&'a str> = self
                    .fragment_usages(operation_definition)
                    .map(PathRoot::Fragment)
                    .chain(std::iter::once(PathRoot::Operation(operation_definition)))
                    .flat_map(|executable_definition| {
                        self.variable_usages
                            .get(&executable_definition)
                            .into_iter()
                            .flatten()
                            .copied()
                    })
                    .collect();

                operation_definition
                    .as_ref()
                    .variable_definitions()
                    .map(move |variable_definitions| {
                        variable_definitions
                            .iter()
                            .filter_map(move |variable_definition| {
                                variable_usages
                                    .contains(variable_definition.variable())
                                    .not()
                                    .then_some(Error::VariableDefinitionUnused {
                                        variable_definition,
                                    })
                            })
                    })
                    .into_iter()
                    .flatten()
            })
            .collect::<Vec<Error<'a, E, S>>>()
            .into_iter()
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for AllVariablesUsed<'a, E, S>
{
    fn new(executable_document: &'a E, _: &'a S, cache: &'a Cache<'a, E, S>) -> Self {
        Self {
            fragment_references: HashMap::new(),
            variable_usages: HashMap::new(),
            cache,
            executable_document,
        }
    }
}
