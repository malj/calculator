use super::tokenizer::Operator;
use crate::engine::Node;
use std::{error, fmt};

#[derive(Debug)]
pub enum Error {
	UnexpectedOperator(Operator),
	UnexpectedNode(Node),
	Empty,
	LeftoverElements,
}

impl error::Error for Error {}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::UnexpectedOperator(operator) => {
				write!(f, "Error: Unexpected {:?} operator", operator)
			}
			Self::UnexpectedNode(node) => write!(f, "Error: Unexpected {:?} node", node),
			Self::Empty => write!(f, "Error: Empty expression"),
			Self::LeftoverElements => write!(f, "Error: Unterminated expression"),
		}
	}
}
