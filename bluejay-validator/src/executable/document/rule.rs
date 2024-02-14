use crate::executable::document::Visitor;
use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::ExecutableDocument;

pub trait Rule<'a, E: ExecutableDocument, S: SchemaDefinition>: Visitor<'a, E, S> {
    type Error;
    type Errors: Iterator<Item = Self::Error>;

    fn into_errors(self) -> Self::Errors;
}

macro_rules! impl_rule {
    ($n:literal) => {
        seq_macro::seq!(N in 0..$n {
            impl<'a, E: ExecutableDocument, S: SchemaDefinition, ER, #(T~N: Rule<'a, E, S, Error = ER>,)*> Rule<'a, E, S> for (#(T~N,)*) {
                type Error = ER;
                type Errors = #(std::iter::Chain<)* std::iter::Empty<ER> #(, <T~N as Rule<'a, E, S>>::Errors>)*;

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

impl_rule!(26);
