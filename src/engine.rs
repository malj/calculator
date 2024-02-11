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

#[cfg(test)]
mod tests {
	use super::{Expr, Node};
	use rust_decimal::Decimal;

	#[test]
	fn raw() {
		assert_eq!(Decimal::ONE, Node::Value(Decimal::ONE).try_into().unwrap());
	}

	#[test]
	fn add() {
		assert_eq!(
			Decimal::TWO,
			Node::Expr(Expr::Add(Decimal::ONE.into(), Decimal::ONE.into()).into())
				.try_into()
				.unwrap()
		);
	}

	#[test]
	fn add_overflow() {
		let error: Result<Decimal, rust_decimal::Error> =
			Node::Expr(Expr::Add(Decimal::MAX.into(), Decimal::ONE.into()).into()).try_into();
		assert_eq!(error, Err(rust_decimal::Error::ExceedsMaximumPossibleValue));
	}

	#[test]
	fn sub() {
		assert_eq!(
			Decimal::ZERO,
			Node::Expr(Expr::Sub(Decimal::ONE.into(), Decimal::ONE.into()).into())
				.try_into()
				.unwrap()
		);
	}

	#[test]
	fn sub_underflow() {
		let error: Result<Decimal, rust_decimal::Error> =
			Node::Expr(Expr::Sub(Decimal::MIN.into(), Decimal::ONE.into()).into()).try_into();
		assert_eq!(
			error,
			Err(rust_decimal::Error::LessThanMinimumPossibleValue)
		);
	}

	#[test]
	fn mul() {
		assert_eq!(
			Decimal::ONE,
			Node::Expr(Expr::Mul(Decimal::ONE.into(), Decimal::ONE.into()).into())
				.try_into()
				.unwrap()
		);
	}

	#[test]
	fn mul_overflow() {
		let error: Result<Decimal, rust_decimal::Error> =
			Node::Expr(Expr::Mul(Decimal::MAX.into(), Decimal::TWO.into()).into()).try_into();
		assert_eq!(error, Err(rust_decimal::Error::ExceedsMaximumPossibleValue));
	}

	#[test]
	fn mul_underflow() {
		let error: Result<Decimal, rust_decimal::Error> =
			Node::Expr(Expr::Mul(Decimal::MIN.into(), Decimal::TWO.into()).into()).try_into();
		assert_eq!(
			error,
			Err(rust_decimal::Error::LessThanMinimumPossibleValue)
		);
	}

	#[test]
	fn div() {
		assert_eq!(
			Decimal::ONE,
			Node::Expr(Expr::Div(Decimal::ONE.into(), Decimal::ONE.into()).into())
				.try_into()
				.unwrap()
		);
	}

	#[test]
	fn div_overflow() {
		let error: Result<Decimal, rust_decimal::Error> =
			Node::Expr(Expr::Div(Decimal::ONE.into(), Decimal::ZERO.into()).into()).try_into();
		assert_eq!(error, Err(rust_decimal::Error::ExceedsMaximumPossibleValue));
	}

	#[test]
	fn div_underflow() {
		let error: Result<Decimal, rust_decimal::Error> =
			Node::Expr(Expr::Div(Decimal::NEGATIVE_ONE.into(), Decimal::ZERO.into()).into())
				.try_into();
		assert_eq!(
			error,
			Err(rust_decimal::Error::LessThanMinimumPossibleValue)
		);
	}

	#[test]
	fn neg() {
		assert_eq!(
			Decimal::NEGATIVE_ONE,
			Node::Expr(Expr::Neg(Decimal::ONE.into()).into())
				.try_into()
				.unwrap()
		);
	}
}
