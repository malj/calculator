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
