use crate::executable::{
    document::{Error, Path, PathRoot, Rule, Visitor},
    Cache,
};
use bluejay_core::definition::{
    BaseInputTypeReference, HasDirectives, InputFieldsDefinition, InputObjectTypeDefinition,
    InputType, InputTypeReference, InputValueDefinition, SchemaDefinition, TypeDefinitionReference,
};
use bluejay_core::executable::{
    ExecutableDocument, FragmentSpread, OperationDefinition, VariableDefinition, VariableType,
    VariableTypeReference,
};
use bluejay_core::Directive;
use bluejay_core::{Argument, AsIter, Indexed, ObjectValue, Value, ValueReference, Variable};
use itertools::Either;
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub struct AllVariableUsagesAllowed<'a, E: ExecutableDocument, S: SchemaDefinition> {
    fragment_references: HashMap<Indexed<'a, E::FragmentDefinition>, BTreeSet<PathRoot<'a, E>>>,
    variable_usages: BTreeMap<PathRoot<'a, E>, Vec<VariableUsage<'a, E, S>>>,
    cache: &'a Cache<'a, E, S>,
    schema_definition: &'a S,
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Visitor<'a, E, S>
    for AllVariableUsagesAllowed<'a, E, S>
{
    fn new(_: &'a E, schema_definition: &'a S, cache: &'a Cache<'a, E, S>) -> Self {
        Self {
            fragment_references: HashMap::new(),
            variable_usages: BTreeMap::new(),
            cache,
            schema_definition,
        }
    }

    fn visit_variable_argument(
        &mut self,
        argument: &'a <E as ExecutableDocument>::Argument<false>,
        input_value_definition: &'a <S as SchemaDefinition>::InputValueDefinition,
        path: &Path<'a, E>,
    ) {
        self.visit_value(
            argument.value(),
            *path.root(),
            VariableUsageLocation::Argument(input_value_definition),
        );
    }

    fn visit_fragment_spread(
        &mut self,
        fragment_spread: &'a E::FragmentSpread,
        _: TypeDefinitionReference<'a, S::TypeDefinition>,
        path: &Path<'a, E>,
    ) {
        if let Some(fragment_definition) = self.cache.fragment_definition(fragment_spread.name()) {
            self.fragment_references
                .entry(Indexed(fragment_definition))
                .or_default()
                .insert(*path.root());
        }
    }
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> AllVariableUsagesAllowed<'a, E, S> {
    fn visit_value(
        &mut self,
        value: &'a <E as ExecutableDocument>::Value<false>,
        root: PathRoot<'a, E>,
        location: VariableUsageLocation<'a, S>,
    ) {
        match value.as_ref() {
            ValueReference::Variable(variable) => {
                self.variable_usages
                    .entry(root)
                    .or_default()
                    .push(VariableUsage { variable, location });
            }
            ValueReference::List(l) => l.iter().for_each(|value| {
                if let InputTypeReference::List(inner, _) =
                    location.r#type().as_ref(self.schema_definition)
                {
                    self.visit_value(value, root, VariableUsageLocation::ListValue(inner));
                }
            }),
            ValueReference::Object(o) => {
                if let BaseInputTypeReference::InputObject(iotd) =
                    location.r#type().base(self.schema_definition)
                {
                    o.iter().for_each(|(key, value)| {
                        if let Some(ivd) = iotd.input_field_definitions().get(key.as_ref()) {
                            self.visit_value(
                                value,
                                root,
                                VariableUsageLocation::ObjectField {
                                    input_value_definition: ivd,
                                    parent_type: iotd,
                                },
                            );
                        }
                    });
                }
            }
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
        if let Some(references) = self.fragment_references.get(&Indexed(fragment_definition)) {
            references.iter().for_each(|reference| {
                if visited.insert(*reference) {
                    if let PathRoot::Fragment(f) = reference {
                        self.visit_fragment_references(f, visited);
                    }
                }
            });
        }
    }

    fn validate_variable_usage(
        &self,
        variable_definition: &'a E::VariableDefinition,
        variable_usage: &VariableUsage<'a, E, S>,
    ) -> Result<(), Error<'a, E, S>> {
        let variable_type = variable_definition.r#type().as_ref();

        // filter non-input types to avoid duplicate error
        if !self.is_input_type(variable_type.name()) {
            return Ok(());
        }

        let VariableUsage { location, .. } = variable_usage;
        let location_type = location.r#type().as_ref(self.schema_definition);
        let input_value_definition = location.input_value_definition();
        let is_nested_one_of = location.is_nested_one_of();

        let is_compatible = if location_type.is_required() && !variable_type.is_required() {
            let has_non_null_variable_default_value =
                matches!(variable_definition.default_value(), Some(v) if !v.as_ref().is_null());
            let has_location_default_value = matches!(input_value_definition.and_then(InputValueDefinition::default_value), Some(v) if !v.as_ref().is_null());

            if !has_non_null_variable_default_value && !has_location_default_value {
                false
            } else {
                self.are_types_compatible(variable_type, location_type.unwrap_nullable())
            }
        } else {
            self.are_types_compatible(variable_type, location_type)
        };

        if !is_compatible {
            Err(Error::InvalidVariableUsage {
                variable: variable_usage.variable,
                variable_type: variable_definition.r#type(),
                location_type: location.r#type(),
            })
        } else if is_nested_one_of && !variable_type.is_required() {
            Err(Error::InvalidOneOfVariableUsage {
                variable: variable_usage.variable,
                variable_type: variable_definition.r#type(),
                parent_type_name: location.parent_type_name().unwrap(),
            })
        } else {
            Ok(())
        }
    }

    #[allow(clippy::only_used_in_recursion)] // making it a class method requires some additional lifetime constraints
    fn are_types_compatible(
        &self,
        variable_type: VariableTypeReference<'a, E::VariableType>,
        location_type: InputTypeReference<'a, S::InputType>,
    ) -> bool {
        match (variable_type, location_type) {
            (
                VariableTypeReference::List(item_variable_type, variable_required),
                InputTypeReference::List(item_location_type, location_required),
            ) if variable_required || !location_required => self.are_types_compatible(
                item_variable_type.as_ref(),
                item_location_type.as_ref(self.schema_definition),
            ),
            (
                VariableTypeReference::Named(base_variable_type, variable_required),
                InputTypeReference::Base(base_location_type, location_required),
            ) if variable_required || !location_required => {
                base_location_type.name() == base_variable_type
            }
            _ => false,
        }
    }

    fn is_input_type(&self, name: &str) -> bool {
        self.schema_definition
            .get_type_definition(name)
            .is_some_and(|tdr| tdr.is_input())
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for AllVariableUsagesAllowed<'a, E, S>
{
    type Error = Error<'a, E, S>;
    type Errors = std::vec::IntoIter<Error<'a, E, S>>;

    fn into_errors(self) -> Self::Errors {
        self.variable_usages
            .iter()
            .filter(|(_, variable_usages)| !variable_usages.is_empty())
            .flat_map(|(root, variable_usages)| {
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
                    variable_usages.iter().filter_map(|variable_usage| {
                        let VariableUsage { variable, .. } = variable_usage;
                        let variable_definition = operation_definition
                            .as_ref()
                            .variable_definitions()
                            .and_then(|variable_definitions| {
                                variable_definitions.iter().find(|variable_definition| {
                                    variable_definition.variable() == variable.name()
                                })
                            });

                        variable_definition.and_then(|variable_definition| {
                            self.validate_variable_usage(variable_definition, variable_usage)
                                .err()
                        })
                    })
                })
            })
            .collect::<Vec<Error<'a, E, S>>>()
            .into_iter()
    }
}

