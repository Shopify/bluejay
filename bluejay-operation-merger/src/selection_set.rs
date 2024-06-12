use crate::{
    Context, EmptyDirectives, Error, Id, MergedField, MergedInlineFragment, MergedSelection,
};
use bluejay_core::{
    executable::{
        ExecutableDocument, Field, FragmentDefinition, FragmentSpread, InlineFragment, Selection,
        SelectionReference, SelectionSet,
    },
    Arguments, AsIter, Indexable,
};

pub struct MergedSelectionSet<'a, E: ExecutableDocument + 'a> {
    selections: Vec<MergedSelection<'a, E>>,
    id: Id,
}

impl<'a, E: ExecutableDocument> AsIter for MergedSelectionSet<'a, E> {
    type Item = MergedSelection<'a, E>;
    type Iterator<'b> = std::slice::Iter<'b, Self::Item> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.selections.iter()
    }
}

impl<'a, E: ExecutableDocument> SelectionSet for MergedSelectionSet<'a, E> {
    type Selection = MergedSelection<'a, E>;
}

impl<'a, E: ExecutableDocument> Indexable for MergedSelectionSet<'a, E> {
    type Id = Id;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl<'a, E: ExecutableDocument> MergedSelectionSet<'a, E> {
    pub(crate) fn new(context: &Context<'a, E>) -> Self {
        Self {
            selections: Vec::new(),
            id: context.next_id(),
        }
    }

    pub(crate) fn merge(
        &mut self,
        selection_set: &'a E::SelectionSet,
        context: &Context<'a, E>,
    ) -> Result<(), Vec<Error<'a, E>>> {
        selection_set.iter().try_for_each(|selection| {
            let selection_reference = selection.as_ref();
            EmptyDirectives::ensure_empty(selection_reference.directives())?;
            match selection_reference {
                SelectionReference::Field(field) => {
                    let existing_idx = self.selections.iter().enumerate().find_map(|(idx, s)| {
                        if let MergedSelection::Field(existing) = s {
                            (existing.response_name() == field.response_name()).then_some(idx)
                        } else {
                            None
                        }
                    });

                    let existing = if let Some(idx) = existing_idx {
                        self.selections[idx].try_as_field_mut().unwrap()
                    } else {
                        self.selections
                            .push(MergedSelection::Field(MergedField::new(
                                field.name(),
                                field.alias(),
                                field.arguments(),
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
                    if !<E::Arguments<false> as Arguments<false>>::equivalent(
                        existing.arguments(),
                        field.arguments(),
                    ) {
                        return Err(vec![Error::ArgumentsNotCompatible {
                            first: existing.arguments(),
                            second: field.arguments(),
                        }]);
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

    fn merged_inline_fragment(
        &mut self,
        type_condition: &'a str,
        context: &Context<'a, E>,
    ) -> &mut MergedInlineFragment<'a, E> {
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
}
