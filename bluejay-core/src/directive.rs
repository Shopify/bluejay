use crate::Arguments;

pub trait Directive<const CONST: bool> {
    type Arguments: Arguments<CONST>;

    fn name(&self) -> &str;
    fn arguments(&self) -> Option<&Self::Arguments>;
}

pub trait ConstDirective: Directive<true> {}
impl<T: Directive<true>> ConstDirective for T {}

pub trait VariableDirective: Directive<false> {}
impl<T: Directive<false>> VariableDirective for T {}

pub trait Directives<const CONST: bool>: AsRef<[Self::Directive]> {
    type Directive: Directive<CONST>;
}

pub trait ConstDirectives: Directives<true> {}
impl<T: Directives<true>> ConstDirectives for T {}

pub trait VariableDirectives: Directives<false> {}
impl<T: Directives<false>> VariableDirectives for T {}
