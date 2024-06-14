use crate::{
    Context, EmptyDirectives, Error, Id, MergedArguments, MergedField, MergedInlineFragment,
    MergedSelection,
};
use bluejay_core::{
    executable::{
        ExecutableDocument, Field, FragmentDefinition, FragmentSpread, InlineFragment, Selection,
        SelectionReference, SelectionSet,
    },
    Argument, Arguments, AsIter, Directive, Indexable, Value,
};
use std::borrow::Cow;

pub struct MergedSelectionSet<'a> {
    selections: Vec<MergedSelection<'a>>,
    id: Id,
}

impl<'a> AsIter for MergedSelectionSet<'a> {
    type Item = MergedSelection<'a>;
    type Iterator<'b> = std::slice::Iter<'b, Self::Item> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.selections.iter()
    }
}

impl<'a> SelectionSet for MergedSelectionSet<'a> {
    type Selection = MergedSelection<'a>;
}

impl<'a> Indexable for MergedSelectionSet<'a> {
    type Id = Id;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl<'a> MergedSelectionSet<'a> {
    pub(crate) fn new<E: ExecutableDocument>(context: &Context<'a, E>) -> Self {
        Self {
            selections: Vec::new(),
            id: context.next_id(),
        }
    }

    pub(crate) fn merge<E: ExecutableDocument>(
        &mut self,
        selection_set: &'a E::SelectionSet,
        context: &Context<'a, E>,
    ) -> Result<(), Vec<Error<'a>>> {
        selection_set.iter().try_for_each(|selection| {
            let selection_reference = selection.as_ref();
            EmptyDirectives::ensure_empty::<false, E>(
                selection_reference.directives(),
                selection_reference.associated_directive_location(),
            )?;
            match selection_reference {
                SelectionReference::Field(field) => {
                    let response_name = self.response_name_for_field(field, context);

                    let existing_idx = self.selections.iter().enumerate().find_map(|(idx, s)| {
                        if let MergedSelection::Field(existing) = s {
                            (existing.response_name() == response_name).then_some(idx)
                        } else {
                            None
                        }
                    });

                    let existing = if let Some(idx) = existing_idx {
                        self.selections[idx].try_as_field_mut().unwrap()
                    } else {
                        let alias = (field.name() != response_name).then_some(response_name);

                        self.selections
                            .push(MergedSelection::Field(MergedField::new(
                                field.name(),
                                alias,
                                field.arguments(),
                                context,
                            )));
                        self.selections
                            .last_mut()
                            .unwrap()
                            .try_as_field_mut()
                            .unwrap()
                    };

                    if existing.name() != field.name() {
                        return Err(vec![Error::DifferingFieldNamesForResponseName {
                            response_name: field.response_name(),
                        }]);
                    }

                    let new_arguments = field
                        .arguments()
                        .map(|arguments| MergedArguments::new(arguments, context));

                    if !MergedArguments::<false>::equivalent(
                        existing.arguments(),
                        new_arguments.as_ref(),
                    ) {
                        return Err(vec![Error::ArgumentsNotCompatible]);
                    }

                    if let Some(selection_set) = field.selection_set() {
                        existing
                            .selection_set_mut()
                            .get_or_insert_with(|| MergedSelectionSet::new(context))
                            .merge(selection_set, context)?;
                    }

                    Ok(())
                }
                SelectionReference::InlineFragment(inline_fragment) => {
                    let Some(type_condition) = inline_fragment.type_condition() else {
                        return self.merge(inline_fragment.selection_set(), context);
                    };

                    let merged_inline_fragment =
                        self.merged_inline_fragment(type_condition, context);

                    merged_inline_fragment
                        .selection_set_mut()
                        .merge(inline_fragment.selection_set(), context)
                }
                SelectionReference::FragmentSpread(fragment_spread) => {
                    let Some(fragment_definition) =
                        context.fragment_definition(fragment_spread.name())
                    else {
                        return Err(vec![Error::FragmentDefinitionNotFound {
                            fragment_name: fragment_spread.name(),
                        }]);
                    };

                    // because we don't have a schema definition, we have no way of knowing if the current type is the same
                    // as the type condition of the fragment definition, so we always use an inline fragment preserving the
                    // type condition of the fragment definition
                    let merged_inline_fragment =
                        self.merged_inline_fragment(fragment_definition.type_condition(), context);

                    merged_inline_fragment
                        .selection_set_mut()
                        .merge(fragment_definition.selection_set(), context)
                }
            }
        })
    }

    fn merged_inline_fragment<E: ExecutableDocument>(
        &mut self,
        type_condition: &'a str,
        context: &Context<'a, E>,
    ) -> &mut MergedInlineFragment<'a> {
        let existing_idx = self.selections.iter().enumerate().find_map(|(idx, s)| {
            if let MergedSelection::InlineFragment(existing) = s {
                (existing.type_condition() == Some(type_condition)).then_some(idx)
            } else {
                None
            }
        });

        if let Some(idx) = existing_idx {
            self.selections[idx].try_as_inline_fragment_mut().unwrap()
        } else {
            self.selections
                .push(MergedSelection::InlineFragment(MergedInlineFragment::new(
                    type_condition,
                    context,
                )));
            self.selections
                .last_mut()
                .unwrap()
                .try_as_inline_fragment_mut()
                .unwrap()
        }
    }

    fn response_name_for_field<E: ExecutableDocument>(
        &self,
        field: &'a E::Field,
        context: &Context<'a, E>,
    ) -> Cow<'a, str> {
        let suffix_on_merge = field.directives().iter().find_map(|directive| {
            if directive.name() == "suffixOnMerge" {
                directive.arguments().and_then(|arguments| {
                    arguments.iter().find_map(|arg| {
                        if arg.name() == "contextKey" {
                            if let Some(context_key) = arg.value().as_ref().as_string() {
                                context.user_provided_context_for_key(context_key)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                })
            } else {
                None
            }
        });

        if let Some(suffix) = suffix_on_merge {
            format!("{}{}", field.response_name(), suffix).into()
        } else {
            field.response_name().into()
        }
    }
}