#[derive(Debug)]
enum VariableUsageLocation<'a, S: SchemaDefinition> {
    Argument(&'a S::InputValueDefinition),
    ObjectField {
        input_value_definition: &'a S::InputValueDefinition,
        parent_type: &'a S::InputObjectTypeDefinition,
    },
    ListValue(&'a S::InputType),
}

impl<'a, S: SchemaDefinition> VariableUsageLocation<'a, S> {
    fn input_value_definition(&self) -> Option<&'a S::InputValueDefinition> {
        match self {
            Self::Argument(ivd) => Some(ivd),
            Self::ObjectField {
                input_value_definition,
                ..
            } => Some(input_value_definition),
            Self::ListValue(_) => None,
        }
    }

    fn r#type(&self) -> &'a S::InputType {
        match self {
            Self::Argument(ivd) => ivd.r#type(),
            Self::ObjectField {
                input_value_definition,
                ..
            } => input_value_definition.r#type(),
            Self::ListValue(t) => t,
        }
    }

    fn is_nested_one_of(&self) -> bool {
        match self {
            Self::ObjectField { parent_type, .. } => parent_type
                .directives()
                .and_then(|d| d.iter().find(|d| d.name() == "oneOf"))
                .is_some(),
            _ => false,
        }
    }

    fn parent_type_name(&self) -> Option<&'a str> {
        match self {
            Self::ObjectField { parent_type, .. } => Some(parent_type.name()),
            _ => None,
        }
    }
}

struct VariableUsage<'a, E: ExecutableDocument, S: SchemaDefinition> {
    variable: &'a <E::Value<false> as Value<false>>::Variable,
    location: VariableUsageLocation<'a, S>,
}
