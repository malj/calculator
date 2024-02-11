use rust_decimal::{prelude::Signed, Decimal};

#[derive(Debug, PartialEq, Eq)]
/// Node containing either a decimal value
/// or an expression which evaluates to a decimal value.
pub enum Node {
	Value(Decimal),
	Expr(Box<Expr>), // requires boxing because of circular reference
}

impl TryFrom<Node> for Decimal {
	type Error = rust_decimal::Error;

	fn try_from(value: Node) -> Result<Self, Self::Error> {
		match value {
			Node::Value(value) => Ok(value),
			Node::Expr(expr) => Decimal::try_from(*expr),
		}
	}
}

impl From<Decimal> for Node {
	fn from(value: Decimal) -> Self {
		Node::Value(value)
	}
}

#[derive(Debug, PartialEq, Eq)]
/// An expression describing an arithmetical operation
/// to perform on its node operand(s).
pub enum Expr {
	/// Addition
	Add(Node, Node),
	/// Subtraction
	Sub(Node, Node),
	/// Multiplication
	Mul(Node, Node),
	/// Division
	Div(Node, Node),
	/// Sign inversion
	Neg(Node),
}

impl TryFrom<Expr> for Decimal {
	type Error = rust_decimal::Error;

	fn try_from(value: Expr) -> Result<Self, Self::Error> {
		match value {
			Expr::Add(lhs, rhs) => {
				let lhs = Decimal::try_from(lhs)?;
				let rhs = Decimal::try_from(rhs)?;
				// Can overflow
				lhs.checked_add(rhs)
					.ok_or(rust_decimal::Error::ExceedsMaximumPossibleValue)
			}
			Expr::Sub(lhs, rhs) => {
				let lhs = Decimal::try_from(lhs)?;
				let rhs = Decimal::try_from(rhs)?;
				// Can underflow
				lhs.checked_sub(rhs)
					.ok_or(rust_decimal::Error::LessThanMinimumPossibleValue)
			}
			Expr::Mul(lhs, rhs) => {
				let lhs = Decimal::try_from(lhs)?;
				let rhs = Decimal::try_from(rhs)?;
				// Can overflow or undeflow depending on operand signs
				lhs.checked_mul(rhs).ok_or(if lhs.signum() == rhs.signum() {
					rust_decimal::Error::ExceedsMaximumPossibleValue
				} else {
					rust_decimal::Error::LessThanMinimumPossibleValue
				})
			}
			Expr::Div(lhs, rhs) => {
				let lhs = Decimal::try_from(lhs)?;
				let rhs = Decimal::try_from(rhs)?;
				// Can overflow or underflow (division by zero)
				lhs.checked_div(rhs).ok_or(if lhs >= Decimal::ZERO {
					rust_decimal::Error::ExceedsMaximumPossibleValue
				} else {
					rust_decimal::Error::LessThanMinimumPossibleValue
				})
			}
			Expr::Neg(value) => Ok(-Decimal::try_from(value)?),
		}
	}
}
