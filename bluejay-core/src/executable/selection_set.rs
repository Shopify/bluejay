use crate::executable::AbstractSelection;

pub trait SelectionSet: AsRef<[Self::Selection]> {
    type Selection: AbstractSelection;
}
