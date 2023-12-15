mod gen;

use crate::{parser::Ty, util::NumTy};
use gen::Gen;

#[derive(Debug, Clone)]
pub enum Stmt {
	Local { name: &'static str },
	Assign { var: Var, val: Expr },
	Throw { msg: String },
	Assert { cond: Expr, msg: String },

	BlockStart,
	NumFor { var: String, start: Expr, end: Expr },
	GenFor { key: String, val: String, expr: Expr },
	If { cond: Expr },
	ElseIf { cond: Expr },
	Else,

	BlockEnd,

	Alloc { into: Var, len: Expr },
	WriteNum { expr: Expr, ty: NumTy, at: Option<Expr> },
	WriteStr { expr: Expr, len: Expr },
	WriteRef { expr: Expr, ref_name: String },
	WriteInst { expr: Expr },

	ReadNum { into: Var, ty: NumTy },
	ReadStr { into: Var, len: Expr },
	ReadRef { into: Var, ref_name: String },
	ReadInst { into: Var },
}

#[derive(Debug, Clone)]
pub enum Var {
	Name(String),

	NameIndex(Box<Var>, String),
	ExprIndex(Box<Var>, Box<Expr>),
}

impl Var {
	fn name_index(self, index: String) -> Var {
		Var::NameIndex(Box::new(self), index)
	}

	fn expr_index(self, index: Expr) -> Var {
		Var::ExprIndex(Box::new(self), Box::new(index))
	}
}

impl From<&str> for Var {
	fn from(string: &str) -> Var {
		Var::Name(string.to_string())
	}
}

#[derive(Debug, Clone)]
pub enum Expr {
	False,
	True,
	Nil,

	Num(f64),
	Str(String),
	Var(Box<Var>),

	EmptyArr,
	EmptyObj,

	Vector3(Box<Expr>, Box<Expr>, Box<Expr>),

	InstanceIsA(Box<Expr>, Box<Expr>),

	Len(Box<Expr>),

	Lt(Box<Expr>, Box<Expr>),
	Gt(Box<Expr>, Box<Expr>),
	Le(Box<Expr>, Box<Expr>),
	Ge(Box<Expr>, Box<Expr>),
	Eq(Box<Expr>, Box<Expr>),
	Or(Box<Expr>, Box<Expr>),

	Add(Box<Expr>, Box<Expr>),
}

impl Expr {
	pub fn is_a(self, ty: Expr) -> Expr {
		Expr::InstanceIsA(Box::new(self), Box::new(ty))
	}

	pub fn len(self) -> Expr {
		Expr::Len(Box::new(self))
	}

	pub fn lt(self, other: Expr) -> Expr {
		Expr::Lt(Box::new(self), Box::new(other))
	}

	#[allow(dead_code)]
	pub fn gt(self, other: Expr) -> Expr {
		Expr::Gt(Box::new(self), Box::new(other))
	}

	pub fn le(self, other: Expr) -> Expr {
		Expr::Le(Box::new(self), Box::new(other))
	}

	pub fn ge(self, other: Expr) -> Expr {
		Expr::Ge(Box::new(self), Box::new(other))
	}

	pub fn eq(self, other: Expr) -> Expr {
		Expr::Eq(Box::new(self), Box::new(other))
	}

	pub fn or(self, other: Expr) -> Expr {
		Expr::Or(Box::new(self), Box::new(other))
	}

	pub fn add(self, other: Expr) -> Expr {
		Expr::Add(Box::new(self), Box::new(other))
	}
}

impl From<Var> for Expr {
	fn from(var: Var) -> Expr {
		Expr::Var(Box::new(var))
	}
}

impl From<&str> for Expr {
	fn from(string: &str) -> Expr {
		Expr::Var(Box::new(Var::Name(string.to_string())))
	}
}

impl From<String> for Expr {
	fn from(string: String) -> Expr {
		Expr::Str(string)
	}
}

impl From<bool> for Expr {
	fn from(boolean: bool) -> Expr {
		if boolean {
			Expr::True
		} else {
			Expr::False
		}
	}
}

impl From<f64> for Expr {
	fn from(num: f64) -> Expr {
		Expr::Num(num)
	}
}

impl From<f32> for Expr {
	fn from(num: f32) -> Expr {
		Expr::Num(num as f64)
	}
}

impl From<u8> for Expr {
	fn from(num: u8) -> Expr {
		Expr::Num(num as f64)
	}
}

impl From<u16> for Expr {
	fn from(num: u16) -> Expr {
		Expr::Num(num as f64)
	}
}

impl From<u32> for Expr {
	fn from(num: u32) -> Expr {
		Expr::Num(num as f64)
	}
}

impl From<i8> for Expr {
	fn from(num: i8) -> Expr {
		Expr::Num(num as f64)
	}
}

impl From<i16> for Expr {
	fn from(num: i16) -> Expr {
		Expr::Num(num as f64)
	}
}

impl From<i32> for Expr {
	fn from(num: i32) -> Expr {
		Expr::Num(num as f64)
	}
}

impl From<usize> for Expr {
	fn from(num: usize) -> Expr {
		Expr::Num(num as f64)
	}
}

pub fn gen_ser(ty: &Ty, from: &str, checks: bool) -> Vec<Stmt> {
	let mut gen = Gen::new(checks, false);
	gen.ser(ty, &from.into());
	gen.output()
}

pub fn gen_des(ty: &Ty, into: &str, checks: bool) -> Vec<Stmt> {
	let mut gen = Gen::new(checks, false);
	gen.des(ty, &into.into());
	gen.output()
}
