use crate::executable::operation::{VariableValues, Visitor};
use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::ExecutableDocument;

pub trait Analyzer<'a, E: ExecutableDocument, S: SchemaDefinition, V: VariableValues>:
    Visitor<'a, E, S, V>
{
    type Output;

    fn into_output(self) -> Self::Output;
}

macro_rules! impl_analyzer {
    ($n:literal) => {
        seq_macro::seq!(N in 0..$n {
            impl<'a, E: ExecutableDocument, S: SchemaDefinition, V: VariableValues, #(T~N: Analyzer<'a, E, S, V>,)*> Analyzer<'a, E, S, V> for (#(T~N,)*) {
                type Output = (#(T~N::Output,)*);

                fn into_output(self) -> Self::Output {
                    (
                        #(self.N.into_output(),)*
                    )
                }
            }
        });
    }
}

seq_macro::seq!(N in 2..=10 {
    impl_analyzer!(N);
});
