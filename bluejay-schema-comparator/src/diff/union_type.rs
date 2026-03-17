use crate::changes::Change;
use crate::diff::directive::diff_directives_into;
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

    #[inline]
    pub fn diff_into(&self, changes: &mut Vec<Change<'a, S>>) {
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

        diff_directives_into::<S, _>(
            self.old_type,
            self.new_type,
            DirectiveLocation::Union,
            self.old_type.name(),
            changes,
        );
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
