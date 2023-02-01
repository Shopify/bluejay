#![feature(iter_intersperse)]

pub mod ast;
mod error;
mod lexical_token;
mod scanner;
mod span;

pub use ast::parse;
pub use error::Error;
pub use scanner::logos_scanner::LogosScanner;
pub use scanner::Scanner;
pub use span::Span;
