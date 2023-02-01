use crate::definition::InterfaceImplementation;
use crate::AsIter;

pub trait InterfaceImplementations: AsIter<Item = Self::InterfaceImplementation> {
    type InterfaceImplementation: InterfaceImplementation;
}
