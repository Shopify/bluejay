use crate::executable::{Cache, Error, Path, PathRoot, Rule, Visitor};
use bluejay_core::definition::{SchemaDefinition, TypeDefinitionReferenceFromAbstract};
use bluejay_core::executable::{
    ExecutableDocument, FragmentSpread, OperationDefinition, VariableDefinition,
};
use bluejay_core::{Argument, AsIter, ObjectValue, Value, ValueReference, Variable};
use itertools::Either;
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub struct AllVariableUsesDefined<'a, E: ExecutableDocument, S: SchemaDefinition> {
    fragment_references: HashMap<&'a E::FragmentDefinition, BTreeSet<PathRoot<'a, E>>>,
    variable_usages:
        BTreeMap<PathRoot<'a, E>, Vec<&'a <E::Value<false> as Value<false>>::Variable>>,
    cache: &'a Cache<'a, E, S>,
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Visitor<'a, E, S>
    for AllVariableUsesDefined<'a, E, S>
{
    fn visit_variable_argument(
        &mut self,
        argument: &'a <E as ExecutableDocument>::Argument<false>,
        _: &'a <S as SchemaDefinition>::InputValueDefinition,
        path: &Path<'a, E>,
    ) {
        self.visit_value(argument.value(), *path.root());
    }

    fn visit_fragment_spread(
        &mut self,
        fragment_spread: &'a E::FragmentSpread,
        _: TypeDefinitionReferenceFromAbstract<'a, S::TypeDefinitionReference>,
        path: &Path<'a, E>,
    ) {
        if let Some(fragment_definition) = self.cache.fragment_definition(fragment_spread.name()) {
            self.fragment_references
                .entry(fragment_definition)
                .or_default()
                .insert(*path.root());
        }
    }
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> AllVariableUsesDefined<'a, E, S> {
    fn visit_value(
        &mut self,
        value: &'a <E as ExecutableDocument>::Value<false>,
        root: PathRoot<'a, E>,
    ) {
        match value.as_ref() {
            ValueReference::Variable(v) => {
                self.variable_usages.entry(root).or_default().push(v);
            }
            ValueReference::List(l) => l.iter().for_each(|value| self.visit_value(value, root)),
            ValueReference::Object(o) => o
                .iter()
                .for_each(|(_, value)| self.visit_value(value, root)),
            _ => {}
        }
    }

    fn operation_definitions_where_fragment_used(
        &self,
        fragment_definition: &'a E::FragmentDefinition,
    ) -> impl Iterator<Item = &'a E::OperationDefinition> {
        let mut references = BTreeSet::new();
        self.visit_fragment_references(fragment_definition, &mut references);
        references
            .into_iter()
            .filter_map(|reference| match reference {
                PathRoot::Operation(o) => Some(o),
                PathRoot::Fragment(_) => None,
            })
    }

    fn visit_fragment_references(
        &self,
        fragment_definition: &'a E::FragmentDefinition,
        visited: &mut BTreeSet<PathRoot<'a, E>>,
    ) {
        if let Some(references) = self.fragment_references.get(fragment_definition) {
            references.iter().for_each(|reference| {
                if visited.insert(*reference) {
                    if let PathRoot::Fragment(f) = reference {
                        self.visit_fragment_references(f, visited);
                    }
                }
            });
        }
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> IntoIterator
    for AllVariableUsesDefined<'a, E, S>
{
    type Item = Error<'a, E, S>;
    type IntoIter = std::vec::IntoIter<Error<'a, E, S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.variable_usages
            .iter()
            .filter(|(_, variables)| !variables.is_empty())
            .flat_map(|(root, variables)| {
                let operation_definitions: Either<std::iter::Once<&'a E::OperationDefinition>, _> =
                    match root {
                        PathRoot::Operation(operation_definition) => {
                            Either::Left(std::iter::once(operation_definition))
                        }
                        PathRoot::Fragment(fragment_definition) => Either::Right(
                            self.operation_definitions_where_fragment_used(fragment_definition),
                        ),
                    };
                operation_definitions.flat_map(|operation_definition| {
                    variables.iter().copied().filter_map(|variable| {
                        operation_definition
                            .as_ref()
                            .variable_definitions()
                            .map_or(true, |variable_definitions| {
                                variable_definitions.iter().all(|variable_definition| {
                                    variable_definition.variable() != variable.name()
                                })
                            })
                            .then_some(Error::VariableNotDefined {
                                variable,
                                operation_definition,
                            })
                    })
                })
            })
            .collect::<Vec<Error<'a, E, S>>>()
            .into_iter()
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for AllVariableUsesDefined<'a, E, S>
{
    fn new(_: &'a E, _: &'a S, cache: &'a Cache<'a, E, S>) -> Self {
        Self {
            fragment_references: HashMap::new(),
            variable_usages: BTreeMap::new(),
            cache,
        }
    }
}
