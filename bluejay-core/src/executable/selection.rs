use crate::{
    definition::DirectiveLocation,
    executable::{Field, FragmentSpread, InlineFragment},
};

#[derive(Debug)]
pub enum SelectionReference<'a, S: Selection> {
    Field(&'a S::Field),
    FragmentSpread(&'a S::FragmentSpread),
    InlineFragment(&'a S::InlineFragment),
}

pub trait Selection: Sized {
    type Field: Field;
    type FragmentSpread: FragmentSpread<Directives = <Self::Field as Field>::Directives>;
    type InlineFragment: InlineFragment<Directives = <Self::Field as Field>::Directives>;

    fn as_ref(&self) -> SelectionReference<'_, Self>;
}

impl<'a, S: Selection> SelectionReference<'a, S> {
    pub fn directives(&self) -> Option<&'a <S::Field as Field>::Directives> {
        match self {
            Self::Field(f) => f.directives(),
            Self::FragmentSpread(fs) => fs.directives(),
            Self::InlineFragment(i) => i.directives(),
        }
    }

    pub fn associated_directive_location(&self) -> DirectiveLocation {
        match self {
            Self::Field(_) => DirectiveLocation::Field,
            Self::FragmentSpread(_) => DirectiveLocation::FragmentSpread,
            Self::InlineFragment(_) => DirectiveLocation::InlineFragment,
        }
    }
}
