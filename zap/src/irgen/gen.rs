use std::fmt::Display;

use num_traits::{Num, NumCast};

use crate::{
	parser::Ty,
	util::{NumTy, Range},
};

use super::{Expr, Stmt, Var};

pub struct Gen {
	stmts: Vec<Stmt>,

	ser_checks: bool,
	des_checks: bool,
}

impl Gen {
	pub fn new(ser_checks: bool, des_checks: bool) -> Self {
		Self {
			stmts: Vec::new(),

			ser_checks,
			des_checks,
		}
	}

	pub fn output(self) -> Vec<Stmt> {
		self.stmts
	}

	fn emit(&mut self, stmt: Stmt) {
		self.stmts.push(stmt);
	}

	fn alloc(&mut self, into: Var, len: Expr) {
		self.emit(Stmt::Alloc { into, len });
	}

	fn write_num(&mut self, expr: Expr, ty: NumTy) {
		self.emit(Stmt::WriteNum { expr, ty, at: None });
	}

	fn write_num_at(&mut self, expr: Expr, ty: NumTy, at: Expr) {
		self.emit(Stmt::WriteNum { expr, ty, at: Some(at) });
	}

	fn write_str(&mut self, expr: Expr, len: Expr) {
		self.emit(Stmt::WriteStr { expr, len });
	}

	fn write_ref(&mut self, expr: Expr, ref_name: String) {
		self.emit(Stmt::WriteRef { expr, ref_name });
	}

	fn write_inst(&mut self, expr: Expr) {
		self.emit(Stmt::WriteInst { expr });
	}

	fn read_num(&mut self, into: Var, ty: NumTy) {
		self.emit(Stmt::ReadNum { into, ty });
	}

	fn read_str(&mut self, into: Var, len: Expr) {
		self.emit(Stmt::ReadStr { into, len });
	}

	fn read_ref(&mut self, into: Var, ref_name: String) {
		self.emit(Stmt::ReadRef { into, ref_name });
	}

	fn read_inst(&mut self, into: Var) {
		self.emit(Stmt::ReadInst { into });
	}

	fn block_start(&mut self) {
		self.emit(Stmt::BlockStart);
	}

	fn num_for(&mut self, var: String, start: Expr, end: Expr) {
		self.emit(Stmt::NumFor { var, start, end });
	}

	fn gen_for(&mut self, key: String, val: String, expr: Expr) {
		self.emit(Stmt::GenFor { key, val, expr });
	}

	fn if_(&mut self, cond: Expr) {
		self.emit(Stmt::If { cond });
	}

	fn else_if(&mut self, cond: Expr) {
		self.emit(Stmt::ElseIf { cond });
	}

	fn else_(&mut self) {
		self.emit(Stmt::Else);
	}

	fn block_end(&mut self) {
		self.emit(Stmt::BlockEnd);
	}

	fn local(&mut self, name: &'static str) {
		self.emit(Stmt::Local { name });
	}

	fn assign(&mut self, var: Var, val: Expr) {
		self.emit(Stmt::Assign { var, val });
	}

	fn throw(&mut self, msg: String) {
		self.emit(Stmt::Throw { msg });
	}

	fn assert(&mut self, cond: Expr, msg: String) {
		self.emit(Stmt::Assert { cond, msg });
	}

