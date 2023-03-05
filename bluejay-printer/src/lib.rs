mod argument;
pub mod definition;
mod directive;
mod string_value;
mod value;
use std::fmt::{Error, Write};

fn write_indent<W: Write>(f: &mut W, indentation: usize) -> Result<(), Error> {
    write!(f, "{: >1$}", "", indentation)
}

const INDENTATION_SIZE: usize = 2;
