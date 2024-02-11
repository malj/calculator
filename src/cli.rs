use crate::parser;
use rust_decimal::Decimal;
use std::{error, fmt, io};

#[derive(Debug)]
pub enum Error {
	Input(io::Error),
	Parse(parser::Error),
	Math(rust_decimal::Error),
}

impl error::Error for Error {}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Error::Input(e) => write!(f, "{e}"),
			Error::Parse(e) => write!(f, "{e}"),
			Error::Math(e) => write!(f, "{e}"),
		}
	}
}

/// Evaluate an arithmetic expression:
/// 1. Read user input
/// 2. Parse the input and generate an abstract syntax tree (AST)
/// 3. Evaluate the AST and return a numeric result
pub fn try_calculate(buffer: &mut String) -> Result<Decimal, Error> {
	io::stdin().read_line(buffer).map_err(Error::Input)?;
	let root_node = parser::parse(buffer).map_err(Error::Parse)?;
	root_node.try_into().map_err(Error::Math)
}
