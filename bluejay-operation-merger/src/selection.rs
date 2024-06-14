use crate::{fragment_spread::MergedFragmentSpread, MergedField, MergedInlineFragment};
use bluejay_core::executable::{Selection, SelectionReference};
use strum::EnumTryAs;

#[derive(EnumTryAs)]
pub enum MergedSelection<'a> {
    Field(MergedField<'a>),
    InlineFragment(MergedInlineFragment<'a>),
    FragmentSpread(MergedFragmentSpread<'a>),
}

impl<'a> Selection for MergedSelection<'a> {
    type Field = MergedField<'a>;
    type InlineFragment = MergedInlineFragment<'a>;
    type FragmentSpread = MergedFragmentSpread<'a>;

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
