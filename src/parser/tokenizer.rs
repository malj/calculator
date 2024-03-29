use rust_decimal::Decimal;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
	Value(Decimal),
	Operator(Operator),
	GroupStart,
	GroupEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
	Add,
	Sub,
	Mul,
	Div,
}

/// Split an input string into stream of tokens.
pub fn tokenize(input: &str) -> impl Iterator<Item = Result<Token, rust_decimal::Error>> + '_ {
	// Since there are only two classes of tokens (static operators and dynamic values)
	// static tokens can be used as separators, splitting the input string.
	// 1. Split the string and separates separators
	// 2. Format and filter remaining chunks
	// 3. Identify chunks and map them to a specific token
	input
		.split_inclusive(is_separator)
		.flat_map(|mut chunk| {
			// `str::split_inclusve` includes separators with the previous chunk.
			// They need to be split from the chunk for easier parsing.
			// Example: `1337+` -> `1337`, `+`
			let mut separator = "";
			if let Some(c) = chunk.chars().last() {
				if is_separator(c) {
					// Separators are split by length (1) because they are single characters.
					// This part needs to be reworked for longer separators.
					(chunk, separator) = chunk.split_at(chunk.len() - 1);
				}
			}
			[chunk, separator].into_iter()
		})
		.flat_map(str::split_whitespace)
		.map(str::trim)
		.filter(|value| !value.is_empty())
		.map(|chunk| match chunk {
			"+" => Ok(Token::Operator(Operator::Add)),
			"-" => Ok(Token::Operator(Operator::Sub)),
			"*" => Ok(Token::Operator(Operator::Mul)),
			"/" => Ok(Token::Operator(Operator::Div)),
			"(" => Ok(Token::GroupStart),
			")" => Ok(Token::GroupEnd),
			value => parse_number(value).map(Token::Value),
		})
}

/// Determine whether a character is a token separator.
fn is_separator(value: char) -> bool {
	matches!(value, '+' | '-' | '*' | '/' | '(' | ')')
}

/// Try converting a string token into a decimal.
fn parse_number(value: &str) -> Result<Decimal, rust_decimal::Error> {
	if let Some(hex_value) = value.strip_prefix("0x") {
		Decimal::from_str_radix(hex_value, 16)
	} else {
		Decimal::from_str(value)
	}
}

#[cfg(test)]
mod tests {
	use super::{parse_number, tokenize, Operator, Token};
	use rust_decimal::Decimal;

	#[test]
	fn parse_integer() {
		assert_eq!(parse_number("0"), Ok(Decimal::ZERO));
		assert_eq!(parse_number("1337"), Ok(Decimal::new(1337, 0)));
	}

	#[test]
	fn parse_float() {
		assert_eq!(parse_number("0.0"), Ok(Decimal::ZERO));
		assert_eq!(parse_number("133.7"), Ok(Decimal::new(1337, 1)));
	}

	#[test]
	fn parse_hexadecimal() {
		assert_eq!(parse_number("0x0"), Ok(Decimal::ZERO));
		assert_eq!(parse_number("0x539"), Ok(Decimal::new(1337, 0)));
	}

	#[test]
	fn tokenize_input() {
		let mut tokens = tokenize("(0 + 0) - 0 * 0 / 0");
		assert_eq!(tokens.next().unwrap(), Ok(Token::GroupStart));
		assert_eq!(tokens.next().unwrap(), Ok(Token::Value(Decimal::ZERO)));
		assert_eq!(tokens.next().unwrap(), Ok(Token::Operator(Operator::Add)));
		assert_eq!(tokens.next().unwrap(), Ok(Token::Value(Decimal::ZERO)));
		assert_eq!(tokens.next().unwrap(), Ok(Token::GroupEnd));
		assert_eq!(tokens.next().unwrap(), Ok(Token::Operator(Operator::Sub)));
		assert_eq!(tokens.next().unwrap(), Ok(Token::Value(Decimal::ZERO)));
		assert_eq!(tokens.next().unwrap(), Ok(Token::Operator(Operator::Mul)));
		assert_eq!(tokens.next().unwrap(), Ok(Token::Value(Decimal::ZERO)));
		assert_eq!(tokens.next().unwrap(), Ok(Token::Operator(Operator::Div)));
		assert_eq!(tokens.next().unwrap(), Ok(Token::Value(Decimal::ZERO)));
		assert!(tokens.next().is_none());
	}

	#[test]
	fn insignificant_whitespace() {
		assert_eq!(
			tokenize("1+1").collect::<Vec<_>>(),
			tokenize("1 + 1").collect::<Vec<_>>()
		);
	}
}