	pub fn ser(&mut self, ty: &Ty, from: &Var) {
		let from_expr: Expr = from.clone().into();

		if self.ser_checks {
			self.checks(from, ty);
		}

		match ty {
			Ty::Bool => {
				self.if_(from_expr.clone());
				self.write_num(Expr::Num(1.0), NumTy::U8);
				self.else_();
				self.write_num(Expr::Num(0.0), NumTy::U8);
				self.block_end();
			}

			Ty::F32(..) => self.write_num(from_expr, NumTy::F32),
			Ty::F64(..) => self.write_num(from_expr, NumTy::F64),

			Ty::U8(..) => self.write_num(from_expr, NumTy::U8),
			Ty::U16(..) => self.write_num(from_expr, NumTy::U16),
			Ty::U32(..) => self.write_num(from_expr, NumTy::U32),

			Ty::I8(..) => self.write_num(from_expr, NumTy::I8),
			Ty::I16(..) => self.write_num(from_expr, NumTy::I16),
			Ty::I32(..) => self.write_num(from_expr, NumTy::I32),

			Ty::Str { len } => {
				if len.is_exact() {
					self.write_str(from_expr, len.min().unwrap().into());
				} else {
					self.block_start();

					self.local("len");
					self.assign("len".into(), from_expr.clone().len());

					self.write_num("len".into(), NumTy::U16);
					self.write_str(from_expr, "len".into());

					self.block_end();
				}
			}

			Ty::Arr { len, ty } => {
				if len.is_exact() {
					self.num_for("i".into(), 1.into(), len.min().unwrap().into());

					self.ser(ty, &from.clone().expr_index("i".into()));

					self.block_end();
				} else {
					self.block_start();

					self.local("len");
					self.assign("len".into(), from_expr.clone().len());

					self.write_num("len".into(), NumTy::U16);

					self.num_for("i".into(), 1.into(), "len".into());

					self.ser(ty, &from.clone().expr_index("i".into()));

					self.block_end();

					self.block_end();
				}
			}

			Ty::Map { key, val } => {
				self.block_start();

				self.local("len");
				self.assign("len".into(), 0.into());

				self.local("len_pos");
				self.alloc("len_pos".into(), 2.into());

				self.gen_for("key".into(), "val".into(), from_expr);

				self.assign("len".into(), Expr::from("len").add(1.into()));

				self.ser(key, &"key".into());
				self.ser(val, &"val".into());

				self.block_end();

				self.write_num_at("len".into(), NumTy::U16, "len_pos".into());
			}

			Ty::Struct { fields } => {
				for (name, ty) in fields {
					self.ser(ty, &from.clone().name_index(name.clone()));
				}
			}

			Ty::Enum { variants } => {
				let num_ty = NumTy::from_f64(0.0, variants.len() as f64 - 1.0);

				for (i, variant) in variants.iter().enumerate() {
					if i == 0 {
						self.if_(from_expr.clone().eq(variant.clone().into()));
					} else {
						self.else_if(from_expr.clone().eq(variant.clone().into()));
					}

					self.write_num(i.into(), num_ty);
				}

				self.else_();

				self.throw("Invalid enum variant!".into());

				self.block_end();
			}

			Ty::Instance(_) => self.write_inst(from_expr),

			Ty::Vector3 => {
				self.write_num(from.clone().name_index("X".into()).into(), NumTy::F32);
				self.write_num(from.clone().name_index("Y".into()).into(), NumTy::F32);
				self.write_num(from.clone().name_index("Z".into()).into(), NumTy::F32);
			}

			Ty::Ref(name) => self.write_ref(from_expr, name.clone()),

			Ty::Optional(ty) => {
				self.if_(from_expr.clone().eq(Expr::Nil));

				self.write_num(Expr::Num(0.0), NumTy::U8);

				self.else_();

				self.write_num(Expr::Num(1.0), NumTy::U8);

				self.ser(ty, from);

				self.block_end();
			}
		}
	}

	pub fn des(&mut self, ty: &Ty, into: &Var) {
		match ty {
			Ty::Bool => {
				self.block_start();

				self.local("val");
				self.read_num("val".into(), NumTy::U8);

				self.if_(Expr::from("val").eq(Expr::Num(0.0)));
				self.assign(into.clone(), Expr::False);
				self.else_();
				self.assign(into.clone(), Expr::True);
				self.block_end();

				self.block_end();
			}

			Ty::F32(..) => self.read_num(into.clone(), NumTy::F32),
			Ty::F64(..) => self.read_num(into.clone(), NumTy::F64),

			Ty::U8(..) => self.read_num(into.clone(), NumTy::U8),
			Ty::U16(..) => self.read_num(into.clone(), NumTy::U16),
			Ty::U32(..) => self.read_num(into.clone(), NumTy::U32),

			Ty::I8(..) => self.read_num(into.clone(), NumTy::I8),
			Ty::I16(..) => self.read_num(into.clone(), NumTy::I16),
			Ty::I32(..) => self.read_num(into.clone(), NumTy::I32),

			Ty::Str { len } => {
				if len.is_exact() {
					self.read_str(into.clone(), len.min().unwrap().into());
				} else {
					self.block_start();

					self.local("len");
					self.read_num("len".into(), NumTy::U16);

					self.read_str(into.clone(), "len".into());

					self.block_end();
				}
			}

			Ty::Arr { len, ty } => {
				self.assign(into.clone(), Expr::EmptyArr);

				if len.is_exact() {
					self.num_for("i".into(), 1.into(), len.min().unwrap().into());

					self.des(ty, &into.clone().expr_index("i".into()));

					self.block_end();
				} else {
					self.block_start();

					self.local("len");
					self.read_num("len".into(), NumTy::U16);

					self.num_for("i".into(), 1.into(), "len".into());

					self.des(ty, &into.clone().expr_index("i".into()));

					self.block_end();

					self.block_end();
				}
			}

			Ty::Map { key, val } => {
				self.assign(into.clone(), Expr::EmptyObj);

				self.block_start();

				self.local("len");
				self.read_num("len".into(), NumTy::U16);

				self.num_for("i".into(), 1.into(), "len".into());

				self.local("key");
				self.des(key, &"key".into());

				self.local("val");
				self.des(val, &"val".into());

				self.assign(into.clone().expr_index("key".into()), "val".into());

				self.block_end();

				self.block_end();
			}

			Ty::Struct { fields } => {
				self.assign(into.clone(), Expr::EmptyObj);

				for (name, ty) in fields {
					self.des(ty, &into.clone().name_index(name.clone()));
				}
			}

			Ty::Enum { variants } => {
				let num_ty = NumTy::from_f64(0.0, variants.len() as f64 - 1.0);

				self.block_start();

				self.local("val");
				self.read_num("val".into(), num_ty);

				for (i, variant) in variants.iter().enumerate() {
					if i == 0 {
						self.if_(Expr::from("val").eq(i.into()));
					} else {
						self.else_if(Expr::from("val").eq(i.into()));
					}

					self.assign(into.clone(), variant.clone().into());
				}

				self.else_();

				self.throw("Invalid enum variant!".into());

				self.block_end();

				self.block_end();
			}

			Ty::Instance(_) => {
				self.read_inst(into.clone());
				self.assert(Expr::from(into.clone()), "Instance could not be found!".into());
			}

			Ty::Vector3 => {
				self.local("x");
				self.local("y");
				self.local("z");

				self.read_num("x".into(), NumTy::F32);
				self.read_num("y".into(), NumTy::F32);
				self.read_num("z".into(), NumTy::F32);

				self.assign(
					into.clone(),
					Expr::Vector3(Box::new("x".into()), Box::new("y".into()), Box::new("z".into())),
				);
			}

			Ty::Ref(name) => self.read_ref(into.clone(), name.clone()),

			Ty::Optional(ty) => match **ty {
				Ty::Instance(_) => self.read_inst(into.clone()),

				_ => {
					self.block_start();

					self.local("val");
					self.read_num("val".into(), NumTy::U8);

					self.if_(Expr::from("val").eq(0.into()));

					self.assign(into.clone(), Expr::Nil);

					self.else_();

					self.des(ty, into);

					self.block_end();

					self.block_end();
				}
			},
		}

		if self.des_checks {
			self.checks(into, ty);
		}
	}

