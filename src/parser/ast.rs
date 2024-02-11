use super::{error::Error, tokenizer::Operator};
use crate::engine::{Expr, Node};
use std::collections::VecDeque;

#[derive(Debug, PartialEq, Eq)]
enum Element {
	Node(Node),
	Operator(Operator),
}

#[derive(Default)]
/// Abstract syntax tree (AST) builder.
/// It combines incoming nodes and operators into parent nodes.
pub struct Builder {
	buffer: VecDeque<Element>,
}

impl Builder {
	pub fn new() -> Self {
		Self::default()
	}

	/// Adds a node element. The order of addition is important and
	/// the operation can fail depending on the previous state.
	pub fn add_node(&mut self, node: Node) -> Result<(), Error> {
		match self.buffer.len() {
			0 => self.buffer.push_back(Element::Node(node)),
			1 => match self.buffer[0] {
				Element::Operator(Operator::Sub) => {
					// Previous minus was unary
					self.buffer.pop_back();
					self.add_node(Node::Expr(Expr::Neg(node).into()))?;
				}
				_ => return Err(Error::LeftoverElements),
			},
			n => match [&self.buffer[n - 2], &self.buffer[n - 1]] {
				[Element::Operator(_), Element::Operator(Operator::Sub)] => {
					// Previous minus was unary
					self.buffer.pop_back();
					self.add_node(Node::Expr(Expr::Neg(node).into()))?;
				}
				[Element::Node(_), Element::Operator(operator)] => match operator {
					Operator::Mul => {
						self.buffer.pop_back();
						// Transfer ownership of the matched element
						let prev_node = match self.buffer.pop_back() {
							Some(Element::Node(prev_node)) => prev_node,
							_ => unreachable!(),
						};
						self.add_node(Node::Expr(Expr::Mul(prev_node, node).into()))?;
					}
					Operator::Div => {
						self.buffer.pop_back();
						// Transfer ownership of the matched element
						let prev_node = match self.buffer.pop_back() {
							Some(Element::Node(prev_node)) => prev_node,
							_ => unreachable!(),
						};
						self.add_node(Node::Expr(Expr::Div(prev_node, node).into()))?;
					}
					_ => {
						// Defer add and sub expression building until the end
						// because future operators might have a higher priority
						self.buffer.push_back(Element::Node(node));
					}
				},
				_ => return Err(Error::UnexpectedNode(node)),
			},
		}
		Ok(())
	}

	/// Adds an operator element. The order of addition is important and
	/// the operation can fail depending on the previous state.
	pub fn add_operator(&mut self, operator: Operator) -> Result<(), Error> {
		if operator != Operator::Sub
			&& matches!(self.buffer.back(), None | Some(Element::Operator(_)))
		{
			Err(Error::UnexpectedOperator(operator))
		} else {
			self.buffer.push_back(Element::Operator(operator));
			Ok(())
		}
	}

	/// Flushes the element buffer and creates a tree root node.
	pub fn build(mut self) -> Result<Node, Error> {
		// Buffer contents are already verified in `add` methods.
		// It is safe to assume `element -> operator [-> element]` order.
		match self.buffer.len() {
			0 | 1 => match self.buffer.pop_back() {
				Some(Element::Node(node)) => Ok(node),
				Some(Element::Operator(opeator)) => Err(Error::UnexpectedOperator(opeator)),
				None => Err(Error::Empty),
			},
			2 => Err(Error::LeftoverElements),
			_ => {
				// Transfer ownership of the matched element
				let mut prev_node = match self.buffer.pop_front() {
					Some(Element::Node(prev_node)) => prev_node,
					_ => unreachable!(),
				};
				// Transfer ownership of the matched element
				let mut prev_operator = match self.buffer.pop_front() {
					Some(Element::Operator(prev_operator)) => prev_operator,
					_ => unreachable!(),
				};
				while let Some(element) = self.buffer.pop_front() {
					match element {
						Element::Node(node) => match prev_operator {
							Operator::Add => {
								prev_node = Node::Expr(Expr::Add(prev_node, node).into());
							}
							Operator::Sub => {
								prev_node = Node::Expr(Expr::Sub(prev_node, node).into());
							}
							_ => unreachable!(),
						},
						Element::Operator(operator) => prev_operator = operator,
					}
				}
				Ok(prev_node)
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::{super::tokenizer::Operator, Builder};
	use crate::engine::Node;
	use rust_decimal::Decimal;

	#[test]
	fn add() {
		let mut builder = Builder::new();
		builder.add_node(Node::Value(Decimal::ONE)).unwrap();
		builder.add_operator(Operator::Add).unwrap();
		builder.add_node(Node::Value(Decimal::ONE)).unwrap();
		let node = builder.build().unwrap();

		assert_eq!(Decimal::TWO, node.try_into().unwrap());
	}

	#[test]
	fn sub() {
		let mut builder = Builder::new();
		builder.add_node(Node::Value(Decimal::ONE)).unwrap();
		builder.add_operator(Operator::Sub).unwrap();
		builder.add_node(Node::Value(Decimal::ONE)).unwrap();
		let node = builder.build().unwrap();

		assert_eq!(Decimal::ZERO, node.try_into().unwrap());
	}

	#[test]
	fn mul() {
		let mut builder = Builder::new();
		builder.add_node(Node::Value(Decimal::ONE)).unwrap();
		builder.add_operator(Operator::Mul).unwrap();
		builder.add_node(Node::Value(Decimal::TWO)).unwrap();
		let node = builder.build().unwrap();

		assert_eq!(Decimal::TWO, node.try_into().unwrap());
	}

	#[test]
	fn div() {
		let mut builder = Builder::new();
		builder.add_node(Node::Value(Decimal::ONE)).unwrap();
		builder.add_operator(Operator::Div).unwrap();
		builder.add_node(Node::Value(Decimal::TWO)).unwrap();
		let node = builder.build().unwrap();

		assert_eq!(Decimal::new(5, 1), node.try_into().unwrap());
	}

	#[test]
	fn neg() {
		let mut builder = Builder::new();
		builder.add_operator(Operator::Sub).unwrap();
		builder.add_node(Node::Value(Decimal::ONE)).unwrap();
		let node = builder.build().unwrap();

		assert_eq!(Decimal::NEGATIVE_ONE, node.try_into().unwrap());
	}

	#[test]
	fn raw() {
		let mut builder = Builder::new();
		builder
			.add_node(Node::Value(Decimal::ONE_THOUSAND))
			.unwrap();
		let node = builder.build().unwrap();

		assert_eq!(Decimal::ONE_THOUSAND, node.try_into().unwrap());
	}
}
