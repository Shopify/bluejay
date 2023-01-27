use strum::{IntoStaticStr, AsRefStr, EnumVariantNames, EnumString, EnumIter, Display};
use crate::definition::ScalarTypeDefinition;

#[derive(IntoStaticStr, AsRefStr, EnumVariantNames, EnumString, EnumIter, Display, Clone, Copy, Debug)]
pub enum BuiltinScalarDefinition {
    Int,
    Float,
    String,
    Boolean,
    ID,
}

impl BuiltinScalarDefinition {
    pub fn name(&self) -> &'static str {
        self.into()
    }
}

impl ScalarTypeDefinition for BuiltinScalarDefinition {
    fn description(&self) -> Option<&str> {
        None
    }

    fn name(&self) -> &str {
        self.into()
    }
}
