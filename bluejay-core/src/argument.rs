use crate::{AsIter, Value};
use std::collections::HashMap;

pub trait Argument<const CONST: bool> {
    type Value: Value<CONST>;

    fn name(&self) -> &str;
    fn value(&self) -> &Self::Value;
}

pub trait ConstArgument: Argument<true> {}
pub trait VariableArgument: Argument<false> {}

impl<T: Argument<true>> ConstArgument for T {}
impl<T: Argument<false>> VariableArgument for T {}

pub trait Arguments<const CONST: bool>: AsIter<Item = Self::Argument> {
    type Argument: Argument<CONST>;

    fn equivalent(optional_self: Option<&Self>, optional_other: Option<&Self>) -> bool {
        let lhs: HashMap<&str, _> = optional_self
            .map(|args| {
                HashMap::from_iter(args.iter().map(|arg| (arg.name(), arg.value().as_ref())))
            })
            .unwrap_or_default();
        let rhs: HashMap<&str, _> = optional_other
            .map(|args| {
                HashMap::from_iter(args.iter().map(|arg| (arg.name(), arg.value().as_ref())))
            })
            .unwrap_or_default();
        lhs == rhs
    }
}

pub trait ConstArguments: Arguments<true> {}
pub trait VariableArguments: Arguments<false> {}

impl<T: Arguments<true>> ConstArguments for T {}
impl<T: Arguments<false>> VariableArguments for T {}
