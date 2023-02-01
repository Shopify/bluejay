use crate::executable::{Field, FragmentSpread, InlineFragment};

#[derive(Debug)]
pub enum Selection<F: Field, FS: FragmentSpread, IF: InlineFragment> {
    Field(F),
    FragmentSpread(FS),
    InlineFragment(IF),
}

pub trait AbstractSelection:
    Into<Selection<Self::Field, Self::FragmentSpread, Self::InlineFragment>>
    + AsRef<Selection<Self::Field, Self::FragmentSpread, Self::InlineFragment>>
{
    type Field: Field;
    type FragmentSpread: FragmentSpread;
    type InlineFragment: InlineFragment;
}

impl<F: Field, FS: FragmentSpread, IF: InlineFragment> AsRef<Selection<F, FS, IF>>
    for Selection<F, FS, IF>
{
    fn as_ref(&self) -> &Selection<F, FS, IF> {
        self
    }
}

impl<F: Field, FS: FragmentSpread, IF: InlineFragment> AbstractSelection for Selection<F, FS, IF> {
    type Field = F;
    type FragmentSpread = FS;
    type InlineFragment = IF;
}
