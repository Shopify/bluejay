use crate::definition::Visitor;
use bluejay_core::definition::SchemaDefinition;

pub trait Rule<'a, S: SchemaDefinition>: Visitor<'a, S> {
    type Error;
    type Errors: Iterator<Item = Self::Error>;

    fn into_errors(self) -> Self::Errors;
}

macro_rules! impl_rule {
    ($n:literal) => {
        seq_macro::seq!(N in 0..$n {
            impl<'a, S: SchemaDefinition, ER, #(T~N: Rule<'a, S, Error = ER>,)*> Rule<'a, S> for (#(T~N,)*) {
                type Error = ER;
                type Errors = #(std::iter::Chain<)* std::iter::Empty<ER> #(, <T~N as Rule<'a, S>>::Errors>)*;

                fn into_errors(self) -> Self::Errors {
                    std::iter::empty() #(.chain(self.N.into_errors()))*
                }
            }
        });
    }
}

seq_macro::seq!(N in 2..=10 {
    impl_rule!(N);
});
