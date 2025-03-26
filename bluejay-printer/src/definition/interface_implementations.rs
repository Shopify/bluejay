use bluejay_core::definition::{InterfaceImplementation, InterfaceImplementations};
use std::fmt::{Display, Formatter, Result};

pub(crate) struct InterfaceImplementationsPrinter<'a, I: InterfaceImplementations>(&'a I);

impl<'a, I: InterfaceImplementations> InterfaceImplementationsPrinter<'a, I> {
    pub(crate) fn new(interface_implementations: &'a I) -> Self {
        Self(interface_implementations)
    }
}

impl<I: InterfaceImplementations> Display for InterfaceImplementationsPrinter<'_, I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self(interface_implementations) = *self;
        if !interface_implementations.is_empty() {
            write!(f, " implements ")?;
            interface_implementations
                .iter()
                .enumerate()
                .try_for_each(|(idx, ii)| {
                    if idx != 0 {
                        write!(f, " & ")?;
                    }
                    write!(f, "{}", ii.name())
                })
        } else {
            Ok(())
        }
    }
}