	fn check_range<T: Num + NumCast + Copy + Display + Into<Expr>>(&mut self, var: &Var, range: &Range<T>) {
		if let Some(min) = range.min() {
			self.assert(
				Expr::from(var.clone()).ge(min.into()),
				format!("Value is less than minimum of {}!", min),
			);
		}

		if let Some(max) = range.max() {
			if range.max_inclusive() {
				self.assert(
					Expr::from(var.clone()).le(max.into()),
					format!("Value is greater than maximum of {}!", max),
				);
			} else {
				self.assert(
					Expr::from(var.clone()).lt(max.into()),
					format!("Value is greater than maximum of {}!", max),
				);
			}
		}
	}

	fn checks(&mut self, var: &Var, ty: &Ty) {
		match ty {
			Ty::F32(range) => self.check_range(var, range),
			Ty::F64(range) => self.check_range(var, range),

			Ty::U8(range) => self.check_range(var, range),
			Ty::U16(range) => self.check_range(var, range),
			Ty::U32(range) => self.check_range(var, range),

			Ty::I8(range) => self.check_range(var, range),
			Ty::I16(range) => self.check_range(var, range),
			Ty::I32(range) => self.check_range(var, range),

			Ty::Str { len } => {
				if len.is_exact() {
					self.assert(
						Expr::from(var.clone()).len().eq(len.min().unwrap().into()),
						format!("String is not exactly {} characters long!", len.min().unwrap()),
					);
				} else {
					if let Some(min) = len.min() {
						self.assert(
							Expr::from(var.clone()).len().ge(min.into()),
							format!("String is shorter than minimum length of {}!", min),
						);
					}

					if let Some(max) = len.max() {
						if len.max_inclusive() {
							self.assert(
								Expr::from(var.clone()).len().le(max.into()),
								format!("String is longer than maximum length of {}!", max),
							);
						} else {
							self.assert(
								Expr::from(var.clone()).len().lt(max.into()),
								format!("String is longer than maximum length of {}!", max),
							);
						}
					}
				}
			}

			Ty::Arr { len, .. } => {
				if len.is_exact() {
					self.assert(
						Expr::from(var.clone()).len().eq(len.min().unwrap().into()),
						format!("Array is not exactly {} elements long!", len.min().unwrap()),
					);
				} else {
					if let Some(min) = len.min() {
						self.assert(
							Expr::from(var.clone()).len().ge(min.into()),
							format!("Array is shorter than minimum length of {}!", min),
						);
					}

					if let Some(max) = len.max() {
						if len.max_inclusive() {
							self.assert(
								Expr::from(var.clone()).len().le(max.into()),
								format!("Array is longer than maximum length of {}!", max),
							);
						} else {
							self.assert(
								Expr::from(var.clone()).len().lt(max.into()),
								format!("Array is longer than maximum length of {}!", max),
							);
						}
					}
				}
			}

			Ty::Instance(Some(class)) => {
				self.assert(
					Expr::from(var.clone()).is_a(class.clone().into()),
					format!("Instance is not of class {}!", class),
				);
			}

			Ty::Optional(ty) => {
				if let Ty::Instance(Some(class)) = *ty.clone() {
					self.assert(
						Expr::from(var.clone())
							.eq(Expr::Nil)
							.or(Expr::from(var.clone()).is_a(class.clone().into())),
						format!("Instance is not of class {}!", class),
					)
				}
			}

			_ => {}
		}
	}
}
