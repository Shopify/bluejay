use crate::changes::Change;
use crate::diff::directive::diff_directives_into;
use crate::diff::directive_definition::DirectiveDefinitionDiff;
use crate::diff::enum_type::EnumTypeDiff;
use crate::diff::input_object_type::InputObjectTypeDiff;
use crate::diff::interface_type::InterfaceTypeDiff;
use crate::diff::object_type::ObjectTypeDiff;
use crate::diff::union_type::UnionTypeDiff;
use bluejay_core::definition::{
    DirectiveDefinition as _, DirectiveLocation, SchemaDefinition, TypeDefinitionReference,
};

pub struct SchemaDiff<'a, S: SchemaDefinition> {
    old_schema_definition: &'a S,
    new_schema_definition: &'a S,
}

impl<'a, S: SchemaDefinition> SchemaDiff<'a, S> {
    pub fn new(old_schema: &'a S, new_schema: &'a S) -> Self {
        Self {
            old_schema_definition: old_schema,
            new_schema_definition: new_schema,
        }
    }

    pub fn diff(&self) -> Vec<Change<'a, S>> {
        let mut changes: Vec<Change<'a, S>> = Vec::with_capacity(256);

        // Type additions
        changes.extend(
            self.added_types()
                .map(|added_type_definition| Change::TypeAdded {
                    added_type_definition,
                }),
        );

        // Type removals + common type diffs in a single pass over old types
        self.old_schema_definition
            .type_definitions()
            .for_each(|old_type| {
                if let Some(new_type) = self
                    .new_schema_definition
                    .get_type_definition(old_type.name())
                {
                    self.changes_in_type(old_type, new_type, &mut changes);
                } else {
                    changes.push(Change::TypeRemoved {
                        removed_type_definition: old_type,
                    });
                }
            });

        // Directive definition additions
        changes.extend(
            self.added_directive_definitions()
                .map(|directive_definition| Change::DirectiveDefinitionAdded {
                    directive_definition,
                }),
        );

        // Directive definition removals + common diffs in a single pass
        self.old_schema_definition
            .directive_definitions()
            .for_each(|old_directive| {
                if let Some(new_directive) = self
                    .new_schema_definition
                    .get_directive_definition(old_directive.name())
                {
                    DirectiveDefinitionDiff::new(old_directive, new_directive)
                        .diff_into(&mut changes);
                } else {
                    changes.push(Change::DirectiveDefinitionRemoved {
                        directive_definition: old_directive,
                    });
                }
            });

        // Schema-level directive additions, removals, and changes
        diff_directives_into::<S, _>(
            self.old_schema_definition,
            self.new_schema_definition,
            DirectiveLocation::Schema,
            "",
            &mut changes,
        );

        changes
    }

    fn changes_in_type(
        &self,
        old_type: TypeDefinitionReference<'a, S::TypeDefinition>,
        new_type: TypeDefinitionReference<'a, S::TypeDefinition>,
        changes: &mut Vec<Change<'a, S>>,
    ) {
        match (old_type, new_type) {
            (
                TypeDefinitionReference::Object(old_type),
                TypeDefinitionReference::Object(new_type),
            ) => {
                ObjectTypeDiff::new(old_type, new_type).diff_into(changes);
            }
            (TypeDefinitionReference::Enum(old_type), TypeDefinitionReference::Enum(new_type)) => {
                EnumTypeDiff::new(old_type, new_type).diff_into(changes);
            }
            (
                TypeDefinitionReference::Union(old_type),
                TypeDefinitionReference::Union(new_type),
            ) => {
                UnionTypeDiff::new(old_type, new_type).diff_into(changes);
            }
            (
                TypeDefinitionReference::Interface(old_type),
                TypeDefinitionReference::Interface(new_type),
            ) => {
                InterfaceTypeDiff::new(old_type, new_type).diff_into(changes);
            }
            (
                TypeDefinitionReference::InputObject(old_type),
                TypeDefinitionReference::InputObject(new_type),
            ) => {
                InputObjectTypeDiff::new(old_type, new_type).diff_into(changes);
            }
            (
                TypeDefinitionReference::BuiltinScalar(_),
                TypeDefinitionReference::BuiltinScalar(_),
            )
            | (
                TypeDefinitionReference::CustomScalar(_),
                TypeDefinitionReference::CustomScalar(_),
            ) => {}
            _ => {
                changes.push(Change::TypeKindChanged {
                    old_type_definition: old_type,
                    new_type_definition: new_type,
                });
            }
        }

        if old_type.description() != new_type.description() {
            changes.push(Change::TypeDescriptionChanged {
                old_type_definition: old_type,
                new_type_definition: new_type,
            });
        }
    }

    fn added_types(&self) -> impl Iterator<Item = TypeDefinitionReference<'a, S::TypeDefinition>> {
        self.new_schema_definition
            .type_definitions()
            .filter(|new_type| {
                self.old_schema_definition
                    .get_type_definition(new_type.name())
                    .is_none()
            })
    }

    fn added_directive_definitions(&self) -> impl Iterator<Item = &'a S::DirectiveDefinition> {
        self.new_schema_definition
            .directive_definitions()
            .filter(|new_directive| {
                self.old_schema_definition
                    .get_directive_definition(new_directive.name())
                    .is_none()
            })
    }
}
