use crate::changes::Change;
use bluejay_core::definition::{DirectiveLocation, HasDirectives, SchemaDefinition};
use bluejay_core::{Argument, Arguments, AsIter, Directive, Value};

pub type ArgumentForDirective<S> =
    <<<S as SchemaDefinition>::Directive as Directive<true>>::Arguments as Arguments<
        true,
    >>::Argument;

pub struct DirectiveDiff<'a, S: SchemaDefinition> {
    old_directive: &'a <S as SchemaDefinition>::Directive,
    new_directive: &'a <S as SchemaDefinition>::Directive,
}

impl<'a, S: SchemaDefinition + 'a> DirectiveDiff<'a, S> {
    pub fn new(
        old_directive: &'a <S as SchemaDefinition>::Directive,
        new_directive: &'a <S as SchemaDefinition>::Directive,
    ) -> Self {
        Self {
            old_directive,
            new_directive,
        }
    }

    #[inline]
    pub fn diff_into(&self, changes: &mut Vec<Change<'a, S>>) {
        // Argument additions
        changes.extend(
            self.argument_additions()
                .map(|arg| Change::DirectiveArgumentAdded {
                    directive: self.new_directive,
                    argument: arg,
                }),
        );

        // Argument removals + common argument diffs in a single pass
        self.old_directive
            .arguments()
            .map(|ii| ii.iter())
            .into_iter()
            .flatten()
            .for_each(|old_arg| {
                let new_arg = self
                    .new_directive
                    .arguments()
                    .map(|ii| ii.iter())
                    .into_iter()
                    .flatten()
                    .find(|new_arg| old_arg.name() == new_arg.name());

                if let Some(new_arg) = new_arg {
                    if !(old_arg.value().as_ref() == new_arg.value().as_ref()) {
                        changes.push(Change::DirectiveArgumentValueChanged {
                            directive: self.old_directive,
                            old_argument: old_arg,
                            new_argument: new_arg,
                        });
                    }
                } else {
                    changes.push(Change::DirectiveArgumentRemoved {
                        directive: self.old_directive,
                        argument: old_arg,
                    });
                }
            });
    }

    fn argument_additions(&self) -> impl Iterator<Item = &'a ArgumentForDirective<S>> {
        self.new_directive
            .arguments()
            .map(|ii| ii.iter())
            .into_iter()
            .flatten()
            .filter(|new_arg| {
                self.old_directive
                    .arguments()
                    .is_none_or(|args| !args.iter().any(|old_arg| old_arg.name() == new_arg.name()))
            })
    }
}

/// Handles directive additions, removals, and common directive diffs.
/// Also emits DirectiveAdded for additions.
#[inline]
pub fn diff_directives_into<
    'a,
    S: SchemaDefinition + 'a,
    T: HasDirectives<Directives = <S as SchemaDefinition>::Directives>,
>(
    old_member: &'a T,
    new_member: &'a T,
    location: DirectiveLocation,
    member_name: &'a str,
    changes: &mut Vec<Change<'a, S>>,
) {
    let old_directives = old_member.directives();
    let new_directives = new_member.directives();

    // Fast path: if neither side has directives, nothing to do
    if old_directives.is_none() && new_directives.is_none() {
        return;
    }

    // Additions: new directives not in old
    new_directives
        .map(|ii| ii.iter())
        .into_iter()
        .flatten()
        .for_each(|new_directive| {
            let found_in_old = old_directives
                .map(|ii| ii.iter())
                .into_iter()
                .flatten()
                .any(|old_directive| old_directive.name() == new_directive.name());
            if !found_in_old {
                changes.push(Change::DirectiveAdded {
                    directive: new_directive,
                    location,
                    member_name,
                });
            }
        });

    // Removals + common diffs in a single pass over old directives
    old_directives
        .map(|ii| ii.iter())
        .into_iter()
        .flatten()
        .for_each(|old_directive| {
            if let Some(new_directive) = new_directives
                .map(|ii| ii.iter())
                .into_iter()
                .flatten()
                .find(|new_directive| old_directive.name() == new_directive.name())
            {
                DirectiveDiff::new(old_directive, new_directive).diff_into(changes);
            } else {
                changes.push(Change::DirectiveRemoved {
                    directive: old_directive,
                    location,
                    member_name,
                });
            }
        });
}
