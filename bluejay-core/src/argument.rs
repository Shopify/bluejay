use crate::{AbstractValue};

pub trait Argument<const CONST: bool> {
    type Value: AbstractValue<CONST>;

    fn name(&self) -> &str;
    fn value(&self) -> &Self::Value;
}

pub trait ConstArgument: Argument<true> {}
pub trait VariableArgument: Argument<false> {}

impl<T: Argument<true>> ConstArgument for T {}
impl<T: Argument<false>> VariableArgument for T {}

pub trait Arguments<const CONST: bool>: AsRef<[Self::Argument]> {
    type Argument: Argument<CONST>;
}

pub trait ConstArguments: Arguments<true> {}
pub trait VariableArguments: Arguments<false> {}

impl<T: Arguments<true>> ConstArguments for T {}
impl<T: Arguments<false>> VariableArguments for T {}
