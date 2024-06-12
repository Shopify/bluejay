use crate::{fragment_spread::MergedFragmentSpread, MergedField, MergedInlineFragment};
use bluejay_core::executable::{ExecutableDocument, Selection, SelectionReference};
use strum::EnumTryAs;

#[derive(EnumTryAs)]
pub enum MergedSelection<'a, E: ExecutableDocument> {
    Field(MergedField<'a, E>),
    InlineFragment(MergedInlineFragment<'a, E>),
    FragmentSpread(MergedFragmentSpread<'a, E>),
}

impl<'a, E: ExecutableDocument + 'a> Selection for MergedSelection<'a, E> {
    type Field = MergedField<'a, E>;
    type InlineFragment = MergedInlineFragment<'a, E>;
    type FragmentSpread = MergedFragmentSpread<'a, E>;

    fn as_ref(&self) -> SelectionReference<'_, Self> {
        match self {
            MergedSelection::Field(field) => SelectionReference::Field(field),
            MergedSelection::InlineFragment(inline_fragment) => {
                SelectionReference::InlineFragment(inline_fragment)
            }
            MergedSelection::FragmentSpread(fragment_spread) => {
                SelectionReference::FragmentSpread(fragment_spread)
            }
        }
    }
}
