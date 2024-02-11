use crate::engine::Node;
use std::iter;
use tokenizer::{tokenize, Token};

mod ast;
mod error;
mod tokenizer;

pub use self::error::Error;

/// Construct a tree of value or expression nodes to be evaluated by the engine.
pub fn parse(input: &str) -> Result<Node, Error> {
	let tokens = &mut tokenize(input).chain(iter::once(Ok(Token::GroupEnd)));
	let root_node = parse_tokens(tokens)?;

	if tokens.next().is_none() {
		Ok(root_node)
	} else {
		Err(Error::UninitializedGroup)
	}
}

/// Convert a stream of tokens into a root tree node.
fn parse_tokens(
	tokens: &mut impl Iterator<Item = Result<Token, rust_decimal::Error>>,
) -> Result<Node, Error> {
	let mut builder = ast::Builder::new();
	let mut is_terminated = false;
	while let Some(token) = tokens.next() {
		match token.map_err(Error::Value)? {
			Token::Value(value) => builder.add_node(Node::Value(value))?,
			Token::Operator(operator) => builder.add_operator(operator)?,
			Token::GroupStart => builder.add_node(parse_tokens(tokens)?)?,
			Token::GroupEnd => {
				is_terminated = true;
				break;
			}
		}
	}
	if is_terminated {
		builder.build()
	} else {
		Err(Error::UnterminatedGroup)
	}
}

#[cfg(test)]
mod tests {
	use super::parse;
	use rust_decimal::Decimal;

	#[test]
	fn add() {
		let node = parse("1 + 1").unwrap();
		assert_eq!(Decimal::TWO, node.try_into().unwrap());
	}

	#[test]
	fn sub() {
		let node = parse("1 - 1").unwrap();
		assert_eq!(Decimal::ZERO, node.try_into().unwrap());
	}

	#[test]
	fn mul() {
		let node = parse("1 * 2").unwrap();
		assert_eq!(Decimal::TWO, node.try_into().unwrap());
	}

	#[test]
	fn div() {
		let node = parse("1 / 2").unwrap();
		assert_eq!(Decimal::new(5, 1), node.try_into().unwrap());
	}

	#[test]
	fn neg() {
		let node = parse("-1").unwrap();
		assert_eq!(Decimal::NEGATIVE_ONE, node.try_into().unwrap());
	}

	#[test]
	fn raw() {
		let node = parse("1000").unwrap();
		assert_eq!(Decimal::ONE_THOUSAND, node.try_into().unwrap());
	}
}
