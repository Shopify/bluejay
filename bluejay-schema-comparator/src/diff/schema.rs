use crate::changes::Change;
use crate::diff::directive::{directive_additions, directive_removals};
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
        let mut changes: Vec<Change<'a, S>> = Vec::new();

        changes.extend(
            self.removed_types()
                .map(|removed_type_definition| Change::TypeRemoved {
                    removed_type_definition,
                }),
        );
        changes.extend(
            self.added_types()
                .map(|added_type_definition| Change::TypeAdded {
                    added_type_definition,
                }),
        );

        self.old_schema_definition
            .type_definitions()
            .for_each(|old_type| {
                let new_type = self
                    .new_schema_definition
                    .get_type_definition(old_type.name());

                if let Some(new_type) = new_type {
                    changes.extend(self.changes_in_type(old_type, new_type));
                }
            });

        changes.extend(
            self.removed_directive_definitions()
                .map(|directive_definition| Change::DirectiveDefinitionRemoved {
                    directive_definition,
                }),
        );
        changes.extend(
            self.added_directive_definitions()
                .map(|directive_definition| Change::DirectiveDefinitionAdded {
                    directive_definition,
                }),
        );

        changes.extend(
            directive_additions::<'a, S, _>(self.old_schema_definition, self.new_schema_definition)
                .map(|directive| Change::DirectiveAdded {
                    directive,
                    location: DirectiveLocation::Schema,
                    member_name: "",
                }),
        );

        changes.extend(
            directive_removals::<'a, S, _>(self.old_schema_definition, self.new_schema_definition)
                .map(|directive| Change::DirectiveRemoved {
                    directive,
                    location: DirectiveLocation::Schema,
                    member_name: "",
                }),
        );

        self.old_schema_definition
            .directive_definitions()
            .for_each(|old_directive| {
                let new_directive = self
                    .new_schema_definition
                    .get_directive_definition(old_directive.name());

                if let Some(new_directive) = new_directive {
                    changes
                        .extend(DirectiveDefinitionDiff::new(old_directive, new_directive).diff());
                }
            });

        changes
    }

    fn changes_in_type(
        &self,
        old_type: TypeDefinitionReference<'a, S::TypeDefinition>,
        new_type: TypeDefinitionReference<'a, S::TypeDefinition>,
    ) -> Vec<Change<'a, S>> {
        let mut changes: Vec<Change<'a, S>> = Vec::new();

        match (old_type, new_type) {
            (
                TypeDefinitionReference::Object(old_type),
                TypeDefinitionReference::Object(new_type),
            ) => {
                changes.extend(ObjectTypeDiff::new(old_type, new_type).diff());
            }
            (TypeDefinitionReference::Enum(old_type), TypeDefinitionReference::Enum(new_type)) => {
                changes.extend(EnumTypeDiff::new(old_type, new_type).diff());
            }
            (
                TypeDefinitionReference::Union(old_type),
                TypeDefinitionReference::Union(new_type),
            ) => {
                changes.extend(UnionTypeDiff::new(old_type, new_type).diff());
            }
            (
                TypeDefinitionReference::Interface(old_type),
                TypeDefinitionReference::Interface(new_type),
            ) => {
                changes.extend(InterfaceTypeDiff::new(old_type, new_type).diff());
            }
            (
                TypeDefinitionReference::InputObject(old_type),
                TypeDefinitionReference::InputObject(new_type),
            ) => {
                changes.extend(InputObjectTypeDiff::new(old_type, new_type).diff());
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

        changes
    }

    fn removed_types(
        &self,
    ) -> impl Iterator<Item = TypeDefinitionReference<'a, S::TypeDefinition>> {
        self.old_schema_definition
            .type_definitions()
            .filter(|old_type| {
                self.new_schema_definition
                    .get_type_definition(old_type.name())
                    .is_none()
            })
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

    fn removed_directive_definitions(&self) -> impl Iterator<Item = &'a S::DirectiveDefinition> {
        self.old_schema_definition
            .directive_definitions()
            .filter(|old_directive| {
                self.new_schema_definition
                    .get_directive_definition(old_directive.name())
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
