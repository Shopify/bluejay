use bluejay_core::definition::{
    InterfaceImplementation, InterfaceImplementations, InterfaceTypeDefinition,
};
use std::fmt::{Error, Write};

pub(crate) struct DisplayInterfaceImplementations;

impl DisplayInterfaceImplementations {
    pub(crate) fn fmt<T: InterfaceImplementations, W: Write>(
        interface_implementations: &T,
        f: &mut W,
    ) -> Result<(), Error> {
        if !interface_implementations.is_empty() {
            write!(f, "implements ")?;
            interface_implementations
                .iter()
                .enumerate()
                .try_for_each(|(idx, ii)| {
                    if idx != 0 {
                        write!(f, " & ")?;
                    }
                    write!(f, "{}", ii.interface().name())
                })
        } else {
            Ok(())
        }
    }
}
