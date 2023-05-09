use crate::executable::{Field, FragmentSpread, InlineFragment};

#[derive(Debug)]
pub enum Selection<'a, F: Field, FS: FragmentSpread, IF: InlineFragment> {
    Field(&'a F),
    FragmentSpread(&'a FS),
    InlineFragment(&'a IF),
}

pub trait AbstractSelection {
    type Field: Field;
    type FragmentSpread: FragmentSpread;
    type InlineFragment: InlineFragment;

    fn as_ref(&self) -> SelectionFromAbstract<'_, Self>;
}

pub type SelectionFromAbstract<'a, T> = Selection<
    'a,
    <T as AbstractSelection>::Field,
    <T as AbstractSelection>::FragmentSpread,
    <T as AbstractSelection>::InlineFragment,
>;

impl<
        'a,
        F: Field,
        FS: FragmentSpread<Directives = F::Directives>,
        IF: InlineFragment<Directives = F::Directives>,
    > Selection<'a, F, FS, IF>
{
    pub fn directives(&self) -> &'a F::Directives {
        match self {
            Self::Field(f) => f.directives(),
            Self::FragmentSpread(fs) => fs.directives(),
            Self::InlineFragment(i) => i.directives(),
        }
    }
}
