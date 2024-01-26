use crate::changes::Change;
use crate::diff::directive::{common_directive_changes, directive_additions, directive_removals};
use bluejay_core::definition::{
    DirectiveLocation, SchemaDefinition, UnionMemberType, UnionMemberTypes, UnionTypeDefinition,
};
use bluejay_core::AsIter;

pub struct UnionTypeDiff<'a, S: SchemaDefinition> {
    old_type: &'a S::UnionTypeDefinition,
    new_type: &'a S::UnionTypeDefinition,
}

impl<'a, S: SchemaDefinition + 'a> UnionTypeDiff<'a, S> {
    pub fn new(old_type: &'a S::UnionTypeDefinition, new_type: &'a S::UnionTypeDefinition) -> Self {
        Self { old_type, new_type }
    }

    pub fn diff(&self) -> Vec<Change<'a, S>> {
        let mut changes = Vec::new();

        changes.extend(
            self.member_additions()
                .map(|union_member_type| Change::UnionMemberAdded {
                    union_type_definition: self.new_type,
                    union_member_type,
                }),
        );

        changes.extend(self.member_removals().map(|union_member_type| {
            Change::UnionMemberRemoved {
                union_type_definition: self.new_type,
                union_member_type,
            }
        }));

        changes.extend(
            directive_additions::<S, _>(self.old_type, self.new_type).map(|directive| {
                Change::DirectiveAdded {
                    directive,
                    location: DirectiveLocation::Union,
                    member_name: self.old_type.name(),
                }
            }),
        );

        changes.extend(
            directive_removals::<S, _>(self.old_type, self.new_type).map(|directive| {
                Change::DirectiveRemoved {
                    directive,
                    location: DirectiveLocation::Union,
                    member_name: self.old_type.name(),
                }
            }),
        );

        changes.extend(common_directive_changes(self.old_type, self.new_type));

        changes
    }

    fn member_removals(&self) -> impl Iterator<Item = &'a S::UnionMemberType> {
        self.old_type
            .union_member_types()
            .iter()
            .filter(|old_member_type| {
                !self
                    .new_type
                    .union_member_types()
                    .contains_type(old_member_type.name())
            })
    }

    fn member_additions(&self) -> impl Iterator<Item = &'a S::UnionMemberType> {
        self.new_type
            .union_member_types()
            .iter()
            .filter(|new_member_type| {
                !self
                    .old_type
                    .union_member_types()
                    .contains_type(new_member_type.name())
            })
    }
}
