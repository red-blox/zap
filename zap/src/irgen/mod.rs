#![allow(dead_code)]
use std::fmt::Display;

mod gen;

pub use gen::{gen_des, gen_ser};

#[derive(Debug, Clone)]
pub enum Stmt {
	Local(&'static str, Option<Expr>),
	Assign(Var, Expr),
	Error(String),
	Assert(Expr, Option<String>),

	Call(Var, Option<String>, Vec<Expr>),

	NumFor {
		var: &'static str,
		from: Expr,
		to: Expr,
	},
	GenFor {
		key: &'static str,
		val: &'static str,
		obj: Expr,
	},
	If(Expr),
	ElseIf(Expr),
	Else,

	End,
}

#[derive(Debug, Clone)]
pub enum Var {
	Name(String),

	NameIndex(Box<Var>, String),
	ExprIndex(Box<Var>, Box<Expr>),
}

impl Var {
	pub fn nindex(self, index: impl Into<String>) -> Self {
		Self::NameIndex(Box::new(self), index.into())
	}

	pub fn eindex(self, index: Expr) -> Self {
		Self::ExprIndex(Box::new(self), Box::new(index))
	}

	pub fn call(self, args: Vec<Expr>) -> Expr {
		Expr::Call(Box::new(self), None, args)
	}
}

impl From<&str> for Var {
	fn from(name: &str) -> Self {
		Self::Name(name.into())
	}
}

impl Display for Var {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Name(name) => write!(f, "{}", name),
			Self::NameIndex(var, index) => write!(f, "{}.{}", var, index),
			Self::ExprIndex(var, index) => write!(f, "{}[{}]", var, index),
		}
	}
}

#[derive(Debug, Clone)]
pub enum Expr {
	// Keyword Literals
	False,
	True,
	Nil,

	// Literals
	Str(String),
	Var(Box<Var>),
	Num(f64),

	// Function Call
	Call(Box<Var>, Option<String>, Vec<Expr>),

	// Table
	EmptyTab,

	// Vector3
	Vector3(Box<Expr>, Box<Expr>, Box<Expr>),

	// Unary Operators
	Len(Box<Expr>),
	Not(Box<Expr>),

	// Boolean Binary Operators
	And(Box<Expr>, Box<Expr>),
	Or(Box<Expr>, Box<Expr>),

	// Comparison Binary Operators
	Gte(Box<Expr>, Box<Expr>),
	Lte(Box<Expr>, Box<Expr>),
	Neq(Box<Expr>, Box<Expr>),
	Gt(Box<Expr>, Box<Expr>),
	Lt(Box<Expr>, Box<Expr>),
	Eq(Box<Expr>, Box<Expr>),

	// Arithmetic Binary Operators
	Add(Box<Expr>, Box<Expr>),
}

impl Expr {
	pub fn len(self) -> Self {
		Self::Len(Box::new(self))
	}

	pub fn not(self) -> Self {
		Self::Not(Box::new(self))
	}

	pub fn and(self, other: Self) -> Self {
		Self::And(Box::new(self), Box::new(other))
	}

	pub fn or(self, other: Self) -> Self {
		Self::Or(Box::new(self), Box::new(other))
	}

	pub fn gte(self, other: Self) -> Self {
		Self::Gte(Box::new(self), Box::new(other))
	}

	pub fn lte(self, other: Self) -> Self {
		Self::Lte(Box::new(self), Box::new(other))
	}

	pub fn neq(self, other: Self) -> Self {
		Self::Neq(Box::new(self), Box::new(other))
	}

	pub fn gt(self, other: Self) -> Self {
		Self::Gt(Box::new(self), Box::new(other))
	}

	pub fn lt(self, other: Self) -> Self {
		Self::Lt(Box::new(self), Box::new(other))
	}

	pub fn eq(self, other: Self) -> Self {
		Self::Eq(Box::new(self), Box::new(other))
	}

	pub fn add(self, other: Self) -> Self {
		Self::Add(Box::new(self), Box::new(other))
	}
}

impl From<String> for Expr {
	fn from(string: String) -> Self {
		Self::Str(string)
	}
}

impl From<Var> for Expr {
	fn from(var: Var) -> Self {
		Self::Var(Box::new(var))
	}
}

impl From<&str> for Expr {
	fn from(name: &str) -> Self {
		Self::Var(Box::new(name.into()))
	}
}

impl From<f64> for Expr {
	fn from(num: f64) -> Self {
		Self::Num(num)
	}
}

impl Display for Expr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::False => write!(f, "false"),
			Self::True => write!(f, "true"),
			Self::Nil => write!(f, "nil"),

			Self::Str(string) => write!(f, "\"{}\"", string),
			Self::Var(var) => write!(f, "{}", var),
			Self::Num(num) => write!(f, "{}", num),

			Self::Call(var, method, args) => match method {
				Some(method) => write!(
					f,
					"{}:{}({})",
					var,
					method,
					args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>().join(", ")
				),

				None => write!(
					f,
					"{}({})",
					var,
					args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>().join(", ")
				),
			},

			Self::EmptyTab => write!(f, "{{}}"),

			Self::Vector3(x, y, z) => write!(f, "Vector3.new({}, {}, {})", x, y, z),

			Self::Len(expr) => write!(f, "#{}", expr),
			Self::Not(expr) => write!(f, "not {}", expr),

			Self::And(lhs, rhs) => write!(f, "{} and {}", lhs, rhs),
			Self::Or(lhs, rhs) => write!(f, "{} or {}", lhs, rhs),

			Self::Gte(lhs, rhs) => write!(f, "{} >= {}", lhs, rhs),
			Self::Lte(lhs, rhs) => write!(f, "{} <= {}", lhs, rhs),
			Self::Neq(lhs, rhs) => write!(f, "{} ~= {}", lhs, rhs),
			Self::Gt(lhs, rhs) => write!(f, "{} > {}", lhs, rhs),
			Self::Lt(lhs, rhs) => write!(f, "{} < {}", lhs, rhs),
			Self::Eq(lhs, rhs) => write!(f, "{} == {}", lhs, rhs),

			Self::Add(lhs, rhs) => write!(f, "{} + {}", lhs, rhs),
		}
	}
}
