use crate::changes::Change;
use bluejay_core::definition::{HasDirectives, SchemaDefinition};
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

    pub fn diff(&self) -> Vec<Change<'a, S>> {
        let mut changes = Vec::new();

        changes.extend(
            self.argument_additions()
                .map(|arg| Change::DirectiveArgumentAdded {
                    directive: self.new_directive,
                    argument: arg,
                }),
        );

        changes.extend(
            self.argument_removals()
                .map(|arg| Change::DirectiveArgumentRemoved {
                    directive: self.old_directive,
                    argument: arg,
                }),
        );

        // diff common arguments
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
                }
            });

        changes
    }

    fn argument_additions(&self) -> impl Iterator<Item = &'a ArgumentForDirective<S>> {
        self.new_directive
            .arguments()
            .map(|ii| ii.iter())
            .into_iter()
            .flatten()
            .filter(|new_arg| {
                self.old_directive.arguments().is_none_or(|args| {
                    !args.iter().any(|old_arg| old_arg.name() == new_arg.name())
                })
            })
    }

    fn argument_removals(&self) -> impl Iterator<Item = &'a ArgumentForDirective<S>> {
        self.old_directive
            .arguments()
            .map(|ii| ii.iter())
            .into_iter()
            .flatten()
            .filter(|old_arg| {
                self.new_directive.arguments().is_some_and(|args| {
                    !args.iter().any(|new_arg| old_arg.name() == new_arg.name())
                })
            })
    }
}

pub fn directive_additions<
    'a,
    S: SchemaDefinition + 'a,
    T: HasDirectives<Directives = <S as SchemaDefinition>::Directives>,
>(
    old_member: &'a T,
    new_member: &'a T,
) -> impl Iterator<Item = &'a <S as SchemaDefinition>::Directive> {
    new_member
        .directives()
        .map(|ii| ii.iter())
        .into_iter()
        .flatten()
        .filter(|new_directive| {
            old_member.directives().is_none_or(|directives| {
                !directives
                    .iter()
                    .any(|old_directive| old_directive.name() == new_directive.name())
            })
        })
}

pub fn directive_removals<
    'a,
    S: SchemaDefinition + 'a,
    T: HasDirectives<Directives = <S as SchemaDefinition>::Directives>,
>(
    old_member: &'a T,
    new_member: &'a T,
) -> impl Iterator<Item = &'a <S as SchemaDefinition>::Directive> {
    old_member
        .directives()
        .map(|ii| ii.iter())
        .into_iter()
        .flatten()
        .filter(|old_directive| {
            new_member.directives().is_none_or(|directives| {
                !directives
                    .iter()
                    .any(|new_directive| old_directive.name() == new_directive.name())
            })
        })
}

pub fn common_directive_changes<
    'a,
    S: SchemaDefinition + 'a,
    T: HasDirectives<Directives = <S as SchemaDefinition>::Directives>,
>(
    old_member: &'a T,
    new_member: &'a T,
) -> impl Iterator<Item = Change<'a, S>> {
    old_member
        .directives()
        .map(|ii| ii.iter())
        .into_iter()
        .flatten()
        .filter_map(move |old_directive| {
            new_member
                .directives()
                .map(|ii| ii.iter())
                .into_iter()
                .flatten()
                .find(|new_directive| old_directive.name() == new_directive.name())
                .map(|new_directive| DirectiveDiff::new(old_directive, new_directive).diff())
        })
        .flat_map(|changes| changes.into_iter())
}
