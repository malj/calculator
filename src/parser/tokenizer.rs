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
